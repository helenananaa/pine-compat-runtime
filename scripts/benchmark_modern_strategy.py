#!/usr/bin/env python3
"""Run an offline release Rust probe; report timings, correctness and resource growth.

Build first: cargo build --release -p pine-runtime --example strategy_benchmark
Each scenario runs in its own process. No Python extension or public API change.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import platform
import statistics
import subprocess
from pathlib import Path
from typing import Any, Callable, Sequence

SCHEMA_VERSION = 2
TOOL_VERSION = 2
ROOT = Path(__file__).resolve().parents[1]


class BenchmarkError(ValueError):
    """Invalid configuration, failed execution or correctness check."""


def sha256_json(value: Any) -> str:
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode()).hexdigest()


def synthetic_bars(count: int, *, seed: int) -> list[dict[str, float | int]]:
    bars = []
    price = 100.0 + seed % 7
    for index in range(count):
        close = price + ((index * 17 + seed * 13) % 11) / 10.0 - 0.5
        bars.append(dict(time=(index + 1) * 60000, open=price, high=max(price, close) + 0.25,
                         low=min(price, close) - 0.25, close=close, volume=100 + index % 5))
        price = close
    return bars


def magnifier_input(bars: list[dict]) -> dict:
    groups = []
    for index, bar in enumerate(bars):
        first = dict(bar, close=bar['open'], volume=bar['volume'] / 2)
        second = dict(bar, time=bar['time'] + 30000, volume=bar['volume'] / 2)
        groups.append(dict(chartBarIndex=index, bars=[first, second]))
    return dict(schemaVersion=1, chartBars=groups)


def stats_ms(values: Sequence[float]) -> dict:
    if not values or any(not math.isfinite(v) or v < 0 for v in values):
        raise BenchmarkError("timings must be nonempty finite nonnegative values")
    result = dict(n=len(values), medianMs=statistics.median(values), stdevMs=statistics.pstdev(values), samplesMs=list(values))
    if len(values) >= 100:
        result['p95Ms'] = sorted(values)[math.ceil(len(values) * .95) - 1]
        result['p95Scope'] = 'pooled operations, not independent processes'
    return result


def default_samples() -> list[dict]:
    return [
        dict(sampleId='trend', sourcePath='tests/fixtures/benchmark/strategy_trend.pine'),
        dict(sampleId='dense', sourcePath='tests/fixtures/benchmark/strategy_dense.pine'),
        dict(sampleId='collection', sourcePath='tests/fixtures/profile/matrix_heavy.pine'),
        dict(sampleId='recalculation', sourcePath='tests/fixtures/benchmark/strategy_recalculation.pine'),
        dict(sampleId='realtime', sourcePath='tests/fixtures/runtime/strategy_calc_on_every_tick.pine'),
        dict(sampleId='magnifier', sourcePath='tests/fixtures/benchmark/strategy_magnifier.pine', magnifier=True),
    ]


def run_probe(binary: Path, payload: dict, timeout: float) -> dict:
    result = subprocess.run([str(binary)], input=json.dumps(payload), text=True,
                            capture_output=True, timeout=timeout, check=False)
    if result.returncode:
        raise BenchmarkError(f'probe exit {result.returncode}: {result.stderr[-4000:]}')
    return json.loads(result.stdout)


def summarize_probe(raw: dict, *, spec: dict, payload: dict) -> dict:
    correctness = raw['correctness']
    if correctness.get('batchEqualsIncremental') is not True or correctness.get('repeatedHistoricalStable') is not True:
        raise BenchmarkError('historical/incremental correctness failed')
    if not spec.get('magnifier') and correctness.get('repeatedLiveStable') is not True:
        raise BenchmarkError('realtime repeatability failed')
    required = {'compile', 'historicalRun', 'incrementalAppend', 'resultSnapshot', 'outputSerialization'}
    if not spec.get('magnifier'):
        required |= {'realtimeSeed', 'formingInitial', 'formingReplace', 'formingConfirm'}
    if not required <= raw['timingsMs'].keys():
        raise BenchmarkError('missing required timing phase')
    for phase in required:
        expected_count = payload['iters'] * (payload['replacements'] if phase == 'formingReplace' else 1)
        if len(raw['timingsMs'][phase]) != expected_count:
            raise BenchmarkError(f'wrong measurement count for {phase}')
    profiles = raw['profile']
    if not profiles or profiles.get('bars') != len(payload['bars']):
        raise BenchmarkError('missing or wrong runtime profile')
    if spec['sampleId'] in {'dense', 'magnifier', 'recalculation'} and raw['orderCount'] == 0:
        raise BenchmarkError('order workload produced no order events')
    if spec['sampleId'] == 'recalculation' and not profiles.get('strategyRecalculationPasses', 0):
        raise BenchmarkError('recalculation workload did not recalculate')
    return dict(
        sampleId=spec['sampleId'], status='measured', sourcePath=spec['sourcePath'],
        sourceHash=hashlib.sha256(payload['source'].encode()).hexdigest(),
        inputHash=sha256_json({'bars': payload['bars'], 'magnifier': payload['magnifier']}),
        barCount=len(payload['bars']), lowerBarCount=len(payload['bars']) * 2 if spec.get('magnifier') else 0,
        formingReplacementsPerIteration=0 if spec.get('magnifier') else payload['replacements'],
        resultHash=sha256_json(raw['result']), liveResultHash=sha256_json(raw['liveResult']) if raw['liveResult'] is not None else None,
        outputBytes=len(json.dumps(raw['result'], separators=(',', ':')).encode()),
        orderCount=raw['orderCount'], correctness=correctness,
        phases={key: stats_ms(value) for key, value in raw['timingsMs'].items()},
        profile=profiles, confirmedRealtimeProfile=raw['confirmedRealtimeProfile'],
        peakRssKiB=raw.get('peakRssKiB'),
        rssStatus='measured' if raw.get('peakRssKiB') is not None else 'unavailable',
        rssScope='fresh Linux process peak through verification, before final probe report rendering; not runtime-only',
        realtimeExclusion=raw.get('realtimeExclusion'),
        formingExecution=('historical-only: excluded' if spec.get('magnifier') else
                          'script executes on replacements' if spec['sampleId'] in {'realtime', 'collection'} else
                          'default strategy: forming returns confirmed state; script executes on confirmation'))


def resource_growth(samples: list[dict]) -> list[dict]:
    groups = {}
    for s in samples:
        if s['status'] == 'measured':
            groups.setdefault(s['sampleId'], []).append(s)
    result = []
    for name, rows in groups.items():
        ordered = sorted(rows, key=lambda x: x['barCount'])
        for a, b in zip(ordered, ordered[1:]):
            result.append(dict(sampleId=name, fromBars=a['barCount'], toBars=b['barCount'],
                               outputBytesDelta=b['outputBytes']-a['outputBytes'],
                               peakRssKiBDelta=b['peakRssKiB']-a['peakRssKiB'] if a['peakRssKiB'] is not None and b['peakRssKiB'] is not None else None,
                               profileDeltas={k: v-a['profile'][k] for k, v in b['profile'].items()
                                              if type(v) is int and type(a['profile'].get(k)) is int}))
    return result


def build_report(*, root: Path, binary: Path, bar_counts: Sequence[int], seed: int,
                 warmup: int, iters: int, replacements: int, timeout: float,
                 runner: Callable = run_probe) -> dict:
    if iters < 1 or warmup < 0 or replacements < 2 or timeout <= 0 or not math.isfinite(timeout):
        raise BenchmarkError('invalid repetitions, warmup, replacements or timeout')
    if not bar_counts or any(n < 2 for n in bar_counts) or len(set(bar_counts)) != len(bar_counts):
        raise BenchmarkError('bar counts must be distinct integers >= 2')
    samples = []
    for spec in default_samples():
        source = (root / spec['sourcePath']).read_text()
        for count in bar_counts:
            bars = synthetic_bars(count, seed=seed)
            payload = dict(source=source, bars=bars, warmup=warmup, iters=iters,
                           replacements=replacements, magnifier=magnifier_input(bars) if spec.get('magnifier') else None)
            try:
                samples.append(summarize_probe(runner(binary, payload, timeout), spec=spec, payload=payload))
            except (BenchmarkError, subprocess.TimeoutExpired, ValueError, KeyError, OSError) as exc:
                samples.append(dict(sampleId=spec['sampleId'], barCount=count, status='failed', reason=str(exc)))
    failed = any(s['status'] == 'failed' for s in samples)
    adequate = iters >= 10 and replacements >= 100 and len(bar_counts) >= 3
    resources = not failed and all(s['rssStatus'] == 'measured' for s in samples)
    revision = subprocess.run(['git', 'rev-parse', 'HEAD'], cwd=root, text=True, capture_output=True, check=True).stdout.strip()
    dirty = subprocess.run(['git', 'status', '--porcelain'], cwd=root, text=True, capture_output=True, check=True).stdout
    return dict(schemaVersion=SCHEMA_VERSION, toolVersion=TOOL_VERSION,
                status='failed' if failed else '基线完成' if adequate and resources else 'partial',
                optimizationCommitted=False, seed=seed, barCounts=list(bar_counts), warmup=warmup,
                iters=iters, replacements=replacements,
                environment=dict(platform=platform.platform(), machine=platform.machine(),
                                 processor=platform.processor(), logicalCpuCount=os.cpu_count(),
                                 cargoLockSha256=hashlib.sha256((root/'Cargo.lock').read_bytes()).hexdigest(),
                                 probeSourceSha256=hashlib.sha256((root/'crates/pine-runtime/examples/strategy_benchmark.rs').read_bytes()).hexdigest(),
                                 rustc=subprocess.run(['rustc', '-vV'], capture_output=True, text=True, check=True).stdout,
                                 buildRevision=revision, worktreeStatus=dirty, binaryPath=str(binary),
                                 binarySha256=hashlib.sha256(binary.read_bytes()).hexdigest(),
                                 buildContract='cargo build --release -p pine-runtime --example strategy_benchmark; record build log'),
                timingScope=dict(historicalRun='append_bars only; excludes construction, result snapshot and JSON',
                                 incrementalAppend='one append_bar per bar on one runtime; excludes result snapshot and JSON',
                                 formingReplace='each changed forming update after seed and initial forming; includes runtime returned snapshot',
                                 formingConfirm='confirmed update only; includes returned snapshot',
                                 peakRss='Linux /proc process high-water mark; unavailable elsewhere'),
                samples=samples, resourceGrowth=resource_growth(samples),
                limitations=['synthetic local regression inputs, not independent TradingView reference',
                             'resource growth observations are not a proved asymptotic bound',
                             'no optimization or speedup claim'])


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=ROOT)
    parser.add_argument('--binary', type=Path, required=True, help='prebuilt release strategy_benchmark example')
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--seed', type=int, default=1)
    parser.add_argument('--bars', default='64,256,1024')
    parser.add_argument('--warmup', type=int, default=2)
    parser.add_argument('--iters', type=int, default=10)
    parser.add_argument('--replacements', type=int, default=100)
    parser.add_argument('--timeout', type=float, default=180)
    args = parser.parse_args(argv)
    try:
        report = build_report(root=args.root.resolve(), binary=args.binary.resolve(),
                              bar_counts=[int(x) for x in args.bars.split(',')], seed=args.seed,
                              warmup=args.warmup, iters=args.iters, replacements=args.replacements, timeout=args.timeout)
    except (BenchmarkError, OSError, ValueError, subprocess.CalledProcessError) as exc:
        parser.error(str(exc))
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True, allow_nan=False)+'\n')
    print(f"{report['status']}: {args.output}")
    return 1 if report['status'] == 'failed' else 0


if __name__ == '__main__':
    raise SystemExit(main())

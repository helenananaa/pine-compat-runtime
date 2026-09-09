#!/usr/bin/env python3
"""Measure a sustained tail; this tool does not award resource acceptance."""
from __future__ import annotations

import argparse
import csv
import hashlib
import json
import math
import subprocess
from pathlib import Path

from benchmark_modern_strategy import BenchmarkError, magnifier_input, run_probe, sha256_json, stats_ms, synthetic_bars

ROOT = Path(__file__).resolve().parents[1]


def summarize(raw: dict, payload: dict) -> dict:
    history, total = payload['historyBars'], len(payload['bars'])
    tail, repeats, replacements = total-history, payload['repetitions'], payload['replacementsPerBar']
    magnified = payload.get('magnifier') is not None
    for key, value in [('schemaVersion', 1), ('historyBars', history), ('tailBars', tail),
                       ('repetitions', repeats), ('replacementsPerBar', replacements)]:
        if raw.get(key) != value:
            raise BenchmarkError(f'wrong reported {key}')
    correctness = raw['correctness']
    if correctness.get('batchEqualsTail') is not True or correctness.get('repeatedHistoricalStable') is not True:
        raise BenchmarkError('historical tail correctness failed')
    counts = dict(compile=1, historySeed=repeats, tailAppend=repeats*tail,
                  resultSnapshot=repeats, outputSerialization=repeats)
    if not magnified:
        if correctness.get('repeatedLiveStable') is not True or raw.get('realtimeExclusion') is not None:
            raise BenchmarkError('live tail correctness failed or silently excluded')
        counts.update(liveSeed=repeats, formingInitial=repeats*tail,
                      formingReplace=repeats*tail*replacements, formingConfirm=repeats*tail,
                      liveResultSnapshot=repeats, liveOutputSerialization=repeats)
    elif (correctness.get('repeatedLiveStable') is not None or raw.get('liveResult') is not None
          or raw.get('realtimeExclusion') != 'historical-only magnifier input'):
        raise BenchmarkError('magnifier live exclusion is incorrect')
    if set(raw['timingsMs']) != set(counts):
        raise BenchmarkError('missing or unexpected timing phases')
    phases = {}
    for phase, count in counts.items():
        values = raw['timingsMs'][phase]
        if len(values) != count:
            raise BenchmarkError(f'wrong operation count for {phase}')
        phases[phase] = stats_ms(values)
    checkpoints = raw['memoryCheckpoints']
    expected_phases = ['afterHistorySeed', 'afterTailAppend'] + ([] if magnified else ['afterLiveSeed', 'afterLiveTail'])
    expected = [(phase, repetition) for repetition in range(repeats) for phase in expected_phases]
    expected.append(('afterVerification', repeats-1))
    if [(row['phase'], row['repetition']) for row in checkpoints] != expected:
        raise BenchmarkError('memory checkpoints do not cover every repetition')
    for row in checkpoints:
        for field in ('peakRssKiB', 'peakCommitKiB'):
            value = row.get(field)
            if value is not None and (type(value) is not int or value <= 0):
                raise BenchmarkError(f'invalid memory measurement {field}')
    historical = json.loads(raw['historicalResult'])
    live = None if magnified else json.loads(raw['liveResult'])
    return dict(status='measured', qualification='notEvaluated', historyBars=history, tailBars=tail,
                repetitions=repeats, replacementsPerBar=replacements, phases=phases,
                memoryCheckpoints=checkpoints, memoryScope=raw['memoryScope'],
                formingTimingScope=raw['formingTimingScope'], formingExecutesScript=raw['formingExecutesScript'],
                correctness=correctness, realtimeExclusion=raw['realtimeExclusion'],
                historicalResultHash=sha256_json(historical), liveResultHash=None if magnified else sha256_json(live),
                historicalOutputBytes=len(raw['historicalResult'].encode()),
                liveOutputBytes=None if magnified else len(raw['liveResult'].encode()))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--history-bars', type=int, required=True)
    parser.add_argument('--tail-bars', type=int, required=True)
    parser.add_argument('--bars-csv', type=Path)
    parser.add_argument('--library-source', action='append', default=[])
    parser.add_argument('--repetitions', type=int, default=2)
    parser.add_argument('--replacements-per-bar', type=int, default=1)
    parser.add_argument('--seed', type=int, default=1)
    parser.add_argument('--magnifier', action='store_true', help='generate two synthetic intrabars per chart bar')
    parser.add_argument('--timeout', type=float, default=180)
    args = parser.parse_args()
    if not math.isfinite(args.timeout) or args.timeout <= 0:
        parser.error('timeout must be positive and finite')
    if min(args.history_bars, args.tail_bars, args.replacements_per_bar) < 1 or args.repetitions < 2:
        parser.error('need positive history/tail/replacements and repetitions >= 2')
    total = args.history_bars + args.tail_bars
    if args.bars_csv:
        if args.magnifier:
            parser.error('synthetic magnifier must not be inferred from a supplied real CSV')
        with args.bars_csv.open(encoding='utf-8-sig', newline='') as stream:
            bars = [dict(time=int(row['time']), **{k: float(row[k]) for k in ('open','high','low','close','volume')})
                    for row in csv.DictReader(stream)]
        if len(bars) != total:
            parser.error('CSV must contain exactly history-bars + tail-bars; no rows are silently trimmed')
    else:
        bars = synthetic_bars(total, seed=args.seed)
    libraries = {}
    for spec in args.library_source:
        key, separator, filename = spec.partition('=')
        if not separator or key in libraries:
            parser.error('library source must be a unique KEY=path')
        libraries[key] = Path(filename).read_text(encoding='utf-8')
    payload = dict(source=args.source.read_text(encoding='utf-8'), libraries=libraries, bars=bars,
                   historyBars=args.history_bars, repetitions=args.repetitions,
                   replacementsPerBar=args.replacements_per_bar,
                   magnifier=magnifier_input(bars) if args.magnifier else None)
    report = dict(schemaVersion=1, payloadHash=sha256_json(payload), sourcePath=str(args.source.resolve()),
                  librarySourceHashes={key: hashlib.sha256(value.encode()).hexdigest() for key,value in libraries.items()},
                  barsHash=sha256_json(bars), seed=args.seed if not args.bars_csv else None,
                  barsCsvPath=str(args.bars_csv.resolve()) if args.bars_csv else None,
                  runnerHash=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                  probeSourceHashes={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in [ROOT/'crates/pine-runtime/examples/long_session_benchmark.rs',ROOT/'crates/pine-runtime/examples/benchmark_support/memory.rs']},
                  binaryPath=str(args.binary.resolve()),
                  sourceHash=hashlib.sha256(payload['source'].encode()).hexdigest(),
                  binaryHash=hashlib.sha256(args.binary.read_bytes()).hexdigest(),
                  revision=subprocess.check_output(['git','rev-parse','HEAD'], cwd=ROOT, text=True).strip(),
                  worktreeStatus=subprocess.check_output(['git','status','--porcelain'], cwd=ROOT, text=True),
                  inputKind='suppliedCsv' if args.bars_csv else 'syntheticRegression',
                  limitations=['measurement only; no frozen budget evaluated', 'internal consistency is not an independent TradingView oracle'])
    try:
        raw = run_probe(args.binary.resolve(), payload, args.timeout)
        report.update(summarize(raw, payload))
    except (BenchmarkError, ValueError, KeyError, OSError, subprocess.TimeoutExpired) as exc:
        report.update(status='failed', qualification='notEvaluated', error=str(exc))
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2, allow_nan=False)+'\n', encoding='utf-8')
    print(report['status'], args.output)
    return 1 if report['status']=='failed' else 0


if __name__ == '__main__':
    raise SystemExit(main())

#!/usr/bin/env python3
"""Offline installed-wheel measurements for fixed history and external feedback.

Run each case in a fresh process. This probe measures and verifies outputs; it
does not evaluate latency/memory budgets or qualify other runtime surfaces.
"""
from __future__ import annotations

import argparse
import ctypes
import gc
import hashlib
import importlib.metadata
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
import time

from benchmark_modern_strategy import stats_ms, synthetic_bars
from resource_json import dump_public_json

ROOT = Path(__file__).resolve().parents[1]
HISTORY_SOURCE = '''//@version=6
indicator("Fixed history resources")
var n = 0
n += 1
plot(n)
plot(close)
plot(ta.sma(close, 14))
plot(last_bar_index)
plot(barstate.islast ? 1 : 0)
'''
EXTERNAL_SOURCE = '''//@version=6
strategy("External feedback resources")
if strategy.position_size == 0
    strategy.entry("L", strategy.long, qty=2)
else
    strategy.close("L")
plot(strategy.position_size)
plot(strategy.position_size[1])
plot(strategy.equity)
plot(close)
plot(ta.sma(close, 14))
'''


def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest() if sys.version_info >= (3, 11) else file_sha(stream)


def file_sha(stream):
    digest = hashlib.sha256()
    for chunk in iter(lambda: stream.read(1024 * 1024), b''):
        digest.update(chunk)
    return digest.hexdigest()


def json_sha(value):
    digest = hashlib.sha256()
    for chunk in json.JSONEncoder(sort_keys=True, separators=(',', ':'), allow_nan=False).iterencode(value):
        digest.update(chunk.encode())
    return digest.hexdigest()


def write(path, value):
    with Path(path).open('w', encoding='utf-8', newline='\n') as stream:
        dump_public_json(value, stream)
        stream.write('\n')


def memory(phase):
    if os.name == 'nt':
        from ctypes import wintypes

        class Counters(ctypes.Structure):
            _fields_ = [('cb', wintypes.DWORD), ('PageFaultCount', wintypes.DWORD)] + [
                (name, ctypes.c_size_t) for name in (
                    'PeakWorkingSetSize', 'WorkingSetSize', 'QuotaPeakPagedPoolUsage',
                    'QuotaPagedPoolUsage', 'QuotaPeakNonPagedPoolUsage',
                    'QuotaNonPagedPoolUsage', 'PagefileUsage', 'PeakPagefileUsage', 'PrivateUsage')]

        kernel = ctypes.WinDLL('kernel32', use_last_error=True)
        kernel.GetCurrentProcess.restype = wintypes.HANDLE
        psapi = ctypes.WinDLL('psapi', use_last_error=True)
        psapi.GetProcessMemoryInfo.argtypes = [wintypes.HANDLE, ctypes.POINTER(Counters), wintypes.DWORD]
        psapi.GetProcessMemoryInfo.restype = wintypes.BOOL
        counters = Counters()
        counters.cb = ctypes.sizeof(counters)
        if not psapi.GetProcessMemoryInfo(kernel.GetCurrentProcess(), ctypes.byref(counters), counters.cb):
            raise ctypes.WinError(ctypes.get_last_error())
        return dict(phase=phase, rssBytes=counters.WorkingSetSize, peakRssBytes=counters.PeakWorkingSetSize,
                    privateBytes=counters.PrivateUsage, source='Windows process working set/private usage')
    if sys.platform.startswith('linux'):
        fields = dict(line.split(':', 1) for line in Path('/proc/self/status').read_text().splitlines() if ':' in line)
        return dict(phase=phase, rssBytes=int(fields['VmRSS'].split()[0]) * 1024,
                    peakRssBytes=int(fields['VmHWM'].split()[0]) * 1024,
                    privateBytes=None, source='Linux process VmRSS/VmHWM')
    raise ValueError('resource measurement requires Windows or Linux')


def timed(metrics, phase, fn):
    start = time.perf_counter_ns()
    result = fn()
    metrics.setdefault(phase, []).append((time.perf_counter_ns() - start) / 1e6)
    return result


def artifact_identity():
    import pine_compat
    distribution = importlib.metadata.distribution('pine-compat-runtime')
    native = [Path(distribution.locate_file(path)).resolve() for path in distribution.files
              if str(path).endswith(('.pyd', '.so'))]
    if not native:
        raise ValueError('installed distribution has no native extension')
    module = Path(pine_compat.__file__).resolve()
    installed = {Path(distribution.locate_file(path)).resolve() for path in distribution.files}
    if module not in installed or distribution.version != pine_compat.__version__:
        raise ValueError('imported module disagrees with installed distribution')
    return dict(version=pine_compat.__version__, module=str(module), python=sys.version,
                executable=sys.executable, native=[dict(path=str(path), sha256=sha(path)) for path in native])


def inputs(total, stream):
    bars = synthetic_bars(total, seed=1 + stream * 13)
    accounts = [dict(time=bar['time'], position_size=0 if i % 3 == 0 else 2,
                     position_avg_price=None if i % 3 == 0 else bar['close'],
                     equity=1000 + stream * 100 + i + stream, initial_capital=1000 + stream * 100,
                     netprofit=float(i), openprofit=float(stream)) for i, bar in enumerate(bars)]
    return bars, accounts


def require(condition, message):
    if not condition:
        raise ValueError(message)


def verify_external(result, bars, accounts):
    plots = result['output']['plots']
    require(result['protocol'] == 'external-broker/1' and not result['output'].get('strategy'), 'native ledger exposed')
    positions = [frame['position_size'] for frame in accounts]
    require(plots[0]['values'] == positions, 'feedback position mismatch')
    require(plots[1]['values'] == [None] + positions[:-1], 'feedback history mismatch')
    require(plots[2]['values'] == [frame['equity'] for frame in accounts], 'feedback equity mismatch')
    require(plots[3]['values'] == [bar['close'] for bar in bars], 'chart mismatch')
    intents = result['intents']
    require(len(intents) == len(bars), 'order intent count mismatch')
    require(all(row['action'] == ('entry' if frame['position_size'] == 0 else 'close')
                for row, frame in zip(intents, accounts)), 'order intent action mismatch')
    require(all(row['bar_index'] == i and (row['action'] != 'entry' or row['qty'] == 2)
                for i, row in enumerate(intents)), 'order intent context mismatch')


def worker(mode, history, tail, sessions, chunk, output):
    import pine_compat
    total = history + tail
    source = HISTORY_SOURCE if mode == 'history' else EXTERNAL_SOURCE
    metrics, checkpoints = {}, [memory('imported')]
    program = timed(metrics, 'compile', lambda: pine_compat.compile_script(source))
    payloads = [inputs(total, stream) for stream in range(sessions)]
    input_hashes = [json_sha(dict(bars=bars, accounts=accounts if mode == 'external' else None))
                    for bars, accounts in payloads]
    checkpoints.append(memory('inputAndProgram'))
    hashes, checks = [], {}
    if mode == 'history':
        active = [timed(metrics, 'create', lambda bars=bars: program.historical_session(bars))
                  for bars, _ in payloads]
        for session in active:
            timed(metrics, 'seedWithOutput', lambda: session.advance(history))
        checkpoints.append(memory('seeded'))
        saved = timed(metrics, 'fork', active[0].fork)
        saved_output = saved.advance(history)
        saved_hash = json_sha(saved_output)
        checkpoints.append(memory('forkHeld'))
        for target in range(history + chunk, total + chunk, chunk):
            target = min(target, total)
            for session in active:
                timed(metrics, 'advanceWithOutput', lambda: session.advance(target))
            require(saved.cursor == history, 'fork cursor changed with parent')
        checkpoints.append(memory('tailAdvanced'))
        for index, (session, (bars, _)) in enumerate(zip(active, payloads)):
            actual = timed(metrics, 'finalSnapshot', lambda: session.advance(total))
            expected = timed(metrics, 'batchControl', lambda: program.run(bars))
            require(actual == expected, 'fixed-history endpoint differs from complete batch')
            require(session.cursor == total, 'incomplete historical cursor')
            values = actual['plots']
            require(values[0]['values'] == list(range(1, total + 1)), 'persistent counter mismatch')
            require(values[1]['values'] == [bar['close'] for bar in bars], 'independent chart mismatch')
            require(values[3]['values'] == [total - 1] * total, 'fixed horizon mismatch')
            require(values[4]['values'] == [0] * (total - 1) + [1], 'endpoint flag mismatch')
            if index == 0:
                require(saved.advance(total) == expected, 'saved fork differs from batch')
                require(json_sha(saved_output) == saved_hash, 'previous Python output mutated')
            path = output.with_name(output.stem + f'-stream{index}.output.json')
            timed(metrics, 'serialization', lambda: write(path, actual))
            hashes.append(sha(path))
            del actual, expected
        checks.update(batchEqualsAdvance=True, independentInputs=True, forkIndependent=True, priorOutputImmutable=True)
        del active, saved, saved_output, session, values, bars, _
    else:
        # This public API is batch-only; repeated calls do not imply incremental
        # external sessions or input retention inside the compiled Program.
        for index, (bars, accounts) in enumerate(payloads):
            prefix = timed(metrics, 'prefixBatch', lambda: program.run_external(bars[:history], accounts[:history]))
            verify_external(prefix, bars[:history], accounts[:history])
            del prefix
            actual = timed(metrics, 'completeBatch', lambda: program.run_external(bars, accounts))
            verify_external(actual, bars, accounts)
            checkpoints.append(memory(f'completeBatch{index}'))
            repeated = timed(metrics, 'repeatBatch', lambda: program.run_external(bars, accounts))
            require(actual == repeated, 'reusing compiled Program changed external result')
            del repeated
            path = output.with_name(output.stem + f'-stream{index}.output.json')
            timed(metrics, 'serialization', lambda: write(path, actual))
            hashes.append(sha(path))
            del actual
        checks.update(feedbackOracle=True, completeIntents=True, repeatedProgramIsolation=True, nativeLedgerAbsent=True)
        del bars, accounts
    checkpoints.append(memory('verified'))
    del program, payloads
    gc.collect()
    checkpoints.append(memory('objectsReleased'))
    return dict(status='passed', qualification='notEvaluated', mode=mode, historyBars=history,
                tailBars=tail, sessions=sessions, chunkBars=chunk, checks=checks,
                sourceSha256=hashlib.sha256(source.encode()).hexdigest(), inputSha256=input_hashes,
                outputSha256=hashes, phases={key: stats_ms(value) for key, value in metrics.items()},
                memoryCheckpoints=checkpoints, artifact=artifact_identity())


def validate(row):
    require(row['status'] == 'passed' and row['qualification'] == 'notEvaluated', 'failed or overclaimed trial')
    require(row['mode'] in ('history', 'external'), 'unknown interface mode')
    require(all(type(row[key]) is int and row[key] > 0
                for key in ['sessions', 'historyBars', 'tailBars', 'chunkBars']), 'invalid trial dimensions')
    count, phases = row['sessions'], row['phases']
    require(len(row['inputSha256']) == count and len(row['outputSha256']) == count, 'missing complete stream')
    required_checks = ({'batchEqualsAdvance', 'independentInputs', 'forkIndependent', 'priorOutputImmutable'}
                       if row['mode'] == 'history' else
                       {'feedbackOracle', 'completeIntents', 'repeatedProgramIsolation', 'nativeLedgerAbsent'})
    require(set(row['checks']) == required_checks and all(value is True for value in row['checks'].values()),
            'missing or failed correctness check')
    expected = dict(compile=1, serialization=count)
    if row['mode'] == 'history':
        expected.update(create=count, seedWithOutput=count, fork=1, finalSnapshot=count, batchControl=count,
                        advanceWithOutput=((row['tailBars'] + row['chunkBars'] - 1) // row['chunkBars']) * count)
    else:
        expected.update(prefixBatch=count, completeBatch=count, repeatBatch=count)
    require(set(phases) == set(expected), 'missing or unexpected measured phase')
    for name, operations in expected.items():
        statistics = stats_ms(phases[name]['samplesMs'])
        require(phases[name] == statistics and statistics['n'] == operations, f'incomplete phase: {name}')
    require({'imported', 'inputAndProgram', 'verified', 'objectsReleased'} <=
            {point['phase'] for point in row['memoryCheckpoints']}, 'missing memory checkpoints')
    require(all(point['rssBytes'] > 0 and point['peakRssBytes'] >= point['rssBytes']
                for point in row['memoryCheckpoints']), 'invalid process memory counters')


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--history-bars', type=int, nargs='+', default=[1024, 100000])
    parser.add_argument('--tail-bars', type=int, default=10000)
    parser.add_argument('--sessions', type=int, nargs='+', default=[1, 4])
    parser.add_argument('--chunk-bars', type=int, default=1000)
    parser.add_argument('--repetitions', type=int, default=2)
    parser.add_argument('--timeout', type=float, default=600)
    parser.add_argument('--worker', choices=['history', 'external'])
    args = parser.parse_args(argv)
    if any(n <= 0 for n in [*args.history_bars, *args.sessions, args.tail_bars, args.chunk_bars, args.repetitions, args.timeout]):
        parser.error('counts and timeout must be positive')
    if len(set(args.history_bars)) != len(args.history_bars) or len(set(args.sessions)) != len(args.sessions):
        parser.error('duplicate case dimensions')
    if args.worker and (len(args.history_bars) != 1 or len(args.sessions) != 1):
        parser.error('workers require one history size and one session count')
    if args.worker:
        try:
            row = worker(args.worker, args.history_bars[0], args.tail_bars, args.sessions[0], args.chunk_bars, args.output)
            validate(row)
        except Exception as error:
            write(args.output, dict(status='failed', error=f'{type(error).__name__}: {error}'))
            raise
        write(args.output, row)
        return 0
    args.output.mkdir(parents=True, exist_ok=False)
    artifact = artifact_identity()
    plan = dict(historyBars=args.history_bars, tailBars=args.tail_bars, sessions=args.sessions,
                chunkBars=args.chunk_bars, repetitions=args.repetitions, timeoutSeconds=args.timeout,
                modes=['history', 'external'], artifact=artifact,
                gitCommit=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                toolSha256=sha(__file__), supportSha256={name: sha(ROOT / 'scripts' / name)
                    for name in ['benchmark_modern_strategy.py', 'resource_json.py']},
                workloadSources={'history': HISTORY_SOURCE, 'external': EXTERNAL_SOURCE})
    write(args.output / 'plan.json', plan)
    report = dict(qualification='notEvaluated', correctness='passed', platform=platform.platform(),
                  planSha256=sha(args.output / 'plan.json'), rows=[],
                  scope='installed Python only; synthetic inputs; history advance returns full outputs; external API is batch-only')
    previous = {}
    for history in args.history_bars:
        for sessions in args.sessions:
            for mode in plan['modes']:
                for repeat in range(args.repetitions):
                    label = f'{mode}-{history}-{sessions}-r{repeat}'
                    output = args.output / f'{label}.json'
                    command = [sys.executable, str(Path(__file__).resolve()), '--worker', mode, '--output', str(output),
                               '--history-bars', str(history), '--tail-bars', str(args.tail_bars),
                               '--sessions', str(sessions), '--chunk-bars', str(args.chunk_bars)]
                    start = time.monotonic()
                    try:
                        with (args.output / f'{label}.log').open('w', encoding='utf-8') as log:
                            subprocess.run(command, stdout=log, stderr=subprocess.STDOUT, timeout=args.timeout, check=True)
                        row = json.loads(output.read_text(encoding='utf-8'))
                        validate(row)
                        require(row['artifact'] == artifact, 'worker artifact changed')
                        key = (mode, history, sessions)
                        fingerprint = (row['inputSha256'], row['outputSha256'])
                        require(key not in previous or previous[key] == fingerprint, 'fresh-process repeat mismatch')
                        previous[key] = fingerprint
                    except Exception as error:
                        row = dict(status='failed', error=f'{type(error).__name__}: {error}')
                        report['correctness'] = 'failed'
                    report['rows'].append(dict(id=label, wallSeconds=time.monotonic() - start,
                                               receiptSha256=sha(output) if output.exists() else None, **row))
                    write(args.output / 'report.json', report)
                    print(label, row['status'], flush=True)
    return 0 if report['correctness'] == 'passed' else 1


if __name__ == '__main__':
    raise SystemExit(main())

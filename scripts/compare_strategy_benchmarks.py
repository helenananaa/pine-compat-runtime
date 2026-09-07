#!/usr/bin/env python3
"""Paired release-probe A/B measurements with frozen correctness/resource gates."""
import argparse
import hashlib
import json
from pathlib import Path
import statistics

import benchmark_modern_strategy as bench

TARGETS = {'trend', 'dense', 'realtime', 'magnifier'}


def compare(pairs):
    if not pairs:
        raise bench.BenchmarkError('no benchmark pairs')
    largest = max(p['barCount'] for p in pairs)
    summary = []
    for name in sorted({p['sampleId'] for p in pairs}):
        rows = [p for p in pairs if p['sampleId'] == name and p['barCount'] == largest]
        a = statistics.median(p['baseline']['phases']['incrementalAppend']['medianMs'] for p in rows)
        b = statistics.median(p['candidate']['phases']['incrementalAppend']['medianMs'] for p in rows)
        if a <= 0:
            raise bench.BenchmarkError('nonpositive baseline time')
        summary.append(dict(sampleId=name, baselineMs=a, candidateMs=b, improvementPercent=(a-b)/a*100))
    correctness = all(all(p['baseline'][key] == p['candidate'][key]
                          for key in ['sourceHash', 'inputHash', 'resultHash', 'liveResultHash']) for p in pairs)
    retained_equal = True
    for p in pairs:
        for name in ['profile', 'confirmedRealtimeProfile']:
            a, b = p['baseline'][name], p['candidate'][name]
            if a is None or b is None:
                retained_equal &= a == b
            else:
                retained_equal &= {k: v for k, v in a.items() if 'capacity' not in k.lower()} == {k: v for k, v in b.items() if 'capacity' not in k.lower()}
    resource_ok = all(p['baseline']['peakRssKiB'] is not None and p['candidate']['peakRssKiB'] is not None and
                      p['candidate']['peakRssKiB']-p['baseline']['peakRssKiB'] <= max(p['baseline']['peakRssKiB']*.2, 2048) for p in pairs)
    targets = [s['improvementPercent'] for s in summary if s['sampleId'] in TARGETS]
    complete = {s['sampleId'] for s in summary} == {s['sampleId'] for s in bench.default_samples()}
    improvement = statistics.median(targets) if targets else None
    control_ok = all(s['improvementPercent'] >= -10 for s in summary if s['sampleId'] not in TARGETS)
    return dict(pairCount=len(pairs), largestScale=summary, correctness=correctness,
                retainedCountsEqual=retained_equal, completeWorkloadSet=complete,
                medianTargetImprovementPercent=improvement, controlBudgetPassed=control_ok,
                resourceBudgetPassed=resource_ok,
                accepted=bool(complete and correctness and retained_equal and resource_ok and control_ok and improvement is not None and improvement >= 5))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--baseline', type=Path, required=True)
    parser.add_argument('--candidate', type=Path, required=True)
    parser.add_argument('--output-dir', type=Path, required=True)
    parser.add_argument('--rounds', type=int, default=6)
    parser.add_argument('--bars', default='1024')
    args = parser.parse_args()
    counts = [int(x) for x in args.bars.split(',')]
    if args.rounds < 2 or any(n < 2 for n in counts) or len(set(counts)) != len(counts):
        parser.error('need >=2 rounds and distinct bar counts >=2')
    args.output_dir.mkdir(parents=True, exist_ok=True)
    binaries = {'baseline': args.baseline.resolve(), 'candidate': args.candidate.resolve()}
    pairs = []
    for round_number in range(args.rounds):
        for spec in bench.default_samples():
            for count in counts:
                bars = bench.synthetic_bars(count, seed=1)
                payload = dict(source=(bench.ROOT/spec['sourcePath']).read_text(), bars=bars,
                               warmup=2, iters=10, replacements=100,
                               magnifier=bench.magnifier_input(bars) if spec.get('magnifier') else None)
                pair = dict(round=round_number, sampleId=spec['sampleId'], barCount=count)
                for name in (['baseline', 'candidate'] if round_number % 2 == 0 else ['candidate', 'baseline']):
                    pair[name] = bench.summarize_probe(bench.run_probe(binaries[name], payload, 180), spec=spec, payload=payload)
                pairs.append(pair)
                (args.output_dir/'paired-results.json').write_text(json.dumps(pairs, indent=2, allow_nan=False)+'\n')
    report = compare(pairs)
    report['binaries'] = {name: hashlib.sha256(path.read_bytes()).hexdigest() for name, path in binaries.items()}
    report['thresholds'] = dict(minimumTargetImprovementPercent=5, maximumControlRegressionPercent=10,
                                peakRssIncrease='max(20%, 2048 KiB)')
    (args.output_dir/'comparison.json').write_text(json.dumps(report, indent=2, allow_nan=False)+'\n')
    print(json.dumps(report, indent=2))
    return 0 if report['accepted'] else 1


if __name__ == '__main__':
    raise SystemExit(main())

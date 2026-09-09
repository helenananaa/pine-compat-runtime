#!/usr/bin/env python3
"""Evaluate a sustained report against a separately frozen, hashed budget plan."""
import argparse
import hashlib
import json
import math
from pathlib import Path


def evaluate(report, plan):
    if plan.get('schemaVersion') != 1 or report.get('schemaVersion') != 1:
        raise ValueError('unsupported budget/report schema')
    if report.get('status') != 'measured':
        return dict(status='failed',failures=[f"probe did not complete: {report.get('error', report.get('status', 'missing status'))}"],timings={})
    expected = plan['expected']
    required_identity = {'historyBars','tailBars','repetitions','replacementsPerBar',
                         'sourceHash','binaryHash','payloadHash','revision','worktreeStatus','formingExecutesScript'}
    if set(expected) != required_identity:
        raise ValueError('budget must freeze the complete execution identity')
    failures = [f'identity mismatch: {key}' for key,value in expected.items() if type(report.get(key)) is not type(value) or report.get(key) != value]
    if report.get('status') != 'measured' or report.get('realtimeExclusion') is not None:
        failures.append('report did not measure the full historical/live workload')
    if any(report.get('correctness',{}).get(key) is not True for key in
           ('batchEqualsTail','repeatedHistoricalStable','repeatedLiveStable')):
        failures.append('execution consistency did not pass')
    if expected['worktreeStatus'] != '':
        raise ValueError('formal budgets require a clean committed source tree')
    repeats, tail, replacements = (expected[k] for k in ('repetitions','tailBars','replacementsPerBar'))
    if any(type(n) is not int or n < 1 for n in (repeats,tail,replacements,expected['historyBars'])) or repeats < 2:
        raise ValueError('invalid frozen execution sizes')
    counts = dict(compile=1, historySeed=repeats, tailAppend=repeats*tail,
                  resultSnapshot=repeats, outputSerialization=repeats, liveSeed=repeats,
                  formingInitial=repeats*tail, formingReplace=repeats*tail*replacements,
                  formingConfirm=repeats*tail, liveResultSnapshot=repeats, liveOutputSerialization=repeats)
    if set(plan['phaseBudgets']) != set(counts) or set(report['phases']) != set(counts):
        raise ValueError('budget and report must cover all phases')
    measurements = {}
    for phase,count in counts.items():
        values = report['phases'][phase]['samplesMs']
        if len(values) != count or report['phases'][phase]['n'] != count:
            failures.append(f'wrong operation count: {phase}')
            continue
        if any(type(v) not in (int,float) or not math.isfinite(v) or v < 0 for v in values):
            raise ValueError(f'invalid timing samples: {phase}')
        budget = plan['phaseBudgets'][phase]
        limit = budget['limitMs']
        if type(limit) not in (int,float) or not math.isfinite(limit) or limit <= 0:
            raise ValueError('budget limits must be positive and finite')
        if budget['statistic'] == 'max':
            actual = max(values)
        elif budget['statistic'] == 'p95' and count >= 100:
            actual = sorted(values)[math.ceil(count*.95)-1]
        else:
            raise ValueError('invalid or undersampled budget statistic')
        measurements[phase] = dict(actualMs=actual, **budget)
        if actual > limit:
            failures.append(f'timing budget exceeded: {phase}')
    checkpoints = report['memoryCheckpoints']
    phases = ['afterHistorySeed','afterTailAppend','afterLiveSeed','afterLiveTail']
    identities = [(phase,repeat) for repeat in range(repeats) for phase in phases] + [('afterVerification',repeats-1)]
    if [(r['phase'],r['repetition']) for r in checkpoints] != identities:
        failures.append('incomplete memory checkpoints')
    memory = plan['memoryBudgets']
    if set(memory) != {'source','peakRssKiB','peakCommitKiB'}:
        raise ValueError('freeze both Windows process-memory budgets and their source')
    if memory['source'] != 'windowsPeakWorkingSet':
        raise ValueError('this budget verifier requires Windows process-memory measurements')
    for field in ('peakRssKiB','peakCommitKiB'):
        if type(memory[field]) is not int or memory[field] <= 0:
            raise ValueError('invalid memory budget')
        for row in checkpoints:
            value = row.get(field)
            if row.get('source') != memory['source'] or type(value) is not int or value <= 0:
                failures.append(f'unavailable or incompatible memory measurement: {field}')
                break
            if value > memory[field]:
                failures.append(f'memory budget exceeded: {field}')
                break
    return dict(status='failed' if failures else 'passed', failures=failures, timings=measurements)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--plan', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    try:
        result = evaluate(json.loads(args.report.read_text(encoding='utf-8')),json.loads(args.plan.read_text(encoding='utf-8')))
    except (ValueError,KeyError,TypeError) as exc:
        result = dict(status='failed',failures=[str(exc)])
    result['planSha256'] = hashlib.sha256(args.plan.read_bytes()).hexdigest()
    result['reportSha256'] = hashlib.sha256(args.report.read_bytes()).hexdigest()
    args.output.write_text(json.dumps(result,indent=2,allow_nan=False)+'\n',encoding='utf-8')
    print(result['status'],args.output)
    return 0 if result['status']=='passed' else 1


if __name__ == '__main__':
    raise SystemExit(main())

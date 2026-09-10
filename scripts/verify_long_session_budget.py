#!/usr/bin/env python3
"""Evaluate a sustained report against a separately frozen, hashed budget plan."""
import argparse
import hashlib
import json
import math
from pathlib import Path


MAGNIFIER_EXCLUSION = 'historical-only magnifier input'


def historical_counts(repeats, tail):
    return dict(compile=1, historySeed=repeats, tailAppend=repeats*tail,
                resultSnapshot=repeats, outputSerialization=repeats)


def live_counts(repeats, tail, replacements):
    counts = historical_counts(repeats, tail)
    counts.update(liveSeed=repeats, formingInitial=repeats*tail,
                  formingReplace=repeats*tail*replacements, formingConfirm=repeats*tail,
                  liveResultSnapshot=repeats, liveOutputSerialization=repeats)
    return counts


def recompute_statistic(values, statistic):
    if statistic == 'max':
        return max(values)
    if statistic == 'p95' and len(values) >= 100:
        return sorted(values)[math.ceil(len(values) * .95) - 1]
    raise ValueError('invalid or undersampled budget statistic')


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
    planned_exclusion = plan.get('realtimeExclusion')
    if planned_exclusion not in (None, MAGNIFIER_EXCLUSION):
        raise ValueError('unsupported realtimeExclusion in budget plan')
    live_required = planned_exclusion is None
    if live_required:
        if report.get('realtimeExclusion') is not None:
            failures.append('report did not measure the full historical/live workload')
        correctness_keys = ('batchEqualsTail','repeatedHistoricalStable','repeatedLiveStable')
    else:
        if report.get('realtimeExclusion') != planned_exclusion:
            failures.append('magnifier historical-only exclusion mismatch')
        if report.get('formingExecutesScript') is not None:
            failures.append('magnifier forming execution flag must be null')
        correctness_keys = ('batchEqualsTail','repeatedHistoricalStable')
    if any(report.get('correctness',{}).get(key) is not True for key in correctness_keys):
        failures.append('execution consistency did not pass')
    if expected['worktreeStatus'] != '':
        raise ValueError('formal budgets require a clean committed source tree')
    repeats, tail, replacements = (expected[k] for k in ('repetitions','tailBars','replacementsPerBar'))
    if any(type(n) is not int or n < 1 for n in (repeats,tail,replacements,expected['historyBars'])) or repeats < 2:
        raise ValueError('invalid frozen execution sizes')
    counts = live_counts(repeats, tail, replacements) if live_required else historical_counts(repeats, tail)
    if set(plan['phaseBudgets']) != set(counts) or set(report['phases']) != set(counts):
        raise ValueError('budget and report must cover all phases')
    if plan.get('modes') is not None:
        expected_modes = ['historical', 'incremental'] + (['realtime'] if live_required else [])
        if list(plan['modes']) != expected_modes:
            raise ValueError('budget modes must match the measured historical/incremental/realtime set')
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
        actual = recompute_statistic(values, budget['statistic'])
        measurements[phase] = dict(actualMs=actual, **budget)
        if actual > limit:
            failures.append(f'timing budget exceeded: {phase}')
    checkpoints = report['memoryCheckpoints']
    phases = ['afterHistorySeed','afterTailAppend'] + (['afterLiveSeed','afterLiveTail'] if live_required else [])
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


def freeze_from_report(report, *, name, realtime_exclusion=None, headroom=2.5,
                       memory_headroom=2.0, observation_timeout=1800, sample_id=None):
    """Record numerical budgets from a completed measurement. This is not acceptance."""
    if report.get('status') != 'measured' or report.get('qualification') not in (None, 'notEvaluated'):
        raise ValueError('freeze only from a complete unevaluated measurement')
    if headroom < 1 or memory_headroom < 1 or not math.isfinite(headroom) or not math.isfinite(memory_headroom):
        raise ValueError('headroom must be a finite value >= 1')
    live_required = realtime_exclusion is None
    repeats, tail, replacements = report['repetitions'], report['tailBars'], report['replacementsPerBar']
    counts = live_counts(repeats, tail, replacements) if live_required else historical_counts(repeats, tail)
    if realtime_exclusion not in (None, MAGNIFIER_EXCLUSION):
        raise ValueError('unsupported realtimeExclusion')
    if live_required and report.get('realtimeExclusion') is not None:
        raise ValueError('cannot freeze a live budget from a report that excluded realtime')
    if not live_required and report.get('realtimeExclusion') != realtime_exclusion:
        raise ValueError('magnifier freeze requires the historical-only exclusion')
    phase_budgets = {}
    for phase, count in counts.items():
        values = report['phases'][phase]['samplesMs']
        if len(values) != count:
            raise ValueError(f'wrong operation count: {phase}')
        statistic = 'p95' if count >= 100 else 'max'
        actual = recompute_statistic(values, statistic)
        phase_budgets[phase] = dict(statistic=statistic, limitMs=actual * headroom)
    peaks = [row['peakRssKiB'] for row in report['memoryCheckpoints']] + [
        row['peakCommitKiB'] for row in report['memoryCheckpoints']]
    if any(type(v) is not int or v <= 0 for v in peaks):
        raise ValueError('freeze requires complete Windows process-memory measurements')
    rss = max(row['peakRssKiB'] for row in report['memoryCheckpoints'])
    commit = max(row['peakCommitKiB'] for row in report['memoryCheckpoints'])
    modes = ['historical', 'incremental'] + (['realtime'] if live_required else [])
    plan = dict(
        schemaVersion=1, name=name,
        expected={key: report[key] for key in
                  ('historyBars','tailBars','repetitions','replacementsPerBar','sourceHash',
                   'binaryHash','payloadHash','revision','worktreeStatus','formingExecutesScript')},
        phaseBudgets=phase_budgets,
        memoryBudgets=dict(source='windowsPeakWorkingSet',
                           peakRssKiB=max(1, math.ceil(rss * memory_headroom)),
                           peakCommitKiB=max(1, math.ceil(commit * memory_headroom))),
        modes=modes, observationTimeoutSeconds=observation_timeout,
        rationale='Frozen from a completed measurement with documented headroom; not an acceptance receipt.',
    )
    if realtime_exclusion is not None:
        plan['realtimeExclusion'] = realtime_exclusion
    if sample_id is not None:
        plan['sampleId'] = sample_id
    return plan


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

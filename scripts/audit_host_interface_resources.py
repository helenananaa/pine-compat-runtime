#!/usr/bin/env python3
"""Audit complete Python host-interface trials, outputs and wheel provenance."""
import argparse
import copy
import hashlib
import itertools
import json
import math
from pathlib import Path
import statistics
import zipfile

import benchmark_host_interfaces as probe


def read(path):
    return json.loads(Path(path).read_text(encoding='utf-8'))


def validate_cross_python(row):
    normalized = copy.deepcopy(row)
    for name, phase in row['phases'].items():
        recomputed = probe.stats_ms(phase['samplesMs'])
        # Python 3.10 and 3.12 can differ by one ULP in statistics.pstdev.
        # Only this derived value is tolerant; samples, counts and all other
        # statistics remain strict. No receipt or acceptance budget changes.
        probe.require(math.isclose(phase['stdevMs'], recomputed['stdevMs'], rel_tol=1e-12, abs_tol=1e-12),
                      'corrupt standard deviation')
        normalized['phases'][name]['stdevMs'] = recomputed['stdevMs']
    probe.validate(normalized)


def audit_platform(root, wheel):
    plan, report = read(root / 'plan.json'), read(root / 'report.json')
    probe.require(report['qualification'] == 'notEvaluated' and report['correctness'] == 'passed',
                  'failed or overclaimed matrix')
    probe.require(report['planSha256'] == probe.sha(root / 'plan.json'), 'changed plan')
    probe.require(plan['toolSha256'] == probe.sha(probe.__file__), 'changed measurement tool')
    for name, digest in plan['supportSha256'].items():
        probe.require(probe.sha(Path(probe.__file__).with_name(name)) == digest, 'changed support tool')
    expected = {f'{mode}-{history}-{sessions}-r{repeat}'
                for mode, history, sessions, repeat in itertools.product(
                    plan['modes'], plan['historyBars'], plan['sessions'], range(plan['repetitions']))}
    probe.require({row['id'] for row in report['rows']} == expected and len(report['rows']) == len(expected),
                  'incomplete or duplicated trial matrix')
    with zipfile.ZipFile(wheel) as archive:
        native = {hashlib.sha256(archive.read(name)).hexdigest()
                  for name in archive.namelist() if name.endswith(('.so', '.pyd'))}
    probe.require(bool(native), 'wheel has no native extension')
    fingerprints, groups, output_count = {}, {}, 0
    for row in report['rows']:
        validate_cross_python(row)
        receipt = root / f"{row['id']}.json"
        probe.require(probe.sha(receipt) == row['receiptSha256'], 'changed trial receipt')
        raw = read(receipt)
        probe.require(raw == {key: value for key, value in row.items()
                              if key not in {'id', 'wallSeconds', 'receiptSha256'}}, 'report differs from trial')
        probe.require(row['artifact'] == plan['artifact'] and row['artifact']['version'] == '0.3.1',
                      'trial artifact identity mismatch')
        probe.require({item['sha256'] for item in row['artifact']['native']} == native,
                      'installed extension differs from supplied wheel')
        source = plan['workloadSources'][row['mode']]
        probe.require(row['sourceSha256'] == hashlib.sha256(source.encode()).hexdigest(), 'changed workload source')
        key = (row['mode'], row['historyBars'], row['sessions'])
        fingerprint = (row['inputSha256'], row['outputSha256'])
        probe.require(key not in fingerprints or fingerprints[key] == fingerprint, 'fresh-process mismatch')
        fingerprints[key] = fingerprint
        groups.setdefault(key, []).append(row)
        for index, digest in enumerate(row['outputSha256']):
            output = root / f"{row['id']}-stream{index}.output.json"
            probe.require(probe.sha(output) == digest, 'changed complete output')
            output_count += 1
    summary = []
    for (mode, history, sessions), rows in groups.items():
        summary.append(dict(mode=mode, historyBars=history, tailBars=plan['tailBars'], streams=sessions,
                            repeats=len(rows), peakRssBytesRange=[
                                min(max(point['peakRssBytes'] for point in row['memoryCheckpoints']) for row in rows),
                                max(max(point['peakRssBytes'] for point in row['memoryCheckpoints']) for row in rows)],
                            phases={name: dict(operationsPerProcess=rows[0]['phases'][name]['n'],
                                              processMedianMs=statistics.median(row['phases'][name]['medianMs'] for row in rows))
                                    for name in rows[0]['phases']}))
    return plan, fingerprints, dict(platform=report['platform'], trials=len(report['rows']),
                                    completeOutputs=output_count, wheel=dict(name=wheel.name, sha256=probe.sha(wheel)),
                                    artifact=plan['artifact'], reportSha256=probe.sha(root / 'report.json'),
                                    planSha256=probe.sha(root / 'plan.json'), summary=summary)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--windows', type=Path, required=True)
    parser.add_argument('--linux', type=Path, required=True)
    parser.add_argument('--windows-wheel', type=Path, required=True)
    parser.add_argument('--linux-wheel', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    windows_plan, windows_outputs, windows = audit_platform(args.windows, args.windows_wheel)
    linux_plan, linux_outputs, linux = audit_platform(args.linux, args.linux_wheel)
    probe.require({key: value for key, value in windows_plan.items() if key != 'artifact'} ==
                  {key: value for key, value in linux_plan.items() if key != 'artifact'}, 'platform plans differ')
    probe.require(windows_outputs == linux_outputs, 'cross-platform complete outputs differ')
    result = dict(date='2026-10-07', coreCommit=windows_plan['gitCommit'], version='0.3.1',
                  correctness='passed', resourceQualification='notEvaluated',
                  measurementToolSha256=windows_plan['toolSha256'], auditToolSha256=probe.sha(__file__),
                  trials=windows['trials'] + linux['trials'],
                  completeOutputs=windows['completeOutputs'] + linux['completeOutputs'],
                  crossPlatformOutputPairs=windows['completeOutputs'], platforms=[windows, linux],
                  limits=['No frozen latency/memory acceptance budgets for these new API workloads.',
                          'Python installed wheels only; no Rust/CLI/WASM resource matrix.',
                          'Synthetic indicator/account feedback; no native live-tick/reference qualification.',
                          f"History advances {windows_plan['chunkBars']} bars per call and materializes a full output each time.",
                          'External execution is sequential batch reuse; four streams are not four persistent sessions.',
                          'Process memory includes Python inputs, outputs, verification and allocator retention.',
                          'Finite two-process repeats do not establish indefinite resource bounds or leak freedom.'])
    probe.write(args.output, result)
    print(f"audited {result['trials']} trials, {result['completeOutputs']} complete outputs; cross-platform hashes agree")


if __name__ == '__main__':
    main()

"""Audit frozen replay receipts and independent state expectations."""
import argparse
import math
from pathlib import Path

from requalify_core_scripts import read, sha, write

REPO = Path(__file__).resolve().parents[1]


def audit(root, artifacts):
    plan = read(root / 'plan.json')
    result = read(root / 'results.json')
    assert result['completed'] and not result['failures']
    for path, digest in result['sourceHashes'].items():
        assert sha(REPO / path) == digest, path
    for path, digest in result['toolHashes'].items():
        assert sha(REPO / path) == digest, path
    for path, digest in result['artifactHashes'].items():
        assert sha(artifacts / path) == digest, path
    assert sha(root / 'core.patch') == result['corePatchSha256']
    oracles = []
    for case, row in zip(plan['cases'], result['cases']):
        assert case['id'] == row['id'] and row['status'] == 'passed'
        assert sha(root / case['payload']) == case['sha256']
        assert len(row['outputs']) == 6
        for surface, digest in row['outputs'].items():
            assert sha(root / f'{case["id"]}-{surface}.trace.json') == digest
        if case['script'] != 'state':
            continue
        payload = read(root / case['payload'])
        trace = read(root / f'{case["id"]}-python-0.trace.json')
        counter = len(payload['bars'])
        active = False
        daily = payload['request']['OFFLINE:TEST:1D'][-1]['close']
        for event, snapshot in zip(payload['events'], trace):
            if event['kind'].startswith('request'):
                daily = event['bar']['close']
                counter += int(active)
            else:
                counter += 1
                active = event['kind'] == 'forming'
            plots = {p['title']: p['values'][-1] for p in snapshot['result']['plots']}
            assert plots['persistent'] == counter, (case['id'], counter)
            assert plots['ordinary'] == snapshot['confirmedBars'] + int(active)
            if active:
                assert math.isclose(plots['daily'], daily, abs_tol=1e-9, rel_tol=1e-12)
        oracles.append({'id': case['id'], 'checkedEvents': len(trace), 'status': 'passed'})
    assert len(oracles) == 2
    return {
        'status': 'passed', 'planSha256': sha(root / 'plan.json'),
        'resultsSha256': sha(root / 'results.json'), 'stateOracles': oracles,
        'auditToolSha256': sha(Path(__file__)),
        'evidenceHashes': {
            p.relative_to(REPO).as_posix(): sha(p)
            for p in [artifacts / 'build.ps1', artifacts / 'build-v2.log',
                      artifacts / 'build-passed.txt', artifacts / 'probe-build-v2.log',
                      artifacts / 'rust-probe/main.rs', artifacts / 'rust-probe/Cargo.lock']
        },
    }


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('root', type=Path)
    parser.add_argument('artifacts', type=Path)
    args = parser.parse_args()
    receipt = audit(args.root.resolve(), args.artifacts.resolve())
    write(args.root / 'audit.json', receipt)
    print(receipt['status'])

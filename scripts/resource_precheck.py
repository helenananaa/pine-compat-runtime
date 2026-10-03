"""Run fresh complete worker cases as a diagnostic subset, never full qualification."""
from __future__ import annotations

import argparse
import math
import os
import platform
import re
import shutil
import signal
import subprocess
import time
from pathlib import Path

from requalify_core_scripts import REPO, read, sha, write
from product_resource_acceptance import resource_metadata
from resource_report import expected_spool_names, validate_report

DEFAULT_CASES = ('rsi-default-1024-1', 'rsi-alternate-1024-1', 'pivot-original-1024-1')
SURFACES = ('rust', 'python', 'wasm')
WORKER_SOURCES = ('scripts/product_resource_acceptance.py',
                  'scripts/product_resource_probe.rs', 'scripts/product_resource_wasm.cjs',
                  'scripts/resource_json.py', 'scripts/resource_report.py',
                  'scripts/requalify_core_scripts.py', 'scripts/resource_precheck.py')


def selected_cases(plan, identifiers=None):
    identifiers = list(DEFAULT_CASES if identifiers is None else identifiers)
    if not identifiers or len(identifiers) != len(set(identifiers)):
        raise ValueError('case selection must be nonempty and unique')
    indexed = {case['id']: case for case in plan['cases']}
    if len(indexed) != len(plan['cases']):
        raise ValueError('duplicate case IDs in input plan')
    selected = []
    for identifier in identifiers:
        if not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9_-]*', identifier):
            raise ValueError(f'invalid case ID: {identifier}')
        if identifier not in indexed:
            raise ValueError(f'unknown case: {identifier}')
        case = indexed[identifier]
        for key in ('history', 'tail', 'sessions'):
            if type(case[key]) is not int or case[key] <= 0:
                raise ValueError(f'invalid {key} in {identifier}')
        selected.append(case)
    return selected


def inside(root, name):
    path = (root / name).resolve()
    if not path.is_relative_to(root.resolve()):
        raise ValueError(f'path leaves its source directory: {name}')
    return path


def worker_command(surface, artifacts, payload, sessions, output):
    if surface == 'rust':
        return [str(artifacts / ('resource-probe.exe' if os.name == 'nt' else 'resource-probe')),
                str(payload), str(sessions), str(output)]
    if surface == 'python':
        return [str(artifacts / ('venv/Scripts/python.exe' if os.name == 'nt' else 'venv/bin/python')),
                str(REPO / 'scripts/product_resource_acceptance.py'), '--python-worker', str(payload),
                '--sessions', str(sessions), '--output', str(output)]
    if surface == 'wasm':
        return ['node', str(REPO / 'scripts/product_resource_wasm.cjs'),
                str(artifacts / 'wasm/pine_wasm.js'), str(payload), str(sessions), str(output)]
    raise ValueError(f'unknown worker surface: {surface}')


def current_identity(artifacts, surfaces):
    provenance = read(artifacts / 'build-provenance.json')
    if provenance.get('profile') != 'release':
        raise ValueError('precheck requires release artifacts')
    core = provenance['coreFiles']
    if not {'Cargo.toml', 'Cargo.lock', 'crates/pine-runtime/src/lib.rs'} <= core.keys():
        raise ValueError('artifact provenance lacks core source hashes')
    for name, digest in core.items():
        if sha(inside(REPO, name)) != digest:
            raise ValueError(f'core differs from artifact source: {name}')
    paths = []
    if 'rust' in surfaces:
        if provenance.get('resourceProbeSourceSha256') != sha(REPO / 'scripts/product_resource_probe.rs'):
            raise ValueError('native worker source changed; rebuild the resource probe')
        paths.append(artifacts / ('resource-probe.exe' if os.name == 'nt' else 'resource-probe'))
    if 'wasm' in surfaces:
        paths.extend(artifacts / name for name in ('wasm/pine_wasm.js', 'wasm/pine_wasm_bg.wasm'))
    if 'python' in surfaces:
        python = artifacts / ('venv/Scripts/python.exe' if os.name == 'nt' else 'venv/bin/python')
        if not python.is_file():
            raise ValueError('installed artifact Python environment is missing')
        wheels = sorted((artifacts / 'wheels').glob('*.whl'))
        if len(wheels) != 1:
            raise ValueError('expected exactly one built Python wheel')
        paths.extend(wheels)
        # Pin the actually installed module as well as the wheel archive.
        query = 'import pathlib,pine_compat; print(pathlib.Path(pine_compat.__file__).resolve())'
        installed = Path(subprocess.check_output([str(python), '-c', query], cwd=REPO,
                                                text=True, timeout=30).strip()).resolve()
        if not installed.is_relative_to((artifacts / 'venv').resolve()):
            raise ValueError('pine_compat was imported outside the artifact environment')
        paths.append(installed)
        paths.extend(sorted(installed.parent.glob('*.pyd')))
        paths.extend(sorted(installed.parent.glob('*.so')))
    return dict(buildProvenanceSha256=sha(artifacts / 'build-provenance.json'),
                sourceCommit=provenance.get('sourceCommit'), coreFiles=core,
                artifactHashes={p.relative_to(artifacts).as_posix(): sha(p) for p in paths},
                workerSourceHashes={name: sha(REPO / name) for name in WORKER_SOURCES})


def expected_counts(payload, case, surface):
    sessions = case['sessions']
    if len(payload['bars']) != case['history'] or len(payload['tail']) != case['tail']:
        raise ValueError('payload lengths differ from frozen case')
    events = payload['events']
    if sum(event['kind'] == 'confirmed' for event in events) != case['tail']:
        raise ValueError('payload confirmation count differs from frozen tail')
    counts = {phase: sum(event['phase'] == phase for event in events) * sessions
              for phase in ('forming', 'replacement', 'confirmation', 'request')}
    counts.update(compile=1, seed=sessions, snapshot=sessions, serialization=sessions)
    counts['replica'] = (case['tail'] * 3 +
                         (counts['request'] // sessions - case['tail']
                          if case['script'].startswith('pivot') else 0)) * sessions
    if surface == 'rust':
        counts.update(append=case['tail'] * sessions, sameContextAppend=case['tail'] * sessions)
    return counts


def execute_worker(command, progress, timeout):
    with progress.open('wb') as log:
        child = subprocess.Popen(command, cwd=REPO, stdout=log, stderr=log,
                                 start_new_session=os.name != 'nt')
        try:
            return child.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            if os.name == 'nt':
                subprocess.run(['taskkill', '/PID', str(child.pid), '/T', '/F'],
                               stdout=log, stderr=log, check=False)
            else:
                try:
                    os.killpg(child.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
            if child.poll() is None:
                child.kill()
            child.wait()
            raise TimeoutError(f'worker exceeded diagnostic timeout of {timeout} seconds')


def precheck(input_root, artifacts, output_root, *, identifiers=None, surfaces=('rust',),
             repetitions=1, timeout=None):
    input_root, artifacts, output_root = (Path(p).resolve() for p in (input_root, artifacts, output_root))
    if type(repetitions) is not int or repetitions < 1:
        raise ValueError('repetitions must be positive')
    if not surfaces or len(set(surfaces)) != len(surfaces) or any(s not in SURFACES for s in surfaces):
        raise ValueError('surfaces must be a nonempty unique selection')
    plan_path = input_root / 'plan.json'
    plan = read(plan_path)
    frozen = REPO / 'docs/PRODUCT_STANDARD_CANDLE_RESOURCE_PLAN.json'
    if plan['budgetSha256'] != sha(frozen) or plan['budgets'] != read(frozen):
        raise ValueError('input plan differs from the frozen resource budget')
    cases = selected_cases(plan, identifiers)
    timeout = plan.get('processTimeoutSeconds', 600) if timeout is None else timeout
    if not isinstance(timeout, (int, float)) or isinstance(timeout, bool) or not math.isfinite(timeout) or timeout <= 0:
        raise ValueError('timeout must be finite and positive')
    identity = current_identity(artifacts, surfaces)
    for case in cases:
        if sha(inside(input_root, case['payload'])) != case['payloadSha256']:
            raise ValueError(f'frozen payload changed: {case["id"]}')
    output_root.mkdir(parents=True, exist_ok=False)
    inputs = output_root / 'inputs'
    inputs.mkdir()
    report = dict(schemaVersion=1, diagnosticPrecheck=True, fullMatrix=False,
                  qualification='notEvaluated', precheckStatus='running',
                  scope='Fresh selected workers; complete inputs, outputs and worker controls; no full-matrix qualification',
                  inputPlan=dict(path=str(plan_path), sha256=sha(plan_path)),
                  budgetSha256=sha(frozen), selectedCases=cases, surfaces=list(surfaces),
                  repetitions=repetitions, processTimeoutSeconds=timeout,
                  artifacts=str(artifacts), identity=identity, platform=platform.platform(),
                  rows=[], failures=[])
    write(output_root / 'precheck-plan.json', report)
    for case in cases:
        payload = inputs / Path(case['payload']).name
        if not payload.exists():
            shutil.copyfile(inside(input_root, case['payload']), payload)
        if sha(payload) != case['payloadSha256']:
            raise ValueError('copied payload collision or changed content')
        counts_by_surface = {surface: expected_counts(read(payload), case, surface) for surface in surfaces}
        for surface in surfaces:
            destination = output_root / surface
            destination.mkdir(exist_ok=True)
            for repeat in range(repetitions):
                tag = f'{case["id"]}-{repeat}'
                output = destination / (tag + '.json')
                progress = destination / (tag + '.progress.log')
                command = worker_command(surface, artifacts, payload, case['sessions'], output)
                row = dict(case=case['id'], surface=surface, repeat=repeat, status='failed', command=command)
                report['rows'].append(row)
                start = time.monotonic()
                try:
                    code = execute_worker(command, progress, timeout)
                    row['exitCode'] = code
                    if code:
                        raise RuntimeError(f'worker exited {code}; see {progress.name}')
                    validate_report(output, case['sessions'], case['history'] + case['tail'],
                                    require_historical=surface == 'rust', seed_count=case['history'])
                    metadata = resource_metadata(output, case['sessions'], case['history'] + case['tail'])
                    for phase, count in counts_by_surface[surface].items():
                        samples = metadata['metrics'].get(phase, [])
                        if not isinstance(samples, list) or len(samples) != count:
                            raise ValueError(f'{phase} sample count differs from complete payload: expected {count}')
                    parts = output.with_name(output.name + '.parts')
                    spool = {name: sha(parts / name)
                             for name in expected_spool_names(case['sessions'], surface == 'rust')}
                    row.update(status='completed', reportPath=output.relative_to(output_root).as_posix(),
                               reportSha256=sha(output), metadataSha256=sha(output.with_name(output.name + '.metadata.json')),
                               spoolOutputSha256=spool, sampleCounts=counts_by_surface[surface],
                               overheadMetrics=metadata.get('overheadMetrics', {}))
                except Exception as error:
                    row['error'] = str(error) or type(error).__name__
                    report['failures'].append(dict(case=case['id'], surface=surface, repeat=repeat, error=row['error']))
                row['wallSeconds'] = time.monotonic() - start
                if progress.exists():
                    row['progressSha256'] = sha(progress)
                write(output_root / 'precheck-results.json', report)
                print(surface, tag, row['status'], row.get('error', ''), flush=True)
    try:
        if identity != current_identity(artifacts, surfaces):
            raise ValueError('artifacts or source changed during precheck')
        if sha(plan_path) != report['inputPlan']['sha256']:
            raise ValueError('input plan changed during precheck')
        for case in cases:
            if sha(inputs / Path(case['payload']).name) != case['payloadSha256']:
                raise ValueError('payload changed during precheck')
    except Exception as error:
        report['failures'].append(dict(error=str(error) or type(error).__name__))
    report['precheckStatus'] = 'failed' if report['failures'] else 'completed'
    write(output_root / 'precheck-results.json', report)
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--input-root', type=Path, required=True)
    parser.add_argument('--artifacts', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True, help='Fresh output directory; never overwrites prior receipts')
    parser.add_argument('--case', dest='identifiers', action='append', help='Exact frozen case ID; repeat to select multiple')
    parser.add_argument('--surface', action='append', choices=SURFACES, help='Repeat to select multiple; default rust')
    parser.add_argument('--repetitions', type=int, default=1)
    parser.add_argument('--timeout', type=float, help='Diagnostic worker timeout, without modifying frozen budgets')
    args = parser.parse_args()
    result = precheck(args.input_root, args.artifacts, args.output, identifiers=args.identifiers,
                      surfaces=args.surface or ['rust'], repetitions=args.repetitions, timeout=args.timeout)
    raise SystemExit(1 if result['failures'] else 0)


if __name__ == '__main__':
    main()

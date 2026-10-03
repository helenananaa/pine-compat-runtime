import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import resource_precheck as precheck
from requalify_core_scripts import read, sha, write


FAKE_WORKER = r'''
import json, pathlib, sys, time
payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
count = int(sys.argv[2]); output = pathlib.Path(sys.argv[3])
mode = payload.get('testMode')
if mode == 'timeout': time.sleep(10)
parts = output.with_name(output.name + '.parts'); parts.mkdir()
for index in range(count):
    for kind in ['live', 'replica', 'batch', 'incremental', 'same-context']:
        (parts / f'{kind}-{index}.json').write_text(json.dumps({'complete': [1, 2, 3], 'stream': index}))
if mode == 'missingControl': (parts / 'batch-0.json').unlink()
metrics = {key: [0.1] * number for key, number in {'compile': 1, 'seed': count,
    'forming': count, 'replacement': count, 'confirmation': count, 'replica': 3 * count,
    'snapshot': count, 'serialization': count, 'append': count, 'sameContextAppend': count}.items()}
if mode == 'badCount': metrics['confirmation'] = []
if mode == 'nonfinite': metrics['forming'] = [float('inf')]
metadata = dict(resourceMetadataVersion=1, resultCount=count, confirmedBars=3, metrics=metrics,
    historicalAppendMatches=False, historicalSameContextMatches=True, overheadMetrics={'reportAssembly': [0.01]})
output.with_name(output.name + '.metadata.json').write_text(json.dumps(metadata))
prefix = output.name + '.parts/'
report = dict(resourceReportVersion=2, metadata={'path': output.name + '.metadata.json'},
    confirmedBars=3, results=[{'path': prefix + f'live-{i}.json'} for i in range(count)],
    historicalAppendMatches=False, historicalSameContextMatches=True,
    historicalContexts=[dict(stream=i, matches=False, sameContextMatches=True, batchDatasetEnd=2,
        initialSeedDatasetEnd=1, batch={'path': prefix + f'batch-{i}.json'},
        incremental={'path': prefix + f'incremental-{i}.json'},
        sameContextIncremental={'path': prefix + f'same-context-{i}.json'}) for i in range(count)])
output.write_text(json.dumps(report))
if mode == 'mutateArtifact': pathlib.Path(sys.argv[4]).write_text('changed artifact')
'''


class ResourcePrecheckTests(unittest.TestCase):
    def fixture(self, directory, mode=None):
        repo = Path(directory) / 'repo'; repo.mkdir()
        for name in precheck.WORKER_SOURCES:
            path = repo / name; path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text('# collector fixture\n')
        core_names = ['Cargo.toml', 'Cargo.lock', 'crates/pine-runtime/src/lib.rs']
        for name in core_names:
            path = repo / name; path.parent.mkdir(parents=True, exist_ok=True); path.write_text('fixture')
        frozen = repo / 'docs/PRODUCT_STANDARD_CANDLE_RESOURCE_PLAN.json'
        frozen.parent.mkdir(); write(frozen, {'repetitions': 2, 'budgets': {'appendP95Ms': 5}})
        inputs = Path(directory) / 'input'; inputs.mkdir()
        payload = dict(bars=[{}, {}], tail=[{}], events=[
            dict(kind='forming', phase='forming'), dict(kind='forming', phase='replacement'),
            dict(kind='confirmed', phase='confirmation')], testMode=mode)
        write(inputs / 'rsi.json', payload)
        case = dict(id='rsi-test-2-1', script='rsi-test', history=2, tail=1, sessions=1,
                    payload='rsi.json', payloadSha256=sha(inputs / 'rsi.json'))
        write(inputs / 'plan.json', dict(budgets=read(frozen), budgetSha256=sha(frozen),
                                        cases=[case], repetitions=2, processTimeoutSeconds=30))
        artifacts = Path(directory) / 'artifacts'; artifacts.mkdir()
        binary = artifacts / ('resource-probe.exe' if precheck.os.name == 'nt' else 'resource-probe')
        binary.write_text('fake native artifact')
        write(artifacts / 'build-provenance.json', dict(profile='release', sourceCommit='fixture',
            coreFiles={name: sha(repo / name) for name in core_names},
            resourceProbeSourceSha256=sha(repo / 'scripts/product_resource_probe.rs')))
        worker = Path(directory) / 'fake_worker.py'; worker.write_text(FAKE_WORKER)
        def command(surface, artifacts, payload, sessions, output):
            return [sys.executable, str(worker), str(payload), str(sessions), str(output), str(binary)]
        return repo, inputs, artifacts, command

    def test_default_selection_is_three_short_single_session_cases(self):
        cases = [dict(id=name, history=1024, tail=128, sessions=1) for name in precheck.DEFAULT_CASES]
        cases.append(dict(id='rsi-default-100000-4', history=100000, tail=10000, sessions=4))
        self.assertEqual([c['id'] for c in precheck.selected_cases({'cases': cases})], list(precheck.DEFAULT_CASES))
        for names in [[], [precheck.DEFAULT_CASES[0]] * 2, ['unknown'], ['../escape']]:
            with self.assertRaises(ValueError): precheck.selected_cases({'cases': cases}, names)

    def test_runs_fresh_workers_retains_complete_payloads_controls_and_diagnostic_scope(self):
        with tempfile.TemporaryDirectory() as directory:
            repo, inputs, artifacts, command = self.fixture(directory)
            output = Path(directory) / 'precheck'
            with patch.object(precheck, 'REPO', repo), patch.object(precheck, 'worker_command', command):
                report = precheck.precheck(inputs, artifacts, output, identifiers=['rsi-test-2-1'], repetitions=2)
                with self.assertRaises(FileExistsError):
                    precheck.precheck(inputs, artifacts, output, identifiers=['rsi-test-2-1'])
            self.assertEqual(report['precheckStatus'], 'completed')
            self.assertTrue(report['diagnosticPrecheck'])
            self.assertFalse(report['fullMatrix'])
            self.assertEqual(report['qualification'], 'notEvaluated')
            self.assertEqual(len(report['rows']), 2)
            self.assertEqual(sha(output / 'inputs/rsi.json'), sha(inputs / 'rsi.json'))
            for row in report['rows']:
                self.assertEqual(row['sampleCounts']['replica'], 3)
                self.assertEqual(len(row['spoolOutputSha256']), 5)
                manifest = read(output / row['reportPath'])
                self.assertEqual(manifest['results'][0], {'path': Path(row['reportPath']).name + '.parts/live-0.json'})

    def test_rejects_changed_payload_or_core_before_starting_a_worker(self):
        for changed in ('payload', 'core'):
            with self.subTest(changed=changed), tempfile.TemporaryDirectory() as directory:
                repo, inputs, artifacts, command = self.fixture(directory)
                output = Path(directory) / 'precheck'
                target = inputs / 'rsi.json' if changed == 'payload' else repo / 'Cargo.toml'
                target.write_text('changed')
                with patch.object(precheck, 'REPO', repo), patch.object(precheck, 'execute_worker') as execute:
                    with self.assertRaises(ValueError):
                        precheck.precheck(inputs, artifacts, output, identifiers=['rsi-test-2-1'])
                    execute.assert_not_called()
                self.assertFalse(output.exists())

    def test_failed_workers_and_missing_controls_never_become_completed_or_full_qualification(self):
        for mode in ('missingControl', 'badCount', 'nonfinite', 'mutateArtifact', 'timeout'):
            with self.subTest(mode=mode), tempfile.TemporaryDirectory() as directory:
                repo, inputs, artifacts, command = self.fixture(directory, mode)
                output = Path(directory) / 'precheck'
                with patch.object(precheck, 'REPO', repo), patch.object(precheck, 'worker_command', command):
                    report = precheck.precheck(inputs, artifacts, output, identifiers=['rsi-test-2-1'],
                                              timeout=0.1 if mode == 'timeout' else 30)
                self.assertEqual(report['precheckStatus'], 'failed')
                self.assertEqual(report['qualification'], 'notEvaluated')
                self.assertTrue(report['failures'])
                self.assertTrue((output / 'precheck-results.json').is_file())
                self.assertTrue((output / 'rust/rsi-test-2-1-0.progress.log').is_file())


if __name__ == '__main__':
    unittest.main()

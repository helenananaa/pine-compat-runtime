"""Real executable tests for sustained-operation counts and failure reporting."""
import copy
import json
from pathlib import Path
import subprocess
import sys
import unittest
import tempfile
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import benchmark_modern_strategy as bench
import benchmark_long_session as sustained


class LongSessionProbeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        build = subprocess.run(['cargo','build','-p','pine-runtime','--example','long_session_benchmark',
                                '--message-format=json'], cwd=bench.ROOT, text=True, capture_output=True, check=True)
        artifacts = [json.loads(line) for line in build.stdout.splitlines() if line.startswith('{')]
        cls.binary = Path(next(a['executable'] for a in artifacts if a.get('executable')
                               and a.get('target',{}).get('name')=='long_session_benchmark'))

    def payload(self, spec=None):
        spec = spec or bench.default_samples()[0]
        bars = bench.synthetic_bars(24, seed=1)
        return dict(source=(bench.ROOT/spec['sourcePath']).read_text(), bars=bars, historyBars=16,
                    repetitions=2, replacementsPerBar=2,
                    magnifier=bench.magnifier_input(bars) if spec.get('magnifier') else None)

    def test_real_tail_operations_and_confirmations_across_all_workloads(self):
        for spec in bench.default_samples():
            with self.subTest(sample=spec['sampleId']):
                payload = self.payload(spec)
                report = sustained.summarize(bench.run_probe(self.binary, payload, 60), payload)
                self.assertEqual(report['qualification'], 'notEvaluated')
                self.assertEqual(report['phases']['tailAppend']['n'], 16)
                if not spec.get('magnifier'):
                    self.assertEqual(report['phases']['formingReplace']['n'], 32)
                    self.assertEqual(report['phases']['formingConfirm']['n'], 16)
                    self.assertEqual(report['formingExecutesScript'], spec['sampleId'] in ('collection','realtime'))
                else:
                    self.assertNotIn('formingReplace', report['phases'])
                    self.assertIsNotNone(report['realtimeExclusion'])
                if sys.platform in ('win32','linux'):
                    self.assertTrue(all(row['peakRssKiB']>0 for row in report['memoryCheckpoints']))

    def test_incomplete_or_failed_measurements_cannot_pass(self):
        payload = self.payload()
        raw = bench.run_probe(self.binary, payload, 60)
        mutations = [lambda r: r['timingsMs']['tailAppend'].pop(),
                     lambda r: r['timingsMs'].pop('formingConfirm'),
                     lambda r: r['correctness'].update(batchEqualsTail=False),
                     lambda r: r['correctness'].update(repeatedLiveStable=False),
                     lambda r: r['memoryCheckpoints'].pop(),
                     lambda r: r['memoryCheckpoints'][0].update(peakRssKiB=0)]
        for mutate in mutations:
            broken = copy.deepcopy(raw)
            mutate(broken)
            with self.assertRaises(bench.BenchmarkError):
                sustained.summarize(broken, payload)

    def test_invalid_ranges_and_timestamps_fail_before_measurement(self):
        payload = self.payload()
        for change in [dict(historyBars=0),dict(historyBars=24),dict(repetitions=1),dict(replacementsPerBar=0)]:
            with self.assertRaises(bench.BenchmarkError):
                bench.run_probe(self.binary, payload | change, 60)
        payload['bars'][1]['time'] = payload['bars'][0]['time']
        with self.assertRaisesRegex(bench.BenchmarkError, 'strictly increasing'):
            bench.run_probe(self.binary, payload, 60)

    def test_complete_library_source_can_be_supplied(self):
        payload = self.payload()
        payload['source'] = '//@version=6\nimport Test/Sample/1 as sample\nindicator("library")\nplot(sample.value())\n'
        payload['libraries'] = {'Test/Sample/1': '//@version=6\nlibrary("Sample")\nexport value() => ta.sma(close,2)\n'}
        report = sustained.summarize(bench.run_probe(self.binary,payload,60),payload)
        self.assertTrue(report['correctness']['batchEqualsTail'])

    def test_progress_and_drop_attribution_preserve_frozen_phase_counts(self):
        payload = self.payload()
        with tempfile.TemporaryDirectory() as folder:
            progress = Path(folder) / 'progress.jsonl'
            raw = sustained.run_sustained_probe(self.binary, payload, 60, progress)
            events = [json.loads(line) for line in progress.read_text().splitlines()]
        self.assertEqual(events[0]['phase'], 'compiled')
        self.assertEqual(events[-1]['phase'], 'verified')
        self.assertEqual([e['repetition'] for e in events if e['phase']=='liveTail'], [0,1])
        self.assertEqual([e['elapsedMs'] for e in events], sorted(e['elapsedMs'] for e in events))
        report = sustained.summarize(raw, payload)
        drops = report['diagnostics']['snapshotDropTimingsMs']
        self.assertEqual(len(drops['liveSeed']), 2)
        self.assertEqual(len(drops['formingReplace']), 32)
        self.assertEqual(len(report['phases']), 11)
        broken = copy.deepcopy(raw)
        broken['diagnostics']['snapshotDropTimingsMs']['formingConfirm'].pop()
        with self.assertRaises(bench.BenchmarkError):
            sustained.summarize(broken, payload)

    def test_timeout_keeps_progress_file_and_original_timeout(self):
        def timeout(*args, **kwargs):
            kwargs['stderr'].write('{"phase":"liveTail","completed":256}\n')
            kwargs['stderr'].flush()
            raise subprocess.TimeoutExpired(args[0], kwargs['timeout'])
        with tempfile.TemporaryDirectory() as folder:
            progress = Path(folder) / 'progress.jsonl'
            with patch.object(sustained.subprocess, 'run', side_effect=timeout):
                with self.assertRaises(subprocess.TimeoutExpired) as caught:
                    sustained.run_sustained_probe(self.binary, self.payload(), 17, progress)
            self.assertEqual(caught.exception.timeout, 17)
            self.assertEqual(json.loads(progress.read_text())['completed'], 256)

    def test_magnifier_fallback_cannot_be_mislabeled_as_intrabar_measurement(self):
        payload = self.payload(bench.default_samples()[-1])
        payload['magnifier'] = None
        with self.assertRaisesRegex(bench.BenchmarkError, 'fallback is not a magnifier measurement'):
            bench.run_probe(self.binary,payload,60)
        payload = self.payload()
        payload['magnifier'] = bench.magnifier_input(payload['bars'])
        with self.assertRaisesRegex(bench.BenchmarkError, 'declaration and intrabar input to agree'):
            bench.run_probe(self.binary,payload,60)


if __name__ == '__main__':
    unittest.main()

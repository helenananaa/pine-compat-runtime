"""Integration checks against the real Rust probe; build outside timed sections."""
import json
from pathlib import Path
import subprocess
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import benchmark_modern_strategy as bench


class StrategyBenchmarkProbeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        build = subprocess.run(['cargo', 'build', '-p', 'pine-runtime', '--example', 'strategy_benchmark',
                                '--message-format=json'], cwd=bench.ROOT, text=True, capture_output=True, check=True)
        artifacts = [json.loads(line) for line in build.stdout.splitlines() if line.startswith('{')]
        cls.binary = Path(next(a['executable'] for a in artifacts
                               if a.get('executable') and a.get('target', {}).get('name') == 'strategy_benchmark'))

    def test_actual_append_and_replacements_across_all_workloads(self):
        for spec in bench.default_samples():
            with self.subTest(sample=spec['sampleId']):
                bars = bench.synthetic_bars(32, seed=1)
                payload = dict(source=(bench.ROOT/spec['sourcePath']).read_text(), bars=bars,
                               warmup=0, iters=2, replacements=3,
                               magnifier=bench.magnifier_input(bars) if spec.get('magnifier') else None)
                raw = bench.run_probe(self.binary, payload, 60)
                summary = bench.summarize_probe(raw, spec=spec, payload=payload)
                self.assertEqual(summary['status'], 'measured')
                if sys.platform in ('win32', 'linux'):
                    self.assertGreater(summary['peakRssKiB'], 0)
                    if sys.platform == 'win32':
                        self.assertGreater(summary['peakCommitKiB'], 0)
                    self.assertEqual(summary['memorySource'], 'windowsPeakWorkingSet' if sys.platform=='win32' else 'linuxVmHWM')
                self.assertEqual(summary['phases']['incrementalAppend']['n'], 2)
                if not spec.get('magnifier'):
                    self.assertEqual(summary['phases']['formingReplace']['n'], 6)
                    self.assertEqual(summary['confirmedRealtimeProfile']['bars'], 32)
                else:
                    self.assertNotIn('formingReplace', summary['phases'])
                    self.assertIsNotNone(summary['realtimeExclusion'])

    def test_invalid_source_and_guardrail_fail_the_probe(self):
        sources = [('//@version=6\nstrategy("bad")\nplot(unknown_value)\n', 'analysis failed'),
                   ('//@version=6\nindicator("loop")\nwhile true\n    x = close\nplot(close)\n', 'exceeded maximum iteration')]
        for source, expected_error in sources:
            with self.subTest(source=source):
                payload = dict(source=source, bars=bench.synthetic_bars(2, seed=1), warmup=0,
                               iters=1, replacements=2, magnifier=None)
                with self.assertRaisesRegex(bench.BenchmarkError, expected_error):
                    bench.run_probe(self.binary, payload, 60)


if __name__ == '__main__':
    unittest.main()

from __future__ import annotations

import copy
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import benchmark_modern_strategy as bench


class BenchmarkModernStrategyTests(unittest.TestCase):
    def test_synthetic_inputs_are_frozen_and_lower_bars_stay_in_chart_interval(self):
        bars = bench.synthetic_bars(8, seed=3)
        self.assertEqual(bars, bench.synthetic_bars(8, seed=3))
        self.assertNotEqual(bars, bench.synthetic_bars(8, seed=4))
        groups = bench.magnifier_input(bars)['chartBars']
        for bar, group in zip(bars, groups):
            self.assertEqual(sum(b['volume'] for b in group['bars']), bar['volume'])
            self.assertTrue(all(bar['time'] <= b['time'] < bar['time'] + 60000 for b in group['bars']))

    def test_invalid_timing_cannot_be_reported_as_success_or_nan(self):
        for values in ([], [float('nan')], [float('inf')], [-1]):
            with self.assertRaises(bench.BenchmarkError):
                bench.stats_ms(values)
        self.assertNotIn('p95Ms', bench.stats_ms([1, 2, 3]))
        self.assertEqual(bench.stats_ms(list(range(100)))['p95Ms'], 94)

    def test_invalid_config_rejected_before_execution(self):
        kwargs = dict(root=bench.ROOT, binary=Path('/not-used'), bar_counts=[4], seed=1,
                      warmup=0, iters=2, replacements=2, timeout=10)
        for update in ({'iters': 0}, {'bar_counts': []}, {'bar_counts': [4, 4]},
                       {'bar_counts': [1]}, {'warmup': -1}, {'replacements': 1}, {'timeout': 0}):
            with self.assertRaises(bench.BenchmarkError):
                bench.build_report(**(kwargs | update))

    def test_mismatch_missing_phase_or_profile_cannot_pass(self):
        spec = dict(sampleId='trend', sourcePath='irrelevant')
        payload = dict(source='x', bars=[{}, {}], magnifier=None, replacements=2, iters=1)
        raw = dict(correctness=dict(batchEqualsIncremental=True, repeatedHistoricalStable=True, repeatedLiveStable=True),
                   timingsMs={k: [1] for k in ('compile', 'historicalRun', 'incrementalAppend', 'resultSnapshot',
                                              'outputSerialization', 'realtimeSeed', 'formingInitial', 'formingReplace', 'formingConfirm')},
                   profile={'bars': 2}, confirmedRealtimeProfile={'bars': 2}, result={}, liveResult={}, orderCount=1, peakRssKiB=100)
        raw['timingsMs']['formingReplace'] = [1, 1]
        for metric in ('peakRssKiB', 'peakCommitKiB'):
            for peak in [0, -1, True, 1.5, float('nan')]:
                broken = copy.deepcopy(raw)
                broken[metric] = peak
                with self.assertRaises(bench.BenchmarkError):
                    bench.summarize_probe(broken, spec=spec, payload=payload)
        for key in raw['correctness']:
            broken = copy.deepcopy(raw)
            broken['correctness'][key] = False
            with self.assertRaises(bench.BenchmarkError):
                bench.summarize_probe(broken, spec=spec, payload=payload)
        broken = copy.deepcopy(raw)
        del broken['timingsMs']['formingReplace']
        with self.assertRaises(bench.BenchmarkError):
            bench.summarize_probe(broken, spec=spec, payload=payload)
        broken = copy.deepcopy(raw)
        broken['profile']['bars'] = 0
        with self.assertRaises(bench.BenchmarkError):
            bench.summarize_probe(broken, spec=spec, payload=payload)

    def test_all_failed_samples_never_mark_baseline_complete(self):
        def fail(*args):
            raise bench.BenchmarkError('injected failure')
        report = bench.build_report(root=bench.ROOT, binary=Path(__file__), bar_counts=[4], seed=1,
                                    warmup=0, iters=1, replacements=2, timeout=10, runner=fail)
        self.assertEqual(report['status'], 'failed')
        self.assertTrue(all(s['status'] == 'failed' for s in report['samples']))

    def test_missing_rss_stays_unavailable_and_growth_keeps_units_separate(self):
        rows = [dict(sampleId='test', status='measured', barCount=n, outputBytes=n*10,
                     peakRssKiB=None, profile={'seriesValues': n*2}) for n in (4, 8)]
        delta = bench.resource_growth(rows)[0]
        self.assertIsNone(delta['peakRssKiBDelta'])
        self.assertIsNone(delta['peakCommitKiBDelta'])
        self.assertEqual(delta['outputBytesDelta'], 40)
        self.assertEqual(delta['profileDeltas']['seriesValues'], 8)


if __name__ == '__main__':
    unittest.main()

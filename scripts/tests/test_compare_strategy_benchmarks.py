import copy
from pathlib import Path
import sys
import unittest
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import compare_strategy_benchmarks as ab


class ComparisonTests(unittest.TestCase):
    def pairs(self):
        pairs = []
        for spec in ab.bench.default_samples():
            a = dict(sourceHash='source', inputHash='bars', resultHash='result', liveResultHash='live',
                     peakRssKiB=10000, profile={'values': 2, 'capacity': 4}, confirmedRealtimeProfile=None,
                     phases={'incrementalAppend': {'medianMs': 10}})
            b = copy.deepcopy(a)
            b['phases']['incrementalAppend']['medianMs'] = 8 if spec['sampleId'] in ab.TARGETS else 10
            pairs.append(dict(sampleId=spec['sampleId'], barCount=1024, baseline=a, candidate=b))
        return pairs

    def test_accepts_improvement_with_same_outputs_and_counts(self):
        p = self.pairs()
        p[0]['candidate']['profile']['capacity'] = 8
        self.assertTrue(ab.compare(p)['accepted'])

    def test_rejects_output_input_or_retention_changes(self):
        for field in ['sourceHash', 'inputHash', 'resultHash', 'liveResultHash', 'profile']:
            p = self.pairs()
            p[0]['candidate'][field] = {'values': 3} if field == 'profile' else 'different'
            self.assertFalse(ab.compare(p)['accepted'])

    def test_rejects_control_regression_memory_growth_and_missing_workloads(self):
        p = self.pairs()
        next(x for x in p if x['sampleId'] == 'collection')['candidate']['phases']['incrementalAppend']['medianMs'] = 12
        self.assertFalse(ab.compare(p)['accepted'])
        p = self.pairs()
        p[0]['candidate']['peakRssKiB'] = 20000
        self.assertFalse(ab.compare(p)['accepted'])
        self.assertFalse(ab.compare(self.pairs()[:-1])['accepted'])

    def test_rejects_no_reproducible_gain(self):
        p = self.pairs()
        for pair in p:
            pair['candidate']['phases']['incrementalAppend']['medianMs'] = 10
        self.assertFalse(ab.compare(p)['accepted'])


if __name__ == '__main__':
    unittest.main()

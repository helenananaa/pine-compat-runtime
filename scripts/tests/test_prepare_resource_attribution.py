import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from prepare_resource_attribution import prefix, prepare, replace_once


class ResourceAttributionTests(unittest.TestCase):
    def test_daily_forming_timestamp_does_not_admit_future_provider_events(self):
        events = []
        for minute in range(3):
            events.extend([
                dict(kind='request_forming', bar=dict(time=0, close=minute)),
                dict(kind='forming', bar=dict(time=minute * 60000)),
                dict(kind='confirmed', bar=dict(time=minute * 60000)),
            ])
        original = dict(tail=[1, 2, 3], events=events)
        result = prefix(original, 1)
        self.assertEqual(result['events'], events[:3])
        self.assertEqual(original['events'], events)
        self.assertEqual(result['tail'], [1])

    def test_provider_confirmation_before_terminal_chart_confirmation_is_retained(self):
        events = [dict(kind='forming'), dict(kind='request_confirmed'), dict(kind='confirmed')]
        self.assertEqual(prefix(dict(tail=[1], events=events), 64)['events'], events)
        with self.assertRaises(ValueError):
            prefix(dict(tail=[1, 2], events=events), 2)

    def test_instrumentation_rejects_stale_or_ambiguous_anchors(self):
        self.assertEqual(replace_once('one anchor', 'anchor', 'probe'), 'one probe')
        for source in ['missing', 'anchor anchor']:
            with self.assertRaises(ValueError):
                replace_once(source, 'anchor', 'probe')

    def test_preparation_instruments_current_collector_and_streamed_report(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            inputs = root / 'inputs'
            inputs.mkdir()
            receipt = prepare(inputs, root / 'prepared', 64)
            source = (root / 'prepared/probe/main.rs').read_text(encoding='utf-8')
            self.assertEqual(receipt['inputs'], [])
            for stage in ['tailComplete', 'replicasReleased', 'snapshotCreated', 'serialized',
                          'publicResultsCompared', 'sessionsReleased',
                          'historicalControlsExecuted', 'reportAssembly']:
                self.assertIn(f'stage("{stage}"', source)
            self.assertIn('same_files(&result_path, &replica_path)?', source)
            self.assertIn('write_report(&args[3]', source)
            self.assertNotIn('// attribution:', source)


if __name__ == '__main__':
    unittest.main()

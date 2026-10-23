"""Reject incomplete, corrupted or overclaimed installed-wheel measurements."""
import copy
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import benchmark_host_interfaces as probe
import audit_host_interface_resources as audit


class HostInterfaceReceiptTests(unittest.TestCase):
    def receipt(self):
        counts = dict(compile=1, serialization=4, create=4, seedWithOutput=4, fork=1,
                      finalSnapshot=4, batchControl=4, advanceWithOutput=12)
        return dict(status='passed', qualification='notEvaluated', mode='history',
                    sessions=4, historyBars=24, tailBars=8, chunkBars=3,
                    inputSha256=['a'] * 4, outputSha256=['b'] * 4,
                    checks=dict(batchEqualsAdvance=True, independentInputs=True,
                                forkIndependent=True, priorOutputImmutable=True),
                    phases={name: probe.stats_ms([1.0] * count) for name, count in counts.items()},
                    memoryCheckpoints=[dict(phase=phase, rssBytes=1, peakRssBytes=2)
                                       for phase in ['imported', 'inputAndProgram', 'verified', 'objectsReleased']])

    def test_nondivisible_tail_requires_every_streams_final_partial_chunk(self):
        receipt = self.receipt()
        probe.validate(receipt)
        receipt['phases']['advanceWithOutput'] = probe.stats_ms([1.0] * 8)
        with self.assertRaisesRegex(ValueError, 'incomplete phase'):
            probe.validate(receipt)

    def test_missing_outputs_checks_or_memory_cannot_pass(self):
        for field in ['outputSha256', 'checks', 'memoryCheckpoints']:
            with self.subTest(field=field):
                receipt = self.receipt()
                receipt[field] = [] if field != 'checks' else {}
                with self.assertRaises(ValueError):
                    probe.validate(receipt)

    def test_cached_statistics_and_nonfinite_samples_cannot_pass(self):
        receipt = self.receipt()
        receipt['phases']['compile']['medianMs'] = 0
        with self.assertRaisesRegex(ValueError, 'incomplete phase'):
            probe.validate(receipt)
        receipt = self.receipt()
        receipt['phases']['compile']['samplesMs'][0] = float('nan')
        with self.assertRaises(ValueError):
            probe.validate(receipt)

    def test_failure_or_resource_qualification_claim_cannot_pass(self):
        for change in [dict(status='failed'), dict(qualification='passed'), dict(mode='unknown'),
                       dict(sessions=True), dict(tailBars=0)]:
            with self.subTest(change=change):
                receipt = self.receipt()
                receipt.update(change)
                with self.assertRaises(ValueError):
                    probe.validate(receipt)

    def test_cross_python_standard_deviation_tolerance_does_not_relax_counts_or_medians(self):
        receipt = self.receipt()
        receipt['phases']['serialization']['stdevMs'] = 1e-15
        audit.validate_cross_python(receipt)
        for field, value in [('stdevMs', 0.1), ('medianMs', 0.1), ('n', 3)]:
            with self.subTest(field=field):
                bad = copy.deepcopy(receipt)
                bad['phases']['serialization'][field] = value
                with self.assertRaises(ValueError):
                    audit.validate_cross_python(bad)

    def test_external_oracle_rejects_missing_feedback_and_truncated_intents(self):
        bars, accounts = probe.inputs(2, 0)
        result = dict(protocol='external-broker/1', output=dict(plots=[
            dict(values=[frame['position_size'] for frame in accounts]),
            dict(values=[None, 0]), dict(values=[frame['equity'] for frame in accounts]),
            dict(values=[bar['close'] for bar in bars])]),
            intents=[dict(action='entry' if frame['position_size'] == 0 else 'close',
                          bar_index=i, qty=2 if frame['position_size'] == 0 else None)
                     for i, (bar, frame) in enumerate(zip(bars, accounts))])
        probe.verify_external(result, bars, accounts)
        for mutation in ['feedback', 'intents', 'ledger', 'context']:
            with self.subTest(mutation=mutation):
                bad = copy.deepcopy(result)
                if mutation == 'feedback':
                    bad['output']['plots'][0]['values'][0] = 100
                elif mutation == 'intents':
                    bad['intents'].pop()
                elif mutation == 'ledger':
                    bad['output']['strategy'] = {'positions': [1]}
                else:
                    bad['intents'][1]['bar_index'] = 0
                with self.assertRaises(ValueError):
                    probe.verify_external(bad, bars, accounts)


if __name__ == '__main__':
    unittest.main()

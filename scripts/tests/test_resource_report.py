import copy
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from resource_report import expected_spool_names, validate_report
from audit_resource_matrix import compare_outputs
from product_resource_acceptance import measurement_directory, require_fresh_trial


class ResourceReportTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.output = self.root / 'result.json'
        self.parts = self.root / 'result.json.parts'
        self.parts.mkdir()
        for name in expected_spool_names(2, True):
            (self.parts / name).write_text('{"plots":[]}', encoding='utf-8')
        self.metadata = dict(resourceMetadataVersion=1, metrics={}, resultCount=2,
            confirmedBars=12, historicalAppendMatches=False, historicalSameContextMatches=True)
        self.manifest = dict(resourceReportVersion=2, metadata={'path':'result.json.metadata.json'},
            results=[{'path':f'result.json.parts/live-{i}.json'} for i in range(2)],
            confirmedBars=12, historicalAppendMatches=False, historicalSameContextMatches=True,
            historicalContexts=[dict(stream=i, matches=False, sameContextMatches=True,
                batchDatasetEnd=11, initialSeedDatasetEnd=9,
                batch={'path':f'result.json.parts/batch-{i}.json'},
                incremental={'path':f'result.json.parts/incremental-{i}.json'},
                sameContextIncremental={'path':f'result.json.parts/same-context-{i}.json'}) for i in range(2)])
        (self.root / 'result.json.metadata.json').write_text(json.dumps(self.metadata))

    def tearDown(self):
        self.temp.cleanup()

    def validate(self, value=None):
        self.output.write_text(json.dumps(value or self.manifest))
        return validate_report(self.output, 2, 12, True, 10)

    def test_references_preserve_all_streams_and_both_historical_contexts(self):
        self.assertEqual(self.validate(), self.manifest)
        self.assertEqual(len(expected_spool_names(2, True)), 10)

    def test_wrong_paths_stream_order_control_counts_and_flags_fail(self):
        variants = []
        for path in ['../outside.json', str(self.parts/'live-0.json'),
                     'result.json.parts/live-1.json']:
            value=copy.deepcopy(self.manifest);value['results'][0]['path']=path;variants.append(value)
        for field, replacement in [('stream', True), ('batchDatasetEnd', 12),
                ('initialSeedDatasetEnd', 11), ('sameContextMatches', False)]:
            value=copy.deepcopy(self.manifest);value['historicalContexts'][0][field]=replacement;variants.append(value)
        value=copy.deepcopy(self.manifest);value['historicalContexts'].pop();variants.append(value)
        value=copy.deepcopy(self.manifest);value['resourceReportVersion']=3;variants.append(value)
        for value in variants:
            with self.subTest(value=value), self.assertRaises(AssertionError):self.validate(value)

    def test_missing_replica_or_control_is_not_accepted_as_complete(self):
        for name in ['replica-0.json','same-context-1.json']:
            path=self.parts/name;data=path.read_bytes();path.unlink()
            with self.assertRaises(AssertionError):self.validate()
            path.write_bytes(data)

    def test_identical_complete_files_do_not_need_two_parsed_trees(self):
        left=self.parts/'live-0.json';right=self.parts/'live-1.json'
        with patch('audit_resource_matrix.read',side_effect=AssertionError('unexpected JSON parse')):
            self.assertEqual(compare_outputs(left,right),[])

    def test_identical_empty_truncated_or_nonfinite_outputs_are_rejected(self):
        left=self.parts/'live-0.json';right=self.parts/'live-1.json'
        for text in ['', '{"x":[1,', '{"x":NaN}', '{"x":1e999}', '[]']:
            left.write_text(text);right.write_text(text)
            with self.subTest(text=text), self.assertRaises((ValueError,AssertionError)):
                compare_outputs(left,right)

    def test_new_measurement_and_resume_preserve_existing_evidence(self):
        outdir=measurement_directory(self.root,'rust',False)
        receipt=outdir/'results.json';receipt.write_bytes(b'original receipt')
        with self.assertRaises(ValueError):measurement_directory(self.root,'rust',False)
        self.assertEqual(measurement_directory(self.root,'rust',True),outdir)
        self.assertEqual(receipt.read_bytes(),b'original receipt')
        receipt.unlink()
        residue=outdir/'case.json.parts';residue.mkdir();(residue/'live-0.json').write_bytes(b'original output')
        with self.assertRaises(ValueError):measurement_directory(self.root,'rust',True)
        with self.assertRaises(ValueError):require_fresh_trial(outdir/'case.json',outdir/'case.progress.log')
        self.assertEqual((residue/'live-0.json').read_bytes(),b'original output')

    def test_different_encoding_still_uses_numeric_tolerance_and_checks_all_fields(self):
        left=self.parts/'live-0.json';right=self.parts/'live-1.json'
        left.write_text('{"x":[1,2.0],"label":"a","flag":true}')
        right.write_text('{"flag":true,"label":"a","x":[1.0,2.0000000001]}')
        self.assertEqual(compare_outputs(left,right),[])
        for value in ['{"x":[1,2],"label":"b","flag":true}',
                      '{"x":[1,2],"label":"a","flag":1}',
                      '{"x":[1,2,3],"label":"a","flag":true}']:
            right.write_text(value)
            self.assertTrue(compare_outputs(left,right))


if __name__ == '__main__':
    unittest.main()

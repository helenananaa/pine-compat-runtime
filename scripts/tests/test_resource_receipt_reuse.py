import copy
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from resource_receipt_reuse import python_only_revision, compatible_plans, validate_surfaces


OLD = '''import json
def measure(x):
    return x + 1
def python_worker(x, stream):
    import pine_compat as pine
    replica_result = x
    result = x
    json.dump(replica_result,stream,allow_nan=False,separators=(',',':'))
    json.dump(result,stream,allow_nan=False,separators=(',',':'))
    json.dump(dict(resourceMetadataVersion=1,metrics=x),stream)
'''
NEW = OLD.replace('    import pine_compat as pine', '''    import pine_compat as pine
    from resource_json import dump_public_json
    serializer_sha = sha(Path(dump_public_json.__code__.co_filename))''').replace(
    "json.dump(replica_result,stream,allow_nan=False,separators=(',',':'))", 'dump_public_json(replica_result,stream)').replace(
    "json.dump(result,stream,allow_nan=False,separators=(',',':'))", 'dump_public_json(result,stream)').replace(
    '    json.dump(dict(', "    assert serializer_sha == sha(Path(dump_public_json.__code__.co_filename)), 'serializer changed during measurement'\n    json.dump(dict(").replace(
    'dict(resourceMetadataVersion=1,', 'dict(resourceMetadataVersion=1,serializerSourceSha256=serializer_sha,')


class ResourceReceiptReuseTests(unittest.TestCase):
    def test_python_serialization_only_revision_has_unchanged_controller_proof(self):
        proof = python_only_revision(OLD, NEW)
        self.assertEqual(proof['normalizations'], [1, 1, 1, 2, 1])
        self.assertEqual(len(proof['untouchedControllerAstSha256']), 64)

    def test_changed_controller_cannot_retain_rust_or_wasm(self):
        with self.assertRaises(AssertionError):
            python_only_revision(OLD, NEW.replace('return x + 1', 'return x + 2'))

    def test_changes_to_python_input_or_execution_cannot_hide_as_serialization(self):
        with self.assertRaises(AssertionError):
            python_only_revision(OLD, NEW.replace('result = x\n', 'result = 42\n'))

    def test_python_receipts_and_reduced_retention_manifest_are_rejected(self):
        valid = ['Windows/rust', 'Windows/wasm', 'Linux/rust', 'Linux/wasm']
        validate_surfaces(valid)
        for invalid in [valid + ['Windows/python'], valid[:-1], valid[:-1] + [valid[0]]]:
            with self.assertRaises(AssertionError):
                validate_surfaces(invalid)

    def test_changed_core_workloads_artifacts_or_budgets_cannot_be_retained(self):
        old = {key: 'same' for key in ['coreCommit', 'sourceIdentitySha256', 'sourceState',
            'budgetSha256', 'budgets', 'cases', 'repetitions', 'processTimeoutSeconds', 'artifactPaths']}
        old['workerSourceHashes'] = {'scripts/product_resource_acceptance.py': 'old', 'wasm': 'same'}
        new = copy.deepcopy(old)
        new['workerSourceHashes']['scripts/product_resource_acceptance.py'] = 'new'
        compatible_plans(old, new)
        for key in ['sourceIdentitySha256', 'budgets', 'cases', 'artifactPaths']:
            invalid = copy.deepcopy(new)
            invalid[key] = 'changed'
            with self.assertRaises(AssertionError):
                compatible_plans(old, invalid)

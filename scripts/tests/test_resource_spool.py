import sys
import tempfile
import subprocess
import json
import io
import unittest
from pathlib import Path

sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from product_resource_acceptance import same_files, resource_metadata
from resource_json import dump_public_json


class ResourceSpoolTests(unittest.TestCase):
    def test_scalar_batches_match_standard_json_across_boundaries_and_nested_values(self):
        values = [None, True, False, -0.0, 0.0, 2**70, '中文🙂"\n\\'] * 9000
        value = {'plots': [{'values': values, 'mixed': [1, {'x': (2, 3)}, 4, [], 5]}],
            'empty': {}, 'numericKeys': {3: 'three', None: 'null'}, 'shared': [values, values]}
        expected = io.StringIO()
        json.dump(value, expected, allow_nan=False, separators=(',', ':'))
        actual = io.StringIO()
        dump_public_json(value, actual)
        self.assertEqual(actual.getvalue(), expected.getvalue())
        self.assertEqual(len(values), 63000)

    def test_batched_json_rejects_nonfinite_values_and_cycles(self):
        cycle = []
        cycle.append(cycle)
        for value in [{'values': [0] * 8192 + [float('nan')]},
                {'nested': (float('inf'),)}, cycle]:
            with self.assertRaises(ValueError):
                dump_public_json(value, io.StringIO())

    def test_metadata_requires_version_counts_and_finite_timings(self):
        with tempfile.TemporaryDirectory() as directory:
            output=Path(directory)/'result.json'
            sidecar=output.with_name(output.name+'.metadata.json')
            valid=dict(resourceMetadataVersion=1,resultCount=4,confirmedBars=110000,
                metrics={'forming':[0.1,0.2]})
            sidecar.write_text(json.dumps(valid))
            self.assertEqual(resource_metadata(output,4,110000),valid)
            for key,value in [('resourceMetadataVersion',2),('resultCount',1),
                    ('confirmedBars',100000),('resultCount',True),
                    ('metrics',{'forming':[float('inf')]}),('metrics',{'forming':[-1]}),
                    ('metrics',{'forming':[True]})]:
                sidecar.write_text(json.dumps(dict(valid,**{key:value})))
                with self.assertRaises(AssertionError):resource_metadata(output,4,110000)
            sidecar.unlink()
            with self.assertRaises(FileNotFoundError):resource_metadata(output,4,110000)

    def test_wasm_diagnostic_scanner_checks_root_strategy_and_escaping(self):
        subprocess.run(['node',str(Path(__file__).with_name('resource_wasm_diagnostics.cjs'))],check=True)

    def test_comparison_checks_last_chunk_and_exact_eof(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);left=root/'left.json';right=root/'right.json'
            data=b'['+b'0,'*70000+b'1]'
            left.write_bytes(data);right.write_bytes(data)
            self.assertTrue(same_files(left,right))
            for changed in [data[:-2]+b'2]',data+b' ',data[:-1]]:
                right.write_bytes(changed)
                self.assertFalse(same_files(left,right))

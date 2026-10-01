"""Protect the acceptance comparator against false positives across bindings."""
import importlib.util
import unittest
from pathlib import Path

spec = importlib.util.spec_from_file_location(
    "core_acceptance", Path(__file__).resolve().parents[1] / "requalify_core_scripts.py"
)
acceptance = importlib.util.module_from_spec(spec)
spec.loader.exec_module(acceptance)


class CompleteOutputComparisonTests(unittest.TestCase):
    def test_extra_orders_and_missing_metadata_fail(self):
        reference = {"strategy": {"orders": [{"qty": 105000}]}, "schemaVersion": 9}
        changed = {"strategy": {"orders": [{"qty": 105000}, {"qty": 30000}]}, "schemaVersion": 9}
        self.assertTrue(acceptance.differences(reference, changed))
        self.assertTrue(acceptance.differences(reference, {"strategy": reference["strategy"]}))

    def test_signal_null_and_boolean_are_not_numeric_zero(self):
        for value in (None, False, True, "0"):
            with self.subTest(value=value):
                self.assertTrue(acceptance.differences({"values": [value]}, {"values": [0]}))

    def test_nonfinite_and_material_account_changes_fail(self):
        self.assertTrue(acceptance.differences(float("nan"), float("nan")))
        self.assertTrue(acceptance.differences(float("inf"), float("inf")))
        self.assertTrue(acceptance.differences({"profit": 1000000.0}, {"profit": 1000000.01}))
        self.assertFalse(acceptance.differences({"profit": 1.23456789}, {"profit": 1.2345678900000002}))


if __name__ == "__main__":
    unittest.main()

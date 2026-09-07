from __future__ import annotations

import json
from pathlib import Path
import sys
import tempfile
import unittest


SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))

import compare_strategy_reference_outputs as compare  # noqa: E402


TRADE = {
    "id": "L",
    "entryBarIndex": 1,
    "exitBarIndex": 2,
    "entryTime": 2,
    "exitTime": 3,
    "entryPrice": 100.0,
    "exitPrice": 110.0,
    "qty": 1.0,
    "profit": 10.0,
}


def payload(trades: list[dict]) -> dict:
    return {"strategy": {"trades": trades}}


def compare_trades(actual: list[dict], reference: list[dict], fields=None) -> dict:
    return compare.compare_reference(
        payload(actual),
        payload(reference),
        fields or compare.PUBLIC_TRADE_FIELDS,
    )


class CompareStrategyReferenceOutputsTests(unittest.TestCase):
    def test_matching_trades_pass_under_frozen_tolerances(self) -> None:
        report = compare_trades([TRADE], [TRADE])
        self.assertEqual(report["status"], "passed")
        self.assertTrue(report["comparable"])
        self.assertEqual(report["mismatches"], 0)

    def test_wrong_price_fails(self) -> None:
        other = dict(TRADE, exitPrice=111.0)
        report = compare_trades([TRADE], [other])
        self.assertEqual(report["status"], "failed")
        self.assertEqual(report["firstDifference"]["field"], "exitPrice")
        self.assertEqual(report["firstDifference"]["reason"], "numeric_mismatch")

    def test_wrong_qty_fails(self) -> None:
        other = dict(TRADE, qty=2.0)
        report = compare_trades([TRADE], [other])
        self.assertEqual(report["status"], "failed")
        self.assertEqual(report["firstDifference"]["field"], "qty")

    def test_missing_fill_fails(self) -> None:
        report = compare_trades([], [TRADE])
        self.assertEqual(report["status"], "failed")
        self.assertEqual(report["reason"], "empty_runtime_trades")

    def test_duplicate_or_extra_fill_fails(self) -> None:
        extra = dict(TRADE, id="M", entryTime=4, exitTime=5)
        report = compare_trades([TRADE, extra], [TRADE])
        self.assertEqual(report["status"], "failed")
        self.assertEqual(report["firstDifference"]["reason"], "duplicate_or_extra_fill")

    def test_wrong_time_fails(self) -> None:
        other = dict(TRADE, exitTime=99)
        report = compare_trades([TRADE], [other])
        self.assertEqual(report["status"], "failed")
        self.assertEqual(report["firstDifference"]["field"], "exitTime")

    def test_different_fees_are_uncovered_not_pass(self) -> None:
        report = compare.compare_reference(
            payload([TRADE]),
            payload([TRADE]),
            ["id", "commission"],
        )
        self.assertEqual(report["status"], "incomparable")
        self.assertEqual(report["reason"], "uncovered_fields")
        self.assertIn("commission", report["uncoveredFields"])
        self.assertNotEqual(report["status"], "passed")

    def test_empty_reference_against_trades_fails(self) -> None:
        report = compare_trades([TRADE], [])
        self.assertEqual(report["status"], "failed")
        self.assertEqual(report["reason"], "empty_reference_trades")

    def test_duplicate_ids_with_identical_identity_fail(self) -> None:
        report = compare_trades([TRADE, TRADE], [TRADE, TRADE])
        self.assertEqual(report["status"], "failed")
        self.assertEqual(report["reason"], "duplicate_trade_identity")

    def test_partial_reference_is_incomparable(self) -> None:
        partial = {"id": "L", "qty": 1.0}
        report = compare.compare_reference(
            payload([TRADE]),
            payload([partial]),
            list(compare.PUBLIC_TRADE_FIELDS),
        )
        self.assertEqual(report["status"], "incomparable")
        self.assertEqual(report["reason"], "partial_reference")
        self.assertNotEqual(report["status"], "passed")

    def test_uncovered_field_declaration_is_incomparable(self) -> None:
        report = compare.compare_reference(
            payload([TRADE]),
            payload([TRADE]),
            ["id", "direction"],
        )
        self.assertEqual(report["status"], "incomparable")
        self.assertIn("direction", report["uncoveredFields"])

    def test_cli_writes_json_and_nonzero_on_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            actual = root / "actual.json"
            reference = root / "reference.json"
            actual.write_text(json.dumps(payload([TRADE])), encoding="utf-8")
            reference.write_text(
                json.dumps(payload([dict(TRADE, qty=9.0)])), encoding="utf-8"
            )
            code = compare.main([str(actual), str(reference), "--format", "json"])
            self.assertEqual(code, 1)


if __name__ == "__main__":
    unittest.main()

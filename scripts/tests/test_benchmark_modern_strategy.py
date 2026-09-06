from __future__ import annotations

import json
from pathlib import Path
import sys
import tempfile
import unittest


SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))

import benchmark_modern_strategy as bench  # noqa: E402


class FakeSession:
    def seed(self, bars):
        self.bars = list(bars)
        return {"plots": []}

    def update_forming(self, bar):
        return {"plots": [{"values": [bar["close"]]}]}

    def update_confirmed(self, bar):
        return {"plots": [{"values": [bar["close"]]}]}


class FakeProgram:
    def run(self, bars):
        return {
            "schemaVersion": 8,
            "plots": [{"id": 1, "values": [row["close"] for row in bars]}],
            "strategy": {"trades": []},
        }

    def realtime_session(self):
        return FakeSession()


class FakePine:
    def compile_script(self, source: str):
        if "FAIL" in source:
            raise ValueError("compile failed")
        return FakeProgram()


class BenchmarkModernStrategyTests(unittest.TestCase):
    def test_synthetic_bars_are_seed_stable(self) -> None:
        first = bench.synthetic_bars(8, seed=3)
        second = bench.synthetic_bars(8, seed=3)
        self.assertEqual(first, second)
        self.assertNotEqual(first, bench.synthetic_bars(8, seed=4))
        self.assertEqual(len(first), 8)

    def test_phases_are_separate_and_hash_is_stable(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            pine = root / "tests/fixtures/runtime"
            pine.mkdir(parents=True)
            source = '//@version=6\nstrategy("bench")\nplot(close)\n'
            (pine / "strategy_trade_counts.pine").write_text(source, encoding="utf-8")
            (pine / "strategy_entry.pine").write_text(source, encoding="utf-8")
            (pine / "generic_input.pine").write_text(source, encoding="utf-8")
            report = bench.build_report(
                root=root,
                pine_compat=FakePine(),
                bar_counts=(4,),
                seed=1,
                warmup=1,
                iters=3,
            )
            self.assertEqual(report["status"], "基线完成")
            self.assertFalse(report["optimizationCommitted"])
            self.assertEqual(len(report["samples"]), 3)
            phases = report["samples"][0]["phases"]
            for name in (
                "compile",
                "historicalRun",
                "incrementalAppend",
                "formingReplace",
                "outputSerialization",
            ):
                self.assertIn(name, phases)
                self.assertEqual(phases[name]["status"], "measured")
            hashes = {item["resultHash"] for item in report["samples"]}
            self.assertEqual(len(hashes), 1)

    def test_zero_iters_does_not_print_100_percent(self) -> None:
        stats = bench.median_ms([])
        self.assertTrue(stats["n"] == 0)


if __name__ == "__main__":
    unittest.main()

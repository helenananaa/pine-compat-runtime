import csv
import hashlib
import importlib.util
import tempfile
import unittest
from pathlib import Path


SPEC = importlib.util.spec_from_file_location(
    "four_update", Path(__file__).resolve().parents[1] / "replay_four_update_reference.py")
REF = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(REF)


class NativeSamplerValidation(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.source = self.root / "source.pine"
        self.source.write_text("//@version=6\n", encoding="utf-8")
        self.start = 600000
        self.plan = dict(captureStart=self.start, sourceSha256=hashlib.sha256(self.source.read_bytes()).hexdigest())
        self.header = ["time", "open", "high", "low", "close", "Volume"] + REF.TITLES
        self.rows = []
        for n, timestamp in enumerate(range(self.start - 240000, self.start + 120000, 60000)):
            row = dict.fromkeys(self.header, "")
            row.update(time=timestamp // 1000, open=100, high=102, low=99, close=101, Volume=12)
            row.update({"RTC now": timestamp + 59000, "RTC ema": 101, "RTC sma": 101, "RTC barCount": n+1})
            if n >= 4:
                for i in range(4):
                    for field in REF.FIELDS:
                        row[f"RT{i} {field}"] = 100
                    row[f"RT{i} now"] = timestamp + i*1000
                    row[f"RT{i} sample"] = i+1
            self.rows.append(row)

    def build(self, header=None):
        path = self.root / "chart.csv"
        with path.open("w", encoding="utf-8", newline="") as stream:
            writer = csv.writer(stream)
            header = self.header if header is None else header
            writer.writerow(header)
            writer.writerows([[r[k] for k in header] for r in self.rows])
        return REF.build_payload(self.source, path, self.plan)

    def test_native_input_and_future_slots(self):
        payload = self.build()
        self.assertEqual(len(payload["seedBars"]), 4)
        self.assertEqual(len(payload["updates"]), 10)
        first = payload["updates"][0]
        self.assertEqual(first["bar"]["close"], 100)
        self.assertIsNone(first["expected"]["RT1 close"])
        self.assertEqual(first["expected"]["RTC now"], self.start)
        self.assertEqual(payload["updates"][4]["bar"]["close"], 101)

    def test_duplicate_plot_rejected(self):
        with self.assertRaisesRegex(ValueError, "duplicate"):
            self.build(self.header + ["RT0 close"])

    def test_missing_sample_rejected(self):
        self.rows[-1]["RT3 sample"] = ""
        with self.assertRaisesRegex(ValueError, "Incomplete"):
            self.build()

    def test_missing_bar_rejected(self):
        self.rows.pop()
        with self.assertRaisesRegex(ValueError, "exactly four"):
            self.build()

    def test_source_changed_rejected(self):
        self.source.write_text("changed", encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "source hash"):
            self.build()

    def test_nonfinite_reference_rejected(self):
        self.rows[-1]["RT3 close"] = "NaN"
        with self.assertRaisesRegex(ValueError, "Non-finite"):
            self.build()

    def test_clock_regression_rejected(self):
        self.rows[-1]["RT1 now"] = 1
        with self.assertRaisesRegex(ValueError, "Regressive"):
            self.build()


if __name__ == "__main__":
    unittest.main()

"""Replay a frozen four-update TradingView CSV through an installed Python wheel.

Inputs come exclusively from the native CSV. This sampler observes four script
updates per target bar, not every exchange price event. A missing observation
must fail validation; a broker discrepancy must remain a failed comparison.
"""

import argparse
import csv
import hashlib
import json
import math
from pathlib import Path


FIELDS = (
    "now", "open", "high", "low", "close", "volume", "flags", "barCount",
    "sample", "ema", "sma", "position", "closed", "entryPrice", "exitPrice",
)
TITLES = [f"RT{i} {field}" for i in range(4) for field in FIELDS]
TITLES += [f"RTC {field}" for field in ("now", "ema", "sma", "barCount")]
EXACT = {"now", "flags", "barCount", "sample", "position", "closed"}


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def number(text):
    if text == "":
        return None
    value = float(text)
    if not math.isfinite(value):
        raise ValueError("Non-finite native value")
    return value


def build_payload(source_path, csv_path, plan):
    if digest(source_path) != plan["sourceSha256"]:
        raise ValueError("Frozen source hash mismatch")
    start = plan["captureStart"]
    with Path(csv_path).open(encoding="utf-8-sig", newline="") as stream:
        reader = csv.reader(stream)
        header = next(reader)
        required = TITLES + ["time", "open", "high", "low", "close", "Volume"]
        if any(header.count(title) != 1 for title in required):
            raise ValueError("Missing or duplicate required native columns")
        indices = {title: header.index(title) for title in required}
        rows = {}
        for raw in reader:
            timestamp = int(raw[indices["time"]]) * 1000
            if start - 240000 <= timestamp < start + 120000:
                if timestamp in rows:
                    raise ValueError("Duplicate bar timestamp")
                rows[timestamp] = {key: number(raw[index]) for key, index in indices.items()}
    times = list(range(start - 240000, start + 120000, 60000))
    if sorted(rows) != times:
        raise ValueError("Need exactly four warmup and two capture bars")

    def bar(row, timestamp):
        return {"time": timestamp, **{f: row[f] for f in ("open", "high", "low", "close")},
                "volume": row["Volume"]}

    seed_rows = [rows[t] for t in times[:4]]
    payload = {
        "source": Path(source_path).read_text(encoding="utf-8"),
        "sourceSha256": digest(source_path), "nativeCsvSha256": digest(csv_path),
        "seedBars": [bar(rows[t], t) for t in times[:4]],
        "seedExecutionTimes": [int(row["RTC now"]) for row in seed_rows],
        "seedExpected": [{title: row[title] for title in TITLES} for row in seed_rows],
        "updates": [], "expectedPlotValues": 896,
        "scope": "4 warmup bars, first 4 observed script updates in each of 2 bars, and confirmations; not every exchange tick",
    }
    previous_clock = payload["seedExecutionTimes"][-1]
    for timestamp in times[4:]:
        row = rows[timestamp]
        for i in range(4):
            fields = {f: row[f"RT{i} {f}"] for f in FIELDS}
            if fields["sample"] != i + 1 or fields["now"] is None:
                raise ValueError("Incomplete or reordered four-update sample")
            if fields["now"] < previous_clock:
                raise ValueError("Regressive execution clock")
            previous_clock = fields["now"]
            expected = {f"RT{j} {f}": row[f"RT{j} {f}"] if j <= i else None
                        for j in range(4) for f in FIELDS}
            expected.update({f"RTC {f}": fields[f] for f in ("now", "ema", "sma", "barCount")})
            payload["updates"].append({
                "kind": "forming", "executionTime": int(fields["now"]),
                "bar": {"time": timestamp, **{f: fields[f] for f in ("open", "high", "low", "close", "volume")}},
                "expected": expected,
            })
        payload["updates"].append({
            "kind": "confirmed", "executionTime": int(row["RTC now"]),
            "bar": bar(row, timestamp), "expected": {title: row[title] for title in TITLES},
        })
    return payload


def replay(payload, expected_module_root):
    import pine_compat

    module = Path(pine_compat.__file__).resolve()
    if not module.is_relative_to(Path(expected_module_root).resolve()):
        raise ValueError(f"Unexpected installed module: {module}")
    session = pine_compat.create_realtime_session(
        payload["source"], chart_symbol="OKX:BTCUSDT", chart_timeframe="1",
        request_bars={"$chart": {"minMove": 1, "priceScale": 10, "quantityPrecision": 6, "pointValue": 1}},
    )
    differences, results = [], []
    count = 0

    def compare(result, expected, index, step):
        nonlocal count
        plots = {p["title"]: p["values"] for p in result["plots"]}
        if len(result["plots"]) != 64 or set(plots) != set(TITLES):
            raise ValueError("Runtime plot schema mismatch")
        for title, target in expected.items():
            actual = plots[title][index]
            exact = title.split(" ", 1)[1] in EXACT
            same = actual == target if exact or target is None else (
                actual is not None and math.isfinite(actual)
                and math.isclose(actual, target, abs_tol=1e-9, rel_tol=1e-9))
            count += 1
            if not same:
                differences.append(dict(step=step, plot=title, actual=actual, expected=target))

    seed = session.seed(payload["seedBars"], execution_times=payload["seedExecutionTimes"])
    for i, expected in enumerate(payload["seedExpected"]):
        compare(seed, expected, i, f"seed-{i}")
    for i, update in enumerate(payload["updates"]):
        method = session.update_forming if update["kind"] == "forming" else session.update_confirmed
        result = method(update["bar"], execution_time=update["executionTime"])
        compare(result, update["expected"], -1, f"{i}-{update['kind']}")
        results.append(result)
    if count != 896:
        raise ValueError("Frozen comparison denominator changed")
    report = dict(status="failed" if differences else "passed", comparedValues=count,
                  mismatchCount=len(differences), differences=differences,
                  pythonModule=str(module), sourceSha256=payload["sourceSha256"],
                  nativeCsvSha256=payload["nativeCsvSha256"], scope=payload["scope"])
    return report, {"seed": seed, "updates": results}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--csv", type=Path, required=True)
    parser.add_argument("--plan", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--module-root", type=Path)
    args = parser.parse_args()
    if args.output.exists():
        raise ValueError("Output directory already exists; preserve previous evidence")
    payload = build_payload(args.source, args.csv, json.loads(args.plan.read_text(encoding="utf-8")))
    args.output.mkdir(parents=True)
    (args.output / "payload.json").write_text(json.dumps(payload, indent=2), encoding="utf-8")
    if args.module_root:
        report, results = replay(payload, args.module_root)
        (args.output / "comparison.json").write_text(json.dumps(report, indent=2), encoding="utf-8")
        (args.output / "results.json").write_text(json.dumps(results), encoding="utf-8")
        print(json.dumps({k: v for k, v in report.items() if k != "differences"}))
        return int(report["status"] != "passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

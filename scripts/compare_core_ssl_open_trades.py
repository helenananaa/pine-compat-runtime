"""Requalify native open legs without equating entry size to reversal transaction size.

The unchanged frozen comparator is executed first and its failure is retained.
A read-only Pine observer must leave every original output field unchanged.
"""
import csv
import datetime as dt
import json
import subprocess
import sys
import time
from collections import defaultdict
from pathlib import Path

from requalify_core_scripts import differences, read, run, sha, write


def compare(root, cli):
    root = Path(root).resolve()
    # Keep the old failure visible; do not edit or weaken the frozen comparator.
    previous = root / "comparison.json"
    if previous.exists():
        previous.rename(root / f"comparison-before-legacy-run-{time.time_ns()}.json")
    old = subprocess.run([sys.executable, str(root / "compare.py")], capture_output=True)
    write(root / "legacy-comparator-receipt.json", {"exitCode": old.returncode,
          "comparatorSha256": sha(root / "compare.py"), "stderr": old.stderr.decode(errors="replace")})
    report = read(root / "comparison.json")
    write(root / "legacy-comparison.json", report)
    assert not report["diagnostics"]
    assert all(x["mismatch_count"] == 0 for x in report["series"].values())
    trades = report["trades"]
    assert not trades["closed_issues"] and not trades["explicit_exit_order_missing"] and not trades["explicit_exit_order_extra"]
    native = defaultdict(dict)
    with (root / "tradingview-trades.csv").open(encoding="utf-8-sig", newline="") as f:
        for row in csv.DictReader(f):
            native[int(row["Trade number"])][row["Type"].split()[0]] = row
    from requalify_core_scripts import load_bars
    bars = load_bars(root / "bars.csv")
    date_only = len(next(iter(native.values()))["Entry"]["Date and time"]) == 10
    zone = dt.timezone.utc if date_only else dt.timezone(dt.timedelta(hours=8))
    fmt = "%Y-%m-%d" if date_only else "%Y-%m-%d %H:%M"
    bar_dates = {dt.datetime.fromtimestamp(b["time"] / 1000, zone).strftime(fmt): b["time"] for b in bars}
    assert len(bar_dates) == len(bars), "Native dates must uniquely identify the frozen chart bars"
    boundary = max(bar_dates)
    entries = [pair["Entry"] for _, pair in sorted(native.items())
               if pair["Entry"]["Date and time"] <= boundary
               and (pair["Exit"]["Date and time"] == "Open" or pair["Exit"]["Date and time"] > boundary)]
    assert len(entries) == trades.get("native_open", trades.get("native_confirmed_boundary_entries"))
    suffix = '\n// Current-core acceptance observer: reads only, no trading commands.\n'
    suffix += 'plot(barstate.islast ? strategy.opentrades : na, title="__accept_open_count")\n'
    for i, entry in enumerate(entries):
        for field in ("size", "entry_price", "entry_time"):
            suffix += f'plot(barstate.islast ? strategy.opentrades.{field}({i}) : na, title="__accept_{field}_{i}")\n'
        suffix += f'plot(barstate.islast ? (strategy.opentrades.entry_id({i}) == {json.dumps(entry["Signal"])} ? 1 : 0) : na, title="__accept_id_{i}")\n'
    observed_source = root / "ssl-open-observed.pine"
    observed_source.write_text((root / "ssl-hybrid-original.pine").read_text(encoding="utf-8-sig") + suffix, encoding="utf-8")
    command = read(root / "local-batch.json.receipt.json")["command"]
    command[0] = str(Path(cli).resolve())
    command[2] = str(observed_source)
    out = root / "ssl-open-observed.json"
    cached = Path(str(out) + ".receipt.json")
    if out.exists() and cached.exists() and read(cached)["command"] == command and read(cached)["outputSha256"] == sha(out):
        assert read(cached)["exitCode"] == 0
    else:
        run(command, out)
    observed = read(out)
    values = {p["title"]: p["values"][-1] for p in observed["plots"] if p.get("title", "").startswith("__accept_")}
    assert values["__accept_open_count"] == len(entries)
    observed["plots"] = [p for p in observed["plots"] if not p.get("title", "").startswith("__accept_")]
    assert not differences(read(root / "local-batch.json"), observed), "Observer changed original output"
    legs = []
    for i, entry in enumerate(entries):
        expected = {"size": float(entry["Size (qty)"]) * (-1 if entry["Type"].endswith("short") else 1),
                    "entry_price": float(entry["Price USD"]),
                    "entry_time": bar_dates[entry["Date and time"]]}
        assert values[f"__accept_id_{i}"] == 1
        actual = {f: values[f"__accept_{f}_{i}"] for f in expected}
        assert not differences(expected, actual), (entry, expected, actual)
        legs.append({"id": entry["Signal"], "expected": expected, "actual": actual})
    assert abs(sum(x["actual"]["size"] for x in legs) - observed["strategy"]["position"][-1]["size"]) <= 1e-9
    receipt = {"validated": True, "nativeOpenLegs": len(legs), "legs": legs,
               "observerSourceSha256": sha(observed_source), "observerOutputSha256": sha(out),
               "originalOutputUnchangedAfterRemovingObserverPlots": True,
               "nativeDateFormat": fmt, "nativeDateTimezone": str(zone), "confirmedBoundary": boundary,
               "nativeOrderTransactionSizeQualified": False,
               "reason": "Native trade CSV entry sizes describe new legs; they do not provide the reversal transaction order list."}
    write(root / "open-leg-observer-comparison.json", receipt)
    trades["legacy_order_size_match_issues"] = trades["open_issues"]
    trades["open_issues"] = []
    trades["open_entry_matches"] = len(legs)
    trades["open_leg_evidence"] = "open-leg-observer-comparison.json"
    report["validated"] = True
    write(root / "current-native-comparison.json", report)
    return report


if __name__ == "__main__":
    result = compare(sys.argv[1], sys.argv[2])
    print(json.dumps({"validated": result["validated"], "nativeOpenLegs": result["trades"]["open_entry_matches"]}))

#!/usr/bin/env python3
"""Fresh-process growth probes for retained drawings, broker output and live HTF data."""
from __future__ import annotations

import argparse
import hashlib
import json
import platform
import subprocess
import sys
from pathlib import Path

from benchmark_streaming import bar, peak_rss_bytes, summary, timed

SOURCES = {
    "drawings": '''//@version=6
indicator("retained drawing workload")
var l = line.new(0, close, 0, close)
var a = label.new(0, close, "value")
var t = table.new(position.top_right, 1, 1)
line.set_xy2(l, bar_index, close)
label.set_xy(a, bar_index, close)
table.cell(t, 0, 0, str.tostring(close))
plot(close)
plotshape(close > open)
plotcandle(open, high, low, close)
bgcolor(close > open ? color.red : color.blue)
''',
    "orders": '''//@version=6
strategy("dense retained broker", process_orders_on_close=true)
if strategy.position_size == 0
    strategy.entry("L", strategy.long, qty=1)
else
    strategy.close("L")
plot(strategy.equity)
plot(strategy.closedtrades)
''',
    "requests": '''//@version=6
indicator("live higher timeframe")
plot(request.security("NYSE:IBM", "5", ta.sma(close, 14)))
plot(close)
''',
}


def worker(name, size, updates, window):
    import pine_compat as pine
    kwargs = {}
    if name == "requests":
        kwargs["request_bars"] = {"NYSE:IBM:5": [bar(i * 5) for i in range(size // 5)]}
    session = pine.create_realtime_session(SOURCES[name], **kwargs)
    session.set_output_retention(window)
    _, seed_ms = timed(lambda: session.seed([bar(i) for i in range(size)]))
    replica = session.replica()
    times = {k: [] for k in ("producer", "replica", "confirm", "request")}

    def consume(change):
        if change is not None:
            _, duration = timed(lambda: replica.apply(change))
            times["replica"].append(duration)

    for i in range(size, size + updates):
        for extra in (0.0, 0.1, 0.2):
            if name == "requests":
                update = bar(i, extra)
                update["time"] = (i // 5) * 300000
                change, elapsed = timed(lambda: session.apply_request_forming("NYSE:IBM", "5", update))
                times["request"].append(elapsed)
                consume(change)
            change, elapsed = timed(lambda: session.apply_forming(bar(i, extra)))
            times["producer"].append(elapsed)
            consume(change)
        if name == "requests" and (i + 1) % 5 == 0:
            update = bar(i, 0.2)
            update["time"] = (i // 5) * 300000
            change, elapsed = timed(lambda: session.apply_request_confirmed("NYSE:IBM", "5", update))
            times["request"].append(elapsed)
            consume(change)
        change, elapsed = timed(lambda: session.apply_confirmed(bar(i, 0.2)))
        times["confirm"].append(elapsed)
        consume(change)
    visible, snapshot_ms = timed(replica.result)
    matches = visible == session.result() and replica.revision == session.revision
    window_holds = all(len(plot["values"]) <= window for plot in visible["plots"])
    return dict(workload=name, history=size, updates=updates, keepConfirmedBars=window,
                sourceSha256=hashlib.sha256(SOURCES[name].encode()).hexdigest(),
                module=pine.__file__, seedMs=seed_ms, snapshotMs=snapshot_ms,
                peakRssBytes=peak_rss_bytes(), finalSnapshotMatches=matches, windowHolds=window_holds,
                metrics={k: summary(v) for k, v in times.items() if v})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--worker", choices=SOURCES)
    parser.add_argument("--history", nargs="+", type=int, default=[1024, 8192, 32768])
    parser.add_argument("--updates", type=int, default=128)
    parser.add_argument("--window", type=int, default=256)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--budget", type=Path)
    args = parser.parse_args()
    if min(args.history + [args.updates, args.window]) < 1:
        parser.error("history, updates and window must be positive")
    if args.worker:
        print(json.dumps(worker(args.worker, args.history[0], args.updates, args.window)))
        return
    if args.output is None:
        parser.error("--output is required")
    budget = json.loads(args.budget.read_text(encoding="utf-8-sig")) if args.budget else None
    rows, failures = [], []
    for name in SOURCES:
        group = []
        for size in args.history:
            run = subprocess.run([sys.executable, str(Path(__file__).resolve()), "--worker", name,
                                  "--history", str(size), "--updates", str(args.updates),
                                  "--window", str(args.window)], text=True, capture_output=True)
            if run.returncode:
                failures.append(f"{name}/{size}: worker failed")
                rows.append(dict(workload=name, history=size, stderr=run.stderr, stdout=run.stdout))
                continue
            row = json.loads(run.stdout)
            rows.append(row)
            group.append(row)
            if not row["finalSnapshotMatches"] or not row["windowHolds"]:
                failures.append(f"{name}/{size}: correctness or retention failed")
            print(f"{name}/{size}: " + ", ".join(f"{k} p95 {v['p95Ms']:.3f} ms"
                  for k, v in row["metrics"].items()), flush=True)
            if budget:
                for key, metric in row["metrics"].items():
                    if metric["p95Ms"] > budget[key + "P95Ms"]:
                        failures.append(f"{name}/{size}: {key} p95 exceeded")
                if row["peakRssBytes"] > budget["peakRssBytes"]:
                    failures.append(f"{name}/{size}: peak RSS exceeded")
        if budget and len(group) == len(args.history):
            first, last = min(group, key=lambda x: x["history"]), max(group, key=lambda x: x["history"])
            for key in first["metrics"]:
                ratio = last["metrics"][key]["p50Ms"] / first["metrics"][key]["p50Ms"]
                if ratio > budget["maxP50GrowthRatio"]:
                    failures.append(f"{name}: {key} median growth {ratio:.3f} exceeded")
    report = dict(status="failed" if failures else "passed", platform=platform.platform(),
                  budget=budget, budgetSha256=hashlib.sha256(args.budget.read_bytes()).hexdigest() if budget else None,
                  results=rows, failures=failures,
                  scope="Finite installed-wheel workloads; no concurrency or indefinite resource guarantee")
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + "\n")
    if failures:
        raise SystemExit("; ".join(failures))


if __name__ == "__main__":
    main()

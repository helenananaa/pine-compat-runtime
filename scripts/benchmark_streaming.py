#!/usr/bin/env python3
"""Installed-wheel streaming benchmark; each size/workload gets a fresh process.

Measures producer update, native replica apply, full snapshot, and process peak
RSS separately. Budgets are explicit caller input and never derived from results.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import platform
import statistics
import subprocess
import sys
import time
from pathlib import Path


SOURCES = {
    "plots": '//@version=6\nindicator("streaming cost")\nplot(close)\nplot(close[1])\n',
    "alerts": '//@version=6\nindicator("alerts cost")\nalert("event", alert.freq_all)\nplot(close)\n',
    "collection": '''//@version=6
indicator("bounded collection cost")
var values = array.new_float(0)
array.push(values, close)
if array.size(values) > 32
    array.shift(values)
plot(array.avg(values))
''',
}


def bar(index: int, extra: float = 0) -> dict:
    value = 100.0 + index * 0.01 + extra
    return dict(time=index * 60000, open=value, high=value, low=value,
                close=value, volume=1.0)


def timed(fn):
    start = time.perf_counter_ns()
    result = fn()
    return result, (time.perf_counter_ns() - start) / 1e6


def summary(values):
    ordered = sorted(values)
    return dict(count=len(values), p50Ms=statistics.median(values),
                p95Ms=ordered[math.ceil(0.95 * len(values)) - 1])


def peak_rss_bytes():
    if sys.platform != "win32":
        import resource
        value = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
        return value if sys.platform == "darwin" else value * 1024
    import ctypes
    from ctypes import wintypes

    class Counters(ctypes.Structure):
        _fields_ = [("cb", wintypes.DWORD), ("PageFaultCount", wintypes.DWORD)] + [
            (name, ctypes.c_size_t) for name in (
                "PeakWorkingSetSize", "WorkingSetSize", "QuotaPeakPagedPoolUsage",
                "QuotaPagedPoolUsage", "QuotaPeakNonPagedPoolUsage",
                "QuotaNonPagedPoolUsage", "PagefileUsage", "PeakPagefileUsage")]

    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel.GetCurrentProcess.restype = wintypes.HANDLE
    psapi = ctypes.WinDLL("psapi", use_last_error=True)
    psapi.GetProcessMemoryInfo.argtypes = [wintypes.HANDLE, ctypes.POINTER(Counters), wintypes.DWORD]
    psapi.GetProcessMemoryInfo.restype = wintypes.BOOL
    counters = Counters()
    counters.cb = ctypes.sizeof(counters)
    if not psapi.GetProcessMemoryInfo(kernel.GetCurrentProcess(), ctypes.byref(counters), counters.cb):
        raise ctypes.WinError(ctypes.get_last_error())
    return counters.PeakWorkingSetSize


def worker(name, size, updates):
    import pine_compat as pine
    source = SOURCES[name]
    session = pine.create_realtime_session(source)
    _, seed_ms = timed(lambda: session.seed([bar(i) for i in range(size)]))
    replica = session.replica()
    producer, consumer, confirms = [], [], []
    max_series_points = 0
    for i in range(size, size + updates):
        for extra in (0.0, 0.1, 0.2):
            change, elapsed = timed(lambda: session.apply_forming(bar(i, extra)))
            producer.append(elapsed)
            applied, elapsed = timed(lambda: replica.apply(change))
            assert applied
            consumer.append(elapsed)
            for series in change["series"]:
                max_series_points = max(max_series_points, len(series.get("values", [])))
        change, elapsed = timed(lambda: session.apply_confirmed(bar(i, 0.2)))
        confirms.append(elapsed)
        replica.apply(change)
    result, snapshot_ms = timed(replica.result)
    assert result == session.result()
    assert replica.revision == session.revision
    assert max_series_points <= 1
    return dict(workload=name, history=size, updates=updates, formingPerBar=3,
                sourceSha256=hashlib.sha256(source.encode()).hexdigest(),
                module=pine.__file__, version=pine.__version__, seedMs=seed_ms,
                producer=summary(producer), replica=summary(consumer),
                confirm=summary(confirms), snapshotMs=snapshot_ms,
                peakRssBytes=peak_rss_bytes(), maxSeriesPoints=max_series_points,
                finalSnapshotMatches=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--worker", choices=SOURCES)
    parser.add_argument("--history", nargs="+", type=int, default=[1024, 16384, 100000])
    parser.add_argument("--updates", type=int, default=128)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--budget", type=Path)
    args = parser.parse_args()
    if args.updates < 1 or not args.history or min(args.history) < 1:
        parser.error("history and updates must be positive")
    if args.worker:
        print(json.dumps(worker(args.worker, args.history[0], args.updates)))
        return
    if args.output is None:
        parser.error("--output is required")
    budget = json.loads(args.budget.read_text(encoding="utf-8-sig")) if args.budget else None
    results, failures = [], []
    for name in SOURCES:
        for size in args.history:
            output = subprocess.check_output([
                sys.executable, str(Path(__file__).resolve()), "--worker", name,
                "--history", str(size), "--updates", str(args.updates)], text=True)
            result = json.loads(output)
            results.append(result)
            print(f"{name} {size}: update p95={result['producer']['p95Ms']:.3f} ms, "
                  f"replica p95={result['replica']['p95Ms']:.3f} ms", flush=True)
            if budget:
                for metric in ("producer", "replica", "confirm"):
                    if result[metric]["p95Ms"] > budget[metric + "P95Ms"]:
                        failures.append(f"{name}/{size}: {metric} p95 exceeded")
                if result["peakRssBytes"] > budget["peakRssBytes"]:
                    failures.append(f"{name}/{size}: peak RSS exceeded")
        if budget:
            group = results[-len(args.history):]
            smallest = min(group, key=lambda x: x["history"])
            largest = max(group, key=lambda x: x["history"])
            for metric in ("producer", "replica"):
                ratio = largest[metric]["p50Ms"] / smallest[metric]["p50Ms"]
                if ratio > budget["maxP50GrowthRatio"]:
                    failures.append(f"{name}: {metric} growth {ratio:.3f} exceeded")
    report = dict(schemaVersion=1, platform=platform.platform(), python=sys.version,
                  status="failed" if failures else "passed", budget=budget,
                  budgetSha256=hashlib.sha256(args.budget.read_bytes()).hexdigest() if budget else None,
                  results=results, failures=failures,
                  scope="Recorded finite workloads only; no indefinite retention or concurrency SLA")
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + "\n")
    if failures:
        raise SystemExit("; ".join(failures))


if __name__ == "__main__":
    main()

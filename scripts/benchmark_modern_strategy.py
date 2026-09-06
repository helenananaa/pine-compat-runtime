#!/usr/bin/env python3
"""Measure compile, historical run, incremental append, forming replace, and JSON serialization.

Phases are timed separately. This harness records a baseline; it does not claim a
speedup by itself.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import statistics
import subprocess
import sys
import time
from pathlib import Path
from typing import Any, Callable, Mapping, Sequence


SCHEMA_VERSION = 1
TOOL_VERSION = 1
ROOT = Path(__file__).resolve().parents[1]
DEFAULT_SEEDS = (1, 2, 3)
DEFAULT_BAR_COUNTS = (16, 64, 256)
WARMUP = 2
ITERS = 10


class BenchmarkError(ValueError):
    """Raised when the benchmark set or environment cannot be measured."""


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_json(value: Any) -> str:
    return sha256_bytes(json.dumps(value, sort_keys=True, separators=(",", ":")).encode("utf-8"))


def synthetic_bars(count: int, *, seed: int) -> list[dict[str, float | int]]:
    bars: list[dict[str, float | int]] = []
    price = 100.0 + (seed % 7)
    for index in range(count):
        delta = ((index * 17 + seed * 13) % 11) / 10.0
        open_ = price
        close = price + delta - 0.5
        high = max(open_, close) + 0.25
        low = min(open_, close) - 0.25
        bars.append(
            {
                "time": index + 1,
                "open": open_,
                "high": high,
                "low": low,
                "close": close,
                "volume": 100 + (index % 5),
            }
        )
        price = close
    return bars


def median_ms(samples: Sequence[float]) -> dict[str, float]:
    if not samples:
        return {"medianMs": float("nan"), "stdevMs": float("nan"), "n": 0}
    values = [item * 1000.0 for item in samples]
    stdev = statistics.pstdev(values) if len(values) > 1 else 0.0
    return {
        "medianMs": statistics.median(values),
        "stdevMs": stdev,
        "n": len(values),
    }


def time_call(fn: Callable[[], Any], *, warmup: int, iters: int) -> tuple[Any, dict[str, float]]:
    result: Any = None
    for _ in range(warmup):
        result = fn()
    samples: list[float] = []
    for _ in range(iters):
        started = time.perf_counter()
        result = fn()
        samples.append(time.perf_counter() - started)
    return result, median_ms(samples)


def load_source(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except OSError as exc:
        raise BenchmarkError(f"failed to read {path}: {exc}") from exc


def environment_record() -> dict[str, Any]:
    return {
        "python": sys.version.split()[0],
        "platform": platform.platform(),
        "machine": platform.machine(),
        "processor": platform.processor(),
        "pid": os.getpid(),
        "rustc": subprocess.run(
            ["rustc", "--version"], capture_output=True, text=True, check=False
        ).stdout.strip(),
    }


def measure_sample(
    *,
    sample_id: str,
    source: str,
    bars: list[dict[str, float | int]],
    pine_compat: Any,
    warmup: int,
    iters: int,
) -> dict[str, Any]:
    compile_result, compile_stats = time_call(
        lambda: pine_compat.compile_script(source), warmup=warmup, iters=iters
    )
    program = compile_result

    hist_result, hist_stats = time_call(
        lambda: program.run(bars), warmup=warmup, iters=iters
    )

    def incremental() -> Any:
        result = None
        for end in range(1, len(bars) + 1):
            result = program.run(bars[:end])
        return result

    _incr, incr_stats = time_call(
        incremental, warmup=max(1, warmup // 2), iters=max(3, iters // 2)
    )

    if len(bars) < 2:
        forming_phase = {"status": "not_run", "reason": "need_two_bars"}
    else:
        def forming_ops() -> Any:
            session = program.realtime_session()
            session.seed(bars[:-1])
            session.update_forming(bars[-1])
            return session.update_confirmed(bars[-1])

        try:
            _forming, forming_stats = time_call(
                forming_ops, warmup=max(1, warmup // 2), iters=max(3, iters // 2)
            )
            forming_phase = {"status": "measured", **forming_stats}
        except Exception as exc:
            forming_phase = {"status": "not_run", "reason": str(exc)}

    rendered, ser_stats = time_call(
        lambda: json.dumps(hist_result, sort_keys=True),
        warmup=warmup,
        iters=iters,
    )
    return {
        "sampleId": sample_id,
        "barCount": len(bars),
        "resultHash": sha256_json(hist_result),
        "outputBytes": len(rendered.encode("utf-8")),
        "phases": {
            "compile": {"status": "measured", **compile_stats},
            "historicalRun": {"status": "measured", **hist_stats},
            "incrementalAppend": {
                "status": "measured",
                "note": "prefix rerun through Program.run; not VM append_bar",
                **incr_stats,
            },
            "formingReplace": forming_phase,
            "outputSerialization": {"status": "measured", **ser_stats},
        },
    }


def default_samples(root: Path) -> list[dict[str, str]]:
    return [
        {
            "sampleId": "original.runtime.strategy_trade_counts.default",
            "sourcePath": "tests/fixtures/runtime/strategy_trade_counts.pine",
        },
        {
            "sampleId": "original.runtime.strategy_entry.default",
            "sourcePath": "tests/fixtures/runtime/strategy_entry.pine",
        },
        {
            "sampleId": "original.runtime.generic_input.default",
            "sourcePath": "tests/fixtures/runtime/generic_input.pine",
        },
    ]


def build_report(
    *,
    root: Path,
    pine_compat: Any,
    bar_counts: Sequence[int],
    seed: int,
    warmup: int,
    iters: int,
) -> dict[str, Any]:
    samples = []
    for spec in default_samples(root):
        source = load_source(root / spec["sourcePath"])
        for count in bar_counts:
            bars = synthetic_bars(count, seed=seed)
            try:
                samples.append(
                    measure_sample(
                        sample_id=f"{spec['sampleId']}.bars{count}",
                        source=source,
                        bars=bars,
                        pine_compat=pine_compat,
                        warmup=warmup,
                        iters=iters,
                    )
                )
            except Exception as exc:
                samples.append(
                    {
                        "sampleId": f"{spec['sampleId']}.bars{count}",
                        "barCount": count,
                        "status": "failed",
                        "reason": str(exc),
                    }
                )
    return {
        "schemaVersion": SCHEMA_VERSION,
        "toolVersion": TOOL_VERSION,
        "status": "基线完成",
        "seed": seed,
        "barCounts": list(bar_counts),
        "warmup": warmup,
        "iters": iters,
        "environment": environment_record(),
        "optimizationCommitted": False,
        "samples": samples,
    }


def parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--seed", type=int, default=1)
    parser.add_argument("--bars", default="16,64,256")
    parser.add_argument("--warmup", type=int, default=WARMUP)
    parser.add_argument("--iters", type=int, default=ITERS)
    return parser.parse_args(argv)


def main(argv: Sequence[str] | None = None) -> int:
    args = parse_args(argv)
    try:
        import pine_compat
    except ImportError as exc:
        raise SystemExit(
            "modern strategy benchmark error: pine_compat is not importable"
        ) from exc
    bar_counts = [int(item) for item in args.bars.split(",") if item.strip()]
    report = build_report(
        root=args.root.resolve(),
        pine_compat=pine_compat,
        bar_counts=bar_counts,
        seed=args.seed,
        warmup=args.warmup,
        iters=args.iters,
    )
    rendered = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if args.output is None:
        print(rendered, end="")
    else:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered, encoding="utf-8")
        print(f"modern strategy benchmark baseline: wrote {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

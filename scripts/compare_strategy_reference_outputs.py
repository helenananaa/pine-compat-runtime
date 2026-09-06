#!/usr/bin/env python3
"""Compare runtime strategy trades with an independent reference trade list.

Tolerances are frozen in this module. Identity fields compare exactly.
Quantity, price, and profit use a tight numeric band that cannot hide a
missing fill or a one-tick price error. Reference files are normalized here;
the runtime JSON schema is not extended.
"""

from __future__ import annotations

import argparse
import json
import math
import sys
from pathlib import Path
from typing import Any, Mapping, Sequence


SCHEMA_VERSION = 1
TOOL_VERSION = 1

# Frozen before inspecting any target-sample results.
EXACT_FIELDS = (
    "id",
    "entryBarIndex",
    "exitBarIndex",
    "entryTime",
    "exitTime",
)
NUMERIC_FIELDS = (
    "qty",
    "entryPrice",
    "exitPrice",
    "profit",
)
PUBLIC_TRADE_FIELDS = EXACT_FIELDS + NUMERIC_FIELDS
ABSOLUTE_TOLERANCE = 1e-9
RELATIVE_TOLERANCE = 1e-9
UNCOVERED_PUBLIC_FIELDS = ("commission", "direction", "exitId")


class StrategyComparisonError(ValueError):
    """Raised when the artifacts cannot be compared without guessing."""


def load_json(path: Path) -> dict[str, Any]:
    try:
        with path.open(encoding="utf-8") as handle:
            value = json.load(handle)
    except (OSError, json.JSONDecodeError) as exc:
        raise StrategyComparisonError(f"failed to read JSON {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise StrategyComparisonError(f"{path} root must be an object")
    return value


def extract_trades(payload: Mapping[str, Any], *, label: str) -> list[dict[str, Any]]:
    if "strategy" in payload and isinstance(payload["strategy"], Mapping):
        raw = payload["strategy"].get("trades")
    else:
        raw = payload.get("trades")
    if raw is None:
        raise StrategyComparisonError(f"{label} has no trades array")
    if not isinstance(raw, list):
        raise StrategyComparisonError(f"{label} trades must be an array")
    trades: list[dict[str, Any]] = []
    for index, item in enumerate(raw):
        if not isinstance(item, Mapping):
            raise StrategyComparisonError(f"{label} trade {index} must be an object")
        trades.append(dict(item))
    return trades


def numeric_equal(actual: float, expected: float) -> bool:
    if not math.isfinite(actual) or not math.isfinite(expected):
        return False
    delta = abs(actual - expected)
    scale = max(abs(actual), abs(expected), 1.0)
    return delta <= ABSOLUTE_TOLERANCE or delta <= RELATIVE_TOLERANCE * scale


def field_present(trade: Mapping[str, Any], name: str) -> bool:
    return name in trade and trade[name] is not None


def compare_field(
    name: str, actual: Mapping[str, Any], expected: Mapping[str, Any]
) -> dict[str, Any] | None:
    if name not in actual or name not in expected:
        return {
            "field": name,
            "reason": "missing_field",
            "actual": actual.get(name),
            "expected": expected.get(name),
        }
    left = actual[name]
    right = expected[name]
    if name in EXACT_FIELDS:
        if left != right:
            return {
                "field": name,
                "reason": "identity_mismatch",
                "actual": left,
                "expected": right,
            }
        return None
    if name in NUMERIC_FIELDS:
        try:
            left_n = float(left)
            right_n = float(right)
        except (TypeError, ValueError):
            return {
                "field": name,
                "reason": "non_numeric",
                "actual": left,
                "expected": right,
            }
        if not numeric_equal(left_n, right_n):
            return {
                "field": name,
                "reason": "numeric_mismatch",
                "actual": left_n,
                "expected": right_n,
                "absoluteError": abs(left_n - right_n),
            }
        return None
    return {
        "field": name,
        "reason": "uncovered_field",
        "actual": left,
        "expected": right,
    }


def duplicate_identity_indexes(trades: Sequence[Mapping[str, Any]]) -> list[int]:
    seen: dict[tuple[Any, ...], int] = {}
    duplicates: list[int] = []
    for index, trade in enumerate(trades):
        key = tuple(trade.get(name) for name in EXACT_FIELDS)
        if key in seen:
            duplicates.append(index)
        else:
            seen[key] = index
    return duplicates


def same_time_ambiguity(trades: Sequence[Mapping[str, Any]]) -> list[int]:
    groups: dict[tuple[Any, Any], list[int]] = {}
    for index, trade in enumerate(trades):
        key = (trade.get("entryTime"), trade.get("exitTime"))
        groups.setdefault(key, []).append(index)
    ambiguous: list[int] = []
    for indexes in groups.values():
        if len(indexes) < 2:
            continue
        identities = [
            tuple(trades[index].get(name) for name in ("id", "qty", "entryPrice", "exitPrice"))
            for index in indexes
        ]
        if len(set(identities)) != len(identities):
            ambiguous.extend(indexes)
    return ambiguous


def compare_reference(
    actual: Mapping[str, Any],
    reference: Mapping[str, Any],
    fields: Sequence[str],
) -> dict[str, Any]:
    declared = [str(name) for name in fields] if fields else list(PUBLIC_TRADE_FIELDS)
    uncovered = [name for name in declared if name in UNCOVERED_PUBLIC_FIELDS]
    if uncovered:
        return {
            "status": "incomparable",
            "reason": "uncovered_fields",
            "comparable": False,
            "coverage": [name for name in declared if name not in uncovered],
            "uncoveredFields": uncovered,
            "mismatches": None,
            "firstDifference": {
                "reason": "uncovered_fields",
                "fields": uncovered,
            },
        }

    try:
        actual_trades = extract_trades(actual, label="runtime")
        reference_trades = extract_trades(reference, label="reference")
    except StrategyComparisonError as exc:
        return {
            "status": "failed",
            "reason": str(exc),
            "comparable": False,
            "coverage": declared,
            "mismatches": None,
            "firstDifference": {"reason": str(exc)},
        }

    if not reference_trades and actual_trades:
        return {
            "status": "failed",
            "reason": "empty_reference_trades",
            "comparable": True,
            "coverage": declared,
            "mismatches": len(actual_trades),
            "firstDifference": {
                "reason": "empty_reference_trades",
                "actualCount": len(actual_trades),
            },
        }
    if reference_trades and not actual_trades:
        return {
            "status": "failed",
            "reason": "empty_runtime_trades",
            "comparable": True,
            "coverage": declared,
            "mismatches": len(reference_trades),
            "firstDifference": {
                "reason": "empty_runtime_trades",
                "expectedCount": len(reference_trades),
            },
        }

    actual_dups = duplicate_identity_indexes(actual_trades)
    reference_dups = duplicate_identity_indexes(reference_trades)
    if actual_dups or reference_dups:
        return {
            "status": "failed",
            "reason": "duplicate_trade_identity",
            "comparable": True,
            "coverage": declared,
            "mismatches": len(actual_dups) + len(reference_dups),
            "firstDifference": {
                "reason": "duplicate_trade_identity",
                "actualIndexes": actual_dups,
                "referenceIndexes": reference_dups,
            },
        }

    ambiguous = same_time_ambiguity(reference_trades) or same_time_ambiguity(actual_trades)
    if ambiguous:
        return {
            "status": "incomparable",
            "reason": "ambiguous_same_time_order",
            "comparable": False,
            "coverage": declared,
            "mismatches": None,
            "firstDifference": {
                "reason": "ambiguous_same_time_order",
                "indexes": ambiguous,
            },
        }

    coverage = [
        name
        for name in declared
        if name in PUBLIC_TRADE_FIELDS
        and all(field_present(trade, name) for trade in reference_trades)
    ]
    missing_declared = [name for name in declared if name not in coverage]
    if missing_declared:
        return {
            "status": "incomparable",
            "reason": "partial_reference",
            "comparable": False,
            "coverage": coverage,
            "mismatches": None,
            "firstDifference": {
                "reason": "partial_reference",
                "missingFields": missing_declared,
            },
        }

    mismatches: list[dict[str, Any]] = []
    limit = max(len(actual_trades), len(reference_trades))
    for index in range(limit):
        if index >= len(actual_trades):
            mismatches.append(
                {
                    "index": index,
                    "reason": "missing_fill",
                    "expected": reference_trades[index],
                }
            )
            continue
        if index >= len(reference_trades):
            mismatches.append(
                {
                    "index": index,
                    "reason": "duplicate_or_extra_fill",
                    "actual": actual_trades[index],
                }
            )
            continue
        for name in coverage:
            diff = compare_field(name, actual_trades[index], reference_trades[index])
            if diff is not None:
                mismatches.append({"index": index, **diff})
                break

    first = mismatches[0] if mismatches else None
    return {
        "status": "passed" if not mismatches else "failed",
        "reason": "compared" if not mismatches else first["reason"] if first else "mismatch",
        "comparable": True,
        "coverage": coverage,
        "mismatches": len(mismatches),
        "firstDifference": first,
        "differences": mismatches,
        "actualCount": len(actual_trades),
        "referenceCount": len(reference_trades),
        "absoluteTolerance": ABSOLUTE_TOLERANCE,
        "relativeTolerance": RELATIVE_TOLERANCE,
    }


def render_report(report: Mapping[str, Any]) -> str:
    return json.dumps(report, indent=2, sort_keys=True) + "\n"


def parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("runtime_json", type=Path)
    parser.add_argument("reference_json", type=Path)
    parser.add_argument(
        "--fields",
        default=",".join(PUBLIC_TRADE_FIELDS),
        help="comma-separated declared compare fields",
    )
    parser.add_argument("--format", choices=("text", "json"), default="json")
    return parser.parse_args(argv)


def main(argv: Sequence[str] | None = None) -> int:
    args = parse_args(argv)
    fields = [item.strip() for item in args.fields.split(",") if item.strip()]
    try:
        report = compare_reference(
            load_json(args.runtime_json),
            load_json(args.reference_json),
            fields,
        )
    except StrategyComparisonError as exc:
        raise SystemExit(f"strategy compare error: {exc}") from exc
    if args.format == "json":
        print(render_report(report), end="")
    else:
        print(
            f"{report['status']}: {report.get('reason')} "
            f"mismatches={report.get('mismatches')} "
            f"first={report.get('firstDifference')}"
        )
    return 0 if report.get("status") == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())

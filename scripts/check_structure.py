#!/usr/bin/env python3
"""Lightweight structural guardrails for Rust source files."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[1]

FACADE_MAX_LINES = 300
MODEL_HELPER_MAX_LINES = 800
IMPLEMENTATION_MAX_LINES = 1_500

# Keep recently split implementation hubs below their old growth ceiling. These
# budgets are deliberately tighter than the generic implementation threshold so
# new responsibilities land in the focused child modules introduced for them.
HOTSPOT_MAX_LINES: dict[str, int] = {
    "crates/pine-sema/src/analyzer/context.rs": 500,
    "crates/pine-sema/src/lowering/mod.rs": 1_200,
    "crates/pine-sema/src/lowering/pure_series.rs": 1_100,
    "crates/pine-sema/src/modules.rs": 1_200,
}


@dataclass(frozen=True)
class AllowlistEntry:
    max_lines: int
    owner: str
    reason: str
    split_plan: str


ALLOWLIST: dict[str, AllowlistEntry] = {
    "crates/pine-ir/src/lib.rs": AllowlistEntry(
        max_lines=320,
        owner="pine-ir",
        reason="public HIR declarations exceed the generic facade limit by six lines",
        split_plan="move remaining HIR model declarations into focused model modules",
    ),
    "crates/pine-python/src/lib.rs": AllowlistEntry(
        max_lines=820,
        owner="pine-python",
        reason="thin PyO3 binding surface still lives in the crate root",
        split_plan="move JSON formatting and conversion helpers out of lib.rs during binding boundary cleanup",
    ),
    "crates/pine-runtime/src/builtins/requests.rs": AllowlistEntry(
        max_lines=1_800,
        owner="pine-runtime",
        reason="request evaluation keeps provider, alignment, and realtime context handling together",
        split_plan="extract lower-timeframe evaluation and provider alignment into focused request modules",
    ),
    "crates/pine-sema/src/analyzer/calls.rs": AllowlistEntry(
        max_lines=1_525,
        owner="pine-sema",
        reason="semantic call binding exceeds the generic implementation limit by eight lines",
        split_plan="separate builtin and user-function call binding into focused modules",
    ),
    "crates/pine-sema/src/analyzer/expressions/type_queries.rs": AllowlistEntry(
        max_lines=1_525,
        owner="pine-sema",
        reason="expression type queries exceed the generic implementation limit by three lines",
        split_plan="extract container and legacy type queries into focused modules",
    ),
    "crates/pine-syntax/src/lexer.rs": AllowlistEntry(
        max_lines=1_525,
        owner="pine-syntax",
        reason="lexer exceeds the generic implementation limit by two lines",
        split_plan="extract literal scanning into a focused lexer module",
    ),
    "crates/pine-builtins/src/namespaces/ta.rs": AllowlistEntry(
        max_lines=IMPLEMENTATION_MAX_LINES,
        owner="pine-builtins",
        reason="table-heavy TA semantic signature registry",
        split_plan="split into TA family signature modules if it approaches 1,500 lines",
    ),
}


HELPER_PATH_PARTS = {
    "algorithms",
    "constants",
    "types",
}

HELPER_FILENAMES = {
    "bar.rs",
    "diagnostic.rs",
    "error.rs",
    "model.rs",
    "profile.rs",
    "registry.rs",
    "retention.rs",
    "returns.rs",
    "series.rs",
    "signature.rs",
    "source.rs",
    "value.rs",
}


def is_test_support_file(path: Path) -> bool:
    posix = path.as_posix()
    return (
        "/src/tests/" in posix
        or path.name == "tests.rs"
        or path.name.endswith("_tests.rs")
    )


def rust_source_files() -> list[Path]:
    crates_dir = ROOT / "crates"
    return sorted(
        path
        for path in crates_dir.glob("*/src/**/*.rs")
        if not is_test_support_file(path)
    )


def line_count(path: Path) -> int:
    return len(path.read_text(encoding="utf-8").splitlines())


def relative(path: Path) -> str:
    return path.relative_to(ROOT).as_posix()


def threshold_for(path: Path) -> tuple[int, str]:
    rel = relative(path)
    if rel in ALLOWLIST:
        return ALLOWLIST[rel].max_lines, "allowlisted"
    if rel in HOTSPOT_MAX_LINES:
        return HOTSPOT_MAX_LINES[rel], "split hotspot"
    if path.name == "lib.rs":
        return FACADE_MAX_LINES, "facade"
    if path.name in HELPER_FILENAMES:
        return MODEL_HELPER_MAX_LINES, "model/helper"
    if any(part in HELPER_PATH_PARTS for part in path.parts):
        return MODEL_HELPER_MAX_LINES, "model/helper"
    return IMPLEMENTATION_MAX_LINES, "implementation"


def validate_allowlist() -> list[str]:
    errors: list[str] = []
    for rel, entry in ALLOWLIST.items():
        path = ROOT / rel
        if not path.exists():
            errors.append(f"allowlist entry does not exist: {rel}")
        if not entry.owner.strip():
            errors.append(f"allowlist entry lacks owner: {rel}")
        if not entry.reason.strip():
            errors.append(f"allowlist entry lacks reason: {rel}")
        if not entry.split_plan.strip():
            errors.append(f"allowlist entry lacks split plan: {rel}")
    return errors


def main() -> int:
    errors = validate_allowlist()
    checked = 0
    largest: list[tuple[int, str, str, int]] = []

    for path in rust_source_files():
        checked += 1
        count = line_count(path)
        threshold, category = threshold_for(path)
        rel = relative(path)
        largest.append((count, rel, category, threshold))
        if count > threshold:
            errors.append(
                f"{rel}: {count} lines exceeds {category} threshold {threshold}"
            )

    if errors:
        print("Structural guardrail failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        print(
            "\nSplit the file, or add a documented allowlist entry with owner, reason, and split plan.",
            file=sys.stderr,
        )
        return 1

    largest.sort(reverse=True)
    print(
        f"Structural guardrail passed: checked {checked} production Rust source files."
    )
    for count, rel, category, threshold in largest[:5]:
        print(f"- {rel}: {count}/{threshold} lines ({category})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

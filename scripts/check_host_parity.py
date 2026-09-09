#!/usr/bin/env python3
"""Guard the explicit CLI/Python/WASM runtime and legacy-analysis parity baselines."""

from __future__ import annotations

import ast
import re
import sys
from collections import Counter
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
FIXTURE_DIR = ROOT / "crates/pine-cli/src/runtime_snapshots/fixtures"
LIBRARY_FIXTURE_FILE = FIXTURE_DIR.with_suffix(".rs")
REQUIRED_MANIFEST = ROOT / "scripts/host_parity_required.txt"
ANALYSIS_FIXTURE_FILE = ROOT / "crates/pine-cli/src/analysis_snapshots.rs"
ANALYSIS_REQUIRED_MANIFEST = ROOT / "scripts/legacy_analysis_parity_required.txt"
WASM_TESTS = ROOT / "crates/pine-wasm/src/tests/mod.rs"
PYTHON_TESTS = ROOT / "python/tests/test_bindings.py"
CLI_DIRECT_RUNTIME_TESTS = ROOT / "crates/pine-cli/src/commands/run/tests.rs"

# A registered CLI runtime snapshot should never silently remain single-host.
# Keep exceptions explicit and reasoned; the normal state is an empty mapping.
UNPAIRED_REGISTERED_ALLOWLIST: dict[str, str] = {}

# rustfmt expands most fixture tuples and leaves a trailing comma before `)`.
# Keep the parser independent of whitespace and of that optional final comma.
SNAPSHOT_FIXTURE = re.compile(
    r'\(\s*"([^"]+\.json)"\s*,\s*"([^"]+\.pine)"\s*,?\s*\)',
    re.DOTALL,
)
LIBRARY_SNAPSHOT_FIXTURE = re.compile(
    r'\(\s*"([^"\n]+\.json)"\s*,\s*"([^"\n]+\.pine)"\s*,\s*&\['
    r'(?:\s*\(\s*"[^"\n]+"\s*,\s*"[^"\n]+\.pine"\s*,?\s*\)\s*,?)*'
    r'\s*\]\s*,?\s*\)',
    re.DOTALL,
)
PYTHON_SNAPSHOT_PATH = re.compile(
    r'(?:^|/)tests/snapshots/([^/]+\.json)$'
)
PYTHON_GOLDEN_ASSERTION_HELPERS = {"assert_json_close", "assert_analysis_snapshot"}
WASM_GOLDEN_ASSERTION_HELPERS = {"assert_snapshot", "assert_analysis_snapshot"}


@dataclass(frozen=True)
class RuntimeSnapshotFixture:
    path: Path
    snapshot: str
    source: str


def parse_runtime_snapshot_fixtures(
    text: str, path: Path
) -> list[RuntimeSnapshotFixture]:
    return [
        RuntimeSnapshotFixture(path, snapshot, source)
        for snapshot, source in SNAPSHOT_FIXTURE.findall(text)
    ]


def runtime_snapshot_fixtures() -> list[RuntimeSnapshotFixture]:
    fixtures: list[RuntimeSnapshotFixture] = []
    for path in sorted(FIXTURE_DIR.glob("*.rs")):
        fixtures.extend(parse_runtime_snapshot_fixtures(path.read_text(), path))
    fixtures.extend(parse_library_snapshot_fixtures(
        LIBRARY_FIXTURE_FILE.read_text(), LIBRARY_FIXTURE_FILE
    ))
    return fixtures


def parse_library_snapshot_fixtures(
    text: str, path: Path
) -> list[RuntimeSnapshotFixture]:
    """Read CLI library triples, including their complete dependency list."""
    fixtures: list[RuntimeSnapshotFixture] = []
    index = 0
    while index < len(text):
        next_code = _skip_rust_trivia(text, index)
        if next_code != index:
            index = next_code
            continue
        raw_end = _rust_raw_string_end(text, index)
        if raw_end is not None:
            index = raw_end
            continue
        if text[index] == '"':
            index = _skip_rust_quoted(text, index, '"')
            continue
        if text[index] == "'":
            char_end = _rust_char_end(text, index)
            if char_end is not None:
                index = char_end
                continue
        match = LIBRARY_SNAPSHOT_FIXTURE.match(text, index)
        if match is not None:
            fixtures.append(RuntimeSnapshotFixture(path, *match.groups()))
            index = match.end()
        else:
            index += 1
    return fixtures


def analysis_snapshot_fixtures() -> list[RuntimeSnapshotFixture]:
    return parse_runtime_snapshot_fixtures(
        ANALYSIS_FIXTURE_FILE.read_text(), ANALYSIS_FIXTURE_FILE
    )


def parse_required_manifest(text: str) -> tuple[list[str], list[str]]:
    snapshots = [
        line.strip()
        for line in text.splitlines()
        if line.strip() and not line.lstrip().startswith("#")
    ]
    errors: list[str] = []
    duplicates = sorted(name for name, count in Counter(snapshots).items() if count > 1)
    if duplicates:
        errors.append("required manifest has duplicate entries: " + ", ".join(duplicates))
    if snapshots != sorted(snapshots):
        errors.append("required manifest entries must be sorted")
    return snapshots, errors


def _skip_rust_block_comment(text: str, start: int) -> int:
    depth = 1
    index = start + 2
    while index < len(text) and depth:
        if text.startswith("/*", index):
            depth += 1
            index += 2
        elif text.startswith("*/", index):
            depth -= 1
            index += 2
        else:
            index += 1
    return index


def _skip_rust_quoted(text: str, start: int, quote: str) -> int:
    index = start + 1
    while index < len(text):
        if text[index] == "\\":
            index += 2
        elif text[index] == quote:
            return index + 1
        else:
            index += 1
    return index


def _rust_char_end(text: str, start: int) -> int | None:
    index = start + 1
    if index >= len(text) or text[index] in {"'", "\n", "\r"}:
        return None
    if text[index] == "\\":
        index += 1
        if index >= len(text):
            return None
        if text[index] == "u" and index + 1 < len(text) and text[index + 1] == "{":
            close = text.find("}", index + 2)
            if close < 0:
                return None
            index = close + 1
        else:
            index += 1
    else:
        index += 1
    return index + 1 if index < len(text) and text[index] == "'" else None


def _rust_raw_string_end(text: str, start: int) -> int | None:
    index = start
    if text.startswith("br", index):
        index += 2
    elif text.startswith("r", index):
        index += 1
    else:
        return None

    hashes = 0
    while index < len(text) and text[index] == "#":
        hashes += 1
        index += 1
    if index >= len(text) or text[index] != '"':
        return None

    terminator = '"' + "#" * hashes
    end = text.find(terminator, index + 1)
    return len(text) if end < 0 else end + len(terminator)


def _skip_rust_trivia(text: str, start: int) -> int:
    index = start
    while index < len(text):
        if text[index].isspace():
            index += 1
        elif text.startswith("//", index):
            newline = text.find("\n", index + 2)
            index = len(text) if newline < 0 else newline + 1
        elif text.startswith("/*", index):
            index = _skip_rust_block_comment(text, index)
        else:
            break
    return index


def _parse_rust_plain_string(text: str, start: int) -> tuple[str, int] | None:
    if start >= len(text) or text[start] != '"':
        return None
    end = _skip_rust_quoted(text, start, '"')
    if end > len(text) or end <= start + 1 or text[end - 1] != '"':
        return None
    try:
        value = ast.literal_eval(text[start:end])
    except (SyntaxError, ValueError):
        return None
    return (value, end) if isinstance(value, str) else None


def wasm_snapshot_assertions(text: str) -> set[str]:
    """Find real assert_snapshot calls while ignoring Rust comments and strings."""

    asserted: set[str] = set()
    index = 0
    while index < len(text):
        if text.startswith("//", index):
            newline = text.find("\n", index + 2)
            index = len(text) if newline < 0 else newline + 1
            continue
        if text.startswith("/*", index):
            index = _skip_rust_block_comment(text, index)
            continue

        raw_end = _rust_raw_string_end(text, index)
        if raw_end is not None:
            index = raw_end
            continue
        if text.startswith('b"', index):
            index = _skip_rust_quoted(text, index + 1, '"')
            continue
        if text[index] == '"':
            index = _skip_rust_quoted(text, index, '"')
            continue
        if text[index] == "'":
            char_end = _rust_char_end(text, index)
            if char_end is not None:
                index = char_end
                continue

        if text[index].isalpha() or text[index] == "_":
            end = index + 1
            while end < len(text) and (text[end].isalnum() or text[end] == "_"):
                end += 1
            if text[index:end] in WASM_GOLDEN_ASSERTION_HELPERS:
                call_start = _skip_rust_trivia(text, end)
                if call_start < len(text) and text[call_start] == "(":
                    argument_start = _skip_rust_trivia(text, call_start + 1)
                    parsed = _parse_rust_plain_string(text, argument_start)
                    if parsed is not None:
                        snapshot, argument_end = parsed
                        comma = _skip_rust_trivia(text, argument_end)
                        if comma < len(text) and text[comma] == "," and snapshot.endswith(".json"):
                            asserted.add(snapshot)
            index = end
            continue

        index += 1

    return asserted


def _python_snapshot_names(node: ast.AST) -> set[str]:
    snapshots: set[str] = set()
    for child in ast.walk(node):
        if not isinstance(child, ast.Constant) or not isinstance(child.value, str):
            continue
        match = PYTHON_SNAPSHOT_PATH.search(child.value)
        if match:
            snapshots.add(match.group(1))
    return snapshots


def python_snapshot_assertions(text: str) -> set[str]:
    tree = ast.parse(text)
    asserted: set[str] = set()

    for function in (
        node
        for node in tree.body
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
        and node.name.startswith("test_")
    ):
        snapshot_aliases: dict[str, set[str]] = {}
        for node in ast.walk(function):
            if not isinstance(node, (ast.Assign, ast.AnnAssign)):
                continue
            snapshots = _python_snapshot_names(node.value)
            if not snapshots:
                continue
            targets = node.targets if isinstance(node, ast.Assign) else [node.target]
            for target in targets:
                for name in ast.walk(target):
                    if isinstance(name, ast.Name):
                        snapshot_aliases[name.id] = snapshots

        for node in ast.walk(function):
            assertion: ast.AST | None = None
            if isinstance(node, ast.Assert):
                assertion = node.test
            elif (
                isinstance(node, ast.Call)
                and isinstance(node.func, ast.Name)
                and node.func.id in PYTHON_GOLDEN_ASSERTION_HELPERS
            ):
                assertion = node
            if assertion is None:
                continue
            asserted.update(_python_snapshot_names(assertion))
            for name in ast.walk(assertion):
                if isinstance(name, ast.Name):
                    asserted.update(snapshot_aliases.get(name.id, set()))

    return asserted


def parity_errors(
    registered: set[str],
    required: set[str],
    wasm_assertions: set[str],
    python_assertions: set[str],
    unpaired_allowlist: dict[str, str] | None = None,
) -> list[str]:
    errors: list[str] = []
    allowlist = unpaired_allowlist or {}

    for snapshot in sorted(required - registered):
        errors.append(f"required snapshot is not registered by the CLI: {snapshot}")

    for snapshot in sorted(required):
        missing_hosts = []
        if snapshot not in wasm_assertions:
            missing_hosts.append("WASM")
        if snapshot not in python_assertions:
            missing_hosts.append("Python")
        if missing_hosts:
            errors.append(
                f"required snapshot {snapshot} is missing a "
                + ", ".join(missing_hosts)
                + " golden assertion"
            )

    single_host_registered = registered & (wasm_assertions ^ python_assertions)
    ordinary_single_host = single_host_registered - required
    for snapshot in sorted(ordinary_single_host - allowlist.keys()):
        host = "WASM" if snapshot in wasm_assertions else "Python"
        errors.append(
            f"registered snapshot {snapshot} has only a {host} golden assertion"
        )

    for snapshot, reason in sorted(allowlist.items()):
        if not reason.strip():
            errors.append(
                f"unpaired snapshot allowlist entry {snapshot} must include a reason"
            )
        if snapshot not in registered:
            errors.append(
                f"unpaired snapshot allowlist entry is not registered by the CLI: {snapshot}"
            )
        elif snapshot in required:
            errors.append(
                f"required snapshot cannot be exempted from host parity: {snapshot}"
            )
        elif snapshot not in ordinary_single_host:
            errors.append(
                f"unpaired snapshot allowlist entry is stale: {snapshot}"
            )

    # Paired assertions are policy, not an accidental side effect. Requiring
    # them to be recorded keeps `registered` and `required` honest and makes a
    # newly paired fixture an explicit baseline change.
    paired_registered = registered & wasm_assertions & python_assertions
    for snapshot in sorted(paired_registered - required):
        errors.append(
            f"paired host snapshot is not recorded in the required manifest: {snapshot}"
        )

    return errors


def main() -> int:
    fixtures = runtime_snapshot_fixtures()
    registered_names = [fixture.snapshot for fixture in fixtures]
    registered_names.extend(
        sorted(wasm_snapshot_assertions(CLI_DIRECT_RUNTIME_TESTS.read_text()))
    )
    registered = set(registered_names)
    required_names, errors = parse_required_manifest(REQUIRED_MANIFEST.read_text())
    required = set(required_names)
    analysis_fixtures = analysis_snapshot_fixtures()
    analysis_registered_names = [fixture.snapshot for fixture in analysis_fixtures]
    analysis_registered = set(analysis_registered_names)
    analysis_required_names, analysis_manifest_errors = parse_required_manifest(
        ANALYSIS_REQUIRED_MANIFEST.read_text()
    )
    errors.extend(analysis_manifest_errors)
    analysis_required = set(analysis_required_names)

    duplicate_registered = sorted(
        name for name, count in Counter(registered_names).items() if count > 1
    )
    if duplicate_registered:
        errors.append(
            "CLI snapshot registry has duplicate entries: "
            + ", ".join(duplicate_registered)
        )

    duplicate_analysis_registered = sorted(
        name
        for name, count in Counter(analysis_registered_names).items()
        if count > 1
    )
    if duplicate_analysis_registered:
        errors.append(
            "CLI analysis snapshot registry has duplicate entries: "
            + ", ".join(duplicate_analysis_registered)
        )
    collisions = sorted(registered & analysis_registered)
    if collisions:
        errors.append(
            "runtime and analysis snapshot registries overlap: " + ", ".join(collisions)
        )

    wasm_assertions = wasm_snapshot_assertions(WASM_TESTS.read_text())
    python_assertions = python_snapshot_assertions(PYTHON_TESTS.read_text())

    errors.extend(
        parity_errors(
            registered,
            required,
            wasm_assertions,
            python_assertions,
            UNPAIRED_REGISTERED_ALLOWLIST,
        )
    )
    errors.extend(
        parity_errors(
            analysis_registered,
            analysis_required,
            wasm_assertions,
            python_assertions,
        )
    )

    if errors:
        print("Host parity guard failed:")
        for error in errors:
            print(f"- {error}")
        return 1

    print(
        "Host parity guard passed: "
        f"found {len(registered)} registered CLI runtime snapshots; "
        f"verified {len(required)} required runtime and "
        f"{len(analysis_required)} required legacy-analysis Python/WASM "
        "golden assertions."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())

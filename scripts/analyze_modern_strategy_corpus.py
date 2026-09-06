#!/usr/bin/env python3
"""Measure a frozen v5/v6 strategy corpus by independent pipeline stages.

Parse is taken from `fmt-ast` diagnostics, not from the process exit code.
Semantic analysis uses `analyze`. Runtime uses `run` only when analysis passed
and required inputs exist. Output comparability and result consistency are
separate metrics; a missing comparator is `not_run`/`tool_unavailable`, never
consistency-pass.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import time
from collections import Counter, defaultdict
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Callable, Iterable, Mapping, Sequence


SCHEMA_VERSION = 1
TOOL_VERSION = 1
MANIFEST_SCHEMA_VERSION = 1
ROOT = Path(__file__).resolve().parents[1]
DEFAULT_SYNTHETIC_BARS = "tests/fixtures/runtime/bars.csv"
CORPUS_REVISION = "modern-strategy-r1"

SAMPLE_CATEGORIES = {
    "modern_strategy",
    "original_regression",
    "negative_control",
    "excluded",
}
LICENSE_CLASSES = {
    "original",
    "user_owned",
    "permissive",
    "private_user_authorized",
}
PINE_VERSIONS = {5, 6}

STAGE_PASSED = "passed"
STAGE_FAILED = "failed"
STAGE_NOT_RUN = "not_run"
STAGE_EXCLUDED = "excluded"
STAGE_MISSING_INPUT = "missing_input"
STAGE_STATUSES = {
    STAGE_PASSED,
    STAGE_FAILED,
    STAGE_NOT_RUN,
    STAGE_EXCLUDED,
    STAGE_MISSING_INPUT,
}

DIAGNOSTIC_RE = re.compile(
    r"^(?P<code>[EW]_[A-Z0-9_]+):(?P<severity>[A-Za-z]+):"
    r"(?P<line>[0-9]+):(?P<column>[0-9]+): (?P<message>.*)$"
)
VERSION_RE = re.compile(
    r"(?m)^[ \t]*//@version[ \t]*=[ \t]*(?P<version>[0-9]+)[ \t]*\r?$"
)
DECLARATION_RE = re.compile(r"(?m)^\s*(?P<mode>study|indicator|strategy)\s*\(")
IMPORT_RE = re.compile(
    r"(?m)^\s*import\s+(?P<key>[^\s]+)\s+as\s+(?P<alias>[A-Za-z_][A-Za-z0-9_]*)"
)
KNOWN_LIBRARY_SOURCES = {
    "user/udt/1": "tests/fixtures/libraries/import_udt_lib.pine",
}
MAGNIFIER_TRUE_RE = re.compile(r"use_bar_magnifier\s*=\s*true")
RUST_PINE_RE = re.compile(r'"(tests/fixtures/[^"]+\.pine)"')
RUST_CSV_RE = re.compile(
    r'include_str!\(\s*"(?:\.\./)+(?P<path>tests/fixtures/[^"]+\.csv)"'
)
SUBJECT_PATTERNS = (
    re.compile(r"unknown function `(?P<subject>[^`]+)`"),
    re.compile(r"unknown symbol `(?P<subject>[^`]+)`"),
    re.compile(r"unknown (?:member|method) `(?P<subject>[^`]+)`"),
    re.compile(r"unsupported (?:feature|call) `(?P<subject>[^`]+)`"),
    re.compile(r"`(?P<subject>[^`]+)` is not supported"),
)

DEFAULT_COMPARE_FIELDS = (
    "id",
    "exitId",
    "qty",
    "entryPrice",
    "exitPrice",
    "entryTime",
    "exitTime",
    "entryBarIndex",
    "exitBarIndex",
    "profit",
)

ELIGIBLE_CATEGORIES = {"modern_strategy", "original_regression"}


class CorpusError(ValueError):
    """Raised when the modern-strategy manifest or an input bundle is malformed."""


@dataclass(frozen=True)
class Missing:
    status: str
    reason: str
    value: Any = None

    def as_dict(self) -> dict[str, Any]:
        payload: dict[str, Any] = {"status": self.status, "reason": self.reason}
        if self.value is not None:
            payload["value"] = self.value
        return payload


@dataclass(frozen=True)
class DiagnosticRecord:
    code: str
    severity: str
    line: int
    column: int
    message: str
    subject: str | None
    root_cause_class: str

    def as_dict(self, stage: str) -> dict[str, Any]:
        payload: dict[str, Any] = {
            "stage": stage,
            "code": self.code,
            "severity": self.severity.lower(),
            "line": self.line,
            "column": self.column,
            "message": self.message,
            "rootCauseClass": self.root_cause_class,
        }
        if self.subject is not None:
            payload["subject"] = self.subject
        return payload


@dataclass
class CommandResult:
    argv: list[str]
    returncode: int
    stdout: str
    stderr: str
    timed_out: bool
    duration_ms: int

    def as_dict(self, *, keep_output: bool = False) -> dict[str, Any]:
        payload: dict[str, Any] = {
            "argv": self.argv,
            "returnCode": self.returncode,
            "timedOut": self.timed_out,
            "durationMs": self.duration_ms,
        }
        if keep_output:
            payload["stdout"] = self.stdout
            payload["stderr"] = self.stderr
        else:
            payload["stdoutExcerpt"] = excerpt_output(self.stdout)
            payload["stderrExcerpt"] = excerpt_output(self.stderr)
        return payload


@dataclass
class Sample:
    raw: dict[str, Any]
    sample_id: str
    script_id: str
    scenario_id: str
    sample_category: str
    pine_version: int
    source_path: str
    source_sha256: str
    ohlcv_path: str
    ohlcv_sha256: str
    license_class: str
    source_modified: bool
    input_overrides: list[dict[str, str]]
    library_sources: list[dict[str, str]]
    magnifier_path: str
    magnifier_sha256: str
    session_windows_path: str
    session_windows_sha256: str
    execution_times_path: str
    execution_times_sha256: str
    request_data: list[dict[str, str]]
    reference_path: str
    reference_sha256: str
    reference_status: str
    reference_fields: list[str]
    reference_reason: str
    timeout_seconds: float
    expected_support_boundary: str
    coverage_kind: str


CommandRunner = Callable[[Sequence[str], Path, float], CommandResult]
CompareFn = Callable[[dict[str, Any], dict[str, Any], Sequence[str]], dict[str, Any]]


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def sha256_text(value: str) -> str:
    return sha256_bytes(value.encode("utf-8"))


def excerpt_output(text: str, limit: int = 4000) -> str:
    diagnostic_lines = [
        line for line in text.splitlines() if DIAGNOSTIC_RE.match(line.strip())
    ]
    if diagnostic_lines:
        joined = "\n".join(diagnostic_lines)
        if len(joined) <= limit:
            return joined
        return joined[:limit]
    if len(text) <= limit:
        return text
    return text[:limit]


def detected_version(source: str) -> int:
    match = VERSION_RE.search(source)
    return int(match.group("version")) if match else 1


def detected_mode(source: str) -> str:
    match = DECLARATION_RE.search(source)
    return match.group("mode") if match else "unknown"


def is_syntax_diagnostic(code: str) -> bool:
    return (
        code.startswith("E_LEX_")
        or code.startswith("E_PARSE_")
        or code in {"E_LANGUAGE_VERSION_DUPLICATE", "E_LANGUAGE_VERSION_PLACEMENT"}
    )


def diagnostic_subject(message: str) -> str | None:
    for pattern in SUBJECT_PATTERNS:
        match = pattern.search(message)
        if match is not None:
            return match.group("subject")
    return None


def classify_root_cause(code: str, message: str, stage: str) -> str:
    lowered = message.lower()
    if code.startswith("E_LEX_") or code.startswith("E_PARSE_"):
        return "language_type"
    if code.startswith("E_LANGUAGE_VERSION_"):
        return "language_type"
    if "missing request" in lowered or "request data" in lowered:
        return "host_data_contract"
    if "execution timestamp" in lowered or "timenow" in lowered:
        return "host_data_contract"
    if "magnifier" in lowered or "session window" in lowered:
        return "host_data_contract"
    if stage == "run" and (
        "timeout" in lowered or code in {"E_RUNTIME_LIMIT", "E_RESOURCE_LIMIT"}
    ):
        return "resource"
    if code in {"E_UNKNOWN_FUNCTION", "E_UNKNOWN_SYMBOL", "E_UNKNOWN_MEMBER"}:
        subject = diagnostic_subject(message) or ""
        if subject.startswith("strategy.") or "strategy" in subject:
            return "strategy_lifecycle"
        return "builtin"
    if code == "E_UNSUPPORTED_FEATURE":
        subject = diagnostic_subject(message) or lowered
        if "strategy" in subject:
            return "strategy_lifecycle"
        return "language_type"
    if code == "E_IMPORT_MISSING_LIBRARY":
        return "host_data_contract"
    if code.startswith("E_CALL_") or code.startswith("E_TYPE_"):
        return "language_type"
    if "commission" in lowered or "currency" in lowered or "precision" in lowered:
        return "account_precision"
    if stage == "run":
        return "strategy_lifecycle"
    if stage in {"parse", "sema"}:
        return "language_type"
    return "tool_environment"


def parse_diagnostics(*outputs: str, stage: str) -> list[DiagnosticRecord]:
    records: list[DiagnosticRecord] = []
    seen: set[tuple[str, str, int, int, str]] = set()
    for output in outputs:
        for line in output.splitlines():
            match = DIAGNOSTIC_RE.match(line.strip())
            if match is None:
                continue
            code = match.group("code")
            message = match.group("message")
            key = (
                code,
                match.group("severity"),
                int(match.group("line")),
                int(match.group("column")),
                message,
            )
            if key in seen:
                continue
            seen.add(key)
            records.append(
                DiagnosticRecord(
                    code=code,
                    severity=match.group("severity"),
                    line=int(match.group("line")),
                    column=int(match.group("column")),
                    message=message,
                    subject=diagnostic_subject(message),
                    root_cause_class=classify_root_cause(code, message, stage),
                )
            )
    return records


def resolve_path(root: Path, value: str) -> Path:
    path = Path(value)
    return path if path.is_absolute() else root / path


def optional_file_status(root: Path, value: str) -> str:
    if not value:
        return "not_supplied"
    return STAGE_PASSED if resolve_path(root, value).is_file() else STAGE_MISSING_INPUT


def stage(status: str, **details: object) -> dict[str, object]:
    if status not in STAGE_STATUSES:
        raise CorpusError(f"unknown stage status {status!r}")
    return {"status": status, **details}


def default_command_runner(
    command: Sequence[str], root: Path, timeout_seconds: float
) -> CommandResult:
    argv = [str(part) for part in command]
    started = time.monotonic()
    try:
        completed = subprocess.run(
            argv,
            cwd=root,
            text=True,
            capture_output=True,
            check=False,
            timeout=timeout_seconds,
        )
    except subprocess.TimeoutExpired as exc:
        duration_ms = int((time.monotonic() - started) * 1000)
        stdout = exc.stdout or ""
        stderr = exc.stderr or ""
        if isinstance(stdout, bytes):
            stdout = stdout.decode("utf-8", errors="replace")
        if isinstance(stderr, bytes):
            stderr = stderr.decode("utf-8", errors="replace")
        return CommandResult(
            argv=argv,
            returncode=-1,
            stdout=stdout,
            stderr=stderr,
            timed_out=True,
            duration_ms=duration_ms,
        )
    duration_ms = int((time.monotonic() - started) * 1000)
    return CommandResult(
        argv=argv,
        returncode=completed.returncode,
        stdout=completed.stdout,
        stderr=completed.stderr,
        timed_out=False,
        duration_ms=duration_ms,
    )


def missing_comparator_result(
    actual: dict[str, Any],
    reference: dict[str, Any],
    fields: Sequence[str],
) -> dict[str, Any]:
    del actual, reference, fields
    return {
        "status": STAGE_NOT_RUN,
        "reason": "tool_unavailable",
        "comparable": False,
        "coverage": [],
        "mismatches": None,
        "firstDifference": None,
    }


def load_default_compare_fn() -> CompareFn:
    try:
        from compare_strategy_reference_outputs import (  # type: ignore
            compare_reference,
        )
    except ImportError:
        return missing_comparator_result
    return compare_reference


def parse_json_object(text: str) -> dict[str, Any] | None:
    decoder = json.JSONDecoder()
    stripped = text.lstrip()
    if not stripped.startswith("{"):
        start = stripped.find("{")
        if start < 0:
            return None
        stripped = stripped[start:]
    try:
        value, _offset = decoder.raw_decode(stripped)
    except json.JSONDecodeError:
        return None
    return value if isinstance(value, dict) else None


def required_string(raw: Mapping[str, Any], key: str, line_number: int) -> str:
    value = raw.get(key)
    if not isinstance(value, str) or not value.strip():
        raise CorpusError(f"manifest line {line_number}: {key} is required")
    return value.strip()


def nested(raw: Mapping[str, Any], key: str) -> Mapping[str, Any]:
    value = raw.get(key)
    return value if isinstance(value, Mapping) else {}


def nested_status(container: Mapping[str, Any], key: str) -> tuple[str, str, str]:
    value = container.get(key)
    if isinstance(value, str):
        return value, "", ""
    if not isinstance(value, Mapping):
        return "", "", ""
    status = str(value.get("status") or "")
    reason = str(value.get("reason") or "")
    path = str(value.get("path") or value.get("value") or "")
    return status, reason, path


def parse_sample(raw: Mapping[str, Any], line_number: int) -> Sample:
    schema_version = raw.get("schemaVersion", MANIFEST_SCHEMA_VERSION)
    if schema_version != MANIFEST_SCHEMA_VERSION:
        raise CorpusError(
            f"manifest line {line_number}: unsupported schemaVersion {schema_version}"
        )
    sample_id = required_string(raw, "sampleId", line_number)
    script_id = required_string(raw, "scriptId", line_number)
    scenario_id = required_string(raw, "scenarioId", line_number)
    sample_category = required_string(raw, "sampleCategory", line_number)
    if sample_category not in SAMPLE_CATEGORIES:
        raise CorpusError(
            f"manifest line {line_number}: unknown sampleCategory {sample_category!r}"
        )
    try:
        pine_version = int(raw.get("pineVersion"))
    except (TypeError, ValueError) as exc:
        raise CorpusError(
            f"manifest line {line_number}: pineVersion must be an integer"
        ) from exc
    source = nested(raw, "source")
    script = nested(raw, "script")
    data = nested(raw, "data")
    host = nested(raw, "hostInputs")
    reference = nested(raw, "reference")
    execution = nested(raw, "execution")
    settings = nested(raw, "settings")
    license_class = str(source.get("licenseClass") or raw.get("licenseClass") or "")
    if license_class not in LICENSE_CLASSES:
        raise CorpusError(
            f"manifest line {line_number}: unknown licenseClass {license_class!r}"
        )
    source_path = str(script.get("sourcePath") or source.get("identifier") or "")
    if not source_path:
        raise CorpusError(f"manifest line {line_number}: source path is required")
    ohlcv_path = str(data.get("ohlcvPath") or "")
    _, _, magnifier_path = nested_status(host, "magnifierBars")
    _, _, session_path = nested_status(host, "sessionWindows")
    _, _, execution_times_path = nested_status(host, "executionTimes")
    reference_status = str(reference.get("status") or "none")
    reference_path = str(reference.get("path") or "")
    reference_fields = [
        str(item)
        for item in reference.get("fieldCoverage") or list(DEFAULT_COMPARE_FIELDS)
        if str(item)
    ]
    timeout = execution.get("timeoutSeconds", 30)
    try:
        timeout_seconds = float(timeout)
    except (TypeError, ValueError) as exc:
        raise CorpusError(
            f"manifest line {line_number}: timeoutSeconds must be numeric"
        ) from exc
    overrides_raw = settings.get("inputOverrides") or []
    if not isinstance(overrides_raw, list):
        raise CorpusError(
            f"manifest line {line_number}: inputOverrides must be an array"
        )
    input_overrides: list[dict[str, str]] = []
    for item in overrides_raw:
        if not isinstance(item, Mapping):
            raise CorpusError(
                f"manifest line {line_number}: input override must be an object"
            )
        name = str(item.get("name") or "")
        call_site_id = str(item.get("callSiteId") or "")
        value = str(item.get("value") or "")
        if not call_site_id or not value:
            raise CorpusError(
                f"manifest line {line_number}: input override needs callSiteId and value"
            )
        input_overrides.append(
            {"name": name, "callSiteId": call_site_id, "value": value}
        )
    library_raw = script.get("imports") or []
    library_sources: list[dict[str, str]] = []
    if not isinstance(library_raw, list):
        raise CorpusError(f"manifest line {line_number}: imports must be an array")
    for item in library_raw:
        if not isinstance(item, Mapping):
            raise CorpusError(f"manifest line {line_number}: import must be an object")
        key = str(item.get("key") or "")
        path = str(item.get("path") or "")
        if not key:
            raise CorpusError(f"manifest line {line_number}: import needs key")
        library_sources.append(
            {
                "key": key,
                "path": path,
                "revision": str(item.get("revision") or ""),
                "sha256": str(item.get("sha256") or ""),
            }
        )
    request_raw = host.get("requestData")
    request_data: list[dict[str, str]] = []
    if isinstance(request_raw, Mapping) and isinstance(request_raw.get("items"), list):
        for item in request_raw["items"]:
            if isinstance(item, Mapping):
                request_data.append(
                    {
                        "spec": str(item.get("spec") or ""),
                        "path": str(item.get("path") or ""),
                        "sha256": str(item.get("sha256") or ""),
                    }
                )
    coverage = data.get("coverage")
    coverage_kind = "unspecified"
    if isinstance(coverage, Mapping):
        coverage_kind = str(coverage.get("status") or coverage_kind)
    elif isinstance(coverage, str):
        coverage_kind = coverage
    return Sample(
        raw=dict(raw),
        sample_id=sample_id,
        script_id=script_id,
        scenario_id=scenario_id,
        sample_category=sample_category,
        pine_version=pine_version,
        source_path=source_path,
        source_sha256=str(source.get("sourceSha256") or ""),
        ohlcv_path=ohlcv_path,
        ohlcv_sha256=str(data.get("ohlcvSha256") or ""),
        license_class=license_class,
        source_modified=bool(script.get("sourceModified") or source.get("sourceModified")),
        input_overrides=input_overrides,
        library_sources=library_sources,
        magnifier_path=magnifier_path,
        magnifier_sha256=str(nested(host, "magnifierBars").get("sha256") or ""),
        session_windows_path=session_path,
        session_windows_sha256=str(nested(host, "sessionWindows").get("sha256") or ""),
        execution_times_path=execution_times_path,
        execution_times_sha256=str(nested(host, "executionTimes").get("sha256") or ""),
        request_data=request_data,
        reference_path=reference_path,
        reference_sha256=str(reference.get("fileHash") or reference.get("sha256") or ""),
        reference_status=reference_status,
        reference_fields=reference_fields,
        reference_reason=str(reference.get("incomparableReason") or reference.get("reason") or ""),
        timeout_seconds=timeout_seconds,
        expected_support_boundary=str(
            execution.get("expectedSupportBoundary") or ""
        ),
        coverage_kind=coverage_kind,
    )


def parse_manifest(path: Path) -> list[Sample]:
    try:
        text = path.read_text(encoding="utf-8")
    except OSError as exc:
        raise CorpusError(f"failed to read corpus manifest {path}: {exc}") from exc
    samples: list[Sample] = []
    seen: set[str] = set()
    for line_number, line in enumerate(text.splitlines(), start=1):
        if not line.strip():
            continue
        try:
            raw = json.loads(line)
        except json.JSONDecodeError as exc:
            raise CorpusError(
                f"manifest line {line_number}: invalid JSON: {exc}"
            ) from exc
        if not isinstance(raw, dict):
            raise CorpusError(f"manifest line {line_number}: sample must be an object")
        sample = parse_sample(raw, line_number)
        if sample.sample_id in seen:
            raise CorpusError(
                f"manifest line {line_number}: duplicate sampleId {sample.sample_id!r}"
            )
        seen.add(sample.sample_id)
        samples.append(sample)
    ids = [sample.sample_id for sample in samples]
    if ids != sorted(ids):
        raise CorpusError("modern strategy manifest rows must be sorted by sampleId")
    return samples


def file_hash_or_blank(path: Path) -> str:
    return sha256_file(path) if path.is_file() else ""


def verify_declared_hash(
    root: Path, relative: str, declared: str, label: str
) -> str | None:
    if not relative or not declared:
        return None
    path = resolve_path(root, relative)
    if not path.is_file():
        return None
    actual = sha256_file(path)
    if actual != declared:
        return f"{label} hash mismatch: declared {declared}, actual {actual}"
    return None


def first_error(records: Sequence[DiagnosticRecord]) -> DiagnosticRecord | None:
    for record in records:
        if record.severity.lower() == "error":
            return record
    return None


def measure_sample(
    sample: Sample,
    *,
    root: Path,
    pine_compat: Path,
    command_runner: CommandRunner,
    compare_fn: CompareFn,
) -> dict[str, Any]:
    item: dict[str, Any] = {
        "sampleId": sample.sample_id,
        "scriptId": sample.script_id,
        "scenarioId": sample.scenario_id,
        "sampleCategory": sample.sample_category,
        "pineVersion": sample.pine_version,
        "licenseClass": sample.license_class,
        "coverageKind": sample.coverage_kind,
        "sourceModified": sample.source_modified,
        "expectedSupportBoundary": sample.expected_support_boundary,
    }
    source_path = resolve_path(root, sample.source_path)
    inputs = {
        "source": optional_file_status(root, sample.source_path),
        "chartBars": optional_file_status(root, sample.ohlcv_path),
        "magnifierBars": optional_file_status(root, sample.magnifier_path),
        "sessionWindows": optional_file_status(root, sample.session_windows_path),
        "executionTimes": optional_file_status(root, sample.execution_times_path),
        "referenceOutput": optional_file_status(root, sample.reference_path),
    }
    item["inputAvailability"] = inputs
    commands: list[dict[str, Any]] = []
    diagnostics: list[dict[str, Any]] = []
    stages: dict[str, dict[str, Any]] = {}
    item["stages"] = stages
    item["commands"] = commands
    item["diagnostics"] = diagnostics

    hash_errors = [
        error
        for error in (
            verify_declared_hash(
                root, sample.source_path, sample.source_sha256, "source"
            ),
            verify_declared_hash(
                root, sample.ohlcv_path, sample.ohlcv_sha256, "ohlcv"
            ),
            verify_declared_hash(
                root,
                sample.magnifier_path,
                sample.magnifier_sha256,
                "magnifierBars",
            ),
            verify_declared_hash(
                root,
                sample.session_windows_path,
                sample.session_windows_sha256,
                "sessionWindows",
            ),
            verify_declared_hash(
                root,
                sample.execution_times_path,
                sample.execution_times_sha256,
                "executionTimes",
            ),
            verify_declared_hash(
                root,
                sample.reference_path,
                sample.reference_sha256,
                "reference",
            ),
        )
        if error is not None
    ]
    if hash_errors:
        stages["parse"] = stage(STAGE_FAILED, errorKind="hash_mismatch")
        stages["sema"] = stage(STAGE_NOT_RUN, errorKind="hash_mismatch")
        stages["run"] = stage(STAGE_NOT_RUN, errorKind="hash_mismatch")
        stages["outputComparability"] = stage(
            STAGE_NOT_RUN, errorKind="hash_mismatch"
        )
        stages["resultConsistency"] = stage(
            STAGE_NOT_RUN, errorKind="hash_mismatch"
        )
        item["hashErrors"] = hash_errors
        return item

    if sample.sample_category == "excluded":
        for name in ("parse", "sema", "run", "outputComparability", "resultConsistency"):
            stages[name] = stage(STAGE_EXCLUDED)
        return item

    if inputs["source"] != STAGE_PASSED:
        stages["parse"] = stage(STAGE_MISSING_INPUT)
        stages["sema"] = stage(STAGE_NOT_RUN)
        stages["run"] = stage(STAGE_NOT_RUN)
        stages["outputComparability"] = stage(STAGE_NOT_RUN)
        stages["resultConsistency"] = stage(STAGE_NOT_RUN)
        return item

    parse_command = [str(pine_compat), "fmt-ast", str(source_path)]
    parsed = command_runner(parse_command, root, sample.timeout_seconds)
    commands.append(parsed.as_dict())
    parse_diagnostics_records = parse_diagnostics(
        parsed.stdout, parsed.stderr, stage="parse"
    )
    diagnostics.extend(record.as_dict("parse") for record in parse_diagnostics_records)
    if parsed.timed_out:
        stages["parse"] = stage(
            STAGE_FAILED, errorKind="timeout", returnCode=parsed.returncode
        )
    else:
        syntax_errors = [
            record
            for record in parse_diagnostics_records
            if record.severity.lower() == "error" and is_syntax_diagnostic(record.code)
        ]
        stages["parse"] = stage(
            STAGE_FAILED if syntax_errors else STAGE_PASSED,
            returnCode=parsed.returncode,
            diagnosticCount=len(syntax_errors),
            usedExitCode=False,
        )

    analyze_command = [str(pine_compat), "analyze", str(source_path)]
    for library in sample.library_sources:
        if not library.get("path"):
            continue
        analyze_command.extend(
            ("--library-source", f"{library['key']}={resolve_path(root, library['path'])}")
        )
    analyzed = command_runner(analyze_command, root, sample.timeout_seconds)
    commands.append(analyzed.as_dict())
    sema_records = parse_diagnostics(analyzed.stdout, analyzed.stderr, stage="sema")
    diagnostics.extend(record.as_dict("sema") for record in sema_records)
    if analyzed.timed_out:
        stages["sema"] = stage(
            STAGE_FAILED, errorKind="timeout", returnCode=analyzed.returncode
        )
    else:
        error_records = [
            record for record in sema_records if record.severity.lower() == "error"
        ]
        stages["sema"] = stage(
            STAGE_PASSED if analyzed.returncode == 0 and not error_records else STAGE_FAILED,
            returnCode=analyzed.returncode,
            diagnosticCount=len(error_records),
        )

    required_host_missing = False
    if sample.magnifier_path and inputs["magnifierBars"] != STAGE_PASSED:
        required_host_missing = True
    if sample.session_windows_path and inputs["sessionWindows"] != STAGE_PASSED:
        required_host_missing = True
    if sample.execution_times_path and inputs["executionTimes"] != STAGE_PASSED:
        required_host_missing = True
    for request in sample.request_data:
        path = request.get("path") or ""
        if path and optional_file_status(root, path) != STAGE_PASSED:
            required_host_missing = True

    can_run = (
        stages["parse"]["status"] == STAGE_PASSED
        and stages["sema"]["status"] == STAGE_PASSED
        and sample.sample_category in ELIGIBLE_CATEGORIES
    )
    runtime_output: dict[str, Any] | None = None
    if not can_run:
        stages["run"] = stage(STAGE_NOT_RUN, reason="sema_not_passed_or_not_eligible")
    elif inputs["chartBars"] != STAGE_PASSED:
        stages["run"] = stage(STAGE_MISSING_INPUT, reason="missing_ohlcv")
    elif required_host_missing:
        stages["run"] = stage(STAGE_MISSING_INPUT, reason="missing_host_input")
    else:
        run_command = [
            str(pine_compat),
            "run",
            str(source_path),
            "--bars",
            str(resolve_path(root, sample.ohlcv_path)),
        ]
        if sample.magnifier_path:
            run_command.extend(
                ("--magnifier-bars", str(resolve_path(root, sample.magnifier_path)))
            )
        if sample.session_windows_path:
            run_command.extend(
                (
                    "--session-windows",
                    str(resolve_path(root, sample.session_windows_path)),
                )
            )
        if sample.execution_times_path:
            run_command.extend(
                (
                    "--execution-times",
                    str(resolve_path(root, sample.execution_times_path)),
                )
            )
        for library in sample.library_sources:
            if not library.get("path"):
                continue
            run_command.extend(
                (
                    "--library-source",
                    f"{library['key']}={resolve_path(root, library['path'])}",
                )
            )
        for request in sample.request_data:
            spec = request.get("spec")
            if spec:
                run_command.extend(("--request-bars", spec))
        for override in sample.input_overrides:
            run_command.extend(
                (
                    "--input-override",
                    f"{override['callSiteId']}={override['value']}",
                )
            )
        executed = command_runner(run_command, root, sample.timeout_seconds)
        commands.append(executed.as_dict())
        run_records = parse_diagnostics(executed.stdout, executed.stderr, stage="run")
        diagnostics.extend(record.as_dict("run") for record in run_records)
        if executed.timed_out:
            stages["run"] = stage(
                STAGE_FAILED, errorKind="timeout", returnCode=executed.returncode
            )
        elif executed.returncode != 0:
            error_kind = "runtime_error"
            combined = f"{executed.stderr}\n{executed.stdout}".lower()
            if "missing request" in combined:
                error_kind = "missing_provider_data"
            stages["run"] = stage(
                STAGE_FAILED,
                returnCode=executed.returncode,
                errorKind=error_kind,
            )
        else:
            runtime_output = parse_json_object(executed.stdout)
            if runtime_output is None:
                stages["run"] = stage(
                    STAGE_FAILED,
                    returnCode=0,
                    errorKind="invalid_runtime_json",
                )
            else:
                trades = runtime_output.get("strategy", {}).get("trades")
                if not isinstance(trades, list):
                    trades = runtime_output.get("trades")
                trade_count = len(trades) if isinstance(trades, list) else 0
                stages["run"] = stage(
                    STAGE_PASSED,
                    returnCode=0,
                    tradeCount=trade_count,
                    noTrades=trade_count == 0,
                )

    compare_result = apply_compare(
        sample,
        root=root,
        inputs=inputs,
        runtime_output=runtime_output,
        run_status=str(stages["run"]["status"]),
        compare_fn=compare_fn,
    )
    stages["outputComparability"] = stage(
        str(compare_result["comparabilityStatus"]),
        reason=compare_result.get("reason"),
        coverage=compare_result.get("coverage"),
        comparable=compare_result.get("comparable"),
    )
    stages["resultConsistency"] = stage(
        str(compare_result["consistencyStatus"]),
        reason=compare_result.get("reason"),
        firstDifference=compare_result.get("firstDifference"),
        mismatches=compare_result.get("mismatches"),
    )
    item["compare"] = compare_result
    first = first_blocking(stages, diagnostics)
    if first is not None:
        item["firstBlocking"] = first
    return item


def apply_compare(
    sample: Sample,
    *,
    root: Path,
    inputs: Mapping[str, str],
    runtime_output: dict[str, Any] | None,
    run_status: str,
    compare_fn: CompareFn,
) -> dict[str, Any]:
    if sample.sample_category == "negative_control":
        return {
            "comparabilityStatus": STAGE_EXCLUDED,
            "consistencyStatus": STAGE_EXCLUDED,
            "reason": "negative_control",
            "comparable": False,
            "coverage": [],
            "mismatches": None,
            "firstDifference": None,
        }
    if sample.reference_status in {"", "none", "not_supplied"} and not sample.reference_path:
        return {
            "comparabilityStatus": STAGE_NOT_RUN,
            "consistencyStatus": STAGE_NOT_RUN,
            "reason": "no_reference",
            "comparable": False,
            "coverage": [],
            "mismatches": None,
            "firstDifference": None,
        }
    if inputs["referenceOutput"] != STAGE_PASSED:
        return {
            "comparabilityStatus": STAGE_MISSING_INPUT,
            "consistencyStatus": STAGE_NOT_RUN,
            "reason": "missing_reference_file",
            "comparable": False,
            "coverage": [],
            "mismatches": None,
            "firstDifference": None,
        }
    if sample.reference_status == "partial":
        return {
            "comparabilityStatus": STAGE_FAILED,
            "consistencyStatus": STAGE_NOT_RUN,
            "reason": sample.reference_reason or "partial_reference",
            "comparable": False,
            "coverage": sample.reference_fields,
            "mismatches": None,
            "firstDifference": None,
        }
    if run_status != STAGE_PASSED or runtime_output is None:
        return {
            "comparabilityStatus": STAGE_NOT_RUN,
            "consistencyStatus": STAGE_NOT_RUN,
            "reason": "run_not_passed",
            "comparable": False,
            "coverage": sample.reference_fields,
            "mismatches": None,
            "firstDifference": None,
        }
    try:
        reference_text = resolve_path(root, sample.reference_path).read_text(
            encoding="utf-8"
        )
        reference_obj = json.loads(reference_text)
    except (OSError, json.JSONDecodeError) as exc:
        return {
            "comparabilityStatus": STAGE_FAILED,
            "consistencyStatus": STAGE_NOT_RUN,
            "reason": f"invalid_reference:{exc}",
            "comparable": False,
            "coverage": sample.reference_fields,
            "mismatches": None,
            "firstDifference": None,
        }
    if not isinstance(reference_obj, dict):
        return {
            "comparabilityStatus": STAGE_FAILED,
            "consistencyStatus": STAGE_NOT_RUN,
            "reason": "reference_root_must_be_object",
            "comparable": False,
            "coverage": sample.reference_fields,
            "mismatches": None,
            "firstDifference": None,
        }
    outcome = compare_fn(runtime_output, reference_obj, sample.reference_fields)
    reason = str(outcome.get("reason") or "")
    status = str(outcome.get("status") or STAGE_NOT_RUN)
    comparable = bool(outcome.get("comparable", status == STAGE_PASSED))
    if reason == "tool_unavailable" or status == STAGE_NOT_RUN:
        return {
            "comparabilityStatus": STAGE_NOT_RUN,
            "consistencyStatus": STAGE_NOT_RUN,
            "reason": reason or "tool_unavailable",
            "comparable": False,
            "coverage": list(outcome.get("coverage") or sample.reference_fields),
            "mismatches": outcome.get("mismatches"),
            "firstDifference": outcome.get("firstDifference"),
        }
    coverage = list(outcome.get("coverage") or sample.reference_fields)
    if status == "incomparable" or (not comparable and status != STAGE_FAILED):
        return {
            "comparabilityStatus": STAGE_FAILED,
            "consistencyStatus": STAGE_NOT_RUN,
            "reason": reason or "incomparable",
            "comparable": False,
            "coverage": coverage,
            "mismatches": outcome.get("mismatches"),
            "firstDifference": outcome.get("firstDifference"),
        }
    if status == STAGE_FAILED:
        return {
            "comparabilityStatus": STAGE_PASSED,
            "consistencyStatus": STAGE_FAILED,
            "reason": reason or "reference_mismatch",
            "comparable": True,
            "coverage": coverage,
            "mismatches": outcome.get("mismatches"),
            "firstDifference": outcome.get("firstDifference"),
        }
    return {
        "comparabilityStatus": STAGE_PASSED,
        "consistencyStatus": STAGE_PASSED,
        "reason": reason or "compared",
        "comparable": True,
        "coverage": coverage,
        "mismatches": outcome.get("mismatches", 0),
        "firstDifference": outcome.get("firstDifference"),
    }


def first_blocking(
    stages: Mapping[str, Mapping[str, Any]], diagnostics: Sequence[Mapping[str, Any]]
) -> dict[str, Any] | None:
    for name in ("parse", "sema", "run"):
        status = str(stages[name]["status"])
        if status in {STAGE_FAILED, STAGE_MISSING_INPUT}:
            matching = [item for item in diagnostics if item.get("stage") == name]
            diagnostic = matching[0] if matching else None
            payload: dict[str, Any] = {
                "stage": name,
                "status": status,
                "errorKind": stages[name].get("errorKind") or stages[name].get("reason"),
            }
            if diagnostic is not None:
                payload["code"] = diagnostic.get("code")
                payload["subject"] = diagnostic.get("subject")
                payload["rootCauseClass"] = diagnostic.get("rootCauseClass")
            elif status == STAGE_MISSING_INPUT:
                payload["rootCauseClass"] = "host_data_contract"
            elif stages[name].get("errorKind") == "timeout":
                payload["rootCauseClass"] = "resource"
            else:
                payload["rootCauseClass"] = "tool_environment"
            return payload
    return None


def metric_bucket() -> dict[str, int]:
    return {
        STAGE_PASSED: 0,
        STAGE_FAILED: 0,
        STAGE_MISSING_INPUT: 0,
        STAGE_NOT_RUN: 0,
        STAGE_EXCLUDED: 0,
    }


def format_rate(passed: int, denominator: int) -> str:
    if denominator == 0:
        return "N/A"
    return f"{passed}/{denominator}"


def summarize_stage(
    items: Sequence[Mapping[str, Any]],
    stage_name: str,
    *,
    denominator: int,
    script_level: bool,
) -> dict[str, Any]:
    counts = metric_bucket()
    seen_scripts: dict[str, str] = {}
    for item in items:
        status = str(item["stages"][stage_name]["status"])
        if script_level:
            script_id = str(item["scriptId"])
            previous = seen_scripts.get(script_id)
            if previous is None:
                seen_scripts[script_id] = status
                counts[status] += 1
            elif previous != STAGE_PASSED and status == STAGE_PASSED:
                counts[previous] -= 1
                counts[STAGE_PASSED] += 1
                seen_scripts[script_id] = STAGE_PASSED
        else:
            counts[status] += 1
    passed = counts[STAGE_PASSED]
    return {
        **counts,
        "denominator": denominator,
        "rate": format_rate(passed, denominator),
        "percent": "N/A" if denominator == 0 else round(100.0 * passed / denominator, 2),
    }


def build_report(
    samples: Sequence[Sample],
    *,
    root: Path,
    manifest_path: Path,
    pine_compat: Path,
    build_revision: str,
    command_runner: CommandRunner | None = None,
    compare_fn: CompareFn | None = None,
) -> dict[str, Any]:
    runner = command_runner or default_command_runner
    comparator = compare_fn or load_default_compare_fn()
    items = [
        measure_sample(
            sample,
            root=root,
            pine_compat=pine_compat,
            command_runner=runner,
            compare_fn=comparator,
        )
        for sample in samples
    ]
    eligible = [
        item
        for item in items
        if item["sampleCategory"] in ELIGIBLE_CATEGORIES
    ]
    negatives = [
        item for item in items if item["sampleCategory"] == "negative_control"
    ]
    excluded = [item for item in items if item["sampleCategory"] == "excluded"]
    script_ids = sorted({item["scriptId"] for item in eligible})
    n_scripts = len(script_ids)
    m_scenarios = len(eligible)
    parse_metric = summarize_stage(
        eligible, "parse", denominator=n_scripts, script_level=True
    )
    sema_metric = summarize_stage(
        eligible, "sema", denominator=n_scripts, script_level=True
    )
    parse_passed_scripts = {
        item["scriptId"]
        for item in eligible
        if item["stages"]["parse"]["status"] == STAGE_PASSED
    }
    sema_after_parse = summarize_stage(
        [item for item in eligible if item["scriptId"] in parse_passed_scripts],
        "sema",
        denominator=len(parse_passed_scripts),
        script_level=True,
    )
    run_metric = summarize_stage(
        eligible, "run", denominator=m_scenarios, script_level=False
    )
    runnable = [
        item
        for item in eligible
        if item["stages"]["sema"]["status"] == STAGE_PASSED
        and item["inputAvailability"]["chartBars"] == STAGE_PASSED
        and item["stages"]["run"]["status"] != STAGE_MISSING_INPUT
    ]
    run_after_ready = summarize_stage(
        runnable, "run", denominator=len(runnable), script_level=False
    )
    comparability = summarize_stage(
        eligible, "outputComparability", denominator=m_scenarios, script_level=False
    )
    comparable_items = [
        item
        for item in eligible
        if item["stages"]["outputComparability"]["status"] == STAGE_PASSED
    ]
    consistency = summarize_stage(
        comparable_items,
        "resultConsistency",
        denominator=len(comparable_items),
        script_level=False,
    )
    negative_reject_passed = sum(
        1
        for item in negatives
        if item["stages"]["sema"]["status"] == STAGE_FAILED
        or item["stages"]["parse"]["status"] == STAGE_FAILED
    )
    ranking = rank_root_causes(eligible)
    manifest_hash = sha256_file(manifest_path) if manifest_path.is_file() else ""
    return {
        "schemaVersion": SCHEMA_VERSION,
        "toolVersion": TOOL_VERSION,
        "corpusRevision": CORPUS_REVISION,
        "buildRevision": build_revision,
        "manifestPath": str(manifest_path.name),
        "manifestSha256": manifest_hash,
        "denominators": {
            "scriptsN": n_scripts,
            "scenariosM": m_scenarios,
            "negativeControls": len(negatives),
            "excluded": len(excluded),
            "missingInputScenarios": sum(
                1
                for item in eligible
                if any(
                    stage_name in item["stages"]
                    and item["stages"][stage_name]["status"] == STAGE_MISSING_INPUT
                    for stage_name in ("parse", "sema", "run")
                )
            ),
        },
        "metrics": {
            "parse": {
                **parse_metric,
                "unit": "scripts",
            },
            "sema": {
                **sema_metric,
                "unit": "scripts",
                "conditionalOnParse": sema_after_parse,
            },
            "run": {
                **run_metric,
                "unit": "scenarios",
                "conditionalOnReadyInputs": run_after_ready,
            },
            "outputComparability": {
                **comparability,
                "unit": "scenarios",
            },
            "resultConsistency": {
                **consistency,
                "unit": "comparable_scenarios",
                "referenceCoverageRate": format_rate(
                    comparability[STAGE_PASSED], m_scenarios
                ),
            },
        },
        "negativeControls": {
            "count": len(negatives),
            "expectedRejectPassed": negative_reject_passed,
            "rate": format_rate(negative_reject_passed, len(negatives)),
        },
        "rootCauseRanking": ranking,
        "items": items,
        "privacy": {
            "sourceTextIncluded": False,
            "privatePathsIncluded": False,
        },
    }


def rank_root_causes(items: Sequence[Mapping[str, Any]]) -> list[dict[str, Any]]:
    clusters: dict[tuple[str, str, str, str], dict[str, Any]] = {}
    for item in items:
        blocking = item.get("firstBlocking")
        if not isinstance(blocking, Mapping):
            continue
        root_class = str(blocking.get("rootCauseClass") or "unclassified")
        stage_name = str(blocking.get("stage") or "")
        code = str(blocking.get("code") or blocking.get("errorKind") or "unknown")
        subject = str(blocking.get("subject") or "")
        key = (root_class, stage_name, code, subject)
        cluster = clusters.setdefault(
            key,
            {
                "rootCauseClass": root_class,
                "firstBlockingStage": stage_name,
                "code": code,
                "subject": subject or None,
                "affectedScripts": set(),
                "affectedScenarios": set(),
                "severity": "blocks_" + stage_name,
                "evidenceQuality": "corpus_diagnostic",
                "dependency": "none",
            },
        )
        cluster["affectedScripts"].add(str(item["scriptId"]))
        cluster["affectedScenarios"].add(str(item["sampleId"]))
    ranking = []
    for cluster in clusters.values():
        scripts = cluster.pop("affectedScripts")
        scenarios = cluster.pop("affectedScenarios")
        ranking.append(
            {
                **cluster,
                "affectedScripts": len(scripts),
                "affectedScenarios": len(scenarios),
            }
        )
    ranking.sort(
        key=lambda item: (
            -int(item["affectedScripts"]),
            -int(item["affectedScenarios"]),
            str(item["rootCauseClass"]),
            str(item["code"]),
            str(item.get("subject") or ""),
        )
    )
    return ranking


def render_report(report: Mapping[str, Any]) -> str:
    return json.dumps(report, indent=2, sort_keys=True) + "\n"


def comparable_stage_view(report: Mapping[str, Any]) -> dict[str, Any]:
    items = []
    for item in report["items"]:
        items.append(
            {
                "sampleId": item["sampleId"],
                "scriptId": item["scriptId"],
                "sampleCategory": item["sampleCategory"],
                "stages": {
                    name: {
                        "status": item["stages"][name]["status"],
                        "errorKind": item["stages"][name].get("errorKind"),
                        "reason": item["stages"][name].get("reason"),
                    }
                    for name in (
                        "parse",
                        "sema",
                        "run",
                        "outputComparability",
                        "resultConsistency",
                    )
                },
                "firstBlocking": item.get("firstBlocking"),
                "diagnostics": [
                    {
                        "stage": diagnostic.get("stage"),
                        "code": diagnostic.get("code"),
                        "subject": diagnostic.get("subject"),
                        "rootCauseClass": diagnostic.get("rootCauseClass"),
                    }
                    for diagnostic in item.get("diagnostics", [])
                ],
            }
        )
    return {
        "metrics": report["metrics"],
        "denominators": report["denominators"],
        "rootCauseRanking": report["rootCauseRanking"],
        "negativeControls": report["negativeControls"],
        "items": items,
    }


def git_revision(root: Path) -> str:
    completed = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=root,
        text=True,
        capture_output=True,
        check=False,
    )
    if completed.returncode != 0:
        return "unknown"
    revision = completed.stdout.strip()
    dirty = subprocess.run(
        ["git", "status", "--porcelain"],
        cwd=root,
        text=True,
        capture_output=True,
        check=False,
    )
    return revision + ("+dirty" if dirty.stdout.strip() else "")


def parse_rust_bars_map(text: str) -> dict[str, str]:
    mapping: dict[str, str] = {}
    collapsed = re.sub(r"include_str!\(\s*", "include_str!(", text)
    pending: list[str] = []
    for line in collapsed.splitlines():
        pending.extend(RUST_PINE_RE.findall(line))
        csv_match = RUST_CSV_RE.search(line)
        if csv_match and pending:
            csv_path = csv_match.group("path")
            for pine in pending:
                mapping[pine] = csv_path
            pending = []
        elif re.search(r"=>\s*(?:None|Some\()", line) and "include_str" not in line:
            pending = []
    return mapping


LIBRARY_PAIR_RE = re.compile(
    r'\("(?P<key>user/[^"]+)",\s*"(?P<path>tests/fixtures/libraries/[^"]+)"\)'
)


def load_known_libraries(root: Path) -> dict[str, str]:
    mapping = dict(KNOWN_LIBRARY_SOURCES)
    for relative in (
        "crates/pine-cli/src/runtime_snapshots/fixtures.rs",
        "crates/pine-runtime/tests/incremental.rs",
        "crates/pine-runtime/tests/realtime.rs",
    ):
        path = root / relative
        if not path.is_file():
            continue
        text = path.read_text(encoding="utf-8")
        for match in LIBRARY_PAIR_RE.finditer(text):
            mapping[match.group("key")] = match.group("path")
    return mapping


def discover_imports(source: str, root: Path, libraries: Mapping[str, str]) -> list[dict[str, str]]:
    imports: list[dict[str, str]] = []
    for match in IMPORT_RE.finditer(source):
        key = match.group("key")
        path = libraries.get(key, "")
        sha = file_hash_or_blank(root / path) if path else ""
        imports.append(
            {
                "key": key,
                "path": path,
                "revision": "",
                "sha256": sha,
                "alias": match.group("alias"),
            }
        )
    return imports


def load_runtime_bars_map(root: Path) -> dict[str, str]:
    mapping: dict[str, str] = {}
    for relative in (
        "crates/pine-cli/src/runtime_snapshots/bars.rs",
        "crates/pine-cli/src/main_tests.rs",
    ):
        path = root / relative
        if path.is_file():
            mapping.update(parse_rust_bars_map(path.read_text(encoding="utf-8")))
    return mapping


def ohlcv_coverage(path: str, default_bars: str) -> dict[str, str]:
    if not path:
        return {
            "status": "not_supplied",
            "reason": "no OHLCV file recorded",
        }
    if path == default_bars:
        return {
            "status": "synthetic_smoke",
            "reason": "default synthetic bars.csv; not a real-symbol backtest",
        }
    return {
        "status": "fixture_bars",
        "reason": "in-repo original fixture bars; still not an independent market series",
    }


def csv_stats(path: Path) -> dict[str, Any]:
    if not path.is_file():
        return Missing("missing", "OHLCV file is not present").as_dict()
    rows = path.read_text(encoding="utf-8").splitlines()
    data_rows = [row for row in rows[1:] if row.strip()] if rows else []
    start = data_rows[0].split(",")[0] if data_rows else None
    end = data_rows[-1].split(",")[0] if data_rows else None
    return {
        "status": "from_file",
        "barCount": len(data_rows),
        "startTime": start,
        "endTime": end,
        "reason": "read from the first time column of the CSV",
    }


def none_reference() -> dict[str, Any]:
    return {
        "status": "none",
        "reason": "no independent Tester or authorized expected fills",
        "source": None,
        "exportedAt": None,
        "fileHash": None,
        "fieldCoverage": [],
        "evidenceGrade": "none",
        "incomparableReason": "no independent reference",
        "path": "",
    }


def host_input(status: str, reason: str, path: str = "", sha256: str = "") -> dict[str, Any]:
    payload: dict[str, Any] = {"status": status, "reason": reason}
    if path:
        payload["path"] = path
    if sha256:
        payload["sha256"] = sha256
    return payload


def script_stem_id(relative: str) -> str:
    path = Path(relative)
    return f"original.{path.parent.name}.{path.stem}"


def extra_session_scenarios(relative: str) -> list[dict[str, str]] | None:
    if relative != "tests/fixtures/runtime/strategy_session_overnight_filled_orders.pine":
        return None
    return [
        {
            "scenarioId": "utc_default",
            "sessionWindows": "",
        },
        {
            "scenarioId": "overnight",
            "sessionWindows": "tests/fixtures/runtime/strategy_session_overnight_windows.json",
        },
        {
            "scenarioId": "window_switch",
            "sessionWindows": "tests/fixtures/runtime/strategy_session_window_switch_windows.json",
        },
    ]


def discover_bars(relative: str, root: Path, rust_map: Mapping[str, str]) -> str:
    if relative in rust_map:
        return rust_map[relative]
    sibling = Path(relative).with_name(Path(relative).stem + "_bars.csv")
    if (root / sibling).is_file():
        return str(sibling).replace("\\", "/")
    if (root / DEFAULT_SYNTHETIC_BARS).is_file():
        return DEFAULT_SYNTHETIC_BARS
    return ""


def discover_magnifier(relative: str, source: str, root: Path) -> tuple[str, str]:
    if "magnifier" not in Path(relative).stem:
        return "", "not_supplied"
    json_path = Path(relative).with_suffix(".json")
    if (root / json_path).is_file():
        return str(json_path).replace("\\", "/"), "present"
    if MAGNIFIER_TRUE_RE.search(source):
        return "", "not_supplied_fallback"
    return "", "not_supplied"


def build_sample_record(
    *,
    root: Path,
    relative: str,
    category: str,
    license_class: str,
    license_id: str,
    revision: str,
    rust_map: Mapping[str, str],
    libraries: Mapping[str, str] | None = None,
    scenario_id: str = "default",
    session_windows: str = "",
    publish_allowed: bool,
    retain_allowed: bool,
    source_kind: str,
) -> dict[str, Any] | None:
    path = root / relative
    try:
        source_bytes = path.read_bytes()
        source = source_bytes.decode("utf-8")
    except (OSError, UnicodeDecodeError):
        return None
    mode = detected_mode(source)
    version = detected_version(source)
    if mode != "strategy":
        return None
    if version not in PINE_VERSIONS:
        category = "excluded"
    source_sha = sha256_bytes(source_bytes)
    bars_path = discover_bars(relative, root, rust_map) if category != "excluded" else ""
    bars_sha = file_hash_or_blank(root / bars_path) if bars_path else ""
    magnifier_path, magnifier_kind = discover_magnifier(relative, source, root)
    magnifier_sha = file_hash_or_blank(root / magnifier_path) if magnifier_path else ""
    session_sha = (
        file_hash_or_blank(root / session_windows) if session_windows else ""
    )
    script_id = script_stem_id(relative) if source_kind.startswith("in_repo") else (
        "permissive." + Path(relative).stem.lower().replace(" ", "_")
    )
    sample_id = f"{script_id}.{scenario_id}"
    stats = csv_stats(root / bars_path) if bars_path else Missing(
        "not_supplied", "no OHLCV recorded"
    ).as_dict()
    if category == "negative_control":
        expected = "expected semantic or parse rejection"
    elif category == "excluded":
        expected = "excluded from modern v5/v6 denominators"
    else:
        expected = "original regression fixture; self-golden only, not Tester-backed"
    return {
        "schemaVersion": MANIFEST_SCHEMA_VERSION,
        "sampleId": sample_id,
        "scriptId": script_id,
        "scenarioId": scenario_id,
        "sampleCategory": category,
        "pineVersion": version,
        "source": {
            "kind": source_kind,
            "identifier": relative,
            "revision": revision,
            "licenseClass": license_class,
            "licenseId": license_id,
            "retainAllowed": retain_allowed,
            "publishAllowed": publish_allowed,
            "sourceSha256": source_sha,
            "sourceModified": False,
        },
        "script": {
            "sourcePath": relative,
            "imports": discover_imports(source, root, libraries or {}),
            "sourceModified": False,
        },
        "settings": {
            "inputOverrides": [],
            "strategyDeclaration": Missing(
                "not_extracted",
                "declaration settings are captured from analyze when measured",
            ).as_dict(),
            "chartType": Missing(
                "unspecified", "fixture does not declare a non-default chart type"
            ).as_dict(),
            "symbol": Missing(
                "synthetic",
                "synthetic smoke uses TEST rather than a real symbol",
                "TEST",
            ).as_dict(),
            "timeframe": Missing(
                "synthetic", "synthetic smoke uses integer bar indexes", "1"
            ).as_dict(),
            "timeUnit": Missing(
                "unspecified", "synthetic bars have no exchange time unit"
            ).as_dict(),
            "timezone": Missing(
                "unspecified", "synthetic bars have no timezone"
            ).as_dict(),
        },
        "data": {
            "ohlcvPath": bars_path,
            "ohlcvSha256": bars_sha,
            "coverage": ohlcv_coverage(bars_path, DEFAULT_SYNTHETIC_BARS),
            "barStats": stats,
            "warmup": Missing(
                "unspecified", "fixture does not declare a warmup range"
            ).as_dict(),
            "gaps": Missing(
                "unspecified", "gap presence is not independently documented"
            ).as_dict(),
        },
        "hostInputs": {
            "requestData": host_input(
                "not_supplied", "fixture does not record request.security bars"
            ),
            "magnifierBars": host_input(
                "present" if magnifier_path else magnifier_kind or "not_supplied",
                "paired magnifier JSON" if magnifier_path else "no magnifier JSON required or present",
                magnifier_path,
                magnifier_sha,
            ),
            "sessionWindows": host_input(
                "present" if session_windows else "not_supplied",
                "paired session-window JSON" if session_windows else "no session windows recorded",
                session_windows,
                session_sha,
            ),
            "executionTimes": host_input(
                "not_supplied", "fixture does not require timenow timestamps"
            ),
        },
        "reference": none_reference(),
        "execution": {
            "mode": "historical",
            "timeoutSeconds": 30,
            "resourceLimits": Missing(
                "runtime_defaults", "uses interpreter default resource guards"
            ).as_dict(),
            "toolVersion": TOOL_VERSION,
            "expectedSupportBoundary": expected,
        },
    }


def iter_candidate_paths(root: Path) -> Iterable[tuple[str, str]]:
    groups = (
        (root / "tests/fixtures/runtime", "original_regression"),
        (root / "tests/fixtures/sema", "original_regression"),
        (root / "tests/fixtures/regressions", "original_regression"),
        (root / "tests/fixtures/profile", "original_regression"),
        (root / "tests/fixtures/realtime", "original_regression"),
        (root / "tests/fixtures/syntax", "original_regression"),
        (root / "tests/fixtures/legacy", "excluded"),
    )
    for directory, default_category in groups:
        if not directory.is_dir():
            continue
        for path in sorted(directory.rglob("*.pine")):
            name = path.name
            relative = str(path.relative_to(root)).replace("\\", "/")
            if default_category == "excluded":
                category = "excluded"
            elif name.startswith("unsupported_"):
                category = "negative_control"
            elif "strategy" in name:
                category = default_category
            else:
                continue
            yield relative, category


def freeze_public_corpus(root: Path, revision: str) -> dict[str, Any]:
    rust_map = load_runtime_bars_map(root)
    libraries = load_known_libraries(root)
    records: list[dict[str, Any]] = []
    inventory: list[dict[str, Any]] = []
    seen_paths: set[str] = set()
    for relative, category in iter_candidate_paths(root):
        extras = extra_session_scenarios(relative)
        scenarios = extras or [{"scenarioId": "default", "sessionWindows": ""}]
        built_any = False
        for extra in scenarios:
            record = build_sample_record(
                root=root,
                relative=relative,
                category=category,
                license_class="original",
                license_id="project-original",
                revision=revision,
                rust_map=rust_map,
                libraries=libraries,
                scenario_id=extra["scenarioId"],
                session_windows=extra["sessionWindows"],
                publish_allowed=True,
                retain_allowed=True,
                source_kind="in_repo_fixture",
            )
            if record is None:
                continue
            built_any = True
            records.append(record)
        if built_any:
            seen_paths.add(relative)
            final_category = next(
                (
                    extra_record["sampleCategory"]
                    for extra_record in reversed(records)
                    if extra_record["script"]["sourcePath"] == relative
                ),
                category,
            )
            inventory.append(
                {
                    "path": relative,
                    "category": final_category,
                    "licenseClass": "original",
                    "publishAllowed": True,
                    "reason": "in-repo original fixture"
                    if final_category != "excluded"
                    else "version or scope excluded from modern v5/v6 denominators",
                }
            )

    hash_groups: dict[str, list[str]] = defaultdict(list)
    normalized_groups: dict[str, list[str]] = defaultdict(list)
    for record in records:
        source_path = record["script"]["sourcePath"]
        source_sha = record["source"]["sourceSha256"]
        hash_groups[source_sha].append(record["sampleId"])
        text = (root / source_path).read_text(encoding="utf-8")
        normalized = re.sub(r"\s+", " ", text.strip())
        normalized_groups[sha256_text(normalized)].append(record["sampleId"])

    representatives: dict[str, str] = {}
    merged: list[dict[str, Any]] = []
    excluded_duplicates: list[dict[str, Any]] = []
    for record in records:
        source_sha = record["source"]["sourceSha256"]
        script_key = source_sha
        if record["sampleCategory"] == "excluded":
            merged.append(record)
            continue
        if script_key not in representatives:
            representatives[script_key] = record["scriptId"]
            merged.append(record)
            continue
        canonical_script = representatives[script_key]
        if record["scriptId"] != canonical_script:
            excluded_duplicates.append(
                {
                    "sampleId": record["sampleId"],
                    "duplicateOf": canonical_script,
                    "reason": "same source SHA-256; extra scenario kept only when scenarioId differs on canonical script",
                }
            )
            if record["scenarioId"] == "default" and any(
                item["scriptId"] == canonical_script
                and item["scenarioId"] == "default"
                for item in merged
            ):
                continue
            record["scriptId"] = canonical_script
            record["sampleId"] = f"{canonical_script}.{record['scenarioId']}"
            if any(item["sampleId"] == record["sampleId"] for item in merged):
                continue
            merged.append(record)
        else:
            merged.append(record)

    merged.sort(key=lambda item: item["sampleId"])
    near_duplicates = [
        {"normalizedSha256": key, "sampleIds": ids}
        for key, ids in sorted(normalized_groups.items())
        if len(set(ids)) > 1
    ]
    eligible = [
        item
        for item in merged
        if item["sampleCategory"] in ELIGIBLE_CATEGORIES
    ]
    n_scripts = len({item["scriptId"] for item in eligible})
    m_scenarios = len(eligible)
    missing_inputs = [
        {
            "sampleId": item["sampleId"],
            "missing": [
                name
                for name, payload in item["hostInputs"].items()
                if isinstance(payload, Mapping)
                and payload.get("status") == "missing"
            ]
            + (
                ["ohlcv"]
                if not item["data"]["ohlcvPath"]
                and item["sampleCategory"] in ELIGIBLE_CATEGORIES
                else []
            ),
        }
        for item in merged
    ]
    missing_inputs = [item for item in missing_inputs if item["missing"]]
    return {
        "records": merged,
        "inventory": inventory,
        "dedup": {
            "method": "source SHA-256 identity; whitespace-normalized hash is advisory only",
            "excludedDuplicates": excluded_duplicates,
            "nearDuplicates": near_duplicates,
            "scriptsN": n_scripts,
            "scenariosM": m_scenarios,
        },
        "missingInputs": missing_inputs,
    }


def freeze_local_permissive(root: Path, revision: str) -> list[dict[str, Any]]:
    candidates_root = root / ".local/upstream-pine-candidates"
    if not candidates_root.is_dir():
        return []
    rust_map = load_runtime_bars_map(root)
    libraries = load_known_libraries(root)
    records: list[dict[str, Any]] = []
    version_re = VERSION_RE
    decl_re = DECLARATION_RE
    for path in sorted(candidates_root.rglob("*")):
        if not path.is_file() or path.suffix.lower() not in {".pine", ".txt"}:
            continue
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        decl = decl_re.search(text)
        if decl is None or decl.group("mode") != "strategy":
            continue
        version_match = version_re.search(text)
        version = int(version_match.group("version")) if version_match else 1
        relative = str(path.relative_to(root)).replace("\\", "/")
        record = build_sample_record(
            root=root,
            relative=relative,
            category="modern_strategy" if version in PINE_VERSIONS else "excluded",
            license_class="permissive",
            license_id=path.parts[path.parts.index("upstream-pine-candidates") + 1]
            if "upstream-pine-candidates" in path.parts
            else "unknown",
            revision=revision,
            rust_map=rust_map,
            libraries=libraries,
            publish_allowed=False,
            retain_allowed=True,
            source_kind="local_permissive_mirror",
        )
        if record is None:
            continue
        records.append(record)
    records.sort(key=lambda item: item["sampleId"])
    return records


def write_jsonl(path: Path, records: Sequence[Mapping[str, Any]]) -> str:
    path.parent.mkdir(parents=True, exist_ok=True)
    lines = [json.dumps(record, sort_keys=True, separators=(",", ":")) for record in records]
    text = "\n".join(lines) + ("\n" if lines else "")
    path.write_text(text, encoding="utf-8")
    return sha256_bytes(text.encode("utf-8"))


def write_json(path: Path, value: Mapping[str, Any] | list[Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def freeze_command(args: argparse.Namespace) -> int:
    root = args.root.resolve()
    revision = args.build_revision or git_revision(root)
    public = freeze_public_corpus(root, revision)
    output_dir = args.output_dir.resolve()
    output_dir.mkdir(parents=True, exist_ok=True)
    manifest_hash = write_jsonl(output_dir / "manifest.jsonl", public["records"])
    write_json(output_dir / "inventory.json", public["inventory"])
    write_json(output_dir / "dedup-report.json", public["dedup"])
    write_json(output_dir / "missing-inputs.json", public["missingInputs"])
    write_json(
        output_dir / "freeze-meta.json",
        {
            "corpusRevision": CORPUS_REVISION,
            "buildRevision": revision,
            "manifestSha256": manifest_hash,
            "scriptsN": public["dedup"]["scriptsN"],
            "scenariosM": public["dedup"]["scenariosM"],
            "sampleCount": len(public["records"]),
        },
    )
    print(
        "modern strategy corpus freeze passed: "
        f"N={public['dedup']['scriptsN']} M={public['dedup']['scenariosM']} "
        f"wrote {output_dir / 'manifest.jsonl'}"
    )
    if args.local_output:
        local_records = freeze_local_permissive(root, revision)
        local_dir = args.local_output.resolve()
        local_hash = write_jsonl(local_dir / "permissive-candidates.jsonl", local_records)
        write_json(
            local_dir / "permissive-candidates-meta.json",
            {
                "count": len(local_records),
                "manifestSha256": local_hash,
                "publishAllowed": False,
            },
        )
        print(
            "local permissive candidate inventory: "
            f"{len(local_records)} samples wrote {local_dir / 'permissive-candidates.jsonl'}"
        )
    return 0


def measure_command(args: argparse.Namespace) -> int:
    root = args.root.resolve()
    manifest_path = args.manifest.resolve()
    pine_compat = args.pine_compat.resolve()
    if not pine_compat.is_file():
        raise SystemExit(
            f"modern strategy corpus error: pine-compat binary not found at {pine_compat}; "
            "run `cargo build -p pine-cli` first"
        )
    try:
        samples = parse_manifest(manifest_path)
        report = build_report(
            samples,
            root=root,
            manifest_path=manifest_path,
            pine_compat=pine_compat,
            build_revision=args.build_revision or git_revision(root),
        )
    except CorpusError as exc:
        raise SystemExit(f"modern strategy corpus error: {exc}") from exc
    rendered = render_report(report)
    if args.output is None:
        print(rendered, end="")
    else:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered, encoding="utf-8")
        print(
            "modern strategy corpus report passed: "
            f"N={report['denominators']['scriptsN']} "
            f"M={report['denominators']['scenariosM']}; wrote {args.output}"
        )
    if args.summary_output is not None:
        args.summary_output.parent.mkdir(parents=True, exist_ok=True)
        args.summary_output.write_text(
            json.dumps(comparable_stage_view(report), indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
    return 0


def parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command")

    freeze = subparsers.add_parser("freeze", help="build the frozen public manifest")
    freeze.add_argument("--root", type=Path, default=ROOT)
    freeze.add_argument(
        "--output-dir",
        type=Path,
        default=ROOT / "tests/fixtures/modern-strategy-corpus/r1",
    )
    freeze.add_argument("--local-output", type=Path)
    freeze.add_argument("--build-revision")

    measure = subparsers.add_parser(
        "measure", help="run staged parse/sema/run/compare metrics"
    )
    measure.add_argument(
        "--manifest",
        type=Path,
        default=ROOT / "tests/fixtures/modern-strategy-corpus/r1/manifest.jsonl",
    )
    measure.add_argument("--root", type=Path, default=ROOT)
    measure.add_argument(
        "--pine-compat",
        type=Path,
        default=ROOT / "target/debug/pine-compat",
    )
    measure.add_argument("--build-revision")
    measure.add_argument("--output", type=Path)
    measure.add_argument("--summary-output", type=Path)

    parser.add_argument("--manifest", type=Path)
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument(
        "--pine-compat",
        type=Path,
        default=ROOT / "target/debug/pine-compat",
    )
    parser.add_argument("--build-revision")
    parser.add_argument("--output", type=Path)
    parser.add_argument("--summary-output", type=Path)
    args = parser.parse_args(argv)
    if args.command is None:
        args.command = "measure"
        if args.manifest is None:
            args.manifest = ROOT / "tests/fixtures/modern-strategy-corpus/r1/manifest.jsonl"
    return args


def main(argv: Sequence[str] | None = None) -> int:
    args = parse_args(argv)
    if args.command == "freeze":
        return freeze_command(args)
    return measure_command(args)


if __name__ == "__main__":
    raise SystemExit(main())

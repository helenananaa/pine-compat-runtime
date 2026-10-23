import hashlib
import json
from pathlib import Path

import pine_compat
import pytest

ROOT = Path(__file__).resolve().parents[2]
CONTEXT_CASES = json.loads((ROOT / "docs/evidence/strategy-library-20261010/constant-bool-declaration-context-oracles-34.json").read_text(encoding="utf8"))["records"]
CASES = json.loads((ROOT / "docs/evidence/strategy-library-20261010/constant-bool-identity-oracles-34.json").read_text(encoding="utf8"))["records"] + [case for case in CONTEXT_CASES if case["native_status"] == "compiled_and_executed"]
BARS = [dict(time=i * 60000, open=10., high=10., low=10., close=10., volume=1.) for i in range(3)]


@pytest.mark.parametrize("case", CASES, ids=lambda case: case["name"])
def test_native_constant_identities_survive_python_and_realtime_boundaries(case):
    raw = (ROOT / case["source"]).read_bytes()
    assert hashlib.sha256(raw).hexdigest() == case["source_sha256"]
    source = raw.decode("utf8")

    def check(result):
        assert [p["values"][-1] for p in result["plots"]] == case["native_values"]

    check(pine_compat.compile_script(source).run(BARS))
    session = pine_compat.create_realtime_session(source)
    session.seed(BARS[:2])
    check(session.update_forming(BARS[2]))
    check(session.update_confirmed(BARS[2]))


def test_native_bool_identity_typeflow_rejections_reach_python_analysis():
    cases = json.loads((ROOT / "docs/evidence/strategy-library-20261010/constant-bool-research-34.json").read_text(encoding="utf8"))["records"]
    for case in cases + CONTEXT_CASES:
        if case["group"] != "typeflow" and case["native_status"] != "analysis_rejected":
            continue
        if case["group"] not in {"typeflow", "typed-declaration-context"}:
            continue
        raw = (ROOT / case["source"]).read_bytes()
        assert hashlib.sha256(raw).hexdigest() == case["source_sha256"]
        report = pine_compat.analyze_script(raw.decode("utf8"))
        assert not report["executable"]
        assert any(d["code"] == "E_CALL_ARG_TYPE" for d in report["diagnostics"])


def test_native_dynamic_declaration_flags_preserve_their_early_compiler_stage():
    cases = json.loads((ROOT / "docs/evidence/strategy-library-20261010/constant-bool-research-34.json").read_text(encoding="utf8"))["records"]
    groups = {"declaration", "not-binding", "config-control", "config-extra", "numeric-config", "na-config", "config"}
    checked = 0
    for case in cases:
        if case["group"] not in groups:
            continue
        raw = (ROOT / case["source"]).read_bytes()
        assert hashlib.sha256(raw).hexdigest() == case["source_sha256"]
        report = pine_compat.analyze_script(raw.decode("utf8"))
        assert report["executable"] == (case["native_status"] == "compiled_and_executed")
        checked += 1
    assert checked == 13


def test_native_input_metadata_defaults_and_titles_match_live_execution():
    case = json.loads((ROOT / "docs/evidence/strategy-library-20261010/constant-bool-input-metadata-oracle-34.json").read_text(encoding="utf8"))
    raw = (ROOT / case["source"]).read_bytes()
    assert hashlib.sha256(raw).hexdigest() == case["source_sha256"]
    source = raw.decode("utf8")
    report = pine_compat.analyze_script(source)
    assert [item["title"] for item in report["inputs"]] == case["native_input_titles"]
    assert [item["default"] for item in report["inputs"]] == case["native_input_defaults"]
    result = pine_compat.compile_script(source).run(BARS)
    assert [plot["values"][-1] for plot in result["plots"]] == case["native_values"]

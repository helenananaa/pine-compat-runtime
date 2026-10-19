import hashlib
import json
from pathlib import Path

import pine_compat
import pytest

ROOT = Path(__file__).resolve().parents[2]
CASES = json.loads((ROOT / "docs/evidence/strategy-library-20261010/constant-param-oracles-35.json").read_text(encoding="utf8"))["records"]
CANDIDATES = [case for case in CASES if case["qualification_scope"] == "candidate_bounded_control"]
BARS = [dict(time=i * 60000, open=10., high=10., low=10., close=10., volume=1.) for i in range(3)]


def source(case):
    raw = (ROOT / case["source"]).read_bytes()
    assert hashlib.sha256(raw).hexdigest() == case["source_sha256"]
    return raw.decode("utf8")


@pytest.mark.parametrize("case", CANDIDATES, ids=lambda case: case["name"])
def test_native_constant_parameter_sources_cross_python_execution_boundaries(case):
    text = source(case)
    report = pine_compat.analyze_script(text)
    if case["native_status"] == "analysis_rejected":
        assert not report["executable"]
        return
    assert report["executable"], report["diagnostics"]

    def check(result):
        assert [plot["values"][-1] for plot in result["plots"]] == case["native_values"]

    check(pine_compat.compile_script(text).run(BARS))
    session = pine_compat.create_realtime_session(text)
    session.seed(BARS[:2])
    check(session.update_forming(BARS[2]))
    check(session.update_confirmed(BARS[2]))


def test_private_library_const_parameter_defaults_and_export_guard():
    root = '//@version=6\nimport test/parameters/1 as lib\nindicator("helper")\nplot(lib.value(true))\n'
    library = '//@version=6\nlibrary("helper")\nhidden(const int x=2)=>x+1\nexport value(simple bool enabled)=>enabled?hidden():0\n'
    result = pine_compat.compile_script(root, library_sources={"test/parameters/1": library}).run(BARS)
    assert result["plots"][0]["values"] == [3, 3, 3]
    exported = next(case for case in CANDIDATES if case.get("library"))
    report = pine_compat.analyze_script(root.replace("plot(lib.value(true))", "plot(close)"), library_sources={"test/parameters/1": source(exported)})
    assert not report["executable"]
    assert any(item["code"] == "E_FUNCTION_PARAM_TYPE" and "exported" in item["message"] for item in report["diagnostics"])

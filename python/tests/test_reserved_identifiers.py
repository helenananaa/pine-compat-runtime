import hashlib
import json
from pathlib import Path

import pine_compat
import pytest

ROOT = Path(__file__).resolve().parents[2]
CASES = json.loads((ROOT / "docs/evidence/strategy-library-20261010/reserved-identifiers-oracles-33.json").read_text(encoding="utf-8"))["cases"]
BARS = [dict(time=i * 60000, open=10., high=10., low=10., close=10., volume=1.) for i in range(3)]


def test_omitted_import_alias_reaches_host_library_resolution():
    case = json.loads((ROOT / "docs/evidence/strategy-library-20261010/reserved-implicit-import-research-33.json").read_text(encoding="utf-8"))
    raw = (ROOT / case["source"]).read_bytes()
    assert hashlib.sha256(raw).hexdigest() == case["source_sha256"]
    report = pine_compat.analyze_script(raw.decode("utf-8"))
    codes = {d["code"] for d in report["diagnostics"]}
    assert not report["executable"]
    assert "E_IMPORT_MISSING_LIBRARY" in codes
    assert "E_RESERVED_IDENTIFIER" not in codes


@pytest.mark.parametrize("case", CASES, ids=lambda case: case["name"])
def test_native_versioned_identifier_rules_across_python_entry_points(case):
    raw = (ROOT / case["source"]).read_bytes()
    assert hashlib.sha256(raw).hexdigest() == case["source_sha256"]
    source = raw.decode("utf-8")
    report = pine_compat.analyze_script(source)
    if case["native_status"] == "analysis_rejected":
        assert not report["executable"]
        assert any(d["code"] == "E_RESERVED_IDENTIFIER" for d in report["diagnostics"])
        with pytest.raises(ValueError, match="reserved identifier"):
            pine_compat.compile_script(source)
        with pytest.raises(ValueError, match="reserved identifier"):
            pine_compat.create_realtime_session(source)
        return

    assert report["executable"], report["diagnostics"]
    expected = [[value] * 3 for value in case["expected_plots"]]
    assert [p["values"] for p in pine_compat.compile_script(source).run(BARS)["plots"]] == expected
    session = pine_compat.create_realtime_session(source)
    session.seed(BARS[:2])
    assert [p["values"] for p in session.update_forming(BARS[2])["plots"]] == expected
    assert [p["values"] for p in session.update_confirmed(BARS[2])["plots"]] == expected

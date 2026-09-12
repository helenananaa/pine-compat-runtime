from pathlib import Path

import pine_compat
import pytest


@pytest.mark.parametrize("version", [6])
def test_active_is_ui_metadata_and_does_not_disable_input_values(version):
    root = Path(__file__).resolve().parents[2]
    source = (root / "tests/fixtures/runtime/input_active_metadata.pine").read_text()
    source = source.replace("version=6", f"version={version}")
    bars = [dict(time=0, open=10., high=10., low=10., close=10., volume=1.)]
    program = pine_compat.compile_script(source)
    result = program.run(bars)
    assert [plot["values"] for plot in result["plots"]] == [[7], [2.5], [1]]
    assert result["plots"][0]["editable"] is False
    report = pine_compat.analyze_script(source)
    length_id = next(item["callSiteId"] for item in report["inputs"] if item["title"] == "Length")
    override = program.run(bars, input_overrides={length_id: 11})
    assert override["plots"][0]["values"] == [11]

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


@pytest.mark.parametrize("version", [3, 4, 5, 6])
def test_udf_inputs_share_source_declaration_and_override(version):
    declaration = "study" if version <= 4 else "indicator"
    number = 'input(2,"Length",type=integer)' if version == 3 else ('input(2,"Length",type=input.integer)' if version == 4 else 'input.int(2,"Length")')
    source = 'input(close,"Source",type=source)' if version == 3 else ('input(close,"Source",type=input.source)' if version == 4 else 'input.source(close,"Source")')
    script = f'//@version={version}\n{declaration}("shared inputs")\nf()=>\n    n={number}\n    s={source}\n    n+s\na=f()\nb=f()\nplot(a)\nplot(b)\n'
    report = pine_compat.analyze_script(script)
    assert len(report["inputs"]) == 2
    length_id = next(x["callSiteId"] for x in report["inputs"] if x["title"] == "Length")
    source_id = next(x["callSiteId"] for x in report["inputs"] if x["title"] == "Source")
    bars = [dict(time=i*60000,open=float(i+1),high=float(i+3),low=float(i),close=float(i+1),volume=1.) for i in range(3)]
    program = pine_compat.compile_script(script)
    assert [p["values"] for p in program.run(bars)["plots"]] == [[3.,4.,5.],[3.,4.,5.]]
    result = program.run(bars,input_overrides={length_id:7,source_id:"high"})
    assert [p["values"] for p in result["plots"]] == [[10.,11.,12.],[10.,11.,12.]]

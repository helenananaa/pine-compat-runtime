import json
from pathlib import Path

import pine_compat
import pytest


ROOT = Path(__file__).resolve().parents[2]


def test_compiled_host_requirements_match_shared_contract_without_data():
    source = (ROOT / "tests/fixtures/host_requirements/strategy.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/host_requirements.json").read_text())
    spans = json.loads((ROOT / "tests/fixtures/host_requirements/source_spans.json").read_text())
    raw = source.encode("utf-8")
    expected["callSites"] = []
    for span in spans:
        text = span["text"].encode("utf-8")
        assert raw.count(text) == 1
        start = raw.index(text)
        expected["callSites"].append({"callSiteId": span["callSiteId"], "source": {
            "sourceId": 0, "libraryKey": None, "start": start, "end": start + len(text)}})
    program = pine_compat.compile_script(source)
    assert program.host_requirements() == expected
    assert program.host_requirements() == expected


@pytest.mark.parametrize("newline", ["\n", "\r\n"])
def test_source_provenance_uses_original_utf8_bytes(newline):
    expression = 'request.security("REMOTE","60",close)'
    source = newline.join(['//@version=6', '// 中文', 'indicator("origin")',
                           f'plot({expression})', ''])
    report = pine_compat.compile_script(source).host_requirements()
    raw = source.encode("utf-8")
    start = raw.index(expression.encode("utf-8"))
    assert report["callSites"] == [{"callSiteId": report["requests"][0]["callSiteId"],
                                  "source": {"sourceId": 0, "libraryKey": None,
                                             "start": start, "end": start + len(expression)}}]


def test_inventory_does_not_require_a_clock_for_an_unreached_read():
    program = pine_compat.compile_script(
        '//@version=6\nindicator("conditional")\nplot(false ? timenow : close)\n'
    )
    assert program.host_requirements()["execution"]["clock"] == "explicitTimestampWhenEvaluated"
    result = program.run([dict(time=0, open=1, high=1, low=1, close=1, volume=1)])
    assert result["plots"][0]["values"] == [1]


def test_unsupported_source_cannot_acquire_an_executable_requirements_report():
    with pytest.raises(ValueError):
        pine_compat.compile_script('//@version=6\nindicator("bad")\nplot(request.financial("X","Y","FQ"))\n')

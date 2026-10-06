import json
from pathlib import Path

import pine_compat
import pytest


ROOT = Path(__file__).resolve().parents[2]


def test_modern_request_options_are_not_reported_as_implicit_defaults():
    program = pine_compat.compile_script('//@version=6\nindicator("merge")\ng=barmerge.gaps_on\nplot(request.security("OTHER","M",close,gaps=g,lookahead=barmerge.lookahead_on))\n')
    report = program.host_requirements()
    assert report["schemaVersion"] == 2
    assert report["requests"][0]["gaps"] == "gapsOn"
    assert report["requests"][0]["lookahead"] == "lookaheadOn"
    assert report["requests"][0]["timeframeRelation"] == "sameOrLowerOrHigherIntegerMultipleExceptCalendarMonths"


@pytest.mark.parametrize("version", [4, 5, 6])
def test_lower_timeframe_inventory_matches_an_executable_provider_request(version):
    declaration = "study" if version == 4 else "indicator"
    function = "security" if version == 4 else "request.security"
    program = pine_compat.compile_script(
        f'//@version={version}\n{declaration}("lower")\n'
        f'plot({function}("REMOTE", "3", close))\n'
    )
    relation = program.host_requirements()["requests"][0]["timeframeRelation"]
    assert relation == "sameOrLowerOrHigherIntegerMultipleExceptCalendarMonths"

    def bar(time, close):
        return dict(time=time, open=close, high=close, low=close, close=close, volume=1)

    result = program.run(
        [bar(0, 10)],
        chart_symbol="CHART",
        chart_timeframe="5",
        request_bars={"REMOTE:3": [bar(0, 1), bar(180_000, 2)]},
    )
    assert result["plots"][0]["values"] == [2]


def test_chart_time_close_variables_and_functions_share_the_host_period():
    program = pine_compat.compile_script(
        '//@version=6\nindicator("close")\nplot(time_close)\n'
        'plot(time_close(""))\nplot(time_close(timeframe.period))\n'
    )
    result = program.run(
        [dict(time=0, open=1, high=1, low=1, close=1, volume=1)],
        chart_timeframe="5",
    )
    assert [plot["values"] for plot in result["plots"]] == [[300_000]] * 3


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

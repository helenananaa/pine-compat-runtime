import pine_compat
import pytest


def bar(time):
    return dict(time=time, open=1, high=1, low=1, close=1, volume=1)


def environment():
    return {"$chart": {"minMove": 1, "priceScale": 10, "currency": "USDT"},
            "$symbolMetadata": {"B": {"minMove": 1, "priceScale": 4,
                                      "quantityPrecision": 3, "currency": "EUR"}},
            "B:2": [bar(0), bar(120000)],
            "B:30S": [bar(i * 30000) for i in range(8)],
            "A:2": [bar(0), bar(120000)]}


@pytest.mark.parametrize("version", [4, 5, 6])
def test_requested_symbol_grid_currency_and_rounding_use_exact_host_metadata(version):
    request = "security" if version == 4 else "request.security"
    declaration = "study" if version == 4 else "indicator"
    source = f'''//@version={version}
{declaration}("Foreign metadata")
plot(syminfo.mintick)
plot({request}("B","2",syminfo.mintick))
plot({request}("B","2",syminfo.currency == "EUR" ? 1 : 0))
'''
    if version >= 5:
        source += f'plot({request}("B","2",syminfo.minmove))\n'
        source += f'plot({request}("B","2",syminfo.pricescale))\n'
        source += f'plot({request}("B","2",math.round_to_mintick(1.13)))\n'
    if version >= 6:
        source += f'plot({request}("B","2",syminfo.mincontract))\n'
    result = pine_compat.compile_script(source).run(
        [bar(i * 60000) for i in range(4)], chart_symbol="A", chart_timeframe="1",
        request_bars=environment())
    expected = [[.1] * 4] + [[None, value, value, value] for value in [.25, 1]]
    if version >= 5:
        expected += [[None, value, value, value] for value in [1, 4, 1.25]]
    if version >= 6:
        expected += [[None, .001, .001, .001]]
    assert [plot["values"] for plot in result["plots"]] == expected


def test_dynamic_nested_requests_recover_the_original_chart_metadata():
    program = pine_compat.compile_script('''//@version=6
indicator("Nested metadata",dynamic_requests=true)
target=input.symbol("B")
root=request.security("A","2",syminfo.mintick)
plot(request.security(target,"2",syminfo.mintick + root))
values=request.security_lower_tf(target,"30S",syminfo.mintick)
plot(array.sum(values))
''')
    bars = [bar(i * 60000) for i in range(4)]
    kwargs = dict(chart_symbol="A", chart_timeframe="1", request_bars=environment())
    result = program.run(bars, **kwargs)
    assert result["plots"][0]["values"] == [None, .35, .35, .35]
    assert result["plots"][1]["values"] == [.5] * 4
    history = program.historical_session(bars, **kwargs)
    for count in range(1, len(bars) + 1):
        continued = history.advance(count)
    assert continued == result
    realtime = program.realtime_session(**kwargs)
    assert realtime.seed(bars) == result


def test_literal_nested_foreign_symbols_keep_their_own_provider_in_string_expressions():
    program = pine_compat.compile_script('''//@version=6
indicator("Literal nested")
plot(request.security("B","2",str.tonumber(str.tostring(request.security("C","2",close)))))
''')
    supplied = {**environment(), "C:2": [{**bar(i * 120000), "open": value, "high": value, "low": value, "close": value}
        for i, value in enumerate((20, 40))]}
    result = program.run([bar(i * 60000) for i in range(4)], chart_symbol="A", chart_timeframe="1", request_bars=supplied)
    assert result["plots"][0]["values"] == [None, 20, 20, 40]


def test_external_append_uses_frozen_symbol_metadata_and_matches_batch():
    from test_external_broker import ACCOUNTS, BARS
    BARS = [{**item, "time": i * 60000} for i, item in enumerate(BARS)]
    ACCOUNTS = [{**item, "time": i * 60000} for i, item in enumerate(ACCOUNTS)]
    program = pine_compat.compile_script('''//@version=6
strategy("Foreign external")
plot(request.security("B","2",syminfo.mintick))
''')
    kwargs = dict(chart_symbol="A", chart_timeframe="1", request_bars=environment())
    session = program.external_session(**kwargs)
    for i in range(len(BARS)):
        result = session.append(BARS[i:i+1], ACCOUNTS[i:i+1])
        assert result == program.run_external(BARS[:i+1], ACCOUNTS[:i+1], **kwargs)
    assert result["output"]["plots"][0]["values"][-1] == .25
    assert pine_compat.SYMBOL_METADATA_SCHEMA_VERSION == 1


@pytest.mark.parametrize("metadata", [[], {"": {"minMove": 1, "priceScale": 4}},
    {"B": {"minMove": True, "priceScale": 4}}, {"B": {"minMove": 0, "priceScale": 4}},
    {"B": {"minMove": 1, "priceScale": 4, "typo": 3}},
    {"B": {"minMove": 1, "priceScale": 4, "pointValue": 2}},
    {"B": {"minMove": 1, "priceScale": 4, "quantityPrecision": 10}},
    {"B": {"minMove": 1, "priceScale": 4, "currency": "eur"}}])
def test_invalid_symbol_metadata_is_rejected_before_execution(metadata):
    with pytest.raises(ValueError):
        pine_compat.compile_script('//@version=6\nindicator("x")\nplot(close)').run(
            [bar(0)], request_bars={"$symbolMetadata": metadata})

import pine_compat


def test_month_seconds_and_inverse_use_supplied_chart_context():
    bars = [{"time": 1704067200000, "open": 1, "high": 1, "low": 1, "close": 1, "volume": 1}]
    for version in [5, 6]:
        source = f'''//@version={version}
indicator("month context")
plot(timeframe.in_seconds())
plot(timeframe.in_seconds(""))
plot(timeframe.in_seconds("12M"))
plot(timeframe.from_seconds(2628003)=="1M"?1:0)
plot(timeframe.from_seconds(31536000)=="12M"?1:0)
plot(timeframe.from_seconds(61)=="2"?1:0)
'''
        program = pine_compat.compile_script(source)
        result = program.run(bars, chart_symbol="BTC", chart_timeframe="1M")
        assert [p["values"] for p in result["plots"]] == [[2628003], [2628003], [31536036], [1], [1], [1]]

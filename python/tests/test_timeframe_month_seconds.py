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


def test_lower_daily_weekly_closes_match_month_chart_native_control():
    source = '//@version=6\nindicator("Lower calendar close oracle 21",precision=0)\nplot(time("D")-time,title="dailyOpen")\nplot(time_close("D")-time,title="dailyClose")\nplot(time("W")-time,title="weeklyOpen")\nplot(time_close("W")-time,title="weeklyClose")\nplot(time_close("D",timeframe_bars_back=1)-time,title="previousDailyClose")\nplot(time_close("D",timeframe_bars_back=-1)-time,title="nextDailyClose")\nplot(time_close("W",timeframe_bars_back=1)-time,title="previousWeeklyClose")\nplot(time_close("W",timeframe_bars_back=-1)-time,title="nextWeeklyClose")\nplot(time_close("D","0930-0945","UTC")-time,title="shortDailyClose")\nplot(time_close("W","0930-0945","UTC")-time,title="shortWeeklyClose")\nplot(time_close("D",bars_back=1)-time,title="chartPreviousDailyClose")\n'
    expected = [[v] for v in [0, 86400000, -259200000, 345600000, 0, 172800000, -259200000, 950400000, None, 294300000, -2505600000]]
    bars = [{"time": 1790812800000, "open": 1, "high": 1, "low": 1, "close": 1, "volume": 1}]
    program = pine_compat.compile_script(source)
    result = program.run(bars, chart_symbol="BINANCE:BTCUSDT", chart_timeframe="M")
    assert [p["values"] for p in result["plots"]] == expected

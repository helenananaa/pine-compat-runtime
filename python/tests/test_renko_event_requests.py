"""Preserve the packaged event-row contract on the current Core source."""
import pytest
import pine_compat


def row(time,value):
    return dict(time=time,open=value-1,high=value,low=value-1,close=value,volume=1)


def test_renko_same_source_time_retains_all_bricks_and_history():
    assert pine_compat.EVENT_REQUEST_SERIES_CONTRACT=="renko-event-rows/1"
    source='''//@version=6
indicator("brick history")
r=ticker.renko(syminfo.tickerid,"Traditional",1)
plot(request.security(r,timeframe.period,close))
plot(request.security(r,timeframe.period,close[1]))
plot(request.security(r,timeframe.period,close,gaps=barmerge.gaps_on))
'''
    symbol='{"chart":"renko","style":"Traditional","param":1,"symbol":"BTC"}'
    main=[row(i*60000,100+i) for i in range(3)]
    bricks=[row(0,101),row(60000,102),row(60000,103),row(120000,104)]
    out=pine_compat.compile_script(source).run(main,chart_symbol="BTC",chart_timeframe="1",request_bars={symbol+":1":bricks})
    assert not out["diagnostics"]
    assert [plot["values"] for plot in out["plots"]]==[[101,103,104],[None,102,103],[101,103,104]]
    assert [item["time"] for item in bricks]==[0,60000,60000,120000]


def test_empty_known_renko_stream_is_na_before_any_confirmed_brick():
    source='//@version=6\nindicator("empty bricks")\nplot(request.security(ticker.renko(syminfo.tickerid,"Traditional",1),timeframe.period,close))'
    symbol='{"chart":"renko","style":"Traditional","param":1,"symbol":"BTC"}'
    out=pine_compat.compile_script(source).run([row(0,100)],chart_symbol="BTC",chart_timeframe="1",request_bars={symbol+":1":[]})
    assert not out["diagnostics"] and out["plots"][0]["values"]==[None]


def test_duplicate_standard_context_and_descending_event_rows_still_rejected():
    program=pine_compat.compile_script('//@version=6\nindicator("ordinary")\nplot(close)')
    with pytest.raises(ValueError,match="duplicate requested bar"):
        program.run([row(0,100)],request_bars={"BTC:1":[row(0,100),row(0,101)]})
    key='{"chart":"renko","symbol":"BTC"}:1'
    with pytest.raises(ValueError,match="not sorted"):
        program.run([row(0,100)],request_bars={key:[row(60000,100),row(0,101)]})

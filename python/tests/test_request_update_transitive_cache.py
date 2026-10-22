import pine_compat
import pytest


def bar(index, value):
    return dict(time=index*60_000,open=value,high=value,low=value,close=value,volume=1)


@pytest.mark.parametrize("dynamic", ["true", "false"])
@pytest.mark.parametrize("reference", [True, False])
def test_inner_feed_growth_invalidates_the_outer_fallback_cache(dynamic, reference):
    body = ('inner=request.security("B","1",close)\nplot(request.security("C","1",inner))'
            if reference else 'plot(request.security("C","1",request.security("B","1",close)))')
    source=f'//@version=6\nindicator("transitive",dynamic_requests={dynamic})\n{body}'
    requests={"B:1":[bar(0,100)],"C:1":[bar(0,1),bar(1,1)]}
    program=pine_compat.compile_script(source)
    session=program.realtime_session(chart_symbol="A",chart_timeframe="1",request_bars=requests)
    assert session.seed([bar(0,1)])["plots"][0]["values"]==[100]
    session.apply_request_confirmed("B","1",bar(1,200))
    actual=session.update_confirmed(bar(1,1))
    expected=program.run([bar(0,1),bar(1,1)],chart_symbol="A",chart_timeframe="1",
                         request_bars={**requests,"B:1":[bar(0,100),bar(1,200)]})
    assert actual["plots"]==expected["plots"]
    assert actual["plots"][0]["values"]==[100,200]
    before,revision=session.result(),session.revision
    with pytest.raises(ValueError):
        session.apply_request_confirmed("B","1",bar(1,999))
    assert session.result()==before and session.revision==revision

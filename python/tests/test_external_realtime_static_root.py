"""Static root dependencies use each observed realtime chart tuple."""
import pytest
import pine_compat


def data():
    bars=[];accounts=[];groups=[]
    for index in range(2):
        opening=10+10*index
        bar=dict(time=index*60000,open=opening,high=opening+2,low=opening-1,close=opening+1,volume=100*(index+1))
        account=dict(time=bar["time"],position_size=0,position_avg_price=None,equity=1000,initial_capital=1000,netprofit=0,openprofit=0)
        high=low=opening;group=[]
        for tick,price in enumerate([opening,bar["high"],bar["low"],bar["close"]]):
            high=max(high,price);low=min(low,price)
            group.append(dict(evaluation=dict(bar={**bar,"high":high,"low":low,"close":price,"volume":bar["volume"]*(tick+1)/4},account=account,event_time_ms=bar["time"]+59999*tick//3,confirmed=tick==3),context=dict(bar_state="realtime",trigger="bar_close" if tick==3 else "realtime_tick",is_new=tick==0)))
        bars.append(bar);accounts.append(account);groups.append(group)
    return bars,accounts,groups


@pytest.mark.parametrize("history",[False,True])
@pytest.mark.parametrize("field",["open","high","low","close","volume"])
def test_observed_static_root_matches_native_realtime_and_append_reconstruction(field,history):
    source=f'''//@version=6
strategy("Static realtime root",calc_on_every_tick=true,calc_on_every_history_tick={str(history).lower()},dynamic_requests=false)
x=request.security(syminfo.tickerid,timeframe.period,{field})
y=request.security("B","1",x)
varip float direct=0
varip float nested=0
direct:=direct*100+x
nested:=nested*100+y
plot(direct)
plot(nested)
'''
    bars,accounts,groups=data()
    requests={"B:1":[dict(time=i*60000,open=50,high=50,low=50,close=50,volume=1) for i in range(2)]}
    options=dict(chart_symbol="A",chart_timeframe="1",request_bars=requests)
    program=pine_compat.compile_script(source)
    actual=program.run_external(bars,accounts,execution_schedule=groups,**options)
    expected=[];total=0
    for group in groups:
        for item in group:
            total=total*100+item["evaluation"]["bar"][field]
        expected.append(total)
    assert [row["values"] for row in actual["output"]["plots"]] == [expected,expected]
    session=program.external_session(scheduled=True,**options)
    for i in range(2):
        appended=session.append(bars[i:i+1],accounts[i:i+1],execution_schedule=groups[i:i+1])
    assert appended == actual
    native=pine_compat.create_realtime_session(source,**options)
    native.seed([])
    for group in groups:
        for item in group:
            evaluation=item["evaluation"]
            output=(native.update_confirmed if evaluation["confirmed"] else native.update_forming)(evaluation["bar"],execution_time=evaluation["event_time_ms"])
    assert [row["values"] for row in output["plots"]] == [expected,expected]

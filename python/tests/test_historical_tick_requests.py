"""Completed historical request contexts do not follow synthetic path clocks."""
from copy import deepcopy
import pytest
import pine_compat
from test_external_stream import data


SOURCE='''//@version=6
strategy("historical requests",calc_on_every_history_tick=true,calc_on_every_tick=true)
varip int n=0
n+=1
a=request.security("B","1",close)
b=request.security("B","2",close)
l=request.security_lower_tf("B","30S",close)
varip array<float> av=array.new_float(4,na)
varip array<float> bv=array.new_float(4,na)
varip array<float> lv=array.new_float(4,na)
i=(n-1)%4
array.set(av,i,a)
array.set(bv,i,b)
array.set(lv,i,array.size(l))
'''+''.join(f'plot(array.get({name},{i}))\n' for name in ('av','bv','lv') for i in range(4))


def updates(index):
    def row(time,price):
        return dict(time=time,open=price,high=price,low=price,close=price,volume=1)
    result=[dict(symbol="B",timeframe="1",kind="confirmed",bar=row(index*60000,100*(index+1)),available_at_ms=index*60000+59999),
            dict(symbol="B",timeframe="30S",kind="confirmed",bar=row(index*60000,10+index),available_at_ms=index*60000+29999),
            dict(symbol="B",timeframe="30S",kind="confirmed",bar=row(index*60000+30000,20+index),available_at_ms=index*60000+59999)]
    if index==1: result.append(dict(symbol="B",timeframe="2",kind="confirmed",bar=row(0,1000),available_at_ms=119999))
    return result


def test_historical_tick_request_values_match_native_and_completed_bar_reference():
    bars,_,groups=data("historical")
    full={"B:1":[],"B:2":[],"B:30S":[]}
    for index in range(2):
        for update in updates(index): full["B:"+update["timeframe"]].append(update["bar"])
    options=dict(chart_symbol="A",chart_timeframe="1")
    program=pine_compat.compile_script(SOURCE)
    native=program.run(bars,request_bars=full,**options)
    session=program.external_session(scheduled=True,request_bars={key:[] for key in full},**options)
    assert session.initialize_history(bars)==0
    for index,(bar,group) in enumerate(zip(bars,groups,strict=True)):
        session.begin_bar(bar,bar_state="historical")
        for ordinal,observation in enumerate(group):
            observation=deepcopy(observation)
            observation["evaluation"]["request_updates"]=updates(index) if ordinal==0 else []
            session.observe(observation,snapshot=False)
        session.commit_bar(snapshot=False)
    assert session.snapshot()["output"]["plots"]==native["plots"]
    assert [plot["values"] for plot in native["plots"]]==[[100,200]]*4+[[None,1000]]*4+[[2,2]]*4


@pytest.mark.parametrize("state,future",[("historical",updates(1)[-1]),("realtime",updates(0)[0])])
def test_historical_reveal_still_rejects_later_bars_and_realtime_rejects_unobserved_data(state,future):
    bars,_,groups=data(state)
    session=pine_compat.compile_script(SOURCE).external_session(scheduled=True,chart_symbol="A",chart_timeframe="1",
        request_bars={"B:1":[],"B:2":[],"B:30S":[]})
    session.begin_bar(bars[0],bar_state=state)
    before=session.snapshot()
    observation=deepcopy(groups[0][0])
    observation["evaluation"]["request_updates"]=[future]
    with pytest.raises(ValueError,match="premature, future or regressed"):
        session.observe(observation)
    assert session.snapshot()==before and session.cursor==0 and session.bar_open

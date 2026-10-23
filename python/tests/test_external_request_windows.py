"""Requested window metadata fixes initial selection without revealing prices."""
from copy import deepcopy

import pytest
import pine_compat
from test_external_stream import data


@pytest.mark.parametrize("expression",[
    'request.security("B","1",ta.cum(close),calc_bars_count=1)',
    'request.security("B","1",ta.cum(close),calc_bars_count=2)',
    'array.sum(request.security_lower_tf("B","30S",close,calc_bars_count=1))',
    'request.security("B","1",request.security("C","1",ta.cum(close),calc_bars_count=1))',
])
def test_known_requested_endpoints_match_native_values_on_every_historical_vertex(expression):
    assert pine_compat.EXTERNAL_REQUEST_WINDOW_PLAN_SCHEMA_VERSION==1
    bars,_,groups=data("historical")
    def row(time,price): return dict(time=time,open=price,high=price,low=price,close=price,volume=1)
    requests={"B:1":[row(0,100),row(60000,200)],"C:1":[row(0,1000),row(60000,2000)],
              "B:30S":[row(i*30000,10*(i+1)) for i in range(4)]}
    histories=[dict(symbol=key.split(":")[0],timeframe=key.split(":")[1],times_ms=[bar["time"] for bar in rows]) for key,rows in requests.items()]
    source='//@version=6\nstrategy("window vertices",calc_on_every_history_tick=true)\nvarip int n=0\nn+=1\nx='+expression+'\nvarip array<float> seen=array.new_float(4,na)\narray.set(seen,(n-1)%4,x)\n'+''.join(f'plot(array.get(seen,{i}))\n' for i in range(4))
    program=pine_compat.compile_script(source)
    native=program.run(bars,request_bars=requests,chart_symbol="A",chart_timeframe="1")
    session=program.external_session(scheduled=True,request_bars={key:[] for key in requests},chart_symbol="A",chart_timeframe="1")
    assert session.initialize_history(bars,requested_history=histories)==0
    assert session.snapshot()["intents"]==[] and session.snapshot()["output"]["plots"]==[]
    for bar,group in zip(bars,groups,strict=True):
        session.begin_bar(bar,bar_state="historical")
        updates=[]
        for key,rows in requests.items():
            symbol,timeframe=key.split(":")
            step=30000 if timeframe=="30S" else 60000
            updates.extend(dict(symbol=symbol,timeframe=timeframe,bar=item,kind="confirmed",available_at_ms=item["time"]+step-1)
                           for item in rows if bar["time"]<=item["time"]<bar["time"]+60000)
        for ordinal,observation in enumerate(group):
            observation=deepcopy(observation)
            observation["evaluation"]["request_updates"]=updates if ordinal==0 else []
            session.observe(observation,snapshot=False)
        session.commit_bar(snapshot=False)
    assert session.snapshot()["output"]["plots"]==native["plots"]
    if expression.startswith('request.security("B","1",ta.cum(close),calc_bars_count=1)'):
        assert [plot["values"] for plot in native["plots"]]==[[None,200]]*4


@pytest.mark.parametrize("history",[
    [dict(symbol="B",timeframe="1",times_ms=[0,0])],
    [dict(symbol=" B",timeframe="1",times_ms=[0])],
    [dict(symbol="A",timeframe="1",times_ms=[0])],
    [dict(symbol="B",timeframe="1",times_ms=[0])]*2,
    [dict(symbol="B",timeframe="1",times_ms=[0],bars=[])],
])
def test_invalid_requested_window_metadata_is_atomic_and_retryable(history):
    bars,_,_=data("historical")
    session=pine_compat.compile_script('//@version=6\nstrategy("metadata",calc_on_every_history_tick=true)\nplot(close)').external_session(scheduled=True,chart_symbol="A",chart_timeframe="1")
    before=session.snapshot()
    with pytest.raises(ValueError): session.initialize_history(bars,requested_history=history)
    assert session.snapshot()==before
    assert session.initialize_history(bars,requested_history=[])==0


def test_known_future_endpoints_neither_populate_prices_nor_allow_future_feed_updates():
    bars,_,groups=data("historical")
    session=pine_compat.compile_script('//@version=6\nstrategy("metadata visibility",calc_on_every_history_tick=true)\nplot(request.security("B","1",close,calc_bars_count=1))').external_session(
        scheduled=True,chart_symbol="A",chart_timeframe="1",request_bars={"B:1":[]})
    session.initialize_history(bars,requested_history=[dict(symbol="B",timeframe="1",times_ms=[0,60000])])
    session.begin_bar(bars[0],bar_state="historical")
    before=session.snapshot()
    future=deepcopy(groups[0][0])
    future["evaluation"]["request_updates"]=[dict(symbol="B",timeframe="1",kind="confirmed",
        bar=dict(time=60000,open=999,high=999,low=999,close=999,volume=1),available_at_ms=119999)]
    with pytest.raises(ValueError,match="premature, future or regressed"): session.observe(future)
    assert session.snapshot()==before
    for observation in groups[0]:
        observation=deepcopy(observation)
        observation["evaluation"]["request_updates"]=[]
        session.observe(observation,snapshot=False)
    session.commit_bar(snapshot=False)
    assert session.snapshot()["output"]["plots"][0]["values"]==[None]


def test_initial_requested_window_keeps_its_origin_when_realtime_data_appends():
    source='//@version=6\nstrategy("window continuation",calc_on_every_history_tick=true,calc_on_every_tick=true)\nplot(request.security("B","1",ta.cum(close),calc_bars_count=1))'
    bars,_,groups=data("historical")
    def row(time,price): return dict(time=time,open=price,high=price,low=price,close=price,volume=1)
    requested=[row(0,100),row(60000,200)]
    native=pine_compat.create_realtime_session(source,chart_symbol="A",chart_timeframe="1",request_bars={"B:1":requested})
    native.seed(bars)
    session=pine_compat.compile_script(source).external_session(scheduled=True,chart_symbol="A",chart_timeframe="1",request_bars={"B:1":[]})
    session.initialize_history(bars,requested_history=[dict(symbol="B",timeframe="1",times_ms=[0,60000])])
    for index,group in enumerate(groups):
        session.begin_bar(bars[index],bar_state="historical")
        for ordinal,item in enumerate(group):
            item=deepcopy(item)
            item["evaluation"]["request_updates"]=[dict(symbol="B",timeframe="1",kind="confirmed",bar=requested[index],available_at_ms=bars[index]["time"]+59999)] if ordinal==0 else []
            session.observe(item,snapshot=False)
        session.commit_bar(snapshot=False)
    current=row(120000,30)
    account=dict(time=120000,position_size=0,equity=1000,initial_capital=1000,netprofit=0,openprofit=0)
    session.begin_bar(current,bar_state="realtime")
    for confirmed in (False,True):
        updates=[]
        if confirmed:
            update=dict(symbol="B",timeframe="1",kind="confirmed",bar=row(120000,300))
            native.apply_request_feed_updates([update])
            updates=[{**update,"available_at_ms":179999}]
        stamp=179999 if confirmed else 120000
        expected=(native.update_confirmed if confirmed else native.update_forming)(current,execution_time=stamp)
        actual=session.observe(dict(evaluation=dict(bar=current,account=account,event_time_ms=stamp,confirmed=confirmed,request_updates=updates),
            context=dict(bar_state="realtime",trigger="bar_close" if confirmed else "realtime_tick",is_new=not confirmed)))
        assert actual["output"]["plots"]==expected["plots"]
    session.commit_bar(snapshot=False)
    assert session.snapshot()["output"]["plots"][0]["values"]==[None,200,500]

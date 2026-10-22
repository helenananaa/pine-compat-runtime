"""Persistent same-bar observations and bounded per-observation result transport."""
from copy import deepcopy
import pytest
import pine_compat

SOURCE='//@version=6\nstrategy("stream",calc_on_every_tick=true,calc_on_every_history_tick=true)\nvar int ordinary=0\nvarip int n=0\nordinary+=1\nn+=1\nplot(ordinary)\nplot(n)\nplot(ta.cum(volume))\nplot(strategy.position_size)\nstrategy.entry(str.tostring(n),strategy.long,qty=1)'


def data(state):
    bars=[];accounts=[];groups=[]
    for index in range(2):
        opening=10+index*10
        bar=dict(time=index*60000,open=opening,high=opening+2,low=opening-1,close=opening+1,volume=100*(index+1))
        group=[];high=low=opening
        for tick,price in enumerate([opening,bar["high"],bar["low"],bar["close"]]):
            high=max(high,price);low=min(low,price)
            observed=bar|dict(high=high,low=low,close=price,volume=bar["volume"]*(tick+1)/4)
            account=dict(time=bar["time"],position_size=tick,position_avg_price=opening if tick else None,
                equity=1000,initial_capital=1000,netprofit=0,openprofit=0)
            trigger="historical_tick" if state=="historical" else "bar_close" if tick==3 else "realtime_tick"
            group.append(dict(evaluation=dict(bar=observed,account=account,event_time_ms=bar["time"]+59999*tick//3,confirmed=tick==3),
                context=dict(bar_state=state,trigger=trigger,is_new=state=="historical" or tick==0)))
        bars.append(bar);accounts.append(group[-1]["evaluation"]["account"]);groups.append(group)
    return bars,accounts,groups


@pytest.mark.parametrize("state",["historical","realtime"])
def test_streaming_returns_only_new_intents_and_matches_complete_schedule(state):
    assert pine_compat.EXTERNAL_STREAM_SCHEMA_VERSION==1
    bars,accounts,groups=data(state)
    program=pine_compat.compile_script(SOURCE)
    complete=program.run_external(bars,accounts,execution_schedule=groups)
    session=program.external_session(scheduled=True)
    intents=[]
    for index,(bar,group) in enumerate(zip(bars,groups,strict=True)):
        session.begin_bar(bar,bar_state=state)
        assert session.cursor==index and session.bar_open
        for observation in group:
            result=session.observe(observation,snapshot=False)
            assert result["stream_schema"]==1 and result["intent_base"]==len(intents)
            assert result["bar_cursor"]==index and result["bar_open"]
            assert "output" not in result and len(result["intents"])==1
            intents.extend(result["intents"])
        committed=session.commit_bar(snapshot=False)
        assert committed["bar_cursor"]==index+1 and not committed["bar_open"]
        assert committed["intents"]==[] and committed["intent_base"]==len(intents)
    assert session.snapshot()==complete
    assert intents==complete["intents"]
    without=session.snapshot(include_intents=False)
    assert without["output"]==complete["output"] and without["intents"]==[]


def test_invalid_append_or_observation_does_not_poison_the_open_bar():
    bars,accounts,groups=data("realtime")
    session=pine_compat.compile_script(SOURCE).external_session(scheduled=True)
    with pytest.raises(ValueError):
        session.begin_bar(bars[0],bar_state="realtime ")
    session.begin_bar(bars[0],bar_state="realtime")
    first=session.observe(groups[0][0])
    with pytest.raises(ValueError):
        session.append(bars[1:],accounts[1:],execution_schedule=groups[1:])
    assert session.snapshot()["output"]==first["output"]
    bad=deepcopy(groups[0][1]);bad["evaluation"]["event_time_ms"]=-1
    with pytest.raises(ValueError):
        session.observe(bad)
    with pytest.raises(ValueError):
        session.commit_bar()
    assert session.cursor==0 and session.bar_open
    for observation in groups[0][1:]:
        session.observe(observation)
    session.commit_bar()
    assert session.cursor==1 and not session.bar_open


def test_execution_error_rejects_snapshot_and_further_updates():
    source='//@version=6\nstrategy("error",calc_on_every_tick=true)\nvarip int n=0\nn+=1\nplot(n)\nif n==2\n    runtime.error("deliberate failure")'
    bars,_,groups=data("realtime")
    session=pine_compat.compile_script(source).external_session(scheduled=True)
    session.begin_bar(bars[0],bar_state="realtime")
    session.observe(groups[0][0])
    with pytest.raises(ValueError,match="deliberate failure"):
        session.observe(groups[0][1])
    for action in [session.snapshot,session.commit_bar,lambda:session.observe(groups[0][2])]:
        with pytest.raises(ValueError,match="E_RUNTIME_POISONED"):
            action()
    assert session.cursor==0 and session.bar_open


def test_realtime_batch_chart_boundaries_match_native_observation_control():
    source='//@version=6\nstrategy("boundaries",calc_on_every_tick=true)\nplot(last_bar_index)\nplot(last_bar_time)'
    bars,accounts,groups=data("realtime")
    actual=pine_compat.compile_script(source).run_external(bars,accounts,execution_schedule=groups)
    session=pine_compat.create_realtime_session(source)
    session.seed([])
    for group in groups:
        for observation in group:
            frame=observation["evaluation"]
            expected=(session.update_confirmed if frame["confirmed"] else session.update_forming)(frame["bar"],execution_time=frame["event_time_ms"])
    assert [p["values"] for p in actual["output"]["plots"]]==[p["values"] for p in expected["plots"]]


@pytest.mark.parametrize("count",[1,2])
def test_known_historical_plan_keeps_endpoint_and_window_without_future_account_frames(count):
    source=f'//@version=6\nstrategy("planned",calc_on_every_history_tick=true,calc_bars_count={count})\nvarip int n=0\nn+=1\nplot(n)\nplot(bar_index)\nplot(last_bar_index)\nplot(barstate.islast?1:0)\nplot(last_bar_time)\nplot(close)'
    bars,accounts,groups=data("historical")
    program=pine_compat.compile_script(source)
    expected=program.run_external(bars,accounts,execution_schedule=groups)
    session=program.external_session(scheduled=True)
    skip=session.initialize_history(bars)
    assert skip==2-count and session.cursor==0 and not session.bar_open
    assert session.snapshot()["intents"]==[] and session.snapshot()["output"]["plots"]==[]
    for bar,group in zip(bars[skip:],groups[skip:],strict=True):
        session.begin_bar(bar,bar_state="historical")
        for observation in group:
            session.observe(observation,snapshot=False)
        session.commit_bar(snapshot=False)
    assert session.snapshot()==expected


def test_static_requested_windows_are_planned_without_evaluating_future_feedback():
    source='//@version=6\nstrategy("planned request",calc_on_every_history_tick=true,calc_bars_count=1)\nplot(request.security("B","1",ta.cum(close),calc_bars_count=2))\nplot(bar_index)\nplot(last_bar_index)'
    bars,accounts,groups=data("historical")
    requests={"B:1":[dict(time=b["time"],open=100,high=100,low=100,close=100,volume=1) for b in bars]}
    options=dict(chart_symbol="A",chart_timeframe="1",request_bars=requests)
    program=pine_compat.compile_script(source)
    expected=program.run_external(bars,accounts,execution_schedule=groups,**options)
    session=program.external_session(scheduled=True,**options)
    skip=session.initialize_history(bars)
    assert skip==1 and session.snapshot()["intents"]==[]
    for bar,group in zip(bars[skip:],groups[skip:],strict=True):
        session.begin_bar(bar,bar_state="historical")
        for observation in group:
            session.observe(observation,snapshot=False)
        session.commit_bar(snapshot=False)
    assert session.snapshot()==expected


def test_unresolved_window_discovery_does_not_invent_an_account_or_poison_metadata_retry():
    source='//@version=6\nstrategy("dynamic plan",calc_on_every_history_tick=true,calc_bars_count=1)\ns=bar_index==0?"B":"C"\nplot(request.security(s,"1",close))'
    bars,_,_=data("historical")
    options=dict(chart_symbol="A",chart_timeframe="1",request_bars={"B:1":bars,"C:1":bars})
    session=pine_compat.compile_script(source).external_session(scheduled=True,**options)
    before=session.snapshot()
    for _ in range(2):
        with pytest.raises(ValueError,match="statically resolved request windows"):
            session.initialize_history(bars)
        assert session.snapshot()==before and session.cursor==0 and not session.bar_open


def test_invalid_historical_metadata_is_rejected_before_publishing_a_plan():
    bars,_,_=data("historical")
    session=pine_compat.compile_script(SOURCE).external_session(scheduled=True)
    with pytest.raises(ValueError):
        session.initialize_history([bars[0],bars[0]])
    assert session.initialize_history(bars)==0
    assert session.cursor==0 and not session.bar_open

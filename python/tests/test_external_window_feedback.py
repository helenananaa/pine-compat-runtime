"""Observed window discovery resets Pine and prices without inventing accounts."""
from copy import deepcopy
import pine_compat
import pytest
from test_external_stream import data

SOURCE='''//@version=6
strategy("observed window",calc_on_every_history_tick=true,calc_bars_count=1)
varip int n=0
n+=1
strategy.entry(str.tostring(n),strategy.long,qty=1)
s=strategy.position_size>0?"A":"B"
plot(request.security(s,"1",ta.cum(close),calc_bars_count=2))
plot(n)
plot(strategy.position_size)
'''


def inputs():
    bars,accounts,groups=data("historical")
    requested=[dict(time=b["time"],open=100*(i+1),high=100*(i+1),low=100*(i+1),close=100*(i+1),volume=1)
               for i,b in enumerate(bars)]
    for group,row in zip(groups,requested,strict=True):
        for ordinal,observation in enumerate(group):
            observation["evaluation"]["request_updates"]=([dict(symbol="B",timeframe="1",kind="confirmed",
                bar=row,available_at_ms=row["time"]+59999)] if ordinal==0 else [])
    return bars,accounts,groups,requested


def session(source=SOURCE):
    return pine_compat.compile_script(source).external_session(scheduled=True,chart_symbol="A",chart_timeframe="1",request_bars={"B:1":[]})


def initialize(engine,bars):
    return engine.initialize_history(bars,observed_feedback=True,
        requested_history=[dict(symbol="B",timeframe="1",times_ms=[b["time"] for b in bars])])


def replay(engine,bars,groups,skip):
    for bar,group in zip(bars[skip:],groups[skip:],strict=True):
        engine.begin_bar(bar,bar_state="historical")
        for observation in group: engine.observe(observation,snapshot=False)
        engine.commit_bar(snapshot=False)


def test_actual_feedback_expands_root_and_resets_prices_varip_intents_and_all_cursors():
    assert pine_compat.EXTERNAL_WINDOW_FEEDBACK_SCHEMA_VERSION==1
    bars,accounts,groups,_=inputs()
    engine=session()
    assert initialize(engine,bars)==1
    assert engine.window_status()==dict(schema_version=1,requires_discovery=True,rebase_required=False,input_skip=1)
    before=engine.snapshot()
    with pytest.raises(ValueError): engine.rebase_window(seal=True)
    assert engine.snapshot()==before
    engine.begin_bar(bars[-1],bar_state="historical")
    assert len(engine.observe(groups[-1][0])["intents"])==1
    with pytest.raises(ValueError,match="initial context window changed"): engine.observe(groups[-1][1])
    assert engine.window_status()==dict(schema_version=1,requires_discovery=True,rebase_required=True,input_skip=0)
    with pytest.raises(ValueError,match="POISONED"): engine.snapshot()
    with pytest.raises(ValueError): engine.rebase_window(seal=True)
    assert engine.rebase_window()==0
    assert not engine.bar_open and engine.cursor==0 and engine.snapshot()["intents"]==[]
    # This earlier requested price must be accepted after discarding the later
    # requested prefix delivered to the first interrupted execution.
    replay(engine,bars,groups,0)
    stable=engine.snapshot()
    assert stable["output"]["plots"][1]["values"]==[4,8]
    assert len(stable["intents"])==8
    assert engine.window_status()["requires_discovery"]
    assert engine.rebase_window(seal=True)==0
    assert not engine.window_status()["requires_discovery"]
    replay(engine,bars,groups,0)
    assert engine.snapshot()==stable
    reference=pine_compat.compile_script(SOURCE).run_external(bars,accounts,execution_schedule=groups,
        chart_symbol="A",chart_timeframe="1",request_bars={"B:1":[]})
    assert reference==stable


def test_initialization_does_not_execute_pine_and_ordinary_errors_stay_poisoned():
    bars,_,groups,_=inputs()
    source=SOURCE+'\nruntime.error("actual failure")'
    engine=session(source)
    initialize(engine,bars)
    assert engine.snapshot()["output"]["plots"]==[]
    engine.begin_bar(bars[-1],bar_state="historical")
    with pytest.raises(ValueError,match="actual failure"): engine.observe(groups[-1][0])
    assert not engine.window_status()["rebase_required"]
    for seal in (False,True):
        with pytest.raises(ValueError): engine.rebase_window(seal=seal)
    with pytest.raises(ValueError,match="POISONED"): engine.observe(groups[-1][1])


@pytest.mark.parametrize("mutation",["price","time","state"])
def test_observed_plan_requires_exact_historical_rows_before_any_execution(mutation):
    bars,_,groups,_=inputs()
    engine=session()
    initialize(engine,bars)
    row=deepcopy(bars[-1]); state="historical"
    if mutation=="price": row["close"]+=1
    if mutation=="time": row["time"]+=60000
    if mutation=="state": state="realtime"
    before=engine.snapshot()
    with pytest.raises(ValueError): engine.begin_bar(row,bar_state=state)
    assert engine.snapshot()==before
    engine.begin_bar(bars[-1],bar_state="historical")
    engine.observe(groups[-1][0])


def test_sealing_still_detects_an_unseen_context_and_requires_discovery_again():
    bars,_,groups,_=inputs()
    engine=session()
    initialize(engine,bars)
    flat=deepcopy(groups)
    for group in flat:
        for item in group:
            item["evaluation"]["account"].update(position_size=0,position_avg_price=None)
    replay(engine,bars,flat,1)
    assert engine.rebase_window(seal=True)==1
    engine.begin_bar(bars[-1],bar_state="historical")
    engine.observe(groups[-1][0])
    with pytest.raises(ValueError,match="initial context window changed"): engine.observe(groups[-1][1])
    assert engine.window_status()["requires_discovery"] and engine.window_status()["rebase_required"]
    assert engine.rebase_window()==0
    assert engine.window_status()["requires_discovery"]


def test_default_initialization_does_not_enable_dynamic_observed_planning():
    bars,_,_,_=inputs()
    engine=session()
    with pytest.raises(ValueError,match="observed account feedback"): engine.initialize_history(bars)
    assert engine.window_status() is None
    assert initialize(engine,bars)==1


def test_batch_append_cannot_bypass_the_observed_history_plan_or_seal_a_different_dataset():
    bars,accounts,groups,_=inputs()
    engine=session()
    initialize(engine,bars)
    before=engine.snapshot()
    with pytest.raises(ValueError,match="begin/observe/commit"):
        engine.append(bars,accounts,execution_schedule=groups)
    # A batch execution rejection retains the existing session failure policy.
    assert not engine.window_status()["rebase_required"]
    with pytest.raises(ValueError): engine.rebase_window(seal=True)


def test_declaration_window_without_live_requests_keeps_the_same_skip_after_reset():
    bars,_,groups,_=inputs()
    engine=session('//@version=6\nstrategy("static window",calc_on_every_history_tick=true,calc_bars_count=1)\nvarip int n=0\nn+=1\nplot(n)')
    assert initialize(engine,bars)==1
    assert engine.window_status()==dict(schema_version=1,requires_discovery=False,rebase_required=False,input_skip=1)
    replay(engine,bars,groups,1)
    assert engine.snapshot()["output"]["plots"][0]["values"]==[4]
    assert engine.rebase_window(seal=True)==1
    replay(engine,bars,groups,1)
    assert engine.snapshot()["output"]["plots"][0]["values"]==[4]

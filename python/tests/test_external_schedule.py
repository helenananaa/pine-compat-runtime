"""Installed binding checks for explicit execution contexts and account authority."""
from copy import deepcopy

import pine_compat
import pytest


def bar(time=0, close=12):
    return dict(time=time, open=10, high=12, low=9, close=close, volume=100)


def account(time=0, position=0):
    return dict(time=time, position_size=position,
                position_avg_price=10 if position else None,
                equity=1000, initial_capital=1000, netprofit=0, openprofit=0)


def scheduled(observation, position=0, *, state="historical", trigger="bar_close",
              commit=True, new=True, clock=None):
    return dict(evaluation=dict(bar=observation, account=account(observation["time"], position),
                               event_time_ms=observation["time"] + 59999 if clock is None else clock,
                               confirmed=commit),
                context=dict(bar_state=state, trigger=trigger, is_new=new))


def compile_strategy(settings="", body="plot(close)"):
    return pine_compat.compile_script(f'//@version=6\nstrategy("Schedule"{settings})\n{body}')


def test_historical_fill_context_keeps_confirmation_and_host_account_authority():
    program = compile_strategy(",calc_on_order_fills=true", """
var int ordinary = 0
varip int calls = 0
ordinary += 1
calls += 1
plot(ordinary)
plot(calls)
plot(barstate.isconfirmed ? 1 : 0)
plot(barstate.isnew ? 1 : 0)
plot(barstate.ishistory ? 1 : 0)
plot(barstate.isrealtime ? 1 : 0)
if barstate.isconfirmed
    if strategy.position_size > 0
        strategy.close("L")
    else
        strategy.entry("L", strategy.long, qty=2)
""")
    group = [scheduled(bar(), 2, trigger="order_fill", commit=False), scheduled(bar())]
    actual = program.run_external([bar()], [account()], execution_schedule=[group])
    assert pine_compat.EXTERNAL_SCHEDULE_SCHEMA_VERSION == 1
    assert [plot["values"] for plot in actual["output"]["plots"]] == [[1], [2], [1], [1], [1], [0]]
    assert [(item["pass_index"], item["action"]) for item in actual["intents"]] == [(0, "close"), (1, "entry")]
    assert "strategy" not in actual["output"]


def realtime_groups():
    bars = [bar(), bar(60000)]
    groups = []
    for observation in bars:
        groups.append([
            scheduled(bar(observation["time"], close), state="realtime", trigger=trigger,
                      commit=commit, new=new, clock=observation["time"] + offset)
            for close, trigger, commit, new, offset in [
                (10, "realtime_tick", False, True, 0),
                (11, "realtime_tick", False, False, 30000),
                (12, "bar_close", True, False, 59999),
            ]
        ])
    return bars, [account(item["time"]) for item in bars], groups


def test_realtime_append_matches_batch_with_rollback_history_and_close_alerts():
    program = compile_strategy(",calc_on_every_tick=true", """
var int ordinary=0
varip float prices=0
varip int news=0
ordinary+=1
prices:=prices*100+close
news:=news*10+(barstate.isnew?1:0)
plot(ordinary)
plot(prices)
plot(news)
plot(close[1])
plot(ta.sma(close,2))
plot(barstate.isrealtime?1:0)
plot(timenow)
strategy.entry("L",strategy.long,qty=1)
alert(str.tostring(timenow),alert.freq_once_per_bar_close)
""")
    bars, accounts, groups = realtime_groups()
    session = program.external_session(scheduled=True)
    for index in range(2):
        actual = session.append(bars[index:index+1], accounts[index:index+1], execution_schedule=groups[index:index+1])
        assert session.cursor == index + 1
        assert actual == program.run_external(bars[:index+1], accounts[:index+1], execution_schedule=groups[:index+1])
    assert [plot["values"] for plot in actual["output"]["plots"]] == [
        [1, 2], [101112, 101112101112], [100, 100100], [None, 12], [None, 12], [1, 1], [59999, 119999]]
    assert len(actual["intents"]) == 6
    assert [item["message"] for item in actual["output"]["alerts"]] == ["59999", "119999"]
    assert session.append([], [], execution_schedule=[]) == actual
    with pytest.raises(ValueError, match="E_EXTERNAL_UNSUPPORTED"):
        program.run_external(bars, accounts)


@pytest.mark.parametrize("change", ["unknown_context", "unknown_envelope", "missing_trigger", "numeric_new", "bad_state"])
def test_schedule_json_rejection_precedes_execution_and_session_remains_usable(change):
    program = compile_strategy()
    group = [[scheduled(bar())]]
    malformed = deepcopy(group)
    item = malformed[0][0]
    if change == "unknown_context":
        item["context"]["surprise"] = True
    elif change == "unknown_envelope":
        item["surprise"] = True
    elif change == "missing_trigger":
        del item["context"]["trigger"]
    elif change == "numeric_new":
        item["context"]["is_new"] = 1
    else:
        item["context"]["bar_state"] = "forming"
    session = program.external_session(scheduled=True)
    with pytest.raises(ValueError):
        session.append([bar()], [account()], execution_schedule=malformed)
    assert session.cursor == 0
    assert session.append([bar()], [account()], execution_schedule=group) == program.run_external(
        [bar()], [account()], execution_schedule=group)


def test_session_and_batch_cannot_mix_legacy_and_explicit_passes():
    program = compile_strategy()
    group = [[scheduled(bar())]]
    with pytest.raises(ValueError, match="mutually exclusive"):
        program.run_external([bar()], [account()], execution_passes=[], execution_schedule=group)
    for session, kwargs in [
        (program.external_session(), dict(execution_schedule=group)),
        (program.external_session(scheduled=True), {}),
        (program.external_session(scheduled=True), dict(execution_schedule=group, execution_passes=[])),
    ]:
        with pytest.raises(ValueError, match="schedule mode changed"):
            session.append([bar()], [account()], **kwargs)
        assert session.cursor == 0


@pytest.mark.parametrize("trigger", ["order_fill", "realtime_tick", "closing_fill"])
def test_declaration_gates_for_triggered_evaluation_and_poisoned_session(trigger):
    program = compile_strategy()
    invalid = [[scheduled(bar(), trigger=trigger)]]
    with pytest.raises(ValueError, match="invalid scheduled execution context"):
        program.run_external([bar()], [account()], execution_schedule=invalid)
    session = program.external_session(scheduled=True)
    with pytest.raises(ValueError, match="invalid scheduled execution context"):
        session.append([bar()], [account()], execution_schedule=invalid)
    assert session.cursor == 0
    with pytest.raises(ValueError, match="POISONED"):
        session.append([bar()], [account()], execution_schedule=[[scheduled(bar())]])


def test_scheduled_closing_fill_keeps_immutable_market_observation():
    program = compile_strategy(",calc_on_order_fills=true,process_orders_on_close=true", "plot(strategy.position_size)")
    group = [[scheduled(bar()), scheduled(bar(), 2, trigger="closing_fill")]]
    assert program.run_external([bar()], [account()], execution_schedule=group, closing_ticks=True)[
        "output"]["plots"][0]["values"] == [2]
    group[0][1]["evaluation"]["event_time_ms"] += 1
    with pytest.raises(ValueError, match="invalid execution pass state"):
        program.run_external([bar()], [account()], execution_schedule=group, closing_ticks=True)


def test_history_to_realtime_boundary_and_append_cannot_return_to_history():
    program = compile_strategy(body="plot(barstate.islastconfirmedhistory?1:0)\nplot(barstate.islast?1:0)")
    bars = [bar(), bar(60000)]
    accounts = [account(), account(60000)]
    groups = [[scheduled(bars[0])], [scheduled(bars[1], state="realtime")]]
    actual = program.run_external(bars, accounts, execution_schedule=groups)
    assert [plot["values"] for plot in actual["output"]["plots"]] == [[1, 0], [0, 1]]
    backwards = [[scheduled(bars[0], state="realtime")], [scheduled(bars[1])]]
    with pytest.raises(ValueError, match="historical schedule after realtime"):
        program.run_external(bars, accounts, execution_schedule=backwards)
    session = program.external_session(scheduled=True)
    session.append(bars[:1], accounts[:1], execution_schedule=backwards[:1])
    with pytest.raises(ValueError, match="historical schedule after realtime"):
        session.append(bars[1:], accounts[1:], execution_schedule=backwards[1:])


@pytest.mark.parametrize("streaming", [False, True])
def test_schedule_refreshes_requested_observations_and_preserves_host_fx(streaming):
    program = compile_strategy(",calc_on_every_tick=true", """
remote=request.security("X:OTHER","1",close)
varip float observed=0
observed:=observed*100+remote
plot(observed)
plot(request.currency_rate("USD","EUR"))
""")
    group = [scheduled(bar(close=10), state="realtime", trigger="realtime_tick", commit=False),
             scheduled(bar(), state="realtime", new=False)]
    for item, price in zip(group, [20, 40]):
        item["evaluation"]["request_data"] = [dict(
            symbol="X:OTHER", timeframe="1",
            bars=[dict(time=0, open=price, high=price, low=price, close=price, volume=1)])]
    host = {"$currencyRates": {"schemaVersion": 1, "pairs": [dict(
        **{"from": "USD", "to": "EUR"}, rates=[dict(effectiveTime=0, rate=0.9)])]}}
    if streaming:
        actual = program.external_session(scheduled=True, chart_symbol="X:MAIN",
                                          chart_timeframe="1", request_bars=host).append(
            [bar()], [account()], execution_schedule=[group])
    else:
        actual = program.run_external([bar()], [account()], execution_schedule=[group],
                                      chart_symbol="X:MAIN", chart_timeframe="1", request_bars=host)
    assert [plot["values"] for plot in actual["output"]["plots"]] == [[2040], [0.9]]


@pytest.mark.parametrize("body", [
    "var a=array.new_int()\nvarip b=array.new_int()\narray.push(a,1)\narray.push(b,1)\nplot(array.size(a))\nplot(array.size(b))",
    "var a=array.new_int(1,0)\nvarip b=array.new_int(1,0)\nx=array.slice(a,0,1)\ny=array.slice(b,0,1)\narray.set(x,0,array.get(x,0)+1)\narray.set(y,0,array.get(y,0)+1)\nplot(array.get(a,0))\nplot(array.get(b,0))",
    "var a=matrix.new<int>(1,1,0)\nvarip b=matrix.new<int>(1,1,0)\nmatrix.set(a,0,0,matrix.get(a,0,0)+1)\nmatrix.set(b,0,0,matrix.get(b,0,0)+1)\nplot(matrix.get(a,0,0))\nplot(matrix.get(b,0,0))",
    "var a=map.new<int,int>()\nvarip b=map.new<int,int>()\nmap.put(a,0,nz(map.get(a,0))+1)\nmap.put(b,0,nz(map.get(b,0))+1)\nplot(map.get(a,0))\nplot(map.get(b,0))",
    "type Counter\n    int ordinary=0\n    varip int retained=0\nvar a=Counter.new()\na.ordinary+=1\na.retained+=1\nplot(a.ordinary)\nplot(a.retained)",
])
def test_schedule_restores_heap_mutations_while_retaining_intrabar_state(body):
    program = compile_strategy(",calc_on_every_tick=true", body)
    bars, frames, groups = realtime_groups()
    # Commit the initial object identities before exercising tick rollback.
    for item_bar, frame, group in zip(bars, frames, groups):
        item_bar["time"] += 60000
        frame["time"] += 60000
        for item in group:
            item["evaluation"]["bar"]["time"] += 60000
            item["evaluation"]["account"]["time"] += 60000
            item["evaluation"]["event_time_ms"] += 60000
    bars.insert(0, bar())
    frames.insert(0, account())
    groups.insert(0, [scheduled(bar())])
    expected = [[1, 2, 3], [1, 4, 7]]
    actual = program.run_external(bars, frames, execution_schedule=groups)
    assert [plot["values"] for plot in actual["output"]["plots"]] == expected
    session = program.external_session(scheduled=True)
    for index in range(3):
        appended = session.append(bars[index:index+1], frames[index:index+1], execution_schedule=groups[index:index+1])
    assert appended == actual


def test_schedule_rolls_back_speculative_drawings_between_ticks():
    program = compile_strategy(",calc_on_every_tick=true", "line.new(bar_index,close,bar_index+1,close)\nplot(array.size(line.all))")
    bars, frames, groups = realtime_groups()
    actual = program.run_external(bars, frames, execution_schedule=groups)
    assert actual["output"]["plots"][0]["values"] == [1, 2]
    assert len(actual["output"]["lines"]) == 2


def test_schedule_price_flow_builtins_commit_once_per_bar():
    program = compile_strategy(",calc_on_every_tick=true", "\n".join(
        f"plot(ta.{name})" for name in ["obv", "accdist", "pvt", "nvi", "pvi", "vwap", "wad"]))
    bars, frames, groups = realtime_groups()
    actual = program.run_external(bars, frames, execution_schedule=groups)
    expected = program.run(bars)
    for left, right in zip(actual["output"]["plots"], expected["plots"]):
        assert left["values"] == pytest.approx(right["values"])

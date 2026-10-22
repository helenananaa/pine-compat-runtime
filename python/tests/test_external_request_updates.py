"""Causal feed observations, independently of any concrete host application."""
import copy

import pine_compat
import pytest


def bar(index, close):
    return dict(time=index * 60_000, open=close, high=close, low=close, close=close, volume=1)


def account(index):
    return dict(time=index * 60_000, position_size=0, position_avg_price=None,
                equity=1000, initial_capital=1000, netprofit=0, openprofit=0)


def update(index, close, *, kind="confirmed", available=None):
    return dict(symbol="B", timeframe="1", bar=bar(index, close), kind=kind,
                available_at_ms=(index + 1) * 60_000 - 1 if available is None else available)


def scheduled(market, feedback, clock, updates, *, trigger="bar_close", new=True):
    return dict(evaluation=dict(bar=market, account=feedback, event_time_ms=clock,
                               confirmed=trigger == "bar_close", request_updates=updates),
                context=dict(bar_state="realtime", trigger=trigger, is_new=new))


def test_feed_only_growth_retains_committed_ta_and_failed_batch_is_retryable():
    program = pine_compat.compile_script('//@version=6\nstrategy("growth")\nplot(request.security("B","1",ta.cum(close)))')
    session = program.external_session(chart_symbol="A", chart_timeframe="1", request_bars={"B:1": []})
    assert pine_compat.EXTERNAL_REQUEST_UPDATE_SCHEMA_VERSION == 1
    with pytest.raises(ValueError, match="availability"):
        session.apply_request_updates([update(0, 100), update(1, 200)], 59_999)
    assert session.cursor == 0
    for index in range(3):
        session.apply_request_updates([update(index, (index + 1) * 100)], (index + 1) * 60_000 - 1)
        result = session.append([bar(index, 1)], [account(index)])
        assert result["output"]["plots"][0]["values"] == [100, 300, 600][:index + 1]
    before = session.append([], [])
    with pytest.raises(ValueError):
        session.apply_request_updates([update(0, 999)], 239_999)
    assert session.append([], []) == before


@pytest.mark.parametrize("expression", ["ta.cum(close)", 'ta.cum(request.security("C","1",close))'])
@pytest.mark.parametrize("window", ["", ",calc_bars_count=2"])
@pytest.mark.parametrize("dynamic", ["true", "false"])
def test_scheduled_growth_matches_the_per_observation_full_snapshot_oracle(expression, window, dynamic):
    source = f'//@version=6\nstrategy("schedule",calc_on_order_fills=true,dynamic_requests={dynamic}{window})\nplot(request.security("B","1",{expression}))'
    program = pine_compat.compile_script(source)
    bars, frames, deltas, snapshots = [], [], [], []
    for index in range(3):
        bars.append(bar(index, 1))
        frames.append(account(index))
        value = dict(bar=bars[-1], account=frames[-1], event_time_ms=(index + 1) * 60_000 - 1,
                     confirmed=True, request_updates=[update(index, (index + 1) * 100)])
        deltas.append([value])
        old = copy.deepcopy(value)
        del old["request_updates"]
        old["request_data"] = [dict(symbol="B", timeframe="1", bars=[bar(i, (i + 1) * 100) for i in range(index + 1)]),
                               dict(symbol="C", timeframe="1", bars=[bar(i, (i + 1) * 10) for i in range(3)])]
        snapshots.append([old])
    environment = {"B:1": [], "C:1": [bar(i, (i + 1) * 10) for i in range(3)]}
    delta = program.run_external(bars, frames, chart_symbol="A", chart_timeframe="1",
                                 request_bars=environment, execution_passes=deltas)
    oracle = program.run_external(bars, frames, chart_symbol="A", chart_timeframe="1",
                                  request_bars=environment, execution_passes=snapshots)
    assert delta == oracle
    session = program.external_session(chart_symbol="A", chart_timeframe="1", request_bars=environment)
    if window:
        # A streaming session freezes its initial calculation window. Rebuilding
        # a later batch would move that window and is a different input contract.
        legacy = program.external_session(chart_symbol="A", chart_timeframe="1", request_bars=environment)
        assert session.append(bars, frames, execution_passes=deltas) == oracle
        assert legacy.append(bars, frames, execution_passes=snapshots) == oracle
        extra = dict(bar=bar(3, 1), account=account(3), event_time_ms=239_999,
                     confirmed=True, request_updates=[update(3, 400)])
        old = copy.deepcopy(extra)
        del old["request_updates"]
        old["request_data"] = copy.deepcopy(snapshots[-1][0]["request_data"])
        old["request_data"][0]["bars"].append(bar(3, 400))
        assert session.append([bar(3, 1)], [account(3)], execution_passes=[[extra]]) == legacy.append(
            [bar(3, 1)], [account(3)], execution_passes=[[old]])
        return
    for index in range(3):
        result = session.append(bars[index:index+1], frames[index:index+1], execution_passes=deltas[index:index+1])
        assert result == program.run_external(bars[:index+1], frames[:index+1], chart_symbol="A", chart_timeframe="1",
                                              request_bars=environment, execution_passes=snapshots[:index+1])


def test_forming_replacement_and_confirmation_do_not_double_count_requested_ta():
    source = '''//@version=6
strategy("forming",calc_on_every_tick=true)
x=request.security("B","1",ta.cum(close))
varip float transcript=0
if barstate.isnew
    transcript:=0
transcript+=x
plot(x)
plot(transcript)
'''
    program = pine_compat.compile_script(source)
    group = [scheduled(bar(0, 1), account(0), 1, [update(0, 100, kind="forming", available=1)], trigger="realtime_tick"),
             scheduled(bar(0, 1), account(0), 2, [update(0, 110, kind="forming", available=2)], trigger="realtime_tick", new=False),
             scheduled(bar(0, 1), account(0), 59_999, [update(0, 120)], new=False)]
    session = program.external_session(chart_symbol="A", chart_timeframe="1", request_bars={"B:1": []}, scheduled=True)
    result = session.append([bar(0, 1)], [account(0)], execution_schedule=[group])
    assert [plot["values"] for plot in result["output"]["plots"]] == [[120], [330]]
    group = [scheduled(bar(1, 1), account(1), 119_999, [update(1, 200)])]
    result = session.append([bar(1, 1)], [account(1)], execution_schedule=[group])
    assert [plot["values"] for plot in result["output"]["plots"]] == [[120, 320], [330, 320]]


def test_lower_timeframe_forming_feed_keeps_observed_prefix_and_intrabar_ta():
    source = '''//@version=6
strategy("ltf",calc_on_every_tick=true)
a=request.security_lower_tf("B","1",close)
b=request.security_lower_tf("B","1",ta.cum(close))
plot(a.sum())
plot(b.sum())
plot(a.size())
'''
    program = pine_compat.compile_script(source)
    group = [scheduled(bar(0, 1), account(0), 1, [update(0, 10, kind="forming", available=1)], trigger="realtime_tick"),
             scheduled(bar(0, 1), account(0), 59_999, [update(0, 10)], trigger="realtime_tick", new=False),
             scheduled(bar(0, 1), account(0), 60_001, [update(1, 20, kind="forming", available=60_001)], trigger="realtime_tick", new=False),
             scheduled(bar(0, 1), account(0), 119_999, [update(1, 30)], new=False)]
    result = program.run_external([bar(0, 1)], [account(0)], chart_symbol="A", chart_timeframe="2",
                                  request_bars={"B:1": []}, execution_schedule=[group])
    assert [plot["values"] for plot in result["output"]["plots"]] == [[40], [50], [2]]


def test_historical_fill_uses_chart_visibility_independent_of_interpolated_fill_clock():
    program = pine_compat.compile_script('//@version=6\nstrategy("history",calc_on_order_fills=true)\nx=request.security("B","1",close)\nvarip float total=0\ntotal+=x\nplot(total)')
    group = [scheduled(bar(0, 1), account(0), 1, [update(0, 100)], trigger="order_fill"),
             scheduled(bar(0, 1), account(0), 59_999, [])]
    for item in group:
        item["context"]["bar_state"] = "historical"
        item["context"]["is_new"] = True
    result = program.run_external([bar(0, 1)], [account(0)], chart_symbol="A", chart_timeframe="1",
                                  request_bars={"B:1": []}, execution_schedule=[group])
    assert result["output"]["plots"][0]["values"] == [200]


@pytest.mark.parametrize("change", [dict(symbol=" B"), dict(timeframe=""), dict(kind="unknown"),
                                    dict(available_at_ms=True), dict(extra=1)])
def test_invalid_update_transport_does_not_poison_feed_only_session(change):
    program = pine_compat.compile_script('//@version=6\nstrategy("strict")\nplot(request.security("B","1",close))')
    session = program.external_session(chart_symbol="A", chart_timeframe="1", request_bars={"B:1": []})
    with pytest.raises(ValueError):
        session.apply_request_updates([{**update(0, 10), **change}], 59_999)
    session.apply_request_updates([update(0, 10)], 59_999)
    assert session.append([bar(0, 1)], [account(0)])["output"]["plots"][0]["values"] == [10]


@pytest.mark.parametrize("clock", [True, 59999.0, "59999"])
def test_observation_clock_requires_integer_milliseconds(clock):
    session = pine_compat.compile_script('//@version=6\nstrategy("clock")\nplot(close)').external_session()
    with pytest.raises(ValueError, match="integer millisecond"):
        session.apply_request_updates([], clock)
    session.apply_request_updates([], 59_999)
    assert session.append([bar(0, 1)], [account(0)])["output"]["plots"][0]["values"] == [1]


def test_calendar_monthly_confirmation_uses_actual_utc_month_closes():
    from datetime import datetime, timezone
    opens = [int(datetime(2024, month, 1, tzinfo=timezone.utc).timestamp() * 1000) for month in range(1, 5)]
    session = pine_compat.compile_script('//@version=6\nstrategy("calendar")\nplot(request.security("B","1M",ta.cum(close)))').external_session(
        chart_symbol="A", chart_timeframe="1M", request_bars={"B:1M": []})
    for index, opened in enumerate(opens[:-1]):
        market, feedback = {**bar(0, 1), "time": opened}, {**account(0), "time": opened}
        observed = opens[index + 1] - 1
        item = dict(symbol="B", timeframe="1M", bar={**bar(0, (index + 1) * 10), "time": opened},
                    kind="confirmed", available_at_ms=observed)
        with pytest.raises(ValueError, match="availability"):
            session.apply_request_updates([{**item, "available_at_ms": observed - 1}], observed)
        session.apply_request_updates([item], observed)
        result = session.append([market], [feedback])
        assert result["output"]["plots"][0]["values"] == [10, 30, 60][:index+1]

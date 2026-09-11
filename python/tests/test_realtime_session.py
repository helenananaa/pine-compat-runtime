from __future__ import annotations

import pytest

import pine_compat


def test_mid_bar_opening_context_preserves_varip_and_rejects_duplicate_opening():
    source = '''//@version=6
indicator("Opening context")
varip int n = 0
if barstate.isnew
    n := 0
n += 1
plot(barstate.isnew ? 1 : 0)
plot(n)
plot(timenow)
'''
    session = pine_compat.create_realtime_session(source)
    session.seed([_bar(60_000, 10)], execution_times=[60_100])
    first = session.update_forming(
        _bar(120_000, 11), execution_time=120_250, opening_update=False
    )
    assert [p["values"][-1] for p in first["plots"]] == [0, 2, 120_250]
    before = session.result()
    with pytest.raises(ValueError, match="opening_update cannot repeat"):
        session.update_forming(
            _bar(120_000, 12), execution_time=120_500, opening_update=True
        )
    assert session.result() == before
    replacement = session.update_forming(_bar(120_000, 12), execution_time=120_500)
    assert [p["values"][-1] for p in replacement["plots"]] == [0, 3, 120_500]
    session.update_confirmed(_bar(120_000, 12), execution_time=179_999)
    next_bar = session.update_forming(_bar(180_000, 13), execution_time=180_001)
    assert [p["values"][-1] for p in next_bar["plots"]] == [1, 1, 180_001]
    assert [p["values"][-1] for p in first["plots"]] == [0, 2, 120_250]


def test_opening_context_is_strict_and_can_describe_a_close_only_observation():
    session = pine_compat.create_realtime_session(
        '//@version=6\nindicator("Opening")\nplot(barstate.isnew ? 1 : 0)\n'
    )
    session.seed([_bar(60_000, 10)])
    before = session.result()
    for invalid in [1, 0, "false", [], {}]:
        with pytest.raises(ValueError, match="opening_update must be a bool"):
            session.update_forming(_bar(120_000, 11), opening_update=invalid)
        assert session.result() == before and session.forming_time is None
    closed = session.update_confirmed(_bar(120_000, 11), opening_update=False)
    assert closed["plots"][0]["values"][-1] == 0
    inferred = session.update_confirmed(_bar(180_000, 12), opening_update=None)
    assert inferred["plots"][0]["values"][-1] == 1


def test_realtime_execution_clock_seed_replacement_and_confirmation():
    source = '''//@version=6
indicator("Clock lifecycle")
var int committed = 0
varip int executions = 0
committed += 1
executions += 1
plot(timenow)
plot(timenow[1])
plot(committed)
plot(executions)
'''
    session = pine_compat.create_realtime_session(source)
    bars = [_bar(60_000, 1), _bar(120_000, 2)]
    with pytest.raises(ValueError, match="execution timestamp"):
        session.seed(bars)
    assert not session.is_seeded
    with pytest.raises(ValueError, match="count"):
        session.seed(bars, execution_times=[1000])
    assert not session.is_seeded
    seeded = session.seed(bars, execution_times=[1000, 2000])
    assert seeded["plots"][0]["values"] == [1000, 2000]
    before = session.result()
    with pytest.raises(ValueError, match="execution timestamp"):
        session.update_forming(_bar(180_000, 3))
    assert session.result() == before and session.forming_time is None
    first = session.update_forming(_bar(180_000, 3), execution_time=3000)
    replacement = session.update_forming(_bar(180_000, 4), execution_time=4000)
    assert first["plots"][0]["values"][-1] == 3000
    assert replacement["plots"][0]["values"][-1] == 4000
    assert replacement["plots"][1]["values"][-1] == 2000
    assert replacement["plots"][2]["values"][-1] == 3
    assert replacement["plots"][3]["values"][-1] == 4
    assert session.confirmed_result() == seeded
    confirmed = session.update_confirmed(_bar(180_000, 5), execution_time=5000)
    assert confirmed["plots"][0]["values"] == [1000, 2000, 5000]
    assert confirmed["plots"][1]["values"] == [None, 1000, 2000]
    assert confirmed["plots"][3]["values"][-1] == 5
    assert session.confirmed_bars == 3 and session.forming_time is None


@pytest.mark.parametrize("value", [True, 1.5, "1000", 2**70])
def test_realtime_execution_clock_rejects_invalid_input_atomically(value):
    session = pine_compat.create_realtime_session('//@version=6\nindicator("clock")\nplot(timenow)\n')
    with pytest.raises(ValueError, match="execution_times"):
        session.seed([_bar(60_000, 1)], execution_times=[value])
    assert not session.is_seeded
    session.seed([_bar(60_000, 1)], execution_times=[1000])
    before = session.result()
    for method in (session.update_forming, session.update_confirmed):
        with pytest.raises(ValueError, match="execution_time"):
            method(_bar(120_000, 2), execution_time=value)
        assert session.result() == before
        assert session.confirmed_bars == 1 and session.forming_time is None


def _session_windows(indices: list[int], name: str = "day") -> dict:
    return {
        "schemaVersion": 1,
        "bars": [
            {"barIndex": index, "windowId": name, "tradingDayId": name}
            for index in indices
        ],
    }


def test_realtime_session_extends_windows_and_retries_missing_coverage() -> None:
    source = '''//@version=6
strategy("Session extension", calc_on_every_tick=true)
strategy.risk.max_intraday_filled_orders(2)
strategy.entry("L", strategy.long)
plot(strategy.position_size)
'''
    session = pine_compat.create_realtime_session(
        source, session_windows=_session_windows([0])
    )
    control = pine_compat.create_realtime_session(
        source, session_windows=_session_windows([0, 1, 2])
    )
    session.seed([_bar(60_000, 10.0)])
    control.seed([_bar(60_000, 10.0)])
    before = session.result()
    with pytest.raises(ValueError, match="E_SESSION_COVERAGE"):
        session.update_confirmed(_bar(120_000, 10.0))
    assert session.result() == before
    assert session.confirmed_bars == 1
    session.extend_session_windows(_session_windows([1]))
    session.update_forming(_bar(120_000, 10.0))
    control.update_forming(_bar(120_000, 10.0))
    forming = session.result()
    with pytest.raises(ValueError, match="E_SESSION_HISTORY_CHANGED"):
        session.extend_session_windows(_session_windows([1, 2], "changed"))
    assert session.result() == forming
    assert session.forming_time == 120_000
    session.extend_session_windows(_session_windows([0, 1, 2]))
    for target in (session, control):
        target.update_forming(_bar(120_000, 10.0))
        target.update_confirmed(_bar(120_000, 10.0))
        target.update_confirmed(_bar(180_000, 10.0))
    assert session.result() == control.result()


def test_realtime_session_extension_validation_is_atomic() -> None:
    session = pine_compat.create_realtime_session(
        '//@version=6\nstrategy("Session")\nplot(close)\n',
        session_windows=_session_windows([0]),
    )
    session.seed([_bar(60_000, 10.0)])
    for payload, code in [
        (_session_windows([0, 1], "changed"), "E_SESSION_HISTORY_CHANGED"),
        (_session_windows([1, 1]), "E_SESSION_DUPLICATE_BAR"),
        ({"schemaVersion": 2, "bars": []}, "E_SESSION_SCHEMA_VERSION"),
    ]:
        with pytest.raises(ValueError, match=code):
            session.extend_session_windows(payload)
        with pytest.raises(ValueError, match="E_SESSION_COVERAGE"):
            session.update_confirmed(_bar(120_000, 10.0))
    session.extend_session_windows(
        '{"schemaVersion":1,"bars":[{"barIndex":1,"windowId":"day","tradingDayId":"day"}]}'
    )
    session.update_confirmed(_bar(120_000, 10.0))
    assert session.confirmed_bars == 2


def test_realtime_session_cannot_switch_executed_utc_history_to_host_windows() -> None:
    session = pine_compat.create_realtime_session(
        '//@version=6\nstrategy("Session")\nplot(close)\n'
    )
    session.seed([_bar(60_000, 10.0)])
    with pytest.raises(ValueError, match="E_SESSION_HISTORY_CHANGED"):
        session.extend_session_windows(_session_windows([1]))


def _bar(time: int, close: float) -> dict[str, float | int]:
    return {
        "time": time,
        "open": close,
        "high": close,
        "low": close,
        "close": close,
        "volume": 1.0,
    }


def _plot_values(result: dict, index: int) -> list[float | None]:
    return result["plots"][index]["values"]


def test_realtime_session_seeds_history_with_the_complete_dataset_endpoint() -> None:
    session = pine_compat.create_realtime_session(
        '''//@version=6
indicator("Dataset endpoint")
plot(last_bar_index)
plot(barstate.islast ? 1 : 0)
'''
    )

    result = session.seed([_bar(60_000, 1.0), _bar(120_000, 2.0)])

    assert pine_compat.REALTIME_SESSION_SCHEMA_VERSION == 1
    assert session.schema_version == 1
    assert session.is_seeded is True
    assert session.confirmed_bars == 2
    assert session.last_confirmed_time == 120_000
    assert session.forming_time is None
    assert _plot_values(result, 0) == [1.0, 1.0]
    assert _plot_values(result, 1) == [0.0, 1.0]


def test_realtime_session_rolls_back_var_and_carries_varip_between_forming_updates() -> None:
    session = pine_compat.compile_script(
        '''//@version=6
indicator("Rollback")
var float regular = 0.0
varip float intrabar = 0.0
regular += 1.0
intrabar += 1.0
plot(regular)
plot(intrabar)
'''
    ).realtime_session()
    session.seed([_bar(60_000, 1.0)])

    first = session.update_forming(_bar(120_000, 2.0))
    second = session.update_forming(_bar(120_000, 3.0))

    assert _plot_values(first, 0) == [1.0, 2.0]
    assert _plot_values(first, 1) == [1.0, 2.0]
    assert _plot_values(second, 0) == [1.0, 2.0]
    assert _plot_values(second, 1) == [1.0, 3.0]
    assert _plot_values(session.confirmed_result(), 0) == [1.0]
    assert _plot_values(session.confirmed_result(), 1) == [1.0]
    assert session.forming_time == 120_000

    confirmed = session.update_confirmed(_bar(120_000, 4.0))

    assert _plot_values(confirmed, 0) == [1.0, 2.0]
    assert _plot_values(confirmed, 1) == [1.0, 4.0]
    assert session.confirmed_bars == 2
    assert session.last_confirmed_time == 120_000
    assert session.forming_time is None
    assert session.result() == session.confirmed_result()


def test_realtime_session_applies_inputs_and_chart_context_once() -> None:
    source = '''//@version=6
indicator("Context")
factor = input.int(2, "Factor")
matches = syminfo.tickerid == "BINANCE:BTCUSDT.P" and timeframe.period == "60"
plot(close * factor + (matches ? 1 : 0))
'''
    analysis = pine_compat.analyze_script(source)
    factor_id = analysis["inputs"][0]["callSiteId"]
    session = pine_compat.create_realtime_session(
        source,
        input_overrides={factor_id: 3},
        chart_symbol="BINANCE:BTCUSDT.P",
        chart_timeframe="60",
    )

    seeded = session.seed([_bar(60_000, 2.0)])
    forming = session.update_forming(_bar(120_000, 4.0))

    assert _plot_values(seeded, 0) == [7.0]
    assert _plot_values(forming, 0) == [7.0, 13.0]


def test_realtime_session_rejects_ambiguous_or_regressive_lifecycle_updates() -> None:
    session = pine_compat.create_realtime_session(
        '//@version=6\nindicator("Lifecycle")\nplot(close)'
    )

    with pytest.raises(ValueError, match="must be seeded"):
        session.update_forming(_bar(60_000, 1.0))

    session.seed([_bar(60_000, 1.0)])
    with pytest.raises(ValueError, match="already been seeded"):
        session.seed([])

    session.update_forming(_bar(120_000, 2.0))
    with pytest.raises(ValueError, match="does not match forming time"):
        session.update_forming(_bar(180_000, 3.0))
    with pytest.raises(ValueError, match="does not match forming time"):
        session.update_confirmed(_bar(180_000, 3.0))

    session.update_confirmed(_bar(120_000, 2.0))
    with pytest.raises(ValueError, match="must be later than confirmed time"):
        session.update_confirmed(_bar(120_000, 2.0))


def _assert_series_is_delta(changes: dict) -> None:
    assert "plots" not in changes
    for item in changes["series"]:
        value_len = len(item.get("values") or item.get("closes") or [])
        assert value_len <= 1
        assert item["op"] in ("append", "replaceLast")


def test_streaming_apply_returns_this_update_changes_not_full_history() -> None:
    session = pine_compat.create_realtime_session(
        '//@version=6\nindicator("stream")\nplot(close)\n'
    )
    snapshot = session.seed([_bar(60_000, 10.0)])
    assert len(_plot_values(snapshot, 0)) == 1
    assert pine_compat.RUNTIME_CHANGES_SCHEMA_VERSION == 3

    replica = session.replica()
    forming = session.apply_forming(_bar(120_000, 12.0))
    assert forming["visibility"] == "preview"
    assert forming["revision"] == session.revision
    _assert_series_is_delta(forming)
    assert forming["series"][0]["op"] == "append"
    assert forming["series"][0]["values"] == [12.0]
    assert pine_compat.apply_runtime_changes(replica, forming)
    applied = replica.result()
    assert applied == session.result()
    assert not pine_compat.apply_runtime_changes(replica, forming)
    again = replica.result()
    assert again == session.result()

    replacement = session.apply_forming(_bar(120_000, 13.0))
    assert replacement["series"][0]["op"] == "replaceLast"
    assert replacement["series"][0]["values"] == [13.0]
    assert replica.apply(replacement)
    applied = replica.result()
    assert applied == session.result()

    confirmed = session.apply_confirmed(_bar(120_000, 13.0))
    assert confirmed["visibility"] == "confirmed"
    assert replica.apply(confirmed)
    applied = replica.result()
    assert applied == session.result() == session.confirmed_result()


def test_streaming_old_update_methods_still_return_complete_snapshots() -> None:
    session = pine_compat.create_realtime_session(
        '//@version=6\nindicator("compat")\nplot(close)\n'
    )
    seeded = session.seed([_bar(60_000, 10.0)])
    assert len(_plot_values(seeded, 0)) == 1
    forming = session.update_forming(_bar(120_000, 12.0))
    assert len(_plot_values(forming, 0)) == 2
    assert session.last_changes() is None
    confirmed = session.update_confirmed(_bar(120_000, 13.0))
    assert len(_plot_values(confirmed, 0)) == 2
    assert confirmed == session.result()


def test_streaming_alerts_are_not_duplicated_when_rereading_revision() -> None:
    session = pine_compat.create_realtime_session(
        '''//@version=6
indicator("alerts")
if close > 2
    alert("high")
plot(close)
'''
    )
    snapshot = session.seed([_bar(60_000, 1.0)])
    replica = session.replica()
    forming = session.apply_forming(_bar(120_000, 3.0))
    assert any(change["action"] == "add" for change in forming["alerts"])
    assert pine_compat.apply_runtime_changes(replica, forming)
    applied = replica.result()
    assert len(applied["alerts"]) == 1
    reread = session.last_changes()
    assert not replica.apply(reread)
    applied = replica.result()
    assert len(applied["alerts"]) == 1
    assert applied == session.result()


def test_streaming_polyline_chart_points_roundtrip_through_apply_changes() -> None:
    session = pine_compat.create_realtime_session(
        '''//@version=6
indicator("stream polyline")
points = array.from(chart.point.from_index(bar_index, close))
polyline.new(points, line_color=color.red)
plot(close)
'''
    )
    snapshot = session.seed([_bar(60_000, 1.0)])
    assert snapshot["polylines"]
    point = snapshot["polylines"][0]["snapshots"][0]["points"][0]
    assert isinstance(point, dict)
    assert set(point) >= {"time", "index", "price"}

    replica = session.replica()
    forming = session.apply_forming(_bar(120_000, 2.0))
    assert pine_compat.apply_runtime_changes(replica, forming)
    applied = replica.result()
    assert applied == session.result()
    applied_point = applied["polylines"][0]["snapshots"][-1]["points"][0]
    assert isinstance(applied_point, dict)
    assert applied_point == session.result()["polylines"][0]["snapshots"][-1]["points"][0]


def test_streaming_new_series_header_roundtrips_after_empty_seed() -> None:
    session = pine_compat.create_realtime_session(
        '''//@version=6
indicator("stream header")
plot(close, title="live", linewidth=2)
'''
    )
    snapshot = session.seed([])
    assert snapshot["plots"] == []

    replica = session.replica()
    forming = session.apply_forming(_bar(60_000, 10.0))
    headers = [item.get("header") for item in forming["series"] if item.get("header")]
    assert headers
    assert headers[0]["title"] == "live"
    assert headers[0]["linewidth"] == 2

    assert pine_compat.apply_runtime_changes(replica, forming)
    applied = replica.result()
    assert applied == session.result()
    assert applied["plots"][0]["title"] == "live"
    assert applied["plots"][0]["linewidth"] == 2


def test_streaming_replica_stale_gap_conflict_and_recovery_are_atomic():
    import copy
    session = pine_compat.create_realtime_session('//@version=6\nindicator("cursor")\nplot(close)')
    session.seed([_bar(0, 10.0)])
    replica = session.replica()
    a = session.apply_forming(_bar(60000, 20.0))
    b = session.apply_forming(_bar(60000, 30.0))
    before = replica.result()
    with pytest.raises(ValueError, match="E_STREAM_GAP"):
        replica.apply(b)
    assert replica.result() == before
    assert replica.apply(a)
    assert not replica.apply(a)
    assert replica.apply(b)
    before = replica.result()
    with pytest.raises(ValueError, match="E_STREAM_STALE"):
        replica.apply(a)
    conflict = copy.deepcopy(b)
    conflict["series"] = []
    with pytest.raises(ValueError, match="E_STREAM_CONFLICT"):
        replica.apply(conflict)
    invalid = copy.deepcopy(b)
    invalid["schemaVersion"] = 100
    with pytest.raises(ValueError, match="E_STREAM_SCHEMA"):
        replica.apply(invalid)
    assert replica.result() == before
    assert replica.revision == b["revision"]
    old_snapshot = replica.result()
    session.apply_forming(_bar(60000, 40.0))
    missed = session.apply_forming(_bar(60000, 50.0))
    with pytest.raises(ValueError, match="E_STREAM_GAP"):
        replica.apply(missed)
    snapshot = session.stream_snapshot()
    replica.reset(
        snapshot["result"],
        revision=snapshot["revision"],
        retained_from=snapshot.get("retainedFrom", 0),
    )
    confirmed = session.apply_confirmed(_bar(60000, 60.0))
    assert replica.apply(confirmed)
    assert replica.result() == session.result()
    assert old_snapshot["plots"][0]["values"] == [10.0, 30.0]
    # Unversioned results can no longer silently accept stale changes.
    with pytest.raises(TypeError):
        pine_compat.apply_runtime_changes(old_snapshot, a)


def test_streaming_replica_requires_explicit_wire_schema_and_base_revision():
    session = pine_compat.create_realtime_session('//@version=6\nindicator("cursor")\nplot(close)')
    session.seed([])
    replica = session.replica()
    update = session.apply_forming(_bar(0, 1.0))
    before = replica.result()
    for field in ("schemaVersion", "baseRevision"):
        invalid = dict(update)
        del invalid[field]
        with pytest.raises(ValueError, match="missing"):
            replica.apply(invalid)
        assert replica.result() == before and replica.revision == 1
    assert replica.apply(update)


def test_streaming_identical_freq_all_events_keep_occurrence_counts():
    session = pine_compat.create_realtime_session('//@version=6\nindicator("occurrences")\nfor i = 0 to int(close) - 1\n    alert("same", alert.freq_all)\nplot(close)')
    session.seed([_bar(0, 1.0)])
    replica = session.replica()
    for value in (3.0, 2.0, 4.0):
        change = session.apply_forming(_bar(60000, value))
        assert replica.apply(change)
        assert replica.result() == session.result()
        assert len(replica.result()["alerts"]) == 1 + int(value)
        assert not replica.apply(change)
    change = session.apply_confirmed(_bar(60000, 2.0))
    replica.apply(change)
    assert replica.result() == session.result()


def test_realtime_replay_replaces_confirmed_history_and_discards_forming() -> None:
    source = '//@version=6\nindicator("replay")\nplot(close)\n'
    session = pine_compat.create_realtime_session(source)
    session.seed([_bar(0, 10.0), _bar(60_000, 20.0)])
    session.apply_forming(_bar(120_000, 30.0))
    assert _plot_values(session.result(), 0) == [10.0, 20.0, 30.0]
    replayed = session.replay([_bar(0, 11.0), _bar(60_000, 21.0)])
    assert _plot_values(replayed, 0) == [11.0, 21.0]
    assert session.forming_time is None
    assert session.confirmed_bars == 2
    control = pine_compat.create_realtime_session(source)
    expected = control.seed([_bar(0, 11.0), _bar(60_000, 21.0)])
    assert _plot_values(replayed, 0) == _plot_values(expected, 0)


def test_realtime_replay_failure_is_atomic_and_replica_must_reset() -> None:
    session = pine_compat.create_realtime_session(
        '//@version=6\nindicator("replay fail")\nplot(timenow)\n'
    )
    session.seed([_bar(0, 10.0)], execution_times=[1000])
    replica = session.replica()
    forming = session.apply_forming(_bar(60_000, 11.0), execution_time=2000)
    replica.apply(forming)
    before = session.result()
    revision = session.revision
    with pytest.raises(ValueError, match="execution timestamp count"):
        session.replay([_bar(0, 12.0)], execution_times=[1000, 2000])
    assert session.result() == before
    assert session.revision == revision
    assert session.forming_time == 60_000
    replayed = session.replay([_bar(0, 12.0)], execution_times=[3000])
    assert _plot_values(replayed, 0) == [3000]
    assert replica.revision != session.revision
    assert replica.apply(forming) is False
    nxt = session.apply_forming(_bar(60_000, 13.0), execution_time=4000)
    with pytest.raises(ValueError, match="E_STREAM_GAP"):
        replica.apply(nxt)
    snapshot = session.stream_snapshot()
    replica.reset(
        snapshot["result"],
        revision=snapshot["revision"],
        retained_from=snapshot.get("retainedFrom", 0),
    )
    assert replica.result() == session.result()


def test_realtime_correct_from_keeps_prefix_and_discards_forming() -> None:
    source = '//@version=6\nindicator("correct")\nplot(close)\n'
    session = pine_compat.create_realtime_session(source)
    session.seed([_bar(0, 10.0), _bar(60_000, 20.0), _bar(120_000, 30.0)])
    session.apply_forming(_bar(180_000, 40.0))
    corrected = session.correct(60_000, [_bar(60_000, 21.0), _bar(120_000, 31.0)])
    assert _plot_values(corrected, 0) == [10.0, 21.0, 31.0]
    assert session.forming_time is None
    assert session.confirmed_bars == 3
    assert session.last_confirmed_time == 120_000
    control = pine_compat.create_realtime_session(source)
    expected = control.seed([_bar(0, 10.0), _bar(60_000, 21.0), _bar(120_000, 31.0)])
    assert _plot_values(corrected, 0) == _plot_values(expected, 0)
    with pytest.raises(ValueError, match="E_HISTORY_CORRECT"):
        session.correct(240_000, [_bar(240_000, 50.0)])


@pytest.mark.parametrize("name", [
    "plotchar", "plotshape", "plotarrow", "plotbar", "plotcandle",
    "line_mutation", "linefill_new", "box_mutation", "box_delete",
    "table_cell", "table_merge_cells", "table_clear", "table_delete",
    "polyline_new", "label_options", "fill_transp",
    "strategy_calc_on_every_tick", "strategy_calc_on_order_fills",
    "strategy_calc_on_order_fills_exit_avg", "strategy_process_orders_on_close",
    "strategy_process_orders_on_close_immediately",
    "strategy_margin_entry_affordability_short", "strategy_margin_entry_affordability_long",
])
def test_streaming_all_output_families_match_snapshot_on_replacements(name):
    from pathlib import Path
    source = (Path(__file__).resolve().parents[2] / "tests/fixtures/runtime" / (name + ".pine")).read_text()
    session = pine_compat.create_realtime_session(source)
    session.seed([_bar(0, 10.0)])
    replica = session.replica()
    for index in range(1, 5):
        for value in (10.0 + index, 12.0 + index, 9.0 + index):
            change = session.apply_forming(_bar(index * 60000, value))
            replica.apply(change)
            assert replica.result() == session.result(), (name, index, value)
        change = session.apply_confirmed(_bar(index * 60000, 11.0 + index))
        replica.apply(change)
        assert replica.result() == session.confirmed_result(), (name, index)

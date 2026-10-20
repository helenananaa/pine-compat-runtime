import pytest
import pine_compat
from test_external_broker import BARS, ACCOUNTS, SOURCE


def test_explicit_execution_clock_matches_batch_and_is_independent_of_bar_open():
    source = SOURCE + '\nplot(timenow)\nif timenow > 100000\n    strategy.close_all()'
    program = pine_compat.compile_script(source)
    clocks = [59999, 119999, 179999]
    session = program.external_session()
    assert pine_compat.EXTERNAL_CLOCK_SCHEMA_VERSION == 1
    for i in range(len(BARS)):
        result = session.append(BARS[i:i+1], ACCOUNTS[i:i+1], execution_times=clocks[i:i+1])
        assert result == program.run_external(BARS[:i+1], ACCOUNTS[:i+1], execution_times=clocks[:i+1])
    assert result['output']['plots'][-1]['values'] == clocks
    assert 'strategy' not in result['output']
    assert session.append([], [], execution_times=[]) == result


@pytest.mark.parametrize('clocks', [[True], [1.5], ['1']])
def test_external_clock_rejects_non_integer_timestamps_before_execution(clocks):
    program = pine_compat.compile_script(SOURCE + '\nplot(timenow)')
    session = program.external_session()
    with pytest.raises(ValueError):
        session.append(BARS[:1], ACCOUNTS[:1], execution_times=clocks)
    assert session.cursor == 0
    assert session.append(BARS[:1], ACCOUNTS[:1], execution_times=[59999]) == program.run_external(
        BARS[:1], ACCOUNTS[:1], execution_times=[59999])


def test_scheduled_fill_passes_keep_their_own_clock():
    source = SOURCE.replace('strategy("External")', 'strategy("External", calc_on_order_fills=true)') + '\nplot(timenow)'
    program = pine_compat.compile_script(source)
    groups = [[dict(bar=bar, account=account, event_time_ms=bar['time']+1, confirmed=True)] for bar, account in zip(BARS, ACCOUNTS)]
    expected = program.run_external(BARS, ACCOUNTS, execution_passes=groups)
    assert program.run_external(BARS, ACCOUNTS, execution_passes=groups, execution_times=[1000]*3) == expected
    session = program.external_session()
    assert session.append(BARS, ACCOUNTS, execution_passes=groups, execution_times=[1000]*3) == expected


def test_append_matches_batch_with_series_arrays_ta_and_authoritative_feedback():
    source = SOURCE + "\nvar float total = 0\ntotal += strategy.position_size\nplot(total)\nplot(ta.sma(close, 2))\nplot(strategy.position_size[1])\nvar values = array.new_float()\narray.push(values, strategy.position_size)\nplot(array.sum(values))\n"
    program = pine_compat.compile_script(source)
    session = program.external_session()
    for i in range(len(BARS)):
        result = session.append(BARS[i:i+1], ACCOUNTS[i:i+1])
        assert session.cursor == i + 1
        assert result == program.run_external(BARS[:i+1], ACCOUNTS[:i+1])
    assert session.append([], []) == result


def test_failed_session_cannot_reuse_partially_committed_state():
    session = pine_compat.compile_script(SOURCE).external_session()
    with pytest.raises(ValueError, match="feedback time/count"):
        session.append(BARS, ACCOUNTS[:1])
    with pytest.raises(ValueError, match="POISONED"):
        session.append(BARS, ACCOUNTS)


def test_passes_append_preserves_account_history():
    source = SOURCE.replace('strategy("External")', 'strategy("External", calc_on_order_fills=true)')
    program = pine_compat.compile_script(source)
    session = program.external_session()
    groups = [[dict(bar=bar, account=account, event_time_ms=bar["time"]+1, confirmed=True)] for bar, account in zip(BARS, ACCOUNTS)]
    groups[1].insert(0, dict(bar=BARS[1], account=ACCOUNTS[0] | {"time": BARS[1]["time"]},
                           event_time_ms=BARS[1]["time"], confirmed=False))
    for i in range(len(BARS)):
        result = session.append(BARS[i:i+1], ACCOUNTS[i:i+1], execution_passes=groups[i:i+1])
        assert result == program.run_external(BARS[:i+1], ACCOUNTS[:i+1], execution_passes=groups[:i+1])


def test_correction_requires_fresh_session():
    session = pine_compat.compile_script(SOURCE).external_session()
    session.append(BARS[:1], ACCOUNTS[:1])
    with pytest.raises(ValueError, match="strictly increasing"):
        session.append(BARS[:1], ACCOUNTS[:1])


def test_session_state_is_owned_by_each_caller():
    program = pine_compat.compile_script(SOURCE)
    first, second = program.external_session(), program.external_session()
    first.append(BARS, ACCOUNTS)
    assert second.cursor == 0
    assert second.append(BARS[:1], ACCOUNTS[:1]) == program.run_external(BARS[:1], ACCOUNTS[:1])


def test_declared_initial_window_is_available_for_host_coordinate_mapping():
    program = pine_compat.compile_script('//@version=6\nindicator("window", calc_bars_count=2)\nplot(close)')
    assert program.calculation_bars_count == 2
    assert pine_compat.compile_script(SOURCE).calculation_bars_count is None


@pytest.mark.parametrize("confirmed", [False, True])
def test_empty_append_keeps_fill_pass_snapshot_without_requiring_passes(confirmed):
    source = SOURCE.replace('strategy("External")', 'strategy("External", calc_on_order_fills=true)')
    session = pine_compat.compile_script(source).external_session()
    assert all(not plot["values"] for plot in session.append([], [])["output"]["plots"])
    assert session.cursor == 0
    group = [[dict(bar=BARS[0], account=ACCOUNTS[0], event_time_ms=1, confirmed=confirmed)]]
    result = session.append(BARS[:1], ACCOUNTS[:1], execution_passes=group)
    assert session.append([], []) == result
    assert session.cursor == 1
    if not confirmed:
        with pytest.raises(ValueError, match="unconfirmed"):
            session.append(BARS[1:2], ACCOUNTS[1:2], execution_passes=[[
                dict(bar=BARS[1], account=ACCOUNTS[1], event_time_ms=61, confirmed=True)
            ]])
        with pytest.raises(ValueError, match="POISONED"):
            session.append([], [])


def test_initial_window_slices_feedback_once_and_preserves_append_origin():
    source = '''//@version=6
strategy("Window", calc_bars_count=2)
var float total = 0
total += strategy.position_size
plot(total)
plot(strategy.equity)
strategy.entry("L", strategy.long)
'''
    program = pine_compat.compile_script(source)
    session = program.external_session()
    frames = [frame | {"equity": 1000 + i} for i, frame in enumerate(ACCOUNTS)]
    assert session.append(BARS, frames) == program.run_external(BARS, frames)
    assert session.cursor == 3
    extra_bar = BARS[-1] | {"time": 180}
    extra_frame = frames[1] | {"time": 180, "equity": 1003}
    result = session.append([extra_bar], [extra_frame])
    assert session.cursor == 4
    assert len(result["output"]["plots"][0]["values"]) == 3
    assert [p["values"] for p in result["output"]["plots"]] == [[1, 1, 2], [1001, 1002, 1003]]
    assert [intent["bar_index"] for intent in result["intents"]] == [0, 1, 2]


def test_decoding_failure_leaves_unexecuted_session_usable():
    session = pine_compat.compile_script(SOURCE).external_session()
    with pytest.raises(ValueError):
        session.append(BARS[:1], [ACCOUNTS[0] | {"time": True}])
    assert session.cursor == 0
    assert len(session.append(BARS[:1], ACCOUNTS[:1])["output"]["plots"][0]["values"]) == 1


def test_partial_execution_failure_requires_a_fresh_session():
    source = '''//@version=6
strategy("Partial")
if bar_index == 1
    strategy.entry("bad", strategy.long, qty=-1)
plot(close)
'''
    session = pine_compat.compile_script(source).external_session()
    with pytest.raises(ValueError, match="quantity"):
        session.append(BARS, ACCOUNTS)
    with pytest.raises(ValueError, match="POISONED"):
        session.append(BARS, ACCOUNTS)


@pytest.mark.parametrize("streaming", [False, True])
def test_pass_request_updates_preserve_host_currency_rates_and_calendar(streaming):
    source = '''//@version=6
strategy("Host inputs", calc_on_order_fills=true)
plot(request.currency_rate("USD", "EUR"))
plot(time_close-time)
plot(syminfo.timezone == "America/New_York" ? 1 : 0)
'''
    host = {
        "$currencyRates": {"schemaVersion": 1, "pairs": [
            {"from": "USD", "to": "EUR", "rates": [{"effectiveTime": 0, "rate": 0.9}]}
        ]},
        "$barCalendars": {"schemaVersion": 1, "symbols": [
            {"symbol": "X:MAIN", "timezone": "America/New_York", "timeframes": [
                {"timeframe": "1", "intervals": [{"open": 0, "close": 45000, "tradingDay": 0}]}
            ]}
        ]},
    }
    program = pine_compat.compile_script(source)
    group = [[dict(bar=BARS[0], account=ACCOUNTS[0], event_time_ms=1, confirmed=True, request_data=[])]]
    if streaming:
        session = program.external_session(chart_symbol="X:MAIN", request_bars=host)
        result = session.append(BARS[:1], ACCOUNTS[:1], execution_passes=group)
    else:
        result = program.run_external(BARS[:1], ACCOUNTS[:1], chart_symbol="X:MAIN",
                                      request_bars=host, execution_passes=group)
    assert [p["values"] for p in result["output"]["plots"]] == [[0.9], [45000], [1]]


@pytest.mark.parametrize("declaration", [
    'indicator("not a strategy")',
    'strategy("ticks", calc_on_every_tick=true)',
    'strategy("fees", commission_type=strategy.commission.percent, commission_value=1)',
])
def test_session_rejects_native_execution_settings(declaration):
    program = pine_compat.compile_script('//@version=6\n' + declaration + '\nplot(close)')
    with pytest.raises(ValueError, match="E_EXTERNAL_UNSUPPORTED"):
        program.external_session()

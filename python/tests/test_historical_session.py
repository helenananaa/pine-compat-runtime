"""Installed-wheel contracts for a fixed dataset, forks, and execution errors."""
import copy

import pine_compat
import pytest

BARS = [dict(time=i * 60000, open=p, high=p + 1, low=p - 1, close=p, volume=1)
        for i, p in enumerate([10, 13, 14, 8, 9])]


@pytest.mark.parametrize("declaration", ['indicator("History")', 'strategy("History")'])
def test_fixed_horizon_flags_and_fork_match_complete_batch(declaration):
    source = f'''//@version=6
{declaration}
var n = 0
n += 1
plot(n)
plot(bar_index)
plot(last_bar_index)
plot(last_bar_time)
plot(barstate.islast ? 1 : 0)
plot(barstate.islastconfirmedhistory ? 1 : 0)
plot(ta.sma(close, 2))
'''
    program = pine_compat.compile_script(source)
    session = program.historical_session(BARS)
    assert isinstance(session, pine_compat.HistoricalSession)
    initial = session.fork()
    first = session.advance(2)
    assert first["plots"][2]["values"] == [4, 4]
    assert first["plots"][4]["values"] == [0, 0]
    saved_output = copy.deepcopy(first)
    checkpoint = session.fork()
    assert session.advance(5) == program.run(BARS)
    assert first == saved_output
    assert checkpoint.cursor == 2
    assert initial.cursor == 0
    assert checkpoint.advance(5) == initial.advance(5) == program.run(BARS)


def test_native_orders_and_backward_seek_by_fork_equal_batch():
    program = pine_compat.compile_script('''//@version=6
strategy("Orders", calc_on_order_fills=true)
if close > 12
    strategy.entry("L", strategy.long, qty=2)
if close < 9
    strategy.close("L")
plot(strategy.position_size)
''')
    session = program.historical_session(BARS)
    session.advance(2)
    saved = session.fork()
    expected = program.run(BARS)
    assert session.advance(5) == expected
    assert saved.advance(3)["plots"][0]["values"] == expected["plots"][0]["values"][:3]
    assert saved.advance(5) == expected


def test_frozen_request_overrides_and_clock_equal_batch():
    program = pine_compat.compile_script('''//@version=6
indicator("Inputs")
factor = input.float(2, "Scale")
plot(request.security("OTHER", "1", close) * factor)
plot(timenow)
''')
    input_id = pine_compat.analyze_script('''//@version=6
indicator("Inputs")
factor = input.float(2, "Scale")
plot(request.security("OTHER", "1", close) * factor)
plot(timenow)
''')["inputs"][0]["callSiteId"]
    supplied = {"OTHER:1": [{**bar, "close": 100, "open": 100, "high": 101, "low": 99} for bar in BARS]}
    options = dict(request_bars=supplied, input_overrides={str(input_id): 3},
                   chart_symbol="MAIN", chart_timeframe="1", execution_times=[b["time"] + 10 for b in BARS])
    expected = program.run(BARS, **options)
    session = program.historical_session(BARS, **options)
    supplied["OTHER:1"][0]["close"] = 900
    session.advance(2)
    assert session.advance(5) == expected


def test_calc_bars_count_matches_batch_without_evaluating_skipped_bars():
    program = pine_compat.compile_script('''//@version=6
indicator("Window", calc_bars_count=3)
plot(bar_index)
plot(last_bar_index)
plot(close)
''')
    session = program.historical_session(BARS)
    assert session.advance(1)["plots"] == []
    assert session.advance(5) == program.run(BARS)


@pytest.mark.parametrize("target", [True, -1, 1.5, "2", 6])
def test_invalid_cursor_does_not_change_valid_session(target):
    program = pine_compat.compile_script('//@version=6\nindicator("Range")\nplot(close)')
    session = program.historical_session(BARS)
    with pytest.raises(ValueError, match="cursor"):
        session.advance(target)
    assert session.cursor == 0
    session.advance(2)
    with pytest.raises(ValueError, match="forward range"):
        session.advance(1)
    assert session.advance(5) == program.run(BARS)


def test_execution_error_poisons_session_but_saved_fork_remains_usable():
    program = pine_compat.compile_script('''//@version=6
indicator("Failure")
if bar_index == 2
    runtime.error("failure receipt")
plot(close)
''')
    session = program.historical_session(BARS)
    prior = session.advance(2)
    saved = session.fork()
    with pytest.raises(ValueError, match="failure receipt"):
        session.advance(5)
    assert session.cursor == 2
    with pytest.raises(ValueError, match="E_RUNTIME_POISONED"):
        session.advance(2)
    with pytest.raises(ValueError, match="E_RUNTIME_POISONED"):
        session.fork()
    assert saved.advance(2) == prior


def test_history_admission_rejects_bad_timestamps_and_clock_count():
    program = pine_compat.compile_script('//@version=6\nindicator("Admission")\nplot(close)')
    for bars, match in [([], "empty"), ([BARS[0], BARS[0]], "duplicate"),
                        ([BARS[1], BARS[0]], "sorted")]:
        with pytest.raises(ValueError, match=match):
            program.historical_session(bars)
    with pytest.raises(ValueError, match="bar count"):
        program.historical_session(BARS, execution_times=[0])

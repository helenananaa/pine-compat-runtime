import pytest
import pine_compat

BARS = [{"time": i * 60, "open": 10, "high": 11, "low": 9, "close": 10, "volume": 1} for i in range(3)]
ACCOUNTS = [{"time": i * 60, "position_size": qty, "position_avg_price": 10 if qty else None,
             "equity": 1000, "initial_capital": 1000, "netprofit": 0, "openprofit": 0} for i, qty in enumerate([0, 1, 0])]
SOURCE = '''//@version=6
strategy("External")
if strategy.position_size == 0
    strategy.entry("L", strategy.long)
else
    strategy.close("L")
plot(strategy.equity)
'''


def test_external_feedback_drives_interpreter_and_has_no_native_account():
    result = pine_compat.compile_script(SOURCE).run_external(BARS, ACCOUNTS)
    assert [item["action"] for item in result["intents"]] == ["entry", "close", "entry"]
    assert "strategy" not in result["output"]
    assert result["output"]["plots"][0]["values"] == [1000, 1000, 1000]


@pytest.mark.parametrize("source", [SOURCE.replace('strategy("External")', 'strategy("External", calc_on_order_fills=true)'),
                                    SOURCE.replace('strategy.entry("L", strategy.long)', 'strategy.entry("L", strategy.long, comment="unsupported")'),
                                    SOURCE.replace('strategy.position_size', 'strategy.wintrades')])
def test_external_unsupported_semantics_never_fall_back(source):
    with pytest.raises(ValueError, match="E_EXTERNAL_UNSUPPORTED"):
        pine_compat.compile_script(source).run_external(BARS, ACCOUNTS)


def test_feedback_time_mismatch_rejected():
    with pytest.raises(ValueError, match="feedback time"):
        pine_compat.compile_script(SOURCE).run_external(BARS, [{**ACCOUNTS[0], "time": -1}, *ACCOUNTS[1:]])


def test_rich_intents_preserve_prices_quantity_and_cancellation():
    source = '''//@version=6
strategy("Orders")
strategy.entry("L", strategy.long, qty=2, limit=close, stop=close+1)
strategy.exit("X", "L", qty_percent=50, limit=close+2, stop=close-2)
strategy.cancel("X")
strategy.cancel_all()
'''
    result = pine_compat.compile_script(source).run_external(BARS, ACCOUNTS)
    first = result["intents"][:4]
    assert [row["action"] for row in first] == ["entry", "exit", "cancel", "cancel_all"]
    assert first[0]["limit"] == 10 and first[0]["stop"] == 11
    assert first[1]["qty_percent"] == 50 and first[1]["from_entry"] == "L"
    assert "strategy" not in result["output"]


def test_external_request_data_uses_supplied_environment():
    bars = [{**bar, "time": i*60000} for i, bar in enumerate(BARS)]
    accounts = [{**frame, "time": i*60000} for i, frame in enumerate(ACCOUNTS)]
    source = '\n'.join(['//@version=6', 'strategy("Requests")',
        'remote = request.security("X:OTHER", "1", close)',
        'if remote > 50', '    strategy.entry("L", strategy.long)', 'plot(remote)'])
    supplied = [{**bar, "open": 100, "high": 101, "low": 99, "close": 100} for bar in bars]
    result = pine_compat.compile_script(source).run_external(bars, accounts,
        chart_symbol="X:MAIN", chart_timeframe="1", request_bars={"X:OTHER:1": supplied})
    assert len(result["intents"]) == 3
    assert result["output"]["plots"][0]["values"] == [100, 100, 100]
    assert not result["output"].get("strategy")


def test_external_known_empty_stream_is_na_but_unknown_stream_fails():
    program = pine_compat.compile_script('//@version=6\nstrategy("Empty")\nplot(request.security("X:OTHER", "2", close))')
    result = program.run_external(BARS[:1], ACCOUNTS[:1], chart_timeframe="1", request_bars={"X:OTHER:2": []})
    assert result["output"]["plots"][0]["values"] == [None]
    with pytest.raises(ValueError, match="missing request data"):
        program.run_external(BARS[:1], ACCOUNTS[:1], chart_timeframe="1")


def test_external_pyramiding_and_tick_exit_intents_are_host_neutral():
    source = '''//@version=6
strategy("Intents", pyramiding=3)
strategy.entry("A", strategy.long, qty=2)
strategy.entry("B", strategy.long, qty=1)
strategy.exit("X", "A", profit=4, loss=2)
strategy.exit("T", "B", trail_points=3, trail_offset=1)
'''
    result = pine_compat.compile_script(source).run_external(BARS[:1], ACCOUNTS[:1])
    assert result["pyramiding"] == 3
    assert [row["id"] for row in result["intents"]] == ["A","B","X","T"]
    assert result["intents"][2]["profit"] == 4
    assert result["intents"][3]["trail_offset"] == 1
    assert not result["output"].get("strategy")


def test_pass_request_cache_does_not_retroactively_expose_later_data():
    bars = [{**bar,"time":i*60000} for i,bar in enumerate(BARS[:2])]
    frames = [{**frame,"time":i*60000} for i,frame in enumerate(ACCOUNTS[:2])]
    empty = [{"symbol":"X:OTHER","timeframe":"2","bars":[]}]
    full = [{**empty[0],"bars":[dict(time=0,open=100,high=101,low=99,close=100,volume=1)]}]
    groups = [[dict(bar=bars[0],account=frames[0],event_time_ms=59999,confirmed=True,request_data=empty)],
        [dict(bar=bars[1],account=frames[1],event_time_ms=60000,confirmed=False,request_data=empty),
         dict(bar=bars[1],account=frames[1],event_time_ms=119999,confirmed=True,request_data=full)]]
    source = '''//@version=6
strategy("Pass requests", calc_on_order_fills=true)
remote = request.security("X:OTHER", "2", close)
if na(remote)
    strategy.entry("L", strategy.long)
else
    strategy.close_all()
'''
    result = pine_compat.compile_script(source).run_external(bars,frames,chart_timeframe="1",execution_passes=groups)
    assert [(row["bar_index"],row["pass_index"],row["action"]) for row in result["intents"]] == [(0,0,"entry"),(1,0,"entry"),(1,1,"close_all")]


def test_external_position_history_uses_feedback_instead_of_native_ledger():
    program = pine_compat.compile_script('''//@version=6
strategy("History")
plot(strategy.position_size[1])
plot(strategy.position_size)
''')
    result = program.run_external(BARS, ACCOUNTS)
    assert result["output"]["plots"][0]["values"] == [None, 0, 1]
    assert result["output"]["plots"][1]["values"] == [0, 1, 0]
    assert "strategy" not in result["output"]


def test_entry_quantity_expression_is_evaluated_once():
    program = pine_compat.compile_script('''//@version=6
strategy("Quantity")
quantities = array.from(1.0, 2.0)
strategy.entry("L", strategy.long, qty=quantities.pop())
plot(quantities.size())
''')
    result = program.run_external(BARS[:1], ACCOUNTS[:1])
    assert result["intents"][0]["qty"] == 2
    assert result["output"]["plots"][0]["values"] == [1]


@pytest.mark.parametrize("with_passes", [False, True])
def test_execution_window_slices_feedback_together_with_chart_bars(with_passes):
    declaration = 'strategy("Window", calc_bars_count=1' + (', calc_on_order_fills=true)' if with_passes else ')')
    program = pine_compat.compile_script('//@version=6\n' + declaration + '\nplot(strategy.equity)\nstrategy.entry("L", strategy.long)')
    frames = [{**frame, "equity": 1000 + i} for i, frame in enumerate(ACCOUNTS)]
    options = {}
    if with_passes:
        options["execution_passes"] = [[dict(bar=bar, account=frame, event_time_ms=bar["time"], confirmed=True)]
                                       for bar, frame in zip(BARS, frames)]
    result = program.run_external(BARS, frames, **options)
    assert result["output"]["plots"][0]["values"] == [1002]
    assert [item["bar_index"] for item in result["intents"]] == [0]


@pytest.mark.parametrize("change", [dict(position_size=float("inf")), dict(position_avg_price=-1),
                                    dict(position_size=1, position_avg_price=None), dict(time=True)])
def test_invalid_account_state_is_rejected(change):
    with pytest.raises(ValueError):
        pine_compat.compile_script(SOURCE).run_external(BARS[:1], [{**ACCOUNTS[0], **change}])


def test_passes_commit_only_final_values_and_preserve_feedback_history():
    program = pine_compat.compile_script('''//@version=6
strategy("Passes", calc_on_order_fills=true)
plot(strategy.position_size)
plot(strategy.position_size[1])
plot(barstate.isnew ? 1 : 0)
if strategy.position_size > 0
    strategy.close_all()
''')
    groups = [[dict(bar=BARS[0], account=ACCOUNTS[0], event_time_ms=1, confirmed=True)],
              [dict(bar=BARS[1], account=ACCOUNTS[1], event_time_ms=61, confirmed=False),
               dict(bar=BARS[1], account=ACCOUNTS[2] | {"time": 60}, event_time_ms=62, confirmed=True)]]
    result = program.run_external(BARS[:2], ACCOUNTS[:2], execution_passes=groups)
    assert result["output"]["plots"][0]["values"] == [0, 0]
    assert result["output"]["plots"][1]["values"] == [None, 0]
    assert result["output"]["plots"][2]["values"] == [1, 0]
    assert [(i["bar_index"], i["pass_index"]) for i in result["intents"]] == [(1, 0)]

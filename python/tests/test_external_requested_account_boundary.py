"""Requested market contexts must not fabricate a separate native account."""
import pine_compat
import pytest

BARS = [dict(time=i*60000,open=10,high=11,low=9,close=10,volume=100) for i in range(2)]
REMOTE = [{**bar,"open":100*(i+1),"high":100*(i+1)+1,"low":100*(i+1)-1,"close":100*(i+1)} for i,bar in enumerate(BARS)]
ACCOUNTS = [dict(time=bar["time"],position_size=2,position_avg_price=10,equity=1000-200*i,
                 initial_capital=1000,netprofit=-200*i,openprofit=0) for i,bar in enumerate(BARS)]
HOST = {"X:OTHER:1": REMOTE,"X:NEST:1":REMOTE}


@pytest.mark.parametrize("expression", ["strategy.equity", "strategy.position_size", "strategy.equity[1]",
                                       "strategy.default_entry_qty(close)",
                                       'request.security("X:NEST","1",strategy.equity)'])
def test_external_requested_account_observations_fail_without_an_account_timeline(expression):
    source = f'''//@version=6
strategy("Requested account",initial_capital=1000000,default_qty_type=strategy.percent_of_equity,default_qty_value=25)
plot(request.security("X:OTHER","1",{expression}))
'''
    # Existing semantic admission rejects account expressions in requested
    # contexts; adding chart-default sizing must preserve that boundary.
    with pytest.raises(ValueError,match="E_UNSUPPORTED_FEATURE:.*`request.security`"):
        pine_compat.compile_script(source)


def test_parent_account_reads_and_requested_market_values_remain_separate():
    program = pine_compat.compile_script('''//@version=6
strategy("Parent account",initial_capital=1000000)
plot(strategy.equity)
plot(request.security("X:OTHER","1",close))
''')
    result = program.run_external(BARS,ACCOUNTS,chart_symbol="X:MAIN",chart_timeframe="1",request_bars=HOST)
    assert [row["values"] for row in result["output"]["plots"]] == [[1000,800],[100,200]]
    assert "strategy" not in result["output"]

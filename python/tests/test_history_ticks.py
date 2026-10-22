"""Installed native and explicit-account history observations."""
import pytest
import pine_compat
from test_external_schedule import account, bar, scheduled


def program(body="plot(close)", flags="calc_on_every_history_tick=true"):
    return pine_compat.compile_script(f'//@version=6\nstrategy("History ticks",{flags})\n{body}')


def test_native_history_ticks_commit_ordinary_state_once_and_retain_varip():
    code = program("var int ordinary=0\nvarip int retained=0\nordinary+=1\nretained+=1\nplot(ordinary)\nplot(retained)\nplot(ta.cum(volume))")
    assert code.host_requirements()["execution"]["calcOnEveryHistoryTick"] is True
    assert pine_compat.EXTERNAL_TICK_SCHEDULE_SCHEMA_VERSION == 1
    assert pine_compat.MAX_STRATEGY_OBSERVATIONS_PER_BAR == 100_000
    result=code.run([bar(),bar(60000)])
    assert [row["values"] for row in result["plots"]] == [[1,2],[4,8],[100,200]]


def test_history_tick_batch_and_append_keep_authoritative_account_and_commit_identity():
    code=program("varip int n=0\nn+=1\nplot(n)\nplot(strategy.equity)\nplot(barstate.isconfirmed?1:0)\nstrategy.entry(\"L\",strategy.long,qty=1)")
    bars=[bar(),bar(60000)]
    accounts=[account(row["time"]) for row in bars]
    groups=[[scheduled(bar(row["time"],close),trigger="historical_tick",commit=i==3,
        clock=row["time"]+i, new=True) for i,close in enumerate([10,12,9,12])] for row in bars]
    actual=code.run_external(bars,accounts,execution_schedule=groups)
    assert [row["values"] for row in actual["output"]["plots"]] == [[4,8],[1000,1000],[1,1]]
    session=code.external_session(scheduled=True)
    for i in range(2):
        incremental=session.append(bars[i:i+1],accounts[i:i+1],execution_schedule=groups[i:i+1])
    assert incremental == actual
    assert session.cursor == 2
    with pytest.raises(ValueError,match="E_EXTERNAL_UNSUPPORTED"):
        code.run_external(bars,accounts)


@pytest.mark.parametrize("historical",[True,False])
def test_explicit_tick_observations_do_not_spend_fill_loop_budget(historical):
    code=program("varip int n=0\nn+=1\nplot(n)",flags="calc_on_every_history_tick=true" if historical else "calc_on_every_tick=true")
    group=[scheduled(bar(),state="historical" if historical else "realtime",
        trigger="historical_tick" if historical else "realtime_tick",
        commit=i==2047,new=historical or i==0,clock=i) for i in range(2048)]
    actual=code.run_external([bar()],[account()],execution_schedule=[group])
    assert actual["output"]["plots"][0]["values"] == [2048]


def test_history_tick_declared_schedule_cannot_silently_fall_back_to_bar_close():
    code=program()
    with pytest.raises(ValueError,match="invalid scheduled execution context"):
        code.run_external([bar()],[account()],execution_schedule=[[scheduled(bar())]])
    with pytest.raises(ValueError):
        pine_compat.compile_script('//@version=5\nstrategy("old",calc_on_every_history_tick=true)')

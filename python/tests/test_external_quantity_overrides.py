"""Explicit sizing uses the caller's quantities and authoritative account frames."""
import pine_compat
import pytest

BARS = [dict(time=i * 60, open=10, high=11, low=9, close=10, volume=100) for i in range(2)]
ACCOUNTS = [dict(time=i * 60, position_size=3 * i, position_avg_price=10 if i else None,
                 equity=1000 + 200 * i, initial_capital=1000, netprofit=0, openprofit=200 * i) for i in range(2)]


@pytest.mark.parametrize("version", [5, 6])
@pytest.mark.parametrize("kind", ["cash", "percent_of_equity"])
@pytest.mark.parametrize("command", ["entry", "order"])
def test_explicit_quantity_overrides_declaration_in_batch_and_append(version, kind, command):
    program = pine_compat.compile_script(f'''//@version={version}
strategy("Override",default_qty_type=strategy.{kind},default_qty_value=25)
var calls=array.new_int(1,0)
quantity() =>
    array.set(calls,0,array.get(calls,0)+1)
    array.get(calls,0)+1
strategy.{command}("L",strategy.long,qty=quantity())
plot(strategy.equity)
plot(strategy.position_size)
plot(array.get(calls,0))
''')
    session = program.external_session()
    for index in range(2):
        actual = session.append(BARS[index:index+1], ACCOUNTS[index:index+1])
        assert actual == program.run_external(BARS[:index+1], ACCOUNTS[:index+1])
    assert [row["qty"] for row in actual["intents"]] == [2, 3]
    assert actual["output"]["plots"][0]["values"] == [1000, 1200]
    assert actual["output"]["plots"][1]["values"] == [0, 3]
    assert actual["output"]["plots"][2]["values"] == [1, 2]
    assert "strategy" not in actual["output"]


@pytest.mark.parametrize("version", [5, 6])
@pytest.mark.parametrize("kind", ["fixed", "cash", "percent_of_equity"])
@pytest.mark.parametrize("command", ["entry", "order"])
def test_zero_quantity_emits_no_replacement_or_cancellation(version, kind, command):
    source = f'''//@version={version}
strategy("Zero",default_qty_type=strategy.{kind},default_qty_value=25)
if bar_index==0
    strategy.{command}("L",strategy.long,qty=2)
strategy.{command}("L",strategy.short,qty=0)
strategy.{command}("Z",strategy.long,qty=-0.0)
'''
    program = pine_compat.compile_script(source)
    result = program.run_external(BARS, ACCOUNTS)
    assert [(row["id"], row["action"], row["direction"], row["qty"]) for row in result["intents"]] == [("L", command, "long", 2)]
    session = program.external_session()
    for i in range(2):
        actual = session.append(BARS[i:i+1], ACCOUNTS[i:i+1])
    assert actual == result


@pytest.mark.parametrize("version", [5, 6])
def test_zero_fixed_defaults_emit_no_intent(version):
    program = pine_compat.compile_script(f'''//@version={version}
strategy("Zero default",default_qty_value=0)
strategy.entry("E",strategy.long)
strategy.order("O",strategy.long,qty=na)
''')
    assert program.run_external(BARS, ACCOUNTS)["intents"] == []


@pytest.mark.parametrize("kind", ["cash", "percent_of_equity"])
@pytest.mark.parametrize("command", ["entry", "order"])
@pytest.mark.parametrize("quantity", ["", ",qty=na"])
def test_unnegotiated_nonfixed_defaults_fail_when_needed(kind, command, quantity):
    program = pine_compat.compile_script(f'//@version=6\nstrategy("Default",default_qty_type=strategy.{kind},default_qty_value=25)\nstrategy.{command}("L",strategy.long{quantity})')
    with pytest.raises(ValueError, match="E_EXTERNAL_UNSUPPORTED: external default sizing"):
        program.run_external(BARS, ACCOUNTS)
    session = program.external_session()
    with pytest.raises(ValueError, match="E_EXTERNAL_UNSUPPORTED: external default sizing"):
        session.append(BARS, ACCOUNTS)
    with pytest.raises(ValueError, match="POISONED"):
        session.append([], [])


@pytest.mark.parametrize("command", ["entry", "order"])
def test_negative_quantity_still_fails(command):
    program = pine_compat.compile_script(f'//@version=6\nstrategy("Invalid")\nstrategy.{command}("L",strategy.long,qty=-1)')
    with pytest.raises(ValueError, match="quantity must be positive and finite"):
        program.run_external(BARS, ACCOUNTS)


@pytest.mark.parametrize("kind", ["cash", "percent_of_equity"])
def test_unused_nonfixed_defaults_do_not_require_a_sizing_provider(kind):
    program = pine_compat.compile_script(f'//@version=6\nstrategy("No orders",default_qty_type=strategy.{kind},default_qty_value=25)\nplot(strategy.equity)')
    result = program.external_session().append(BARS, ACCOUNTS)
    assert result == program.run_external(BARS, ACCOUNTS)
    assert result["intents"] == []
    assert result["output"]["plots"][0]["values"] == [1000, 1200]

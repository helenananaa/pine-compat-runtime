"""Default quantities use Pine arithmetic and the active Host account/pass."""
import pine_compat
import pytest

BARS = [dict(time=i*60000, open=p, high=p+1, low=p-1, close=p, volume=100)
        for i, p in enumerate([10, 20, 5])]
ACCOUNTS = [dict(time=i*60000, position_size=0, position_avg_price=None,
                 equity=equity, initial_capital=1000, netprofit=equity-1000, openprofit=0)
            for i, equity in enumerate([1000, 800, 1600])]


@pytest.mark.parametrize("version", [5, 6])
@pytest.mark.parametrize("kind,expected", [("cash", [10, 5, 20]), ("percent_of_equity", [100, 40, 320])])
@pytest.mark.parametrize("command", ["entry", "order"])
@pytest.mark.parametrize("quantity", ["", ",qty=na"])
def test_default_quantities_use_placement_price_and_host_equity(version, kind, expected, command, quantity):
    source = f'''//@version={version}
strategy("Host sizing",initial_capital=1000000,default_qty_type=strategy.{kind},default_qty_value=100)
strategy.{command}("L",strategy.long{quantity})
plot(strategy.default_entry_qty(10))
plot(strategy.equity)
'''
    program = pine_compat.compile_script(source)
    assert program.host_requirements()["account"]["defaultQuantity"] == {"kind":kind,"value":"100"}
    actual = program.run_external(BARS, ACCOUNTS)
    assert pine_compat.EXTERNAL_SIZING_SCHEMA_VERSION == 1
    assert [row["qty"] for row in actual["intents"]] == expected
    assert actual["output"]["plots"][0]["values"] == ([10]*3 if kind == "cash" else [100, 80, 160])
    assert actual["output"]["plots"][1]["values"] == [1000, 800, 1600]
    assert "strategy" not in actual["output"]
    session = program.external_session()
    for i in range(3):
        prefix = session.append(BARS[i:i+1], ACCOUNTS[i:i+1])
        assert prefix == program.run_external(BARS[:i+1], ACCOUNTS[:i+1])
    assert prefix == actual


@pytest.mark.parametrize("kind", ["cash", "percent_of_equity"])
@pytest.mark.parametrize("command", ["entry", "order"])
@pytest.mark.parametrize("price_argument", ["", ",limit=30", ",stop=11", ",stop=11,limit=30"])
def test_default_placement_quantity_matches_native_across_a_price_gap(kind, command, price_argument):
    program = pine_compat.compile_script(f'''//@version=6
strategy("Gap",initial_capital=1000,default_qty_type=strategy.{kind},default_qty_value=100,margin_long=0)
if bar_index==0
    strategy.{command}("L",strategy.long{price_argument})
''')
    native = program.run(BARS)
    external = program.run_external(BARS, ACCOUNTS)
    assert len(external["intents"]) == 1
    assert native["strategy"]["orders"]
    assert external["intents"][0]["qty"] == native["strategy"]["orders"][0]["qty"]
    assert external["intents"][0]["qty"] == (10 if kind == "cash" else 100)


@pytest.mark.parametrize("kind,value", [("fixed", 1.239), ("cash", 12.39), ("percent_of_equity", 1.239)])
def test_default_quantities_and_query_share_the_supplied_quantity_grid(kind, value):
    program = pine_compat.compile_script(f'''//@version=6
strategy("Grid",default_qty_type=strategy.{kind},default_qty_value={value})
strategy.entry("L",strategy.long)
plot(strategy.default_entry_qty(close))
''')
    output = program.run_external(BARS[:1], ACCOUNTS[:1], request_bars={"$chart":dict(minMove=1,priceScale=10,quantityPrecision=2)})
    assert output["intents"][0]["qty"] == 1.23
    assert output["output"]["plots"][0]["values"] == [1.23]


@pytest.mark.parametrize("quantity,precision,expected", [(1.239, 2, 1.23), (.0009, 3, None), (.129515, 6, .129515)])
def test_explicit_quantities_follow_native_decimal_truncation(quantity, precision, expected):
    program = pine_compat.compile_script(f'//@version=6\nstrategy("Grid")\nstrategy.entry("L",strategy.long,qty={quantity})')
    output = program.run_external(BARS[:1], ACCOUNTS[:1],request_bars={"$chart":dict(minMove=1,priceScale=10,quantityPrecision=precision)})
    assert [row["qty"] for row in output["intents"]] == ([] if expected is None else [expected])


def test_negative_subcontract_quantity_is_not_truncated_to_a_noop():
    program = pine_compat.compile_script('//@version=6\nstrategy("Negative")\nstrategy.entry("L",strategy.long,qty=-0.0009)')
    with pytest.raises(ValueError,match="quantity must be positive and finite"):
        program.run_external(BARS[:1],ACCOUNTS[:1],request_bars={"$chart":dict(minMove=1,priceScale=10,quantityPrecision=3)})


@pytest.mark.parametrize("kind", ["fixed", "cash", "percent_of_equity"])
def test_zero_default_budgets_emit_no_intents(kind):
    program = pine_compat.compile_script(f'''//@version=6
strategy("Zero default",default_qty_type=strategy.{kind},default_qty_value=0)
strategy.entry("E",strategy.long)
strategy.order("O",strategy.short,qty=na)
plot(strategy.default_entry_qty(close))
''')
    result = program.run_external(BARS,ACCOUNTS)
    assert result["intents"] == []
    assert result["output"]["plots"][0]["values"] == [0,0,0]


def test_fill_pass_sizing_uses_each_current_account_instead_of_the_base_frame():
    program = pine_compat.compile_script('''//@version=6
strategy("Pass sizing",initial_capital=1000000,default_qty_type=strategy.percent_of_equity,default_qty_value=25,calc_on_order_fills=true)
strategy.order("L",strategy.long)
plot(strategy.default_entry_qty(close))
''')
    group = [[dict(bar=BARS[0],account=ACCOUNTS[0],event_time_ms=59999,confirmed=True)],
             [dict(bar=BARS[1],account={**ACCOUNTS[1],"equity":equity},event_time_ms=60001+i,confirmed=i==2)
              for i,equity in enumerate([800,600,400])]]
    options = dict(execution_passes=group)
    result = program.run_external(BARS[:2],ACCOUNTS[:2],**options)
    assert [row["qty"] for row in result["intents"]] == [25,10,7.5,5]
    assert result["output"]["plots"][0]["values"] == [25,5]
    session = program.external_session()
    for i in range(2):
        incremental = session.append(BARS[i:i+1],ACCOUNTS[i:i+1],execution_passes=group[i:i+1])
    assert incremental == result


@pytest.mark.parametrize("api", ["entry", "default_entry_qty"])
def test_negative_host_equity_keeps_the_native_error_and_poisons_the_session(api):
    call = 'strategy.entry("L",strategy.long)' if api == "entry" else 'plot(strategy.default_entry_qty(10))'
    program = pine_compat.compile_script('//@version=6\nstrategy("Negative equity",default_qty_type=strategy.percent_of_equity,default_qty_value=25)\n'+call)
    session = program.external_session()
    with pytest.raises(ValueError,match="equity is less than 0"):
        session.append(BARS[:1],[{**ACCOUNTS[0],"equity":-1}])
    with pytest.raises(ValueError,match="POISONED"):
        session.append([],[])


def test_currency_conversion_is_shared_with_native_sizing_without_native_account_values():
    program = pine_compat.compile_script('''//@version=6
strategy("Currency sizing",currency=currency.EUR,initial_capital=1000000,default_qty_type=strategy.percent_of_equity,default_qty_value=25)
strategy.entry("L",strategy.long)
plot(strategy.default_entry_qty(10))
''')
    rates = dict(schemaVersion=1,pairs=[dict(**{"from":"USD","to":"EUR"},rates=[dict(effectiveTime=0,rate=.5)])])
    host = {"$chart":dict(minMove=1,priceScale=10,currency="USD"),"$currencyRates":rates}
    output = program.run_external(BARS,ACCOUNTS,request_bars=host)
    assert [row["qty"] for row in output["intents"]] == [50,20,160]
    assert output["output"]["plots"][0]["values"] == [50,40,80]
    with pytest.raises(ValueError,match="missing currency rate data"):
        program.run_external(BARS,ACCOUNTS)

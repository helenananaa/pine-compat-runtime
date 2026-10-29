import pine_compat
import pytest


def bars(base):
    return [dict(time=i * 60000, open=base + i, high=base + i + 2,
                 low=base + i - 2, close=base + i, volume=10) for i in range(5)]


@pytest.mark.parametrize("command", ["entry", "order"])
@pytest.mark.parametrize("price", ["na", "float(na)"])
def test_na_prices_produce_market_trades(command, price):
    source = f'''//@version=6
strategy("na")
if bar_index == 0
    strategy.{command}("E", strategy.long, qty=1, limit={price}, stop={price})
if bar_index == 3
    strategy.close_all()
'''
    result = pine_compat.compile_script(source).run(bars(100))
    assert len(result["strategy"]["trades"]) == 1
    assert result["strategy"]["trades"][0]["entryPrice"] == 101
    assert result["strategy"]["diagnostics"] == []


def test_named_nz_and_function_alert():
    source = '''//@version=5
indicator("names")
emit() =>
    alert("inside")
    nz(replacement=7.0, source=close[1])
plot(emit())
'''
    result = pine_compat.compile_script(source).run(bars(100))
    assert result["plots"][0]["values"] == [7, 100, 101, 102, 103]
    assert [a["message"] for a in result["alerts"]] == ["inside"] * 5


def test_standard_prices_reach_python_broker_without_replacing_chart_series():
    source = '''//@version=6
strategy("dual",fill_orders_on_standard_ohlc=true)
if bar_index==0
    strategy.entry("E",strategy.long,qty=1)
if bar_index==3
    strategy.close_all()
plot(close)
'''
    chart, standard = bars(1000), bars(100)
    host = {"$chart": {"minMove": 1, "priceScale": 100, "chartType": "heikinashi"},
            "$standardOhlc": {"schemaVersion": 1, "observations": [
                {"observation": {"chart": c, "kind": "historical", "executionTime": None}, "standard": s}
                for c, s in zip(chart, standard)]}}
    program = pine_compat.compile_script(source)
    result = program.run(chart, request_bars=host)
    assert result["plots"][0]["values"] == [1000, 1001, 1002, 1003, 1004]
    trade = result["strategy"]["trades"][0]
    assert (trade["entryPrice"], trade["exitPrice"]) == (101, 104)
    del host["$standardOhlc"]
    with pytest.raises(ValueError, match="E_STANDARD_OHLC_INPUT"):
        program.run(chart, request_bars=host)

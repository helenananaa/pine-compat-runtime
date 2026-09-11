import pine_compat
import pytest


BARS = [dict(time=0, open=10, high=10, low=10, close=10, volume=1)]
SOURCE = '''//@version=6
indicator("grid")
plot(syminfo.mintick)
plot(math.round_to_mintick(10.26))
plot(str.tostring(10.26, format.mintick) == "10.3" ? 1 : 0)
'''


def test_explicit_unit_point_value_preserves_execution_output():
    source = '//@version=6\nindicator("point value")\nplot(syminfo.pointvalue)\n'
    program = pine_compat.compile_script(source)
    for value in (1, 1.0):
        assert program.run(BARS, request_bars={"$chart": {"minMove": 1, "priceScale": 100, "pointValue": value}}) == program.run(BARS)


@pytest.mark.parametrize("value", [True, None, "1", 0, -1, 0.5, 5, 1.0000000001, float("nan"), float("inf"), float("-inf")])
def test_explicit_point_value_rejects_unsupported_or_invalid_profiles(value):
    with pytest.raises(ValueError, match="pointValue"):
        pine_compat.run_script(SOURCE, BARS, request_bars={"$chart": {"minMove": 1, "priceScale": 10, "pointValue": value}})


def test_chart_quantity_precision_is_per_execution_and_preserves_default():
    source = "//@version=6\nindicator(\"quantity\")\nf(simple float value=syminfo.mincontract) => value\nplot(f())\n"
    program = pine_compat.compile_script(source)
    configured = program.run(BARS, request_bars={"$chart": {"minMove": 1, "priceScale": 10, "quantityPrecision": 6}})
    assert configured["plots"][0]["values"] == [0.000001]
    assert program.run(BARS)["plots"][0]["values"] == [1]


@pytest.mark.parametrize("precision", [True, -1, 1.5, "6", 10, 4294967296])
def test_chart_quantity_precision_rejects_invalid_metadata(precision):
    with pytest.raises(ValueError):
        pine_compat.run_script(SOURCE, BARS, request_bars={"$chart": {"minMove": 1, "priceScale": 10, "quantityPrecision": precision}})


def test_chart_price_grid_is_per_execution_and_preserves_default():
    program = pine_compat.compile_script(SOURCE)
    result = program.run(BARS, request_bars={"$chart": {"minMove": 1, "priceScale": 10}})
    assert result["plots"][0]["values"] == [0.1]
    assert result["plots"][1]["values"] == pytest.approx([10.3])
    assert result["plots"][2]["values"] == [1]
    assert program.run(BARS)["plots"][0]["values"] == [0.01]


@pytest.mark.parametrize("grid", [
    {"minMove": 0, "priceScale": 10}, {"minMove": 1, "priceScale": 0},
    {"minMove": True, "priceScale": 10}, {"minMove": 1, "priceScale": 1.5},
    {"minMove": 1}, {"minMove": 1, "priceScale": 10, "unexpected": 1},
])
def test_chart_price_grid_rejects_invalid_metadata(grid):
    with pytest.raises(ValueError):
        pine_compat.run_script(SOURCE, BARS, request_bars={"$chart": grid})

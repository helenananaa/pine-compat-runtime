import pine_compat
import pytest


BARS = [dict(time=0, open=10, high=10, low=10, close=10, volume=1)]
SOURCE = '''//@version=6
indicator("grid")
plot(syminfo.mintick)
plot(math.round_to_mintick(10.26))
plot(str.tostring(10.26, format.mintick) == "10.3" ? 1 : 0)
'''


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

from pathlib import Path

import pine_compat
import pytest

EVIDENCE = Path(__file__).resolve().parents[2] / "docs/evidence/strategy-library-20261010"
BARS = [dict(time=1700000000000+i*60000, open=10., high=10., low=10., close=10., volume=1.) for i in range(6)]


def test_v4_direction_constants_are_boolean_through_aliases_and_order_conditions():
    source = (EVIDENCE / "legacy-direction-v4-oracle-27.pine").read_text(encoding="utf-8")
    result = pine_compat.compile_script(source).run(BARS)
    assert [p["values"] for p in result["plots"]] == [[n]*6 for n in [1,0,1,1,1]] + [[0,1,1,1,0,0]]


@pytest.mark.parametrize("variant,expected", [("comment",0), ("bool",1)])
def test_v5_close_keeps_comment_and_legacy_when_overloads(variant, expected):
    source = (EVIDENCE / f"close-{variant}-v5-oracle-27.pine").read_text(encoding="utf-8")
    result = pine_compat.compile_script(source+"\nplot(strategy.position_size)\n").run(BARS)
    assert result["plots"][1]["values"][-1] == expected


@pytest.mark.parametrize("version", [5,6])
def test_direction_alias_cannot_select_close_string_comment_overload(version):
    with pytest.raises(ValueError,match="strategy_direction"):
        pine_compat.compile_script(f'//@version={version}\nstrategy("guard")\nd=strategy.long\nf(x)=>x\nstrategy.close("L",f(d))\n')

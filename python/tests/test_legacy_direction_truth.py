from pathlib import Path

import pine_compat
import pytest

EVIDENCE = Path(__file__).resolve().parents[2] / "docs/evidence/strategy-library-20261010"
BARS = [dict(time=1700000000000+i*60000, open=10., high=10., low=10., close=10., volume=1.) for i in range(6)]


@pytest.mark.parametrize("version",[1,2,3])
def test_legacy_direction_native_truth_and_identity_matrix(version):
    source=(EVIDENCE/f"legacy-direction-truth-v{version}-oracle-28.pine").read_text(encoding="utf-8")
    result=pine_compat.compile_script(source).run(BARS)
    assert [p["values"] for p in result["plots"]]==[[v]*6 for v in [1,1,0,0,0,0,1,1,0,1,0,0,1,1,1,1,1,1]]


@pytest.mark.parametrize("variant,positions,orders,trades",[
    ("orders",[0,1,1,1,-1,0],3,2),
    ("when",[0,0,1,1,1,0],2,1),
])
def test_direction_order_conditions_preserve_identity(variant,positions,orders,trades):
    source=(EVIDENCE/f"legacy-direction-{variant}-v3-oracle-28.pine").read_text(encoding="utf-8")
    result=pine_compat.compile_script(source).run(BARS)
    assert result["plots"][0]["values"]==positions
    assert len(result["strategy"]["orders"])==orders
    assert len(result["strategy"]["trades"])==trades

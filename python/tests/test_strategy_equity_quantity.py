import json
from pathlib import Path

import pine_compat
import pytest

CASES = json.loads((Path(__file__).resolve().parents[2] / "docs/evidence/strategy-library-20261010/equity-quantity-binding-cases-30.json").read_text(encoding="utf-8"))
BARS = [dict(time=1700000000000+i*60000, open=p, high=p, low=p, close=p, volume=1.) for i,p in enumerate([100.,100.,300.,300.])]


@pytest.mark.parametrize("case", CASES, ids=[r["name"] for r in CASES])
def test_equity_quantity_boundaries(case):
    program = pine_compat.compile_script(case["source"])
    if "error" in case:
        with pytest.raises(ValueError, match=case["error"]):
            program.run(BARS)
        return
    result = program.run(BARS)
    assert [p["values"] for p in result["plots"]] == case["values"]
    assert len(result["strategy"]["orders"]) == case["orders"]

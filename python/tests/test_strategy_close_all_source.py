import json
from pathlib import Path

import pine_compat
import pytest

EVIDENCE = Path(__file__).resolve().parents[2] / "docs/evidence/strategy-library-20261010"
RECORDS = json.loads((EVIDENCE / "close-all-oracles-29.json").read_text(encoding="utf-8"))
BARS = [dict(time=1700000000000+i*60000, open=10., high=10., low=10., close=10., volume=1.) for i in range(6)]


@pytest.mark.parametrize("record", RECORDS, ids=[r["stem"] for r in RECORDS])
def test_close_all_source_overloads_and_eager_arguments(record):
    source = (EVIDENCE / (record["stem"] + ".pine")).read_text(encoding="utf-8")
    if record["native_status"] == "analysis_rejected":
        with pytest.raises(ValueError):
            pine_compat.compile_script(source)
        return
    result = pine_compat.compile_script(source).run(BARS)
    assert [plot["values"][-1] for plot in result["plots"]] == record["native_values"]
    if record["expected_exit_bar"] is not None:
        assert result["plots"][0]["values"] == [0,1,1,1,0,0]
        assert len(result["strategy"]["trades"]) == 1
        assert len(result["strategy"]["alerts"]) == record["expected_alert_count"]
    else:
        assert result["plots"][0]["values"] == [record["native_values"][0]]*6
        assert not result["strategy"]["trades"]

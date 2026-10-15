from pathlib import Path

import pine_compat
import pytest

EVIDENCE = Path(__file__).resolve().parents[2] / "docs/evidence/strategy-library-20261010"
BARS = [dict(time=0, open=1.0, high=1.0, low=1.0, close=1.0, volume=1.0)]


@pytest.mark.parametrize("version", [4, 5, 6])
def test_native_template_source_uses_explicit_clock_and_forward_plot_bindings(version):
    source = (EVIDENCE / f"alert-template-v{version}-oracle-26.pine").read_text(encoding="utf-8")
    program = pine_compat.compile_script(source)
    with pytest.raises(ValueError, match="explicit execution timestamp"):
        program.run(BARS)
    result = program.run(BARS, execution_times=[1700000000123])
    assert result["diagnostics"] == []
    assert result["alerts"][0]["message"] == "P0=1|P19=20|NOW=2023-11-14T22:13:20Z"
    assert [p["values"] for p in result["plots"]] == [[i] for i in range(1, 21)]


def test_template_clock_stays_lazy_and_alert_time_is_utc_literal_z():
    source = '//@version=6\nindicator("lazy")\nalertcondition(false,"no clock","{{timenow}}")\nalertcondition(true,"bar time","{{time}}")\nplot(close)\n'
    result = pine_compat.run_script(source, BARS)
    assert result["diagnostics"] == []
    assert [a["message"] for a in result["alerts"]] == ["1970-01-01T00:00:00Z"]

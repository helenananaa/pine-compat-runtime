from pathlib import Path

import pine_compat
import pytest

FIXTURES = Path(__file__).resolve().parents[2] / "tests/fixtures/runtime"
CASES = [("bool_control_defaults_v6.pine", 4, 1), ("bool_control_functions_history_v6.pine", 6, 3)]
BARS = [dict(time=i * 60000, open=10., high=10., low=10., close=10., volume=1.) for i in range(3)]


def check(result, version, count, bool_count, other_count):
    expected = [1 if version == 6 else 0] * bool_count + [1] * other_count
    assert [plot["values"] for plot in result["plots"]] == [[value] * count for value in expected]


@pytest.mark.parametrize("filename,bool_count,other_count", CASES)
@pytest.mark.parametrize("version", [5, 6])
def test_native_boolean_control_defaults_in_batch(filename, bool_count, other_count, version):
    source = (FIXTURES / filename).read_text(encoding="utf-8").replace("version=6", f"version={version}")
    check(pine_compat.compile_script(source).run(BARS), version, 3, bool_count, other_count)


@pytest.mark.parametrize("filename,bool_count,other_count", CASES)
@pytest.mark.parametrize("version", [5, 6])
def test_native_boolean_control_defaults_survive_forming_replacement_and_confirmation(filename, bool_count, other_count, version):
    source = (FIXTURES / filename).read_text(encoding="utf-8").replace("version=6", f"version={version}")
    session = pine_compat.create_realtime_session(source)
    check(session.seed(BARS[:1]), version, 1, bool_count, other_count)
    for close in [10., 20.]:
        bar = {**BARS[1], "open": close, "high": close, "low": close, "close": close}
        check(session.update_forming(bar), version, 2, bool_count, other_count)
    check(session.update_confirmed(bar), version, 2, bool_count, other_count)
    check(session.update_forming(BARS[2]), version, 3, bool_count, other_count)

from pathlib import Path

import pine_compat
import pytest


@pytest.mark.parametrize("version", [5, 6])
def test_udf_push_native_values_and_realtime_replica(version):
    root = Path(__file__).resolve().parents[2]
    source = (root / "tests/fixtures/runtime/udf_array_push.pine").read_text()
    source = source.replace("version=6", f"version={version}")
    bars = [dict(time=i * 60000, open=10., high=10., low=10., close=10., volume=1.) for i in range(8)]
    program = pine_compat.compile_script(source)
    result = program.run(bars)
    assert [p["values"][-1] for p in result["plots"]] == [1, 3, 19, 9, 10, 5]
    session = program.realtime_session()
    session.seed(bars[:2])
    replica = session.replica()
    for bar in bars[2:]:
        replica.apply(session.apply_forming(bar))
        replica.apply(session.apply_forming(bar))
        replica.apply(session.apply_confirmed(bar))
    assert replica.result() == session.result() == result

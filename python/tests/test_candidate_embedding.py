"""Candidate embedding path: compile through owned results on the shipped Python API."""
from __future__ import annotations

import copy

import pine_compat
import pytest


SOURCE = """//@version=6
indicator("candidate embed")
factor = input.float(2.0, "Scale")
plot(close * factor, "scaled")
plot(timenow, "clock")
"""

BARS = [
    dict(time=0, open=10.0, high=10.0, low=10.0, close=10.0, volume=1.0),
    dict(time=60_000, open=11.0, high=11.0, low=11.0, close=11.0, volume=1.0),
]


def test_package_identity_is_the_coordinated_prerelease():
    assert pine_compat.__version__ == "0.3.0-rc.1"
    assert pine_compat.RUNTIME_SCHEMA_VERSION == 8
    assert pine_compat.ANALYSIS_SCHEMA_VERSION == 5


def test_compile_requirements_historical_realtime_error_and_owned_result():
    program = pine_compat.compile_script(SOURCE)
    requirements = program.host_requirements()
    assert requirements["schemaVersion"] == 1
    assert requirements["execution"]["clock"] == "explicitTimestampWhenEvaluated"
    clocks = [1_000, 2_000]
    historical = program.run(BARS, execution_times=clocks)
    assert historical["schemaVersion"] == 8
    assert historical["plots"][0]["values"] == [20.0, 22.0]
    assert historical["plots"][1]["values"] == [1_000, 2_000]
    owned = copy.deepcopy(historical)
    historical["plots"][0]["values"][0] = 999.0
    assert owned["plots"][0]["values"][0] == 20.0

    session = program.realtime_session()
    seeded = session.seed(BARS, execution_times=clocks)
    assert seeded["plots"][0]["values"] == owned["plots"][0]["values"]
    before = session.result()
    with pytest.raises(ValueError, match="execution timestamp"):
        session.update_forming(dict(time=120_000, open=12.0, high=12.0, low=12.0, close=12.0, volume=1.0))
    assert session.result() == before
    forming = dict(time=120_000, open=12.0, high=12.0, low=12.0, close=12.0, volume=1.0)
    preview = session.update_forming(forming, execution_time=3_000)
    assert preview["plots"][0]["values"][-1] == 24.0
    preview["plots"][0]["values"][-1] = 0.0
    confirmed = session.update_confirmed(
        dict(time=120_000, open=12.0, high=13.0, low=12.0, close=13.0, volume=1.0),
        execution_time=4_000,
    )
    assert confirmed["plots"][0]["values"] == [20.0, 22.0, 26.0]
    assert confirmed["plots"][1]["values"] == [1_000, 2_000, 4_000]
    assert session.confirmed_bars == 3


def test_last_bar_resource_limit_is_atomic_on_the_python_session():
    source = """//@version=6
indicator("limit")
if bar_index >= 2
    while true
        x = close
plot(close)
"""
    session = pine_compat.create_realtime_session(source)
    seeded = session.seed(
        [
            dict(time=60_000, open=1.0, high=1.0, low=1.0, close=1.0, volume=1.0),
            dict(time=120_000, open=2.0, high=2.0, low=2.0, close=2.0, volume=1.0),
        ]
    )
    owned = copy.deepcopy(seeded)
    with pytest.raises(ValueError, match="exceeded maximum iteration"):
        session.update_forming(dict(time=180_000, open=3.0, high=3.0, low=3.0, close=3.0, volume=1.0))
    assert session.confirmed_result()["plots"][0]["values"] == owned["plots"][0]["values"]
    assert session.confirmed_bars == 2
    owned["plots"][0]["values"][0] = 999.0
    assert session.confirmed_result()["plots"][0]["values"][0] == 1.0

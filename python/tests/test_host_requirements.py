import json
from pathlib import Path

import pine_compat
import pytest


ROOT = Path(__file__).resolve().parents[2]


def test_compiled_host_requirements_match_shared_contract_without_data():
    source = (ROOT / "tests/fixtures/host_requirements/strategy.pine").read_text()
    expected = json.loads((ROOT / "tests/snapshots/host_requirements.json").read_text())
    program = pine_compat.compile_script(source)
    assert program.host_requirements() == expected
    assert program.host_requirements() == expected


def test_inventory_does_not_require_a_clock_for_an_unreached_read():
    program = pine_compat.compile_script(
        '//@version=6\nindicator("conditional")\nplot(false ? timenow : close)\n'
    )
    assert program.host_requirements()["execution"]["clock"] == "explicitTimestampWhenEvaluated"
    result = program.run([dict(time=0, open=1, high=1, low=1, close=1, volume=1)])
    assert result["plots"][0]["values"] == [1]


def test_unsupported_source_cannot_acquire_an_executable_requirements_report():
    with pytest.raises(ValueError):
        pine_compat.compile_script('//@version=6\nindicator("bad")\nplot(request.financial("X","Y","FQ"))\n')

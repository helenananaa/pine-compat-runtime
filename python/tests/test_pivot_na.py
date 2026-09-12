import json
from pathlib import Path
import pine_compat
import pytest

ROOT=Path(__file__).resolve().parents[2]

@pytest.mark.parametrize("version",[5,6])
def test_native_pivot_na_values_and_realtime_match(version):
    source=(ROOT/'tests/fixtures/runtime/pivot_na_boundaries.pine').read_text().replace('version=6',f'version={version}')
    expected=json.loads((ROOT/'tests/fixtures/pivot_na_native_values.json').read_text())
    bars=[dict(time=i*60000,open=10.,high=10.,low=10.,close=10.,volume=1.) for i in range(109)]
    program=pine_compat.compile_script(source)
    result=program.run(bars)
    for plot in result['plots']: assert plot['values']==expected[plot['title']]
    session=program.realtime_session()
    session.seed(bars[:4])
    replica=session.replica()
    for bar in bars[4:]:
        replica.apply(session.apply_forming(bar))
        replica.apply(session.apply_forming(bar))
        replica.apply(session.apply_confirmed(bar))
    assert replica.result()==session.result()==result

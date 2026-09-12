from pathlib import Path

import pine_compat
import pytest

ROOT = Path(__file__).resolve().parents[2]
BARS = [dict(time=i*60000, open=10.+i, high=10.+i, low=10.+i, close=10.+i, volume=1.) for i in range(4)]


@pytest.mark.parametrize('version', [5, 6])
def test_native_udt_identity_source_and_streaming(version):
    source = (ROOT/'tests/fixtures/runtime/udt_reference_identity.pine').read_text().replace('version=6', f'version={version}')
    program = pine_compat.compile_script(source)
    expected = program.run(BARS)
    assert expected['plots'][0]['values'] == [11., 12., 13., 14.]
    assert expected['plots'][3]['values'] == [13., 14., 15., 16.]
    assert expected['plots'][4]['values'] == [14., 15., 16., 17.]
    assert expected['plots'][5]['values'] == [None, 1., 2., 3.]
    session = program.realtime_session()
    session.seed(BARS[:1])
    replica = session.replica()
    for bar in BARS[1:]:
        replica.apply(session.apply_forming(bar))
        replica.apply(session.apply_forming(bar))
        replica.apply(session.apply_confirmed(bar))
    assert replica.result() == session.result() == expected


def test_udt_field_varip_preserves_field_policy_in_replica():
    source = (ROOT/'tests/fixtures/runtime/udt_field_varip.pine').read_text()
    session = pine_compat.compile_script(source).realtime_session()
    session.seed(BARS[:1])
    replica = session.replica()
    for tick in range(1, 4):
        replica.apply(session.apply_forming(BARS[1]))
        result = session.result()
        assert replica.result() == result
        assert [plot['values'][1] for plot in result['plots']] == [2., 1.+tick, 2., 1.+tick, tick, tick, 1.]

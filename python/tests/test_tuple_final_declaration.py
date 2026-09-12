from pathlib import Path

import pine_compat
import pytest

SOURCE = (Path(__file__).resolve().parents[2] / 'tests/fixtures/runtime/tuple_final_declaration.pine').read_text()


@pytest.mark.parametrize('version', [5, 6])
def test_tuple_final_declaration_batch_and_stream(version):
    program = pine_compat.compile_script(SOURCE.replace('version=6', f'version={version}'))
    bars = [dict(time=i * 60000, open=10.+i, high=10.+i, low=10.+i, close=10.+i, volume=1.) for i in range(8)]
    expected = program.run(bars)
    assert expected['plots'][4]['values'] == [10.+i for i in range(8)]
    assert expected['plots'][8]['values'] == list(range(1, 9))
    assert expected['plots'][9]['values'] == list(range(10, 90, 10))
    session = program.realtime_session()
    session.seed(bars[:1])
    replica = session.replica()
    for bar in bars[1:]:
        replica.apply(session.apply_forming(bar))
        replica.apply(session.apply_forming(bar))
        replica.apply(session.apply_confirmed(bar))
    assert replica.result() == session.result() == expected

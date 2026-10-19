import hashlib
import json
from pathlib import Path

import pine_compat
import pytest

ROOT = Path(__file__).resolve().parents[2]
CASES = json.loads((ROOT / 'docs/evidence/strategy-library-20261010/point-reference-oracles-41.json').read_text(encoding='utf8'))['records']
BARS = [dict(time=i * 60000, open=10., high=10., low=10., close=10., volume=1.) for i in range(3)]


@pytest.mark.parametrize('case', CASES, ids=lambda case: case['name'])
def test_native_point_references_across_python_boundaries(case):
    raw = (ROOT / case['source']).read_bytes()
    assert hashlib.sha256(raw).hexdigest() == case['source_sha256']
    text = raw.decode('utf8')
    report = pine_compat.analyze_script(text)
    if case['native_status'] == 'analysis_rejected':
        assert not report['executable']
        with pytest.raises(ValueError):
            pine_compat.compile_script(text)
        return
    assert report['executable'], report['diagnostics']
    prepared = pine_compat.compile_script(text)
    if case['native_status'] == 'runtime_error':
        with pytest.raises(ValueError, match='E_CHART_POINT_NA_FIELD'):
            prepared.run(BARS)
        with pytest.raises(ValueError, match='E_CHART_POINT_NA_FIELD'):
            pine_compat.create_realtime_session(text).seed(BARS[:2])
        return

    def check(result, tick=None):
        actual = [plot['values'][-1] for plot in result['plots']]
        if case['execution_kind'] == 'intrabar':
            expected = [tick if value == 'tick' else value for value in case['intrabar_expected']]
        elif case.get('native_marker_suffix'):
            expected = case['native_marker_suffix']
            actual = actual[-len(expected):]
        else:
            expected = case['native_values']
        assert actual == expected, case['name']

    if case['execution_kind'] == 'static':
        check(prepared.run(BARS))
    session = pine_compat.create_realtime_session(text)
    session.seed(BARS[:2])
    for tick in range(1, 4):
        check(session.update_forming(BARS[2]), tick)
    check(session.update_confirmed(BARS[2]), 4)

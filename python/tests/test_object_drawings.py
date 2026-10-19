import hashlib
import json
from pathlib import Path

import pine_compat
import pytest

ROOT = Path(__file__).resolve().parents[2]
CASES = json.loads((ROOT / 'docs/evidence/strategy-library-20261010/object-drawing-oracles-40.json').read_text(encoding='utf8'))['records']
BARS = [dict(time=i * 60000, open=10., high=10., low=10., close=10., volume=1.) for i in range(3)]


@pytest.mark.parametrize('case', CASES, ids=lambda case: case['name'])
def test_native_object_drawings_across_python_execution_boundaries(case):
    raw = (ROOT / case['source']).read_bytes()
    assert hashlib.sha256(raw).hexdigest() == case['source_sha256']
    text = raw.decode('utf8')
    report = pine_compat.analyze_script(text)
    if case['native_status'] == 'analysis_rejected':
        assert not report['executable'], case['name']
        with pytest.raises(ValueError):
            pine_compat.compile_script(text)
        return
    assert report['executable'], report['diagnostics']
    if case['native_status'] == 'runtime_error':
        with pytest.raises(ValueError, match='invalid extend value'):
            pine_compat.compile_script(text).run(BARS)
        with pytest.raises(ValueError, match='invalid extend value'):
            pine_compat.create_realtime_session(text).seed(BARS[:2])
        return

    def check(result):
        assert [plot['values'][-1] for plot in result['plots']] == case['native_values']

    check(pine_compat.compile_script(text).run(BARS))
    session = pine_compat.create_realtime_session(text)
    session.seed(BARS[:2])
    check(session.update_forming(BARS[2]))
    check(session.update_confirmed(BARS[2]))

import hashlib
import json
from pathlib import Path

import pine_compat
import pytest

ROOT = Path(__file__).resolve().parents[2]
CASES = json.loads((ROOT / 'docs/evidence/strategy-library-20261010/array-persistence-oracles-42.json').read_text(encoding='utf8'))['records']
BARS = [dict(time=i * 60000, open=10., high=10., low=10., close=10., volume=1.) for i in range(3)]


def forming_values(case, tick, history):
    profile = case.get('intrabar_profile')
    if profile == 'opening':
        return [tick, tick, 0, 1, 0, -1 if tick == 1 else 0]
    if profile == 'late_history':
        return [tick, tick, 0, int(tick >= 2)]
    if profile == 'forming_reassign':
        return [tick, 2 if tick == 1 else 1, tick + 1, int(tick >= 2)]
    if profile == 'transient':
        return [1, history + tick, history + tick]
    return case['forming_expected']


@pytest.mark.parametrize('case', CASES, ids=lambda case: case['name'])
def test_native_array_bindings_across_python_boundaries(case):
    raw = (ROOT / case['source']).read_bytes()
    assert hashlib.sha256(raw).hexdigest() == case['source_sha256']
    source = raw.decode('utf8')
    report = pine_compat.analyze_script(source)
    if case['native_status'] == 'analysis_rejected':
        assert not report['executable']
        with pytest.raises(ValueError):
            pine_compat.compile_script(source)
        return
    assert report['executable'], report['diagnostics']
    prepared = pine_compat.compile_script(source)
    if case['native_status'] == 'runtime_error':
        with pytest.raises(ValueError, match=case['runtime_error_pattern']):
            prepared.run(BARS)
        with pytest.raises(ValueError, match=case['runtime_error_pattern']):
            pine_compat.create_realtime_session(source).seed(BARS[:2])
        return
    values = lambda result: [plot['values'][-1] for plot in result['plots']]
    assert values(prepared.run(BARS)) == case['historical_expected']
    for history in ([1, 2] if case['execution_kind'] == 'intrabar' else [2]):
        session = pine_compat.create_realtime_session(source)
        session.seed(BARS[:history])
        for tick in range(1, 4):
            assert values(session.update_forming(BARS[history])) == forming_values(case, tick, history)
        assert values(session.update_confirmed(BARS[history])) == forming_values(case, 4, history)

"""Native-backed direct chart-context reference captures on the shipped API."""
import hashlib
import json
from pathlib import Path

import pine_compat
import pytest

ROOT = Path(__file__).resolve().parents[2]
CASES = json.loads((ROOT / 'docs/evidence/strategy-library-20261010/request-reference-oracles-43.json').read_text(encoding='utf8'))['records']
BARS = [dict(time=i * 60000, open=10., high=10., low=10., close=10., volume=1.) for i in range(3)]


@pytest.mark.parametrize('case', CASES, ids=lambda case: case['name'])
def test_native_reference_capture_historical_and_realtime(case):
    raw = (ROOT / case['source']).read_bytes()
    assert hashlib.sha256(raw).hexdigest() == case['source_sha256']
    if case['native_status'] == 'analysis_rejected':
        with pytest.raises(ValueError):
            pine_compat.compile_script(raw.decode('utf8'))
        return
    program = pine_compat.compile_script(raw.decode('utf8'))
    result = program.run(BARS)
    assert [p['values'][-1] for p in result['plots']] == case['historical_expected']
    session = program.realtime_session()
    session.seed(BARS[:2])
    for _ in range(3):
        result = session.update_forming(BARS[2])
        assert [p['values'][-1] for p in result['plots']] == case['forming_expected']
    result = session.update_confirmed(BARS[2])
    assert [p['values'][-1] for p in result['plots']] == case['historical_expected']

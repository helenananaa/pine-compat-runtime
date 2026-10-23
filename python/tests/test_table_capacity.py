import hashlib
import json
from pathlib import Path

import pine_compat
import pytest

ROOT = Path(__file__).resolve().parents[2]
CASES = json.loads((ROOT / 'docs/evidence/strategy-library-20261010/table-capacity-oracles-36.json').read_text(encoding='utf8'))['records']
# The dense source is checked in Rust; serializing its full mutation history is a distinct workload.
SPARSE = [case for case in CASES if not case['name'].startswith('original-') and 'populated' not in case['name']]
BARS = [dict(time=i * 60000, open=10., high=10., low=10., close=10., volume=1.) for i in range(3)]


@pytest.mark.parametrize('case', SPARSE, ids=lambda case: case['name'])
def test_native_sparse_table_capacity_and_lifetime_cross_python_boundaries(case):
    raw = (ROOT / case['source']).read_bytes()
    assert hashlib.sha256(raw).hexdigest() == case['source_sha256']
    text = raw.decode('utf8')
    report = pine_compat.analyze_script(text)
    assert report['executable'], report['diagnostics']
    prepared = pine_compat.compile_script(text)
    if case['native_status'] == 'analysis_or_runtime_rejected':
        with pytest.raises(ValueError, match='table cell count cannot exceed 10000'):
            prepared.run(BARS)
        return

    def check(result):
        assert [plot['values'][-1] for plot in result['plots']] == case['native_values']
        assert result['tables']

    check(prepared.run(BARS))
    session = pine_compat.create_realtime_session(text)
    session.seed(BARS[:2])
    check(session.update_forming(BARS[2]))
    check(session.update_confirmed(BARS[2]))

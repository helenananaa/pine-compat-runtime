import hashlib
import json
from pathlib import Path

import pine_compat
import pytest

ROOT = Path(__file__).resolve().parents[2]
CASES = json.loads((ROOT / 'docs/evidence/strategy-library-20261010/function-body-positive-migration-oracles-46.json').read_text(encoding='utf8'))['records']
BARS = [dict(time=i * 60000, open=10., high=10., low=10., close=10., volume=1.) for i in range(12)]
BODY_CASES = []
for manifest in ['function-body-oracles-46.json', 'function-body-shape-oracles-46.json', 'function-body-supplementary-oracles-46.json', 'function-body-method-forwarding-oracles-46.json']:
    BODY_CASES.extend(case for case in json.loads((ROOT / 'docs/evidence/strategy-library-20261010' / manifest).read_text(encoding='utf8'))['records'] if case.get('qualification_scope') != 'confirmed_remaining_parser_gap')


@pytest.mark.parametrize('case', BODY_CASES, ids=lambda case: case['name'])
def test_unused_declaration_admission_through_python(case):
    raw = (ROOT / case['source']).read_bytes()
    assert hashlib.sha256(raw).hexdigest() == case['source_sha256']
    source = raw.decode('utf8')
    libraries = {}
    if case.get('kind') == 'exported':
        libraries['local/Unused/1'] = source
        source = f'//@version={case["version"]}\nimport local/Unused/1 as c\nindicator("unused library")\nplot(1)\n'
    report = pine_compat.analyze_script(source, library_sources=libraries)
    if case['native_status'] == 'analysis_rejected':
        assert not report['executable'], case['name']
        with pytest.raises(ValueError):
            pine_compat.compile_script(source, library_sources=libraries)
    else:
        assert report['executable'], report['diagnostics']
        result = pine_compat.compile_script(source, library_sources=libraries).run(BARS)
        assert [plot['values'][-1] for plot in result['plots']] == case['native_values']


@pytest.mark.parametrize('case', CASES, ids=lambda case: case['name'])
def test_native_body_checks_and_matrix_dimensions_through_python(case):
    raw = (ROOT / case['source']).read_bytes()
    assert hashlib.sha256(raw).hexdigest() == case['source_sha256']
    source = raw.decode('utf8')
    report = pine_compat.analyze_script(source)
    if case['native_status'] == 'analysis_rejected':
        assert not report['executable'], case['name']
        with pytest.raises(ValueError):
            pine_compat.compile_script(source)
        return
    assert report['executable'], report['diagnostics']

    def check(result):
        assert [plot['values'][-1] for plot in result['plots']] == case['native_values']

    check(pine_compat.compile_script(source).run(BARS))
    session = pine_compat.create_realtime_session(source)
    session.seed(BARS[:11])
    check(session.update_forming(BARS[11]))
    check(session.update_forming(BARS[11]))
    check(session.update_confirmed(BARS[11]))

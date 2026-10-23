"""Native collection parameter controls through installed Python entry points."""
import hashlib
import json
from pathlib import Path

import pine_compat
import pytest

ROOT = Path(__file__).resolve().parents[2]
CASES = json.loads((ROOT / 'docs/evidence/strategy-library-20261010/collection-parameter-oracles-45.json').read_text(encoding='utf8'))['records']
SUPPLEMENT = json.loads((ROOT / 'docs/evidence/strategy-library-20261010/collection-import-supplementary-45.json').read_text(encoding='utf8'))['records']
CASES += [case for case in SUPPLEMENT if case['native_status'] == 'compiled_and_executed']
BARS = [dict(time=i*60000, open=10., high=10., low=10., close=10., volume=1.) for i in range(3)]


@pytest.mark.parametrize('case', CASES, ids=lambda case: case['name'])
def test_native_collection_parameter_templates_and_references(case):
    raw = (ROOT / case['source']).read_bytes()
    assert hashlib.sha256(raw).hexdigest() == case['source_sha256']
    if case['native_status'] == 'analysis_rejected':
        with pytest.raises(ValueError):
            pine_compat.compile_script(raw.decode('utf8'))
        return
    program = pine_compat.compile_script(raw.decode('utf8'))
    values = lambda result: [p['values'][-1] for p in result['plots']]
    expected = case['local_three_bar_values']
    assert values(program.run(BARS)) == expected
    assert values(program.historical_session(BARS).advance(3)) == expected
    session = program.realtime_session()
    session.seed(BARS[:2])
    for _ in range(3):
        assert values(session.update_forming(BARS[2])) == expected
    assert values(session.update_confirmed(BARS[2])) == expected


@pytest.mark.parametrize('case', [case for case in SUPPLEMENT if case['native_status'] == 'compiled_library'], ids=lambda case: case['name'])
def test_native_valid_library_mutates_supplied_collection_parameters(case):
    raw = (ROOT / case['source']).read_bytes()
    assert hashlib.sha256(raw).hexdigest() == case['source_sha256']
    version = case['version']
    if 'mutable-array' in case['name']:
        body = 'var values=array.new<float>(1,10)\nplot(c.step(values))\nplot(values.size())'
        expected = [[2,3,4]]*2
    else:
        body = 'var m=map.new<string,int>()\nvar x=matrix.new<int>(1,1,1)\nif barstate.isfirst\n    m.put("x",1)\nplot(c.bump(m))\nplot(m.get("x"))\nplot(c.step(x))\nplot(x.get(0,0))'
        expected = [[1.5,2,2.5]]*4
    source = f'//@version={version}\nimport local/Contract/1 as c\nindicator("import contract")\n{body}\n'
    program = pine_compat.compile_script(source, library_sources={'local/Contract/1':raw.decode('utf8')})
    values = lambda result: [p['values'] for p in result['plots']]
    assert values(program.run(BARS)) == expected
    assert values(program.historical_session(BARS).advance(3)) == expected
    session = program.realtime_session()
    session.seed(BARS[:2])
    for _ in range(3):
        assert values(session.update_forming(BARS[2])) == expected
    assert values(session.update_confirmed(BARS[2])) == expected

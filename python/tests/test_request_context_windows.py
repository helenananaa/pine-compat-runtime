"""Whole-source Native window and qualifier controls through the shipped API."""
import hashlib
import json
from pathlib import Path
import pine_compat
import pytest

ROOT = Path(__file__).resolve().parents[2]
CASES = json.loads((ROOT / 'docs/evidence/strategy-library-20261010/request-window-oracles-44.json').read_text(encoding='utf8'))['records']

def bar(time):
    return dict(time=time, open=10., high=10., low=10., close=10., volume=1.)

BARS = [bar(1502928000000+i*86400000) for i in range(3350)]
REQUESTS = {'BINANCE:BTCUSDT:W': [bar(1502668800000+i*604800000) for i in range(479)], 'NASDAQ:AAPL:D': BARS}
CONTEXT = dict(chart_symbol='BINANCE:BTCUSDT', chart_timeframe='D', request_bars=REQUESTS)

@pytest.mark.parametrize('case', CASES, ids=lambda case: case['name'])
def test_native_initial_context_window(case):
    raw = (ROOT / case['source']).read_bytes()
    assert hashlib.sha256(raw).hexdigest() == case['source_sha256']
    if case['native_status'] == 'analysis_rejected':
        with pytest.raises(ValueError):
            pine_compat.compile_script(raw.decode('utf8'))
        return
    program = pine_compat.compile_script(raw.decode('utf8'))
    session = program.realtime_session(**CONTEXT)
    if case['native_status'] == 'runtime_error':
        with pytest.raises(ValueError, match=case['runtime_error_message']):
            program.run(BARS, **CONTEXT)
        with pytest.raises(ValueError, match=case['runtime_error_message']):
            session.seed(BARS)
        with pytest.raises(ValueError, match=case['runtime_error_message']):
            program.historical_session(BARS, **CONTEXT).advance(len(BARS))
        return
    for result in [program.run(BARS, **CONTEXT), session.seed(BARS), program.historical_session(BARS, **CONTEXT).advance(len(BARS))]:
        assert [p['values'][-1] for p in result['plots']] == case['native_values']
        assert len(result['plots'][0]['values']) == case['native_values'][-1]


def test_native_request_window_origin_extends_in_realtime():
    cases = json.loads((ROOT / 'docs/evidence/strategy-library-20261010/request-window-live-origin-oracles-44.json').read_text(encoding='utf8'))['records']
    initial, continued = cases
    raw = (ROOT / initial['source']).read_bytes()
    assert hashlib.sha256(raw).hexdigest() == initial['source_sha256']
    program = pine_compat.compile_script(raw.decode('utf8'))
    session = program.realtime_session(chart_symbol='BINANCE:BTCUSDT', chart_timeframe='1')
    time = initial['native_values'][3]
    values = lambda result: [p['values'][-1] for p in result['plots']]
    assert values(session.seed([bar(time-60000), bar(time)])) == initial['native_values']
    for _ in range(3):
        assert values(session.update_forming(bar(time+60000))) == [2, 2, 3, time+60000]
    session.update_confirmed(bar(time+60000))
    for _ in range(3):
        assert values(session.update_forming(bar(time+120000))) == continued['native_values']
    assert values(session.update_confirmed(bar(time+120000))) == continued['native_values']


NAME_CASES = json.loads((ROOT / 'docs/evidence/strategy-library-20261010/simple-declaration-name-oracles-44.json').read_text(encoding='utf8'))['records']


@pytest.mark.parametrize('case', NAME_CASES, ids=lambda case: case['name'])
def test_simple_type_name_is_not_consumed_as_a_qualifier(case):
    raw = (ROOT / case['source']).read_bytes()
    assert hashlib.sha256(raw).hexdigest() == case['source_sha256']
    program = pine_compat.compile_script(raw.decode('utf8'))
    bars = [bar(i*60000) for i in range(3)]
    expected = case['native_values'] * 3
    assert program.run(bars)['plots'][0]['values'] == expected
    assert program.historical_session(bars).advance(3)['plots'][0]['values'] == expected
    session = program.realtime_session()
    session.seed(bars[:2])
    assert session.update_forming(bars[2])['plots'][0]['values'] == expected
    assert session.update_confirmed(bars[2])['plots'][0]['values'] == expected

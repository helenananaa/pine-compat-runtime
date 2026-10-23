"""Typed provider misses cannot be forged by runtime.error text."""
from copy import deepcopy
import json

import pine_compat
import pytest
from test_external_stream import data


KEY='{"chart":"renko","style":"Traditional","param":1,"symbol":"B"}'


def session(expression, requests=None):
    bars,_,groups=data('historical')
    program=pine_compat.compile_script('//@version=6\nstrategy("typed request failure",calc_on_every_history_tick=true)\n'+expression)
    value=program.external_session(scheduled=True,chart_symbol='A',chart_timeframe='1',
        request_bars=requests if requests is not None else {'C:1':[dict(time=0,open=10,high=10,low=10,close=10,volume=1)]})
    value.begin_bar(bars[0],bar_state='historical')
    return value,deepcopy(groups[0][0])


@pytest.mark.parametrize('expression,expected',[
    ('plot(request.security(ticker.renko("B","Traditional",1),"1",close))',(KEY,'1')),
    ('plot(request.security("missing","1",close))',('missing','1')),
    ('plot(request.security("C","1",request.security("missing","1",close)))',('missing','1')),
    ('plot(request.security("C","1",request.security('+json.dumps(KEY)+',"1",close)))',(KEY,'1')),
])
def test_failed_provider_context_is_typed_including_nested_request_forks(expression,expected):
    value,observation=session(expression)
    assert value.failed_request_context() is None
    with pytest.raises(ValueError,match='missing request data'):
        value.observe(observation)
    assert value.failed_request_context()==expected
    with pytest.raises(ValueError,match='E_RUNTIME_POISONED'):
        value.snapshot()
    assert value.failed_request_context()==expected


@pytest.mark.parametrize('prefix',['','plot(request.security("C","1",close))\n'])
def test_user_error_with_identical_text_has_no_provider_context(prefix):
    message=f'missing request data for symbol `{KEY}` timeframe `1`'
    value,observation=session(prefix+'runtime.error('+json.dumps(message)+')')
    with pytest.raises(ValueError,match='missing request data'):
        value.observe(observation)
    assert value.failed_request_context() is None


def test_validation_failure_does_not_create_a_typed_provider_miss():
    value,observation=session('plot(request.security(ticker.renko("B","Traditional",1),"1",close))')
    bad=deepcopy(observation)
    bad['evaluation']['event_time_ms']=-1
    with pytest.raises(ValueError):value.observe(bad)
    assert value.failed_request_context() is None
    with pytest.raises(ValueError,match='missing request data'):value.observe(observation)
    assert value.failed_request_context()==(KEY,'1')


def test_known_empty_event_stream_is_not_an_unknown_provider():
    value,observation=session('plot(request.security(ticker.renko("B","Traditional",1),"1",close))',
        {KEY+':1':[]})
    value.observe(observation)
    assert value.failed_request_context() is None

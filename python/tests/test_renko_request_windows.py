"""Event row ordinals must survive bounded windows and causal request feeds."""
from copy import deepcopy

import pine_compat
import pytest

from test_external_stream import data


SYMBOL = '{"chart":"renko","style":"Traditional","param":1,"symbol":"B"}'
KEY = SYMBOL + ':1'


def row(time, value):
    return dict(time=time, open=value, high=value, low=value, close=value, volume=1)


def source(mode='indicator'):
    declaration = 'indicator("brick window")' if mode == 'indicator' else 'strategy("brick window",calc_on_every_history_tick=true)'
    return ('//@version=6\n' + declaration + '\n'
            'r=ticker.renko("B","Traditional",1)\n'
            'plot(request.security(r,"1",ta.cum(close),calc_bars_count=1))')


@pytest.mark.parametrize('expression,values', [
    ('ta.cum(close)', [None,3,3,3]),
    ('close[1]', [None,None,None,None]),
    ('ta.cum(close)+(barstate.islast?0.0:0.0)', [None,3,3,3]),
])
def test_native_bounded_window_keeps_the_exact_brick_ordinal_across_chart_bars(expression, values):
    bricks = [row(0, 1), row(60000, 2), row(60000, 3)]
    output = pine_compat.compile_script(source().replace('ta.cum(close)', expression)).run(
        [row(i*60000, 10) for i in range(4)], request_bars={KEY: bricks},
        chart_symbol='A', chart_timeframe='1')
    assert not output['diagnostics']
    assert output['plots'][0]['values'] == values


def test_renko_confirmed_feed_appends_distinct_same_time_rows_without_script_reexecution():
    program = source().replace(',calc_bars_count=1', '')
    session = pine_compat.create_realtime_session(program, chart_symbol='A', chart_timeframe='1',
        request_bars={KEY: [row(0, 1)]})
    session.seed([row(0, 10)])
    before = deepcopy(session.result())
    updates = [dict(symbol=SYMBOL, timeframe='1', kind='confirmed', bar=row(60000, value)) for value in (2, 3)]
    session.apply_request_feed_updates(updates)
    assert session.result() == before
    session.update_confirmed(row(60000, 10), execution_time=119999)
    assert session.result()['plots'][0]['values'] == [1, 6]
    with pytest.raises(ValueError):
        session.apply_request_feed_updates([dict(symbol=SYMBOL, timeframe='1', kind='confirmed', bar=row(0, 999))])


def test_external_known_event_times_anchor_an_ordinal_without_revealing_future_prices():
    bars, _, groups = data('historical')
    bricks = [row(0, 1), row(60000, 2), row(60000, 3)]
    program = pine_compat.compile_script(source('strategy'))
    expected = program.run(bars, request_bars={KEY: bricks}, chart_symbol='A', chart_timeframe='1')
    session = program.external_session(scheduled=True, request_bars={KEY: []}, chart_symbol='A', chart_timeframe='1')
    session.initialize_history(bars, requested_history=[dict(symbol=SYMBOL, timeframe='1', times_ms=[0,60000,60000])])
    assert session.snapshot()['output']['plots'] == []
    for index, group in enumerate(groups):
        session.begin_bar(bars[index], bar_state='historical')
        for ordinal, observation in enumerate(group):
            observation = deepcopy(observation)
            observation['evaluation']['request_updates'] = [dict(symbol=SYMBOL, timeframe='1', kind='confirmed',
                bar=brick, available_at_ms=bars[index]['time']+59999) for brick in bricks
                if brick['time']==bars[index]['time']] if ordinal == 0 else []
            session.observe(observation, snapshot=False)
        session.commit_bar(snapshot=False)
    assert session.snapshot()['output']['plots'] == expected['plots']
    assert expected['plots'][0]['values'] == [None, 3]


def test_cross_stream_failure_does_not_partially_append_a_same_time_event_cohort():
    program = source().replace(',calc_bars_count=1', '')
    session = pine_compat.create_realtime_session(program, chart_symbol='A', chart_timeframe='1',
        request_bars={KEY:[row(0,1)], 'C:1':[row(0,10)]})
    session.seed([row(0,10)])
    first = dict(symbol=SYMBOL, timeframe='1', kind='confirmed', bar=row(60000,2))
    second = {**first, 'bar':row(60000,3)}
    before = deepcopy(session.result())
    with pytest.raises(ValueError):
        session.apply_request_feed_updates([first,second,
            dict(symbol='C',timeframe='1',kind='confirmed',bar=row(0,999))])
    assert session.result() == before
    session.apply_request_feed_updates([first,second])
    session.update_confirmed(row(60000,10),execution_time=119999)
    assert session.result()['plots'][0]['values'] == [1,6]


def test_forming_event_then_confirmation_preserves_each_bricks_history():
    program = source().replace('ta.cum(close)', 'close[1]').replace(',calc_bars_count=1', '')
    session = pine_compat.create_realtime_session(program,chart_symbol='A',chart_timeframe='1',
        request_bars={KEY:[row(0,1)]})
    session.seed([row(0,10)])
    for value in (2,3):
        session.apply_request_feed_updates([
            dict(symbol=SYMBOL,timeframe='1',kind='forming',bar=row(60000,value)),
            dict(symbol=SYMBOL,timeframe='1',kind='confirmed',bar=row(60000,value))])
    session.update_confirmed(row(60000,10),execution_time=119999)
    assert session.result()['plots'][0]['values'] == [None,2]

"""Known request windows never prepopulate a realtime chart's future values."""
from copy import deepcopy

import pine_compat
import pytest
from test_external_stream import data


SOURCE='''//@version=6
strategy("realtime window",calc_on_every_tick=true,calc_on_every_history_tick=true)
varip int n=0
n+=1
plot(n)
plot(last_bar_index)
plot(request.security(syminfo.tickerid,timeframe.period,close))
plot(request.security("B","1",ta.cum(close),calc_bars_count=1))
plot(barstate.isrealtime?1:0)
'''


def prepared(source=SOURCE):
    bars,_,groups=data('realtime')
    session=pine_compat.compile_script(source).external_session(scheduled=True,
        chart_symbol='A',chart_timeframe='1',request_bars={'B:1':[]})
    skip=session.initialize_realtime_history(bars,requested_history=[dict(symbol='B',timeframe='1',times_ms=[0,60000])])
    return session,bars,groups,skip


def play(session,bars,groups):
    for bar,group in zip(bars,groups,strict=True):
        session.begin_bar(group[0]['evaluation']['bar'],bar_state='realtime')
        for ordinal,observation in enumerate(group):
            item=deepcopy(observation)
            item['evaluation']['request_updates']=[]
            if ordinal==3:
                price=100*(bar['time']//60000+1)
                item['evaluation']['request_updates']=[dict(symbol='B',timeframe='1',kind='confirmed',
                    bar=dict(time=bar['time'],open=price,high=price,low=price,close=price,volume=1),
                    available_at_ms=bar['time']+59999)]
            output=session.observe(item)['output']
            assert output['plots'][1]['values'][-1]==bar['time']//60000
            assert output['plots'][2]['values'][-1]==item['evaluation']['bar']['close']
        session.commit_bar(snapshot=False)


def test_realtime_plan_preserves_partial_chart_values_and_exact_requested_origin_after_rebase():
    session,bars,groups,skip=prepared()
    assert skip==0 and session.snapshot()['output']['plots']==[]
    play(session,bars,groups)
    expected=[[4,8],[0,1],[11,21],[None,200],[1,1]]
    assert [plot['values'] for plot in session.snapshot()['output']['plots']]==expected
    assert session.rebase_window(seal=True)==0
    play(session,bars,groups)
    assert [plot['values'] for plot in session.snapshot()['output']['plots']]==expected


def test_wrong_bar_state_and_future_canonical_tuple_reject_before_opening_realtime_bar():
    session,bars,groups,_=prepared()
    before=session.snapshot()
    with pytest.raises(ValueError,match='planned history'):
        session.begin_bar(bars[0],bar_state='historical')
    invalid={**groups[0][0]['evaluation']['bar'],'high':bars[0]['high']+1}
    with pytest.raises(ValueError,match='planned history'):
        session.begin_bar(invalid,bar_state='realtime')
    assert session.snapshot()==before
    play(session,bars,groups)


def test_wrong_confirmed_tuple_is_retryable_and_cannot_seal_incomplete_window():
    session,bars,groups,_=prepared()
    session.begin_bar(groups[0][0]['evaluation']['bar'],bar_state='realtime')
    for item in groups[0][:3]:session.observe(item,snapshot=False)
    before=session.snapshot()
    bad=deepcopy(groups[0][3])
    bad['evaluation']['bar']['close']-=0.5
    with pytest.raises(ValueError,match='frozen dataset'):session.observe(bad)
    assert session.snapshot()==before
    session.observe(groups[0][3],snapshot=False)
    session.commit_bar(snapshot=False)
    with pytest.raises(ValueError,match='complete stable pass'):session.rebase_window(seal=True)


def test_realtime_static_account_branches_reserve_windows_without_seeding_future_chart_prices():
    source='''//@version=6
strategy("live discovery",calc_on_every_tick=true,calc_bars_count=1)
plot(strategy.position_size>0?request.security("A","1",close,calc_bars_count=2):request.security("B","1",close,calc_bars_count=1))
plot(last_bar_index)
'''
    session,bars,groups,skip=prepared(source)
    assert skip==0
    session.begin_bar(groups[0][0]['evaluation']['bar'],bar_state='realtime')
    first=deepcopy(groups[0][0])
    first['evaluation']['account']['position_size']=0
    session.observe(first)
    changed=deepcopy(groups[0][1])
    changed['evaluation']['account']['position_size']=1
    output=session.observe(changed)['output']
    assert not session.window_status()['rebase_required']
    assert output['plots'][0]['values']==[12]
    assert output['plots'][1]['values']==[0]


def test_realtime_plan_does_not_admit_an_unseen_dynamic_request_context():
    source=SOURCE.replace('request.security("B","1",ta.cum(close),calc_bars_count=1)',
        'request.security(strategy.position_size>0?"A":"B","1",close,calc_bars_count=1)')
    session,bars,groups,_=prepared(source)
    session.begin_bar(groups[0][0]['evaluation']['bar'],bar_state='realtime')
    with pytest.raises(ValueError,match='new dynamic context'):
        session.observe(groups[0][0])
    assert not session.window_status()['rebase_required']
    assert session.failed_request_context() is None
    with pytest.raises(ValueError,match='POISONED'):session.snapshot()


def test_realtime_plan_cannot_be_reinitialized_even_before_its_first_observation():
    session,bars,_,_=prepared()
    before=session.snapshot()
    for initialize in (session.initialize_realtime_history,session.initialize_history):
        with pytest.raises(ValueError,match='fresh scheduled runtime'):initialize(bars)
        assert session.snapshot()==before


@pytest.mark.parametrize('field',['open','high','low','close','volume'])
def test_static_realtime_self_request_matches_each_observed_field_and_native_control(field):
    source=f'''//@version=6
strategy("planned realtime field",calc_on_every_tick=true,dynamic_requests=false)
varip float seen=0
seen:=seen*100+request.security(syminfo.tickerid,timeframe.period,{field})
plot(seen)
plot(last_bar_time)
'''
    session,bars,groups,_=prepared(source)
    native=pine_compat.create_realtime_session(source,chart_symbol='A',chart_timeframe='1')
    native.seed([])
    for group in groups:
        session.begin_bar(group[0]['evaluation']['bar'],bar_state='realtime')
        for observation in group:
            frame=observation['evaluation']
            expected=(native.update_confirmed if frame['confirmed'] else native.update_forming)(
                frame['bar'],execution_time=frame['event_time_ms'])
            assert session.observe(observation)['output']['plots']==expected['plots']
        session.commit_bar(snapshot=False)

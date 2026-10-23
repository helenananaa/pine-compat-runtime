from copy import deepcopy

import pytest
import pine_compat


SOURCE = '''//@version=6
strategy("Close",process_orders_on_close=true,calc_on_order_fills=true)
var int ordinary=0
varip int persistent=0
ordinary+=1
persistent+=1
if barstate.isnew
    strategy.entry("L",strategy.long,qty=2)
if strategy.position_size>0
    strategy.close("L")
plot(ordinary)
plot(persistent)
plot(strategy.position_size)
plot(timenow)
'''


def data():
    bars=[dict(time=i*60000,open=10,high=11,low=9,close=10,volume=100) for i in range(2)]
    def account(bar,position):
        return dict(time=bar['time'],position_size=position,position_avg_price=10 if position else None,
            equity=1000,initial_capital=1000,netprofit=0,openprofit=0)
    groups=[[dict(bar=bar.copy(),account=account(bar,p),event_time_ms=bar['time']+59999,confirmed=True)
             for p in (0,2,0)] for bar in bars]
    return bars,[group[-1]['account'] for group in groups],groups


def test_external_closing_ticks_require_explicit_opt_in_and_keep_rollback_and_account_authority():
    bars,accounts,groups=data()
    program=pine_compat.compile_script(SOURCE)
    with pytest.raises(ValueError,match='E_EXTERNAL_UNSUPPORTED'):
        program.run_external(bars,accounts,execution_passes=groups)
    actual=program.run_external(bars,accounts,execution_passes=groups,closing_ticks=True)
    assert pine_compat.EXTERNAL_CLOSING_TICK_SCHEMA_VERSION==1
    assert actual['output']['plots'][0]['values']==[1,2]
    assert actual['output']['plots'][1]['values']==[3,6]
    assert actual['output']['plots'][2]['values']==[0,0]
    assert actual['output']['plots'][3]['values']==[59999,119999]
    assert [(x['bar_index'],x['pass_index'],x['action']) for x in actual['intents']]==[
        (0,0,'entry'),(0,1,'close'),(1,0,'entry'),(1,1,'close')]
    assert 'strategy' not in actual['output']
    session=program.external_session(closing_ticks=True)
    for i in range(2):
        assert session.append(bars[i:i+1],accounts[i:i+1],execution_passes=groups[i:i+1])==program.run_external(
            bars[:i+1],accounts[:i+1],execution_passes=groups[:i+1],closing_ticks=True)


@pytest.mark.parametrize('change',['forming','price','clock','request'])
def test_a_confirmed_closing_pass_cannot_revise_market_data_or_advance_the_tick(change):
    bars,accounts,groups=data()
    changed=deepcopy(groups)
    item=changed[0][1]
    if change=='forming':
        item['confirmed']=False
    elif change=='price':
        item['bar'].update(open=20,high=21,low=19,close=20)
    elif change=='clock':
        item['event_time_ms']+=1
    else:
        item['request_data']=[]
    with pytest.raises(ValueError,match='invalid execution pass state'):
        pine_compat.compile_script(SOURCE).run_external(bars,accounts,execution_passes=changed,closing_ticks=True)


def test_closing_tick_opt_in_does_not_allow_repeated_confirmed_passes_without_the_declaration():
    bars,accounts,groups=data()
    program=pine_compat.compile_script(SOURCE.replace('process_orders_on_close=true,',''))
    with pytest.raises(ValueError,match='invalid execution pass state'):
        program.run_external(bars,accounts,execution_passes=groups,closing_ticks=True)


def test_single_closing_pass_works_without_fill_recalculation_and_old_api_stays_fail_closed():
    program=pine_compat.compile_script('//@version=6\nstrategy("Close",process_orders_on_close=true)\nstrategy.entry("L",strategy.long)')
    bars,accounts,_=data()
    with pytest.raises(ValueError,match='E_EXTERNAL_UNSUPPORTED'):
        program.external_session()
    assert program.external_session(closing_ticks=True).append(bars,accounts)==program.run_external(bars,accounts,closing_ticks=True)

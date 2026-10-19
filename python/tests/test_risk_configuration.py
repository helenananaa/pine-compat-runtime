import pine_compat
import pytest


SOURCE='''//@version=6
strategy("Risk configuration")
n=input.int(3,minval=1)
if false
    strategy.risk.max_position_size(n+1)
strategy.risk.allow_entry_in(strategy.direction.long)
strategy.risk.max_drawdown(10,strategy.percent_of_equity,"DD")
strategy.risk.max_intraday_loss(20,strategy.cash)
strategy.risk.max_intraday_filled_orders(2)
strategy.risk.max_cons_loss_days(3)
strategy.risk.max_position_size(8)
if bar_index==0
    strategy.entry("L",strategy.long,qty=5)
plot(strategy.position_size)
'''


def data():
    bars=[dict(time=i*60000,open=10,high=11,low=9,close=10,volume=100) for i in range(3)]
    accounts=[dict(time=row['time'],position_size=i,position_avg_price=10 if i else None,
        equity=1000,initial_capital=1000,netprofit=0,openprofit=0) for i,row in enumerate(bars)]
    return bars,accounts


def test_all_rules_are_declarative_parameterized_and_preserved_separately():
    program=pine_compat.compile_script(SOURCE)
    result=program.strategy_risk_configuration()
    assert result['schemaVersion']==pine_compat.STRATEGY_RISK_CONFIGURATION_SCHEMA_VERSION==1
    assert result['unresolvedCallSiteIds']==[]
    assert len(result['rules'])==7 and all(row['resolved'] for row in result['rules'])
    assert [row['value'] for row in result['rules'] if row['function'].endswith('max_position_size')]==[4,8]
    input_id=program.host_requirements()['inputCallSiteIds'][0]
    changed=program.strategy_risk_configuration(input_overrides={input_id:6})
    assert changed['rules'][0]['value']==7
    assert result['rules'][2]['valueType']=='strategy.percent_of_equity'
    assert result['rules'][2]['alertMessage']=='DD'


def test_external_opt_in_exports_rules_without_a_native_broker_or_silent_host_caps():
    program=pine_compat.compile_script(SOURCE)
    bars,accounts=data()
    with pytest.raises(ValueError,match='explicit host enforcement'):
        program.run_external(bars,accounts)
    with pytest.raises(ValueError,match='explicit host enforcement'):
        program.external_session()
    actual=program.run_external(bars,accounts,risk_rules=True)
    assert actual['risk_rules']==program.strategy_risk_configuration()
    assert actual['output']['plots'][0]['values']==[0,1,2]
    assert actual['intents'][0]['qty']==5  # The Host owns enforcement; no native fill/cap.
    assert 'strategy' not in actual['output']
    session=program.external_session(risk_rules=True)
    for index in range(len(bars)):
        assert session.append(bars[index:index+1],accounts[index:index+1])==program.run_external(
            bars[:index+1],accounts[:index+1],risk_rules=True)


def test_default_callers_reject_even_an_unreachable_risk_call():
    program=pine_compat.compile_script('//@version=6\nstrategy("Hidden")\nif false\n    strategy.risk.max_position_size(2)')
    with pytest.raises(ValueError,match='explicit host enforcement'):
        program.run_external([],[])


def test_no_risk_output_field_is_added_to_ordinary_external_results():
    program=pine_compat.compile_script('//@version=6\nstrategy("No rules")\nplot(close)')
    bars,accounts=data()
    assert 'risk_rules' not in program.run_external(bars,accounts,risk_rules=True)


def test_context_math_and_inlined_simple_udfs_are_resolved():
    source='''//@version=6
strategy("Context")
f(simple float n)=>n*2
if false
    strategy.risk.max_position_size(f(syminfo.mintick*8))
while true
    runtime.error("must not execute")
'''
    program=pine_compat.compile_script(source)
    result=program.strategy_risk_configuration(chart_symbol='EXCHANGE:TEST',chart_timeframe='1',
        request_bars={'$chart':{'minMove':1,'priceScale':4}})
    assert result['rules'][0]['value']==4 and result['unresolvedCallSiteIds']==[]


def test_side_effecting_arguments_do_not_execute_and_keep_the_profile_closed():
    program=pine_compat.compile_script('//@version=6\nstrategy("Effect")\nf()=>\n    runtime.error("metadata side effect")\n    2.0\nstrategy.risk.max_position_size(f())')
    result=program.strategy_risk_configuration()
    assert result['unresolvedCallSiteIds']==[result['rules'][0]['callSiteId']]
    with pytest.raises(ValueError,match='fully resolved configuration'):
        program.run_external([],[],risk_rules=True)


@pytest.mark.parametrize('value',[0,-1,1.5,1e99])
def test_invalid_runtime_input_count_never_becomes_a_resolved_rule(value):
    program=pine_compat.compile_script('//@version=6\nstrategy("Count")\nn=input.float(2)\nstrategy.risk.max_intraday_filled_orders(n)')
    key=program.host_requirements()['inputCallSiteIds'][0]
    result=program.strategy_risk_configuration(input_overrides={key:value})
    assert result['unresolvedCallSiteIds']
    with pytest.raises(ValueError,match='fully resolved configuration'):
        program.external_session(input_overrides={key:value},risk_rules=True)

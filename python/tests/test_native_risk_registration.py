"""Native broker directives are independent of branch and declaration order."""
import pine_compat
import pytest


def bars(prices=(100, 100, 96, 96)):
    return [dict(time=i*60000, open=price, high=price, low=price,
                 close=price, volume=100) for i, price in enumerate(prices)]


def compile_rules(rules, body='if bar_index==0\n    strategy.entry("L",strategy.long,qty=10)'):
    return pine_compat.compile_script(f'''//@version=6
strategy("Native rules",initial_capital=1000,margin_long=0,margin_short=0)
{rules}
{body}
plot(strategy.position_size)
''')


@pytest.mark.parametrize('rules', [
    'if false\n    strategy.risk.max_position_size(2)',
    'strategy.risk.max_position_size(2)\nstrategy.risk.max_position_size(5)',
    'strategy.risk.max_position_size(5)\nstrategy.risk.max_position_size(2)',
])
def test_native_position_caps_are_declarative_and_do_not_overwrite(rules):
    result = compile_rules(rules).run(bars((100,)*3))
    assert result['plots'][0]['values'] == [0, 2, 2]
    assert result['strategy']['orders'][0]['qty'] == 2


@pytest.mark.parametrize('function', ['max_drawdown', 'max_intraday_loss'])
@pytest.mark.parametrize('strict,loose', [
    ('2,strategy.percent_of_equity', '10000,strategy.cash'),
    ('20,strategy.cash', '99,strategy.percent_of_equity'),
])
@pytest.mark.parametrize('reverse', [False, True])
def test_cash_and_percentage_limits_remain_independent(function, strict, loose, reverse):
    values = [strict, loose] if not reverse else [loose, strict]
    source = '\n'.join(f'strategy.risk.{function}({value})' for value in values)
    result = compile_rules(source).run(bars())
    assert result['plots'][0]['values'] == [0, 10, 0, 0]


def test_input_overrides_and_chart_grid_are_bound_before_registration():
    program = compile_rules('''n=input.int(2)
f(simple float value)=>value*2
if false
    strategy.risk.max_position_size(f(n*syminfo.mintick))''')
    key = program.host_requirements()['inputCallSiteIds'][0]
    result = program.run(bars((100,)*3), input_overrides={key: 4},
                         request_bars={'$chart': {'minMove': 1, 'priceScale': 4}})
    assert result['plots'][0]['values'] == [0, 2, 2]


def test_unresolved_hidden_rules_do_not_silently_disappear():
    program = compile_rules('''f()=>
    runtime.error("must not execute")
    2.0
if false
    strategy.risk.max_position_size(f())''')
    with pytest.raises(ValueError, match='E_NATIVE_RISK_UNSUPPORTED') as error:
        program.run(bars())
    assert 'must not execute' not in str(error.value)


def test_external_account_enforcement_stays_with_the_host():
    program = compile_rules('if false\n    strategy.risk.max_position_size(2)')
    inputs = bars((100,)*3)
    accounts = [dict(time=row['time'], position_size=0, position_avg_price=None,
                     equity=1000, initial_capital=1000, netprofit=0, openprofit=0)
                for row in inputs]
    result = program.run_external(inputs, accounts, risk_rules=True)
    assert result['intents'][0]['qty'] == 10
    assert result['output']['plots'][0]['values'] == [0, 0, 0]
    assert 'strategy' not in result['output']


@pytest.mark.parametrize('expression,expected', [
    ('str.length(syminfo.ticker)', 10), ('str.length(syminfo.prefix)', 4),
    ('timeframe.multiplier', 15), ('timeframe.in_seconds()/60', 15),
    ('timestamp(2020,1,1,0,0)>0 ? 2 : 3', 2),
    ('str.length(str.match(syminfo.ticker,"LONG"))', 4),
])
def test_native_and_metadata_share_the_actual_chart_context(expression, expected):
    program = compile_rules(f'if false\n    strategy.risk.max_position_size({expression})',
                            'if bar_index==0\n    strategy.entry("L",strategy.long,qty=100)')
    context = dict(chart_symbol='TEST:LONGTICKER', chart_timeframe='15')
    configuration = program.strategy_risk_configuration(**context)
    assert configuration['rules'][0]['value'] == expected
    result = program.run(bars((100,)*3), **context)
    assert result['plots'][0]['values'] == [0, expected, expected]


@pytest.mark.parametrize('flow', ['for i=1 to n\n        total+=1',
                                 'while total<n\n        total+=1'])
def test_scalar_local_loops_resolve_without_executing_original_program_scope(flow):
    program = compile_rules(f'''f(simple int n)=>
    total=0
    {flow}
    total
if false
    strategy.risk.max_position_size(f(2))''')
    assert program.strategy_risk_configuration()['rules'][0]['value'] == 2
    assert program.run(bars((100,)*3))['plots'][0]['values'] == [0, 2, 2]


def test_calendar_timezone_is_preserved_in_discovery_native_and_external_contracts():
    from datetime import datetime
    start = int(datetime.fromisoformat('2026-10-08T13:30:00+00:00').timestamp()*1000)
    day = int(datetime.fromisoformat('2026-10-08T00:00:00+00:00').timestamp()*1000)
    inputs = bars((100,)*3)
    for i, row in enumerate(inputs):
        row['time'] = start+i*60000
    calendar = {'schemaVersion': 1, 'symbols': [{
        'symbol': 'TEST:LONGTICKER', 'timezone': 'America/New_York',
        'timeframes': [{'timeframe': '1', 'intervals': [
            {'open': row['time'], 'close': row['time']+60000, 'tradingDay': day}
            for row in inputs]}]}]}
    context = dict(chart_symbol='TEST:LONGTICKER', chart_timeframe='1',
                   request_bars={'$barCalendars': calendar})
    program = compile_rules('if false\n    strategy.risk.max_position_size(str.length(syminfo.timezone))',
                            'if bar_index==0\n    strategy.entry("L",strategy.long,qty=100)')
    configuration = program.strategy_risk_configuration(**context)
    assert configuration['rules'][0]['value'] == 16
    assert program.run(inputs, **context)['plots'][0]['values'] == [0, 16, 16]
    accounts = [dict(time=row['time'], position_size=0, position_avg_price=None,
                     equity=1000, initial_capital=1000, netprofit=0, openprofit=0)
                for row in inputs]
    external = program.run_external(inputs, accounts, risk_rules=True, **context)
    assert external['risk_rules'] == configuration
    assert external['intents'][0]['qty'] == 100

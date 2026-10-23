"""Contract value regressions; Decimal expected values frozen before implementation."""
import pine_compat
import pytest


def bars(prices=(100, 100, 110, 110)):
    return [dict(time=i*60000, open=p, high=p, low=p, close=p, volume=1000)
            for i,p in enumerate(prices)]


def context(multiplier):
    return {'$chart': dict(minMove=1, priceScale=100, pointValue=multiplier)}


def program(commission, fee, qty):
    return pine_compat.compile_script(f'''//@version=6
strategy("Contract valuation",initial_capital=10000,margin_long=0,margin_short=0,
 commission_type=strategy.commission.{commission},commission_value={fee})
if bar_index==0
    strategy.entry("L",strategy.long,qty={qty})
if bar_index==2
    strategy.close_all()
plot(strategy.equity)
plot(syminfo.pointvalue)
''')


@pytest.mark.parametrize('multiplier,commission,fee,qty,profit', [
    (50., 'cash_per_contract', 2, 2, 992),
    (50., 'percent', .1, 2, 979),
    (.01, 'cash_per_contract', .02, 100, 6),
    (.01, 'percent', .1, 100, 9.79),
    (50., 'cash_per_order', 2, 2, 996),
])
def test_contract_valuation_accounts_for_commission_units(multiplier,commission,fee,qty,profit):
    result=program(commission,fee,qty).run(bars(),request_bars=context(multiplier))
    strategy=result['strategy']
    assert result['plots'][1]['values']==[multiplier]*4
    assert strategy['orders'][0]['qty']==qty
    assert strategy['trades'][0]['profit']==pytest.approx(profit)
    assert strategy['equity'][-1]['cash']==pytest.approx(10000+profit)
    assert strategy['equity'][-1]['equity']==pytest.approx(10000+profit)
    assert strategy['equity'][-1]['marketValue']==0


def test_contract_reversal_retains_transaction_quantities_and_allocated_fees():
    source='''//@version=6
strategy("Contract reversal",initial_capital=10000,margin_long=0,margin_short=0,
 commission_type=strategy.commission.cash_per_order,commission_value=2)
if bar_index==0
    strategy.entry("L",strategy.long,qty=2)
if bar_index==2
    strategy.entry("S",strategy.short,qty=3)
if bar_index==4
    strategy.close_all()
'''
    result=pine_compat.compile_script(source).run(bars((100,100,110,110,100,100)),request_bars=context(50))['strategy']
    assert [a['qty'] for a in result['alerts']]==[2,5,3]
    assert [t['profit'] for t in result['trades']]==pytest.approx([997.2,1496.8])
    assert result['equity'][-1]['equity']==pytest.approx(12494)

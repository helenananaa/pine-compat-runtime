"""Exact contract metadata at the installed Python/native boundary."""
import pine_compat
import pytest

BARS=[dict(time=i*60000,open=100,high=100,low=100,close=100,volume=1000) for i in range(3)]


def grid(**extra):
    return {'$chart':dict(minMove=1,priceScale=100,quantityStep='0.25',**extra)}


def source(qty):
    return f'''//@version=6
strategy("Quantity contract",margin_long=0)
if bar_index==0
    strategy.entry("L",strategy.long,qty={qty})
plot(syminfo.pointvalue)
plot(syminfo.mincontract)
'''


def test_contract_capability_and_requirements_describe_actual_host_metadata():
    assert pine_compat.CONTRACT_METADATA_SCHEMA_VERSION==1
    requirements=pine_compat.compile_script(source(2)).host_requirements()
    assert requirements['schemaVersion']==5
    assert requirements['chart']['contractMetadataSchemaVersion']==1
    assert requirements['account']['profile']=='linearHostPointValueWithHostCurrencyRates'
    assert requirements['account']['defaultPointValue']==1
    assert 'nonUnitContractMultiplier' not in requirements['account']['unsupportedProfiles']


@pytest.mark.parametrize('qty,expected',[(1.249999999999,1),(1.25,1.25),(1.49,1.25)])
def test_explicit_quantity_is_truncated_on_the_actual_increment(qty,expected):
    result=pine_compat.compile_script(source(qty)).run(BARS,request_bars=grid(pointValue=50))
    assert result['strategy']['orders'][0]['qty']==expected
    assert result['strategy']['equity'][1]['marketValue']==expected*100*50
    assert result['plots'][0]['values']==[50]*3


def test_minimum_is_separate_and_metadata_does_not_leak_between_program_runs():
    program=pine_compat.compile_script(source(1.99))
    blocked=program.run(BARS,request_bars=grid(minContract='2'))
    assert not blocked['strategy']['orders']
    assert blocked['plots'][1]['values']==[2]*3
    ordinary=program.run(BARS)
    assert ordinary['strategy']['orders'][0]['qty']==1.99
    assert ordinary['plots'][1]['values']==[1]*3


@pytest.mark.parametrize('field',['quantityStep','minContract'])
@pytest.mark.parametrize('value',[True,0.25,None,'0','-1','NaN','inf','1/4','1e-19','0.100000000000000001'])
def test_quantity_contract_rejects_non_decimal_or_unrepresentable_metadata(field,value):
    metadata=dict(minMove=1,priceScale=100)
    metadata[field]=value
    with pytest.raises(ValueError,match=field):
        pine_compat.compile_script(source(2)).run(BARS,request_bars={'$chart':metadata})


def test_precision_and_step_must_be_consistent():
    program=pine_compat.compile_script(source(1.49))
    assert program.run(BARS,request_bars=grid(quantityPrecision=2))['strategy']['orders'][0]['qty']==1.25
    with pytest.raises(ValueError,match='quantityPrecision'):
        program.run(BARS,request_bars=grid(quantityPrecision=1))


def test_unrepresentable_large_quantity_cannot_be_emitted_as_an_off_grid_order():
    program=pine_compat.compile_script(source('1000000000000'))
    with pytest.raises(ValueError,match='quantity'):
        program.run(BARS,request_bars={'$chart':dict(minMove=1,priceScale=100,quantityStep='3e-18')})

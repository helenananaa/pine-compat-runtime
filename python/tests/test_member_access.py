from pathlib import Path
import pine_compat
import pytest

SOURCE=(Path(__file__).resolve().parents[2]/'tests/fixtures/runtime/member_access.pine').read_text()

@pytest.mark.parametrize('version',[5,6])
def test_member_access_fields_methods_and_history(version):
    source=SOURCE.replace('version=6',f'version={version}')
    bars=[dict(time=i*60000,open=10.+i,high=10.+i,low=10.+i,close=10.+i,volume=1.) for i in range(8)]
    program=pine_compat.compile_script(source)
    expected=program.run(bars)
    assert expected['plots'][0]['values']==[10.+i for i in range(8)]
    assert expected['plots'][1]['values']==list(range(1,9))
    assert expected['plots'][2]['values']==[None,*range(1,8)]
    assert expected['plots'][4]['values']==list(range(1,9))
    assert expected['plots'][5]['values']==[None,*[10.+i for i in range(7)]]
    assert len(expected['lines'])==8
    session=program.realtime_session()
    session.seed(bars[:1]);replica=session.replica()
    for bar in bars[1:]:
        replica.apply(session.apply_forming(bar))
        replica.apply(session.apply_forming(bar))
        replica.apply(session.apply_confirmed(bar))
    assert replica.result()==session.result()==expected

@pytest.mark.parametrize('version',[5,6])
def test_undefined_member_read_reports_runtime_error(version):
    source=f'//@version={version}\nindicator("undefined")\ntype Item\n    float value\nItem item=na\nplot(item.value)\n'
    program=pine_compat.compile_script(source)
    with pytest.raises(ValueError, match='E_UDT_NA_FIELD'):
        program.run([dict(time=0,open=1.,high=1.,low=1.,close=1.,volume=1.)])

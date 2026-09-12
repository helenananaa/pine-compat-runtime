import copy
from pathlib import Path

import pine_compat
import pytest

SOURCE = (Path(__file__).resolve().parents[2] / "tests/fixtures/runtime/gradient_fill.pine").read_text()

def bar(i, close):
    return dict(time=i*60000,open=close,high=close,low=close,close=close,volume=1.)

def test_gradient_python_delta_replica_retention_and_rollback():
    program=pine_compat.compile_script(SOURCE)
    session=program.realtime_session()
    session.set_output_retention(7)
    session.seed([bar(0,10.)])
    replica=session.replica()
    for i in range(1,270):
        for fn, close in [(session.apply_forming,20.),(session.apply_forming,30.),(session.apply_confirmed,40.)]:
            change=fn(bar(i,close))
            assert change["schemaVersion"]==4
            replica.apply(change)
            assert replica.result()==session.result()
            fill=replica.result()["fills"][0]
            assert len(fill["gradient"])==len(fill["colors"])<=8
            assert fill["gradient"][-1]["bottomValue"]==close-1
    assert session.result()["schemaVersion"]==9

def test_gradient_old_schema_and_malformed_samples_reject_atomically():
    session=pine_compat.compile_script(SOURCE).realtime_session()
    session.seed([bar(0,10.)])
    replica=session.replica()
    before=replica.result()
    change=session.apply_forming(bar(1,20.))
    old=copy.deepcopy(change)
    old["schemaVersion"]=3
    with pytest.raises(ValueError,match="schema"):
        replica.apply(old)
    for key,value in [("topColor",True),("topValue",float("inf")),("topColor",-1)]:
        bad=copy.deepcopy(change)
        item=next(item for item in bad["fills"] if item["action"]=="setGradient")
        item["values"][0][key]=value
        with pytest.raises((TypeError,ValueError,OverflowError)):
            replica.apply(bad)
        assert replica.result()==before
    replica.apply(change)
    assert replica.result()==session.result()

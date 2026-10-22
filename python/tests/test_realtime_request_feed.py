"""Provider-only input transactions retain exactly one script execution per observation."""
from copy import deepcopy

import pine_compat
import pytest

SOURCE='//@version=6\nindicator("feed")\nvarip int n=0\nn+=1\nplot(n)\na=request.security_lower_tf("B","1",close)\nplot(array.size(a))\nplot(array.sum(a))'


def bar(time,price=10):
    return dict(time=time,open=price,high=price,low=price,close=price,volume=1)


def update(time,price=10,**changes):
    return dict(symbol="B",timeframe="1",bar=bar(time,price),kind="confirmed",**changes)


def seeded():
    session=pine_compat.create_realtime_session(SOURCE,chart_symbol="A",chart_timeframe="5",
        request_bars={"B:1":[bar(i*60000) for i in range(5)],"C:1":[bar(0)]})
    session.seed([bar(0)])
    before=session.result()
    session.apply_request_feed_updates([update(300000)])
    assert session.result()==before
    session.update_forming(bar(300000),execution_time=359999)
    return session


def last(session):
    return [p["values"][-1] for p in session.result()["plots"]]


def test_feed_batch_preserves_output_delta_revision_and_varip_until_next_observation():
    assert pine_compat.REALTIME_REQUEST_FEED_SCHEMA_VERSION==1
    session=seeded()
    before=deepcopy(session.result())
    changes=deepcopy(session.last_changes())
    revision=session.revision
    session.apply_request_feed_updates([update(360000,20),update(420000,30)])
    assert session.result()==before
    assert session.last_changes()==changes and session.revision==revision
    session.update_forming(bar(300000,11),execution_time=479999)
    assert last(session)==[3,3,60]
    session.update_confirmed(bar(300000,11),execution_time=599999)
    assert last(session)==[4,3,60] and session.confirmed_bars==2


def test_later_cross_stream_failure_keeps_the_complete_transaction_retryable():
    session=seeded()
    before=deepcopy(session.result())
    bad=update(0,999)
    bad["symbol"]="C"
    with pytest.raises(ValueError):
        session.apply_request_feed_updates([update(360000,20),bad])
    assert session.result()==before
    session.apply_request_feed_updates([update(360000,20)])
    session.update_forming(bar(300000,11),execution_time=419999)
    assert last(session)==[3,2,30]


@pytest.mark.parametrize("change",[
    {"symbol":" B"},{"timeframe":"bad"},{"kind":"unknown"},
    {"bar":bar(360000,20)|{"high":19}},{"bar":bar(360000,20)|{"volume":float("nan")}},
    {"unknown":True},
])
def test_invalid_feed_payload_cannot_partially_advance_live_state(change):
    session=seeded()
    before=deepcopy(session.result())
    with pytest.raises(ValueError):
        session.apply_request_feed_updates([update(360000,20),update(420000,30)|change])
    assert session.result()==before
    session.apply_request_feed_updates([])
    session.apply_request_feed_updates([update(360000,20)])
    session.update_forming(bar(300000,11),execution_time=419999)
    assert last(session)==[3,2,30]


def test_existing_auto_recalculation_remains_available_and_close_queries_use_calendar():
    session=seeded()
    session.apply_request_confirmed("B","1",bar(360000,20))
    assert last(session)==[3,2,30]
    assert session.request_bar_close("B","30S",0)==30000
    assert session.request_bar_close("B","1M",1706745600000)==1709251200000
    for invalid in [True,60000.5,"60000",None]:
        with pytest.raises(ValueError):
            session.request_bar_close("B","1",invalid)

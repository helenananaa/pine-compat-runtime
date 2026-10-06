import pytest

import pine_compat


def bar(index, close):
    return {
        "time": index * 60_000,
        "open": close,
        "high": close,
        "low": close,
        "close": close,
        "volume": 1,
    }


EVENT_SOURCE = '''//@version=6
indicator("retained events")
plot(ta.valuewhen(true, close, bar_index % 3))
plot(ta.valuewhen(true, close * 2, bar_index % 3))
'''


def test_valuewhen_limit_counts_callsites_and_rolls_back_realtime_failures():
    session = pine_compat.create_realtime_session(EVENT_SOURCE)
    assert session.valuewhen_limit is None
    session.seed([bar(0, 1), bar(1, 2)])
    assert session.valuewhen_retained_values == 4
    session.set_valuewhen_limit(6)
    session.apply_forming(bar(2, 3))
    session.apply_forming(bar(2, 4))
    assert session.valuewhen_retained_values == 6
    assert session.confirmed_valuewhen_retained_values == 4
    before, changes, revision = session.result(), session.last_changes(), session.revision
    with pytest.raises(ValueError, match="E_VALUEWHEN_BUDGET"):
        session.set_valuewhen_limit(5)
    assert session.valuewhen_limit == 6
    assert (session.result(), session.last_changes(), session.revision) == (before, changes, revision)
    session.apply_confirmed(bar(2, 4))
    assert session.confirmed_valuewhen_retained_values == 6
    before, changes, revision = session.result(), session.last_changes(), session.revision
    with pytest.raises(ValueError, match="E_VALUEWHEN_BUDGET"):
        session.apply_forming(bar(3, 5))
    assert (session.result(), session.last_changes(), session.revision) == (before, changes, revision)
    assert session.forming_time is None
    assert session.valuewhen_retained_values == 6
    session.set_valuewhen_limit(8)
    session.apply_forming(bar(3, 5))
    session.apply_confirmed(bar(3, 5))
    assert session.valuewhen_retained_values == 8
    assert session.confirmed_valuewhen_retained_values == 8
    before, revision = session.result(), session.revision
    with pytest.raises(ValueError, match="E_VALUEWHEN_BUDGET"):
        session.replay([bar(index, index + 1) for index in range(5)])
    assert (session.result(), session.revision) == (before, revision)
    assert session.valuewhen_limit == 8
    session.set_valuewhen_limit()
    session.replay([bar(index, index + 1) for index in range(5)])
    assert session.valuewhen_retained_values == 10
    assert session.valuewhen_limit is None


@pytest.mark.parametrize("invalid", [True, False, -1, 1.5, "6", float("inf"), 2**100])
def test_valuewhen_limit_rejects_invalid_input_without_changing_configuration(invalid):
    session = pine_compat.create_realtime_session(EVENT_SOURCE)
    session.set_valuewhen_limit(6)
    session.seed([bar(0, 1)])
    before = session.result()
    with pytest.raises(ValueError, match="nonnegative integer"):
        session.set_valuewhen_limit(invalid)
    assert session.valuewhen_limit == 6
    assert session.result() == before
    assert session.valuewhen_retained_values == 2


def test_alert_deltas_own_python_payloads_and_preserve_duplicate_occurrences():
    count = 257
    prefix = "告警🧠" * 600
    source = f'''//@version=6
indicator("alert delta ownership")
if barstate.isrealtime
    for iteration = 0 to {count - 1}
        alert("{prefix}" + (close == 17 ? "A" : "B"), alert.freq_all)
plot(close)
'''
    session = pine_compat.create_realtime_session(source)
    session.seed([bar(0, 1)])
    replica = session.replica()
    first = session.apply_forming(bar(1, 17))
    assert len(first["alerts"]) == count
    assert all(change["action"] == "add" and change["event"]["message"] == prefix + "A"
               for change in first["alerts"])
    assert replica.apply(first) is True
    assert replica.apply(first) is False
    assert replica.result() == session.result()
    first["alerts"][0]["event"]["message"] = "independent Python object"
    assert session.last_changes()["alerts"][0]["event"]["message"] == prefix + "A"
    replacement = session.apply_forming(bar(1, 18))
    assert len(replacement["alerts"]) == count * 2
    assert [change["action"] for change in replacement["alerts"]] == ["remove"] * count + ["add"] * count
    assert [change["event"]["message"] for change in replacement["alerts"]] == [prefix + "A"] * count + [prefix + "B"] * count
    assert replica.apply(replacement) is True
    assert replica.apply(replacement) is False
    assert replica.result() == session.result()
    confirmed = session.apply_confirmed(bar(1, 18))
    assert replica.apply(confirmed) is True
    assert replica.result() == session.confirmed_result()
    assert len(session.confirmed_result()["alerts"]) == count

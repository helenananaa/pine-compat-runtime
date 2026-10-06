from __future__ import annotations

import copy
import sys
import threading
from collections.abc import Sequence

import pytest

import pine_compat


def _bar(index: int, close: float = 10.0) -> dict:
    return dict(time=index * 60_000, open=close, high=close, low=close, close=close, volume=1.0)


STATE_SOURCE = '''//@version=6
indicator("Seed state")
var int committed = 0
varip int intrabar = 0
committed += 1
intrabar += 1
var a = array.new_float(129, 0)
var m = matrix.new<float>(2, 2, 0)
var t = map.new<int,float>()
array.set(a, 128, close)
matrix.set(m, 0, 0, close)
map.put(t, 1, close)
plot(committed)
plot(intrabar)
plot(timenow)
plot(array.get(a, 128) + matrix.get(m, 0, 0) + map.get(t, 1))
plot(ta.sma(close, 3))
label.new(bar_index, close, str.tostring(close))
alert("execution", alert.freq_all)
'''


@pytest.mark.parametrize("retention", [None, 2])
def test_seed_state_preserves_snapshot_cursor_replica_and_realtime_rollback(retention):
    program = pine_compat.compile_script(STATE_SOURCE)
    state = program.realtime_session()
    control = program.realtime_session()
    for session in (state, control):
        session.set_output_retention(retention)
    bars = [_bar(index, index + 1.0) for index in range(12)]
    clocks = [index * 60_000 + 10 for index in range(12)]
    assert state.seed_state(bars, execution_times=clocks) is None
    seeded = control.seed(bars, execution_times=clocks)
    assert state.result() == state.confirmed_result() == seeded
    assert state.stream_snapshot() == control.stream_snapshot()
    assert state.is_seeded and state.confirmed_bars == 12
    assert state.last_confirmed_time == bars[-1]["time"]
    assert state.forming_time is None and state.revision == control.revision == 1
    preserved = copy.deepcopy(seeded)
    replica = state.replica()
    for method, close, clock in [("apply_forming", 20.0, 720_010),
                                 ("apply_forming", 21.0, 720_020),
                                 ("apply_confirmed", 22.0, 720_030)]:
        changes = getattr(state, method)(_bar(12, close), execution_time=clock)
        assert changes == getattr(control, method)(_bar(12, close), execution_time=clock)
        assert replica.apply(changes)
        assert replica.result() == state.result() == control.result()
        assert state.stream_snapshot() == control.stream_snapshot()
    assert seeded == preserved
    assert state.replay([_bar(13, 23.0)], execution_times=[780_010]) == control.replay(
        [_bar(13, 23.0)], execution_times=[780_010]
    )
    assert state.stream_snapshot() == control.stream_snapshot()
    assert state.confirmed_bars == control.confirmed_bars == 1
    assert state.last_confirmed_time == 780_000 and state.forming_time is None


def test_seed_state_failure_is_atomic_and_can_retry_with_complete_history():
    source = '''//@version=6
indicator("Seed failure")
var int committed = 0
committed += 1
var a = array.new_float(129, 0)
array.set(a, 128, close)
if bar_index == 1 and close < 0
    runtime.error("seed rejected")
plot(committed)
plot(array.get(a, 128))
plot(timenow)
'''
    program = pine_compat.compile_script(source)
    session = program.realtime_session()
    before = session.stream_snapshot()
    attempts = [
        ([_bar(0, 1.0), _bar(1, -1.0)], [10, 60_010], "seed rejected"),
        ([_bar(0, 1.0), _bar(1, 2.0)], None, "execution timestamp"),
        ([_bar(0, 1.0), _bar(1, 2.0)], [10], "count"),
        ([_bar(0, 1.0), _bar(0, 2.0)], [10, 20], "duplicate bar time"),
    ]
    for bars, clocks, message in attempts:
        with pytest.raises(ValueError, match=message):
            session.seed_state(bars, execution_times=clocks)
        assert not session.is_seeded and session.confirmed_bars == 0
        assert session.last_confirmed_time is None and session.forming_time is None
        assert session.stream_snapshot() == before
    bars, clocks = [_bar(0, 1.0), _bar(1, 2.0)], [10, 60_010]
    control = program.realtime_session()
    assert session.seed_state(bars, execution_times=clocks) is None
    assert session.result() == control.seed(bars, execution_times=clocks)
    committed = session.stream_snapshot()
    for method in (session.seed_state, session.seed):
        with pytest.raises(ValueError, match="already been seeded"):
            method([])
        assert session.stream_snapshot() == committed


def test_empty_seed_state_starts_revision_and_allows_updates():
    program = pine_compat.compile_script('//@version=6\nindicator("Empty")\nplot(close)\n')
    state = program.realtime_session()
    control = program.realtime_session()
    assert state.seed_state([]) is None
    assert state.result() == control.seed([])
    assert state.is_seeded and state.revision == 1 and state.confirmed_bars == 0
    assert state.last_confirmed_time is None
    assert state.apply_confirmed(_bar(0, 1.0)) == control.apply_confirmed(_bar(0, 1.0))
    assert state.result() == control.result()


@pytest.mark.parametrize("operation", ["seed_state", "apply_forming"])
def test_rust_execution_allows_other_python_thread_to_run(operation):
    if sys.implementation.name != "cpython" or (
        hasattr(sys, "_is_gil_enabled") and not sys._is_gil_enabled()
    ):
        pytest.skip("GIL-release regression requires CPython with its GIL enabled")
    program = pine_compat.compile_script('''//@version=6
indicator("Thread progress")
float total = 0
for counter = 0 to 49999
    total += math.sin(close + counter)
plot(total)
''')
    session = program.realtime_session()
    if operation == "apply_forming":
        session.seed_state([])
    execute = getattr(session, operation)

    started = threading.Event()
    ready = threading.Event()
    completed = threading.Event()
    observed = []

    def worker():
        ready.set()
        if started.wait(10):
            observed.append(completed.is_set())

    def bars():
        yield _bar(0)
        # parse_bars finishes consuming this iterator while holding the GIL.
        # Signal at exhaustion so thread progress cannot come from early input
        # preparation or Python's normal periodic interpreter switching.
        started.set()

    class SignalBar(Sequence):
        def __len__(self):
            return 6

        def __getitem__(self, index):
            fields = (0, 10.0, 10.0, 10.0, 10.0, 1.0)
            if index == 5:
                started.set()
            return fields[index]

    thread = threading.Thread(target=worker, daemon=True)
    thread.start()
    assert ready.wait(5), "worker did not start"
    previous_interval = sys.getswitchinterval()
    try:
        # Native execution must explicitly release the GIL. A bounded 50k
        # iteration workload gives the already-waiting worker time to run;
        # there is no assertion about wall-clock speed or parallel throughput.
        sys.setswitchinterval(60.0)
        execute(SignalBar() if operation == "apply_forming" else bars())
        completed.set()
    finally:
        completed.set()
        sys.setswitchinterval(previous_interval)
        started.set()
        thread.join(5)
    assert not thread.is_alive(), "worker did not finish"
    assert observed == [False], "Python worker could not run during Rust execution"

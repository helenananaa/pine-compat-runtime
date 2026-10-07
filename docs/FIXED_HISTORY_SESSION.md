# Fixed historical horizon sessions

Available in official Python wheels starting with `0.3.1`:

```python
session = program.historical_session(
    bars, input_overrides=None, chart_symbol=None, chart_timeframe=None,
    request_bars=None, magnifier_bars=None, execution_times=None, session_windows=None,
)
saved = session.fork()
output = session.advance(target)  # exclusive cursor into supplied bars
cursor = session.cursor
```

The complete nonempty dataset is frozen and validated at construction. Only
bars advanced to the cursor execute. last_bar_index, last_bar_time, chart
endpoint metadata and last-bar flags refer to the fixed full execution horizon,
not the current prefix. No future OHLCV enters script evaluation.

Native strategy execution, requested data, magnifier, input overrides, session
windows and explicit clocks follow Program.run. Final output equals a full
batch with identical inputs. calc_bars_count skips the same leading inputs as
batch execution; cursors still refer to the original supplied dataset. A prefix
before that window has no outputs. All timestamps are integer milliseconds.

advance accepts a nonnegative integer from the current cursor through dataset
length. Invalid cursor requests leave the session usable. Move backwards by
using an earlier independent fork or rebuilding. Output dictionaries are owned
snapshots; later execution cannot change them. Forks share immutable inputs and
program data while cloning independent runtime state.

Execution errors disable further advance and fork, including attempts to read
output at the unchanged cursor. Recreate the session or use an earlier healthy
fork. Previously returned outputs and saved healthy forks remain valid.

Realtime updates use realtime_session. Acquisition, eviction, processes and
durable checkpoints belong to callers. Rust exposes set_historical_horizon
before execution; the existing historical_dataset iterator remains available
for scoped batch execution.

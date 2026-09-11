# Realtime Model

## Incremental Session Windows

Python `RealtimeSession.extend_session_windows(payload)` accepts the same v1
dict or JSON string as the optional constructor `session_windows` argument.
Supply only the next known bars; there is no need to precompute an unlimited calendar:

```python
session.extend_session_windows({
    "schemaVersion": 1,
    "bars": [{"barIndex": next_index, "windowId": window_id,
              "tradingDayId": trading_day_id}],
})
session.update_forming(next_bar)
session.update_confirmed(closed_bar)
```

Use host ids before the first executed bar to opt into session-window mode.
Missing coverage raises `E_SESSION_COVERAGE`; extend the mapping and retry the
same bar. Extensions merge rows atomically, preserve prior rows, and reject
changes to executed confirmed/forming identities with `E_SESSION_HISTORY_CHANGED`.
Repeating identical rows is allowed. An extension does not execute a script,
reset risk counters, discard a forming result or change lifecycle timestamps.
Rust `HistoricalRuntime` and `RealtimeRuntime` expose the same extension operation.
Their replacement builder `with_session_windows` now returns `Result<Self, RuntimeError>`.

This document defines the implemented realtime bar model and its state partitions.

The historical runtime executes only closed bars. Realtime execution adds the
concept of a forming bar: the latest bar may be evaluated multiple times before
it is confirmed.

## Bar Update Kinds

The runtime model exposes three update kinds:

```rust
enum BarUpdateKind {
    Historical,
    Forming,
    Confirmed,
}
```

`Historical` means a closed bar loaded from history. It is equivalent to the
existing `HistoricalRuntime::append_bar` behavior.

`Forming` means an intrabar update for the current open bar. A forming update
must not commit series history. Before script execution, ordinary user state and output roll back to the
confirmed checkpoint; `varip` and successful broker events persist. An update
that does not execute the script retains its latest user state and output.

`Confirmed` means the final update for a realtime bar. Confirmed updates execute
like forming updates, then commit current series values to historical buffers.

## State Partitions

Realtime execution needs explicit state partitions:

- committed series history
- current update values
- output side effects for committed bars
- temporary output side effects for the forming bar
- persistent `var` state
- intrabar `varip` state
- live broker order, fill, position, risk and cash state
- callsite state for TA functions
- immutable request provider data
- deterministic request result cache

Rollback semantics must specify which partition is restored and which partition
survives repeated forming updates.

## Commit Rules

Only these updates commit series:

- `Historical`
- `Confirmed`

`Forming` updates are visible in the current evaluation but not visible through
positive history references on later evaluations until the bar is confirmed.

## Runtime API

Rollback execution is exposed through `RealtimeRuntime`:

```rust
let mut runtime = RealtimeRuntime::new(&hir);
runtime.update(BarUpdate::historical(bar))?;
runtime.update(BarUpdate::forming(partial_bar))?;
runtime.update(BarUpdate::forming(updated_partial_bar))?;
runtime.update(BarUpdate::confirmed(final_bar))?;
```

Those methods still return a complete `RuntimeResult`. A streaming path applies the same forming/confirmed lifecycle without constructing that snapshot:

```rust
let mut replica = runtime.replica();
let changes = runtime.apply_update(BarUpdate::forming(partial_bar))?;
replica.apply(&changes)?;
assert_eq!(replica.result(), &runtime.result());
```

`RuntimeChanges` schema 3 carries this-update series append/current-bar replace,
drawing tails and deletion, order/fill/alert changes, preview/confirmed visibility,
`baseRevision`, `revision` and the absolute display origin `retainedFrom`.
A cursor-bearing `RuntimeReplica` applies changes
in place. Identical retransmission returns false; stale, conflicting, wrong-schema
and missing revisions fail before mutation. After a gap, capture a producer
snapshot with its revision and reset the consumer; do not infer a missing base.
Hosts bind replicas to one stream and own cross-session routing/identity.

Python uses `session.replica()`, `apply_forming` / `apply_confirmed`, then
`replica.apply(changes)`. `apply_runtime_changes(replica, changes)` is the same
in-place operation and returns a bool; the unreleased dictionary-to-dictionary
helper is replaced. `session.stream_snapshot()` atomically captures a result and
revision and `retainedFrom` for replica construction or `replica.reset(...)`.
Preserve all three fields when restoring a physically pruned stream:

```python
snapshot = session.stream_snapshot()
replica.reset(
    snapshot["result"],
    revision=snapshot["revision"],
    retained_from=snapshot["retainedFrom"],
)
```

Only explicit `replica.result()` constructs a complete Python dictionary.
Existing `update_forming`, `update_confirmed`, `result()` and `confirmed_result()`
retain their complete snapshot contracts (runtime output schema 8 unchanged).

A host-owned historical correction is `correct_historical(from_time, bars)` /
Python `session.correct(from_time, bars)` / WASM `correct(fromTime, barsCsv)`.
The session keeps confirmed bars with `time < from_time` and re-executes that
prefix plus the supplied suffix from a blank runtime that preserves the same
program, inputs, request environment, request feed, magnifier and session
windows, then discards forming state. Confirmed request-feed extras that close
after the new last confirmed chart bar are dropped so replay cannot see
`barstate.islast` or HTF close times from a discarded tail. Later forming
request extras are kept for the next chart forming bar. An empty suffix
truncates from `from_time`.
`replay_historical` / `session.replay` still replaces the entire confirmed
history when the host already has the combined list. Failed correct/replay
restores the previous confirmed/forming snapshots, retained bars, clocks and
revision. Neither operation is a linear change; `last_changes` is cleared and
replicas must `reset` from the new snapshot.

Ordinary plot values/colors and alert history use a persistent append tree;
checkpoint updates copy a bounded leaf plus a logarithmic branch path. Drawing
deltas retain the entire mutable bar's suffix, since multiple snapshots can
exist on one bar. Alert differences retain occurrence counts for identical
`alert.freq_all` events. Source execution still rolls back and runs the current
bar; the output optimization does not change Pine execution scheduling.
Other output families, broker records and user-owned collections can still
incur copies; finite measured workloads do not establish indefinite bounded
retention. See [streaming acceptance](STREAMING_INCREMENTAL_AUDIT.md).

`RealtimeRuntime` internally keeps:

- a confirmed `HistoricalRuntime` snapshot
- an optional forming `HistoricalRuntime` snapshot

Each forming script execution restores ordinary user state from the confirmed
snapshot, while inheriting successful live broker state and `varip`. This rolls back:

- current update values
- uncommitted series values
- temporary output side effects
- `var` updates made during the previous forming execution
- callsite state changes made during the previous forming execution
- array storage mutations made during the previous forming execution
- label, line, box, and table creation, mutation, deletion, and cell snapshots
  made during the previous forming execution
- alert events made during the previous forming execution
- request cache entries and requested-context runtime state created during the
  previous forming execution

Request provider data is the immutable historical seed. A live session may also
append, replace, or confirm bars on a `RequestKey` through `apply_request_update`.
Forming requested bars are visible only while the chart bar itself is forming;
confirmed chart evaluation ignores them. Alignment still uses the existing
lookahead/gaps close rules, so a higher-timeframe forming bar cannot appear
before it closes. A failed request-feed update restores the previous feed,
cache, forming snapshot, and revision.

Confirmed and historical updates replace the confirmed snapshot and clear the
forming snapshot.

Forming `RuntimeResult` values include the current forming bar's alert events
so hosts can inspect the live update result. Those events are not visible
through `confirmed_result()` and are discarded by the next forming update unless
a confirmed update commits an alert event for that bar. Repeated forming
updates recompute alerts from the confirmed snapshot: if the first forming
update triggers multiple alert sites and a later forming update triggers fewer
or none, only the latest forming result contains alert events. A confirming
update commits the events produced by that confirmed execution and is expected
to match equivalent historical execution for the same final bar data.

## `var` and `varip`

`var` is supported under rollback. A `var` update made during a forming update
is temporary; the next forming update starts again from the last confirmed
snapshot. A confirmed update persists the new `var` value. Single
`chart.point` values follow this ordinary `var` rollback path, including field
mutation on the current point value.

Scalar, single chart-point value, scalar typed-array, same-local scalar-tree UDT
array, scalar map, and supported matrix `varip` declarations are supported as a
separate intrabar persistence path. The first forming update for a bar starts
from the confirmed snapshot. Later forming updates for that same bar seed
`varip` slots from the previous forming update. For supported array ids referenced by `varip` slots,
the runtime also copies the previous forming backing array contents, element
kind, and UDT element metadata, and advances the next array id past retained
ids. For supported map ids referenced by `varip` slots,
the runtime copies the previous forming backing map contents and advances the
next map id past retained ids. For supported matrix ids referenced by `varip`
slots, the runtime copies the previous forming backing matrix contents and
advances the next matrix id past retained ids. Ordinary `var`, non-`varip`
arrays/maps/matrices, drawing objects, outputs, request caches, callsite state,
and history reads stay on the confirmed rollback path. A confirmed update also
seeds from the latest forming `varip` slots before executing, then stores the
resulting values in the confirmed snapshot for the next bar.

Historical execution treats the supported `varip` subset like `var` because
historical bars have one committed evaluation. Local scalar declaration sites
inside `if`, `for`, `while`, and UDF bodies initialize only when first reached;
each lowered scalar UDF callsite has independent storage. Branch-local
scalar-array `varip` declaration sites initialize on first reach, but array
mutation inside UDFs remains rejected by the existing function side-effect
rules. Drawing object ids are rejected for `varip` before runtime because
retaining only an id would become dangling when the label, line, box, or table
object store rolls back. Tuples, non-scalar UDT arrays, and value families
outside the fixture-backed `varip` subset remain rejected with compatibility
diagnostics instead of being approximated.

## Current Status

Phase 7 defines the model and implements rollback for repeated forming updates.
Phase I closes the fixture-backed scalar and scalar typed-array `varip` subset
described in `docs/PHASE_I_AUDIT.md`, with later slices adding scalar map,
runtime-owned matrix, and same-local scalar-tree UDT array backing-store
handoff for `varip`. Phase H adds
fixture-backed alert forming event rollback. Realtime fixtures cover temporary
output rollback, alert event rollback, drawing-object lifecycle rollback for
labels, lines, boxes, and tables, `var` rollback, scalar, chart-point value,
scalar typed-array, same-local scalar-tree UDT array, scalar map, and
runtime-owned matrix `varip` intrabar persistence, stateful TA
callsite rollback inside conditional branches, array, map, and matrix rollback,
request provider immutability and cache rollback, and dynamic history reads
from confirmed history during forming updates.

Next work:

- broaden realtime fixtures for more stateful built-ins and nested scopes

## Observed realtime broker ticks

A forming or confirmed update supplies an observed close and OHLC range.
For pending market entries and closes, a same-timestamp update that expands
exactly one extreme uses that new high or low. First observations, unchanged
ranges and simultaneous high/low expansion use close; price-condition orders
continue to evaluate close. The two-sided fallback is not native-qualified.
See [the scoped market-order correction](REALTIME_MARKET_EXTREME_AUDIT.md).
OHLC fields remain available to the script but are not replayed as historical
price paths. Supply every required price observation through the host adapter;
the core cannot reconstruct unobserved fills from candle summaries. Pending
orders may execute even when `calc_on_every_tick=false`. A fill-triggered script
execution replaces the ordinary pass for that observation. `process_orders_on_close`
uses a confirmed closing update, not every forming update. Native multi-order
coverage and remaining limitations are recorded in LIVE_TICK_REFERENCE_AUDIT.md.


## Retention and requested-context qualification

Physical display pruning uses storage-relative indexes while Pine and delta bar
indexes remain absolute. Growing or clearing a retention limit does not move
an already pruned origin backwards; use replay plus snapshot reset to recover
expired output. Eligible requested expressions reuse a checkpoint before the
terminal requested bar; complex and dataset-end-dependent expressions retain
full evaluation. See [streaming expansion](STREAMING_EXPANSION_AUDIT.md) for
current verification, measured workloads and explicit remaining boundaries.

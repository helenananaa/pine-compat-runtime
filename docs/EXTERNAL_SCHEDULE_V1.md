# Explicit external execution schedule (development schema 1)

This host-neutral Rust/Python contract separates the script's bar state from
the completion of a market observation. It is the prerequisite for a Host
historical fill scheduler and realtime strategy ticks, not an implementation of
a concrete broker clock. Detect `EXTERNAL_SCHEDULE_SCHEMA_VERSION == 1` in Python.
Development artifacts retain package version 0.3.2; identify them by hash.

## Python

```python
schedule = [[{
    "evaluation": {
        "bar": bars[0],
        "account": accounts[0],
        "event_time_ms": 59999,
        "confirmed": True,
    },
    "context": {
        "bar_state": "historical",
        "trigger": "bar_close",
        "is_new": True,
    },
}]]
result = program.run_external(bars, accounts, execution_schedule=schedule)
session = program.external_session(scheduled=True)
assert session.append(bars, accounts, execution_schedule=schedule) == result
```

Supply one nonempty group per chart bar, with at most 65 evaluations per group.
The existing `ExternalPass` fields live inside `evaluation`; optional
`request_data` has the same provider-replacement behavior as legacy passes.
`execution_schedule` and `execution_passes` are mutually exclusive. A session's
schedule mode is fixed at construction, including empty appends. Explicit mode
requires `execution_schedule=[]` on an empty append.

Rust exports `ExternalScheduledPass`, `ExternalPassContext`, `ExternalBarState`
and `ExternalEvaluationTrigger`. Use `with_external_schedule(frames, schedule,
options)` and `append_external_schedule(bars, frames, schedule, execution_times)`.
Existing `ExternalPass`, `ExternalExecutionOptions` and legacy methods retain
their public shapes. WASM exports `externalScheduleSchemaVersion()` as metadata;
there is no WASM external-account executor.

## Context and validation

`bar_state` is constant within a group:

| State | Script state |
| --- | --- |
| `historical` | `ishistory=true`, `isrealtime=false`, `isconfirmed=true`, `isnew=true` on every pass |
| `realtime` | `ishistory=false`, `isrealtime=true`, `islast=true`, `isconfirmed=evaluation.confirmed`, `isnew=context.is_new` |

Realtime `is_new=true` is allowed only on a group's first evaluation. A strategy
that first executes after an opening update can supply `false` even there.
The last historical group before a known realtime group has
`islastconfirmedhistory=true`. Historical dataset-end variables otherwise retain
existing batch/streaming endpoint semantics; incremental append does not
retroactively recompute earlier endpoints. After entering realtime state, a
schedule cannot return to historical state without reconstructing the runtime.

| Trigger | Required declaration / observation |
| --- | --- |
| `bar_close` | `evaluation.confirmed=true` |
| `order_fill` | `calc_on_order_fills=true` |
| `realtime_tick` | realtime state and `calc_on_every_tick=true` |
| `closing_fill` | a prior committed observation in the group, `calc_on_order_fills=true`, `process_orders_on_close=true`, `closing_ticks=True` |

`evaluation.confirmed` closes the market observation. Historical fill callbacks
before the closing observation use `false` here while Pine still sees
`barstate.isconfirmed=true`. Only the final group may end uncommitted. Append
cannot extend that bar: reconstruct its complete transcript before proceeding.
Passes after a committed observation must be `closing_fill`; their bar, event
timestamp and requested streams must be identical to that observation. Accounts
can change after an actual Host fill.

The core validates shape, trigger admission, finite feedback, timestamps and
immutable closing observations. The Host remains responsible for the truth of
each trigger, actual fills, final versus evolving OHLCV visibility, market-data
causality, matching, persistence and replay. Passing a context does not prove
that a fill occurred. Existing fixed-sizing, cost, magnifier and risk admission
rules still apply. Risk and closing execution require their separate opt-ins.

Each pass restores the prior committed ordinary-variable and TA checkpoint;
`varip` survives passes. Only the final plot/series values commit for the chart
bar, while intents preserve every pass index. Alerts with close frequency wait
for realtime confirmation. Resource budgets remain shared per chart bar.
External execution emits intents and consumes authoritative accounts without
creating native broker fills. Python parse/mode errors leave a session usable;
errors returned by runtime append poison the Python session, as in the legacy
API. Rust preflight rejection preserves runtime state.

Rollback includes referenced arrays, slices, maps, matrices, UDT fields and
drawings. The same sparse heap roots and varip object-transfer rules used by
realtime replacement preserve retained contents without cloning the whole
runtime or scanning scalar collection payloads. Stateful price/volume builtins
are recomputed from the prior bar for each external observation. Speculative
drawings disappear between passes; object allocation identities remain monotonic.

## Qualification boundary

The separation follows the current official [execution model](https://www.tradingview.com/pine-script-docs/language/execution-model/)
and [strategy calculation settings](https://www.tradingview.com/pine-script-docs/concepts/strategies/#calc_on_every_history_tick).
Core tests and installed-binding tests exercise state, rollback, account
authority, alert timing, batch/append equivalence and invalid schedules.
These checks do not establish TradingView output parity or a connected Host
BAR/tick scheduler. `calc_on_every_history_tick` declaration/native scheduling,
lower-timeframe paths and arbitrary historical-detail support remain work to do.

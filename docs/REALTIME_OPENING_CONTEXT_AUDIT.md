# Realtime opening observation context

Status: Windows development-artifact qualification passed, 2026-09-10.
Base runtime: `a6a28528e`.

## Native observation

The `Realtime Update Trace 64` control retains its first 64 executions in a
fixed three-bar window. Fifteen fields include observed OHLCV, execution clock,
bar time, flags, execution count, positions, trade count and entry/exit prices
and times. Browser records have a contiguous sequence 1 through 64 and no
truncation. The final 15-field record agrees exactly with an independent chart
CSV export. Raw source and records are in
`.local/tv-goal-20260910/live-r4-trace/`.

The control was actually attached after the opening of its first capture bar.
Its first native execution is at 10:19:25.020 UTC for the 10:19 bar, with
`barstate.isnew` false. The existing session inferred true because this was
its first forming observation. Replay compares 975 values (one inert historical
seed plus 64 records): exactly one flag differs. All 32 fill prices and recorded
entry/exit times agree. This batch did not contain the same-close/new-extreme
geometry from the unresolved price samples, so it does not settle that question.

The wire transport's `1e100` unavailable-value encoding is normalized only for
absent trade identity fields in this control. It is not supplied as market
input. Source initializes the record to `na`; native values and tolerances
remain unchanged.

## Host-neutral contract

Rust adds `RealtimeUpdateContext` and `RealtimeRuntime::update_with_context`.
Python forming and confirmation updates accept optional `opening_update`.
Existing calls retain first-observation inference. A host attaching to an
already open bar can explicitly provide false. This conveys a lifecycle fact;
the runtime does not guess it from the execution clock or price.

The marker changes Pine's opening-update context, not the bar's storage index
or commit lifecycle. Historical bars retain their required opening status.
An opening marker cannot repeat after a forming observation exists. Invalid
metadata fails before mutation. Python accepts actual booleans; `None` retains
the old inference. Result and session schema versions are unchanged because
the input addition is optional.

```python
snapshot = session.update_forming(
    forming_bar, execution_time=host_clock_ms, opening_update=False
)
```

The host still owns history and observation completeness. This does not
synthesize missed updates or fix unresolved market-price observations.

## Qualification

Before this addition, the installed candidate rejects the keyword and the
unchanged native trace has one startup-flag difference. New tests cover varip
continuity, unchanged prior results, duplicate-opening rejection, historical
context, strict types and close-only observations. Full Windows qualification
passes 6676 Rust / 717 fresh installed-wheel Python / 130 tool tests and actual
WASM. A separately installed retained wheel replays the same native source and
records with `opening_update=False` only for the known mid-bar attachment:
975/975 values pass. No other input or expected output changed.

Final source commit `6c31e2b22c1c7f27a42db4d6df3da76ee425d791` also passes
Ubuntu-native qualification: 6676 Rust / 717 installed-wheel Python / actual
WASM; 130 tool tests run with one Windows-only skip. Separately built Windows
and Linux optimized release wheels each pass all 717 installed tests and the
same 975-value native trace. Complete release CLI output for the native r1
commission companion agrees across Windows, Linux and the qualified baseline.
The Linux release wheel is manylinux_2_35_x86_64; this does not qualify a new
manylinux2014 distribution. Logs, artifacts and hashes are retained under
`.local/tv-goal-20260910/`, indexed by `final-evidence-manifest.json`.

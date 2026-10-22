# External per-observation execution

`EXTERNAL_STREAM_SCHEMA_VERSION=1` adds a persistent same-bar lifecycle in Rust
and Python. The runtime evaluates only newly submitted observations. The
embedding host remains responsible for orders, matching, accounts, event clocks
and recording the input transcript. No concrete host service is part of Core.

Create a scheduled external evaluator, open a chart bar, submit its scheduled
observations, then commit its history once. Rust exposes `begin_external_bar`,
`observe_external_bar` and `commit_external_bar`; Python `ExternalSession`
exposes `begin_bar`, `observe` and `commit_bar`. Bar and event times use Core
milliseconds. Each observation carries its actual account and explicit context.

Historical opening headers contain canonical OHLCV. Historical observations stay
inside that envelope and the confirmed observation equals it. Realtime opening
headers contain the current observed tuple. Realtime last-bar metadata advances
with each observed chart bar; a complete batch does not expose a later chart
boundary early. Static request roots also follow the observed realtime tuple.

Historical requested updates may reveal data through the canonical chart close
at every synthetic vertex. This preserves ordinary historical merge semantics,
including completed lower-timeframe arrays, while chart/account observations
remain progressive. Data beyond that chart boundary is rejected. Realtime
updates remain limited to the actual observation time. Bounded fresh TradingView
samples cover completed bars with same/lower/higher requested contexts; they do
not qualify arbitrary feeds or scripts.

Known requested calculation windows have a separate metadata input. Rust's
`initialize_external_stream_dataset_with_requested_history` and Python's
`initialize_history(..., requested_history=...)` accept exact symbol/timeframe
keys and ordered `times_ms` arrays. They anchor initial requested windows to
the full known endpoint rather than the first observed provider prefix. No
prices, account frames or Pine executions are supplied by these headers. Active
providers still receive only causally available updates. Nested requested
evaluators retain the same header plan. Duplicate keys, unordered times and
conflicting root metadata reject atomically. Python and WASM expose requested
window-plan schema 1; the existing JS binding still has no external-account
session API. Dynamic historical discovery still needs observed-feedback/rebase
coordination and is not enabled by this metadata-only capability.

Observed window feedback is a separate opt-in capability (schema 1). Rust
`initialize_external_stream_dataset_with_observed_feedback` and Python
`initialize_history(..., observed_feedback=True)` resolve static reservations
without running Pine or fabricating account frames. `window_status()` reports
whether an actual-observation discovery pass is required. A window change
interrupts the current execution; status remains readable and
`rebase_window(seal=False)` resets the entire tentative Pine state and initial
requested feed while retaining observed context reservations. All tentative
intents and Host matching state must be discarded by the caller. No failed
observation's partial intents may be settled.

After a complete, uninterrupted pass over the exact planned historical rows,
`rebase_window(seal=True)` starts a fresh final execution with the discovered
plan. The host must complete discovery before publishing durable prefixes.
Partial passes and ordinary script failures cannot be sealed or reset. Later
window drift still interrupts and requires discovery again. This opt-in mode
uses begin/observe/commit exclusively; legacy batch appends are rejected.
Core owns only reservations, control status and deterministic reset. It never
replays a Host ledger, performs matching, or supplies future account feedback.
Host orchestration, recovery journals, attempt limits, deadlines and resource
qualification are separate responsibilities. WASM exposes the capability number
but still has no JS external-account session API.

Ordinary variables, referenced heaps, TA and price flow roll back between
observations; varip persists. Market confirmation does not end the input group:
explicit closing-fill feedback can still arrive before the host commits. The
commit stores Pine series once. It does not perform Host matching or synthesize
account feedback. Existing observation/pass and fill-loop budgets still apply.

For a known historical dataset, Rust
`initialize_external_stream_dataset` or Python `initialize_history` plans its
endpoint and static request windows before any chart execution. It returns the
input prefix skip; subsequent cursor and intent indices are relative to the
retained dataset. No future account frames are supplied or invented. Invalid
planning input preserves the original evaluator. Unresolved dynamic window
discovery is rejected: its observed-feedback/rebase path still needs host
integration, rather than running a disposable native broker as a substitute.

Python observations return only newly emitted intents with `intent_base`,
`bar_cursor`, `bar_open` and `stream_schema`. `snapshot=False` omits the full plot
snapshot. `commit_bar(snapshot=False)` returns its cursor/empty intent delta.
`snapshot()` explicitly retrieves complete output; `include_intents=False`
avoids resending previous intents. These controls bound per-message history
serialization; they do not themselves qualify long-session memory or latency.

Invalid begin/observation/commit input is retryable. Execution failures poison
the evaluator and keep the failure state for inspection; Python refuses further
output snapshots. Legacy append keeps its existing failure policy. An append
attempt while a streaming bar is open is rejected before entering that route.
Committed bars can still use the legacy append API. A new evaluator reconstructs
from a host-recorded transcript; no external broker settlement is repeated by
Core. Recording, restart coordination and durable ledger recovery remain host
responsibilities.

Development checks cover complete/streamed equivalence, rollback, arrays,
authoritative feedback, closing fills, requested transactions between passes,
clone continuation, rejected-input retry, execution failure and 2,048
observations. Independent generic realtime controls expose and lock the repaired
last-bar boundary behavior. Rust/Python targeted receipts do not substitute for
the combined full gate, application clock drivers, resource qualification or a
packaged application. The WASM surface exposes the capability number; its
existing JavaScript binding still has no external-account session API.

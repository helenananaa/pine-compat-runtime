# Incremental external evaluation

The Python `Program.external_session(...)` API retains interpreter state between
strictly forward batches. It uses authoritative account frames supplied by the
caller and does not run the native broker. This is an unreleased addition to the
development branch; package metadata remains aligned with main. Identify local
wheels by source revision or artifact hash rather than package version alone.

```python
program = pine_compat.compile_script(source)
session = program.external_session(
    input_overrides=None, chart_symbol="EXCHANGE:SYMBOL", chart_timeframe="1",
    request_bars=None,
)
result = session.append(new_bars, matching_account_frames)
assert session.cursor == len(new_bars)
```

`result` uses the existing `external-broker/1` envelope: protocol, pyramiding,
intents, and output. Intents and graphics remain cumulative snapshots. Only the
new batch executes. Base request inputs and input overrides are frozen at
construction. Explicit `execution_passes` request-data snapshots replace only
the supplied request streams; chart metadata, calendars and currency rates remain
bound to the session. Sessions are independently owned, process-local and nonpersistent.
Create a new session to replay corrections or changed inputs. An append rejected
by the core or failing during execution poisons the Python session, preventing
use of partially advanced state. Argument decoding errors raised before entering
the core leave the session unchanged and may be corrected and retried.

`append(..., execution_passes=groups)` supports the existing fill-recalculation
contract. Each bar requires matching account feedback; appended timestamps must
increase strictly. An unconfirmed final pass must be replaced by a fresh replay
before another bar can append. Empty appends return the current snapshot, even
for fill-pass sessions, without requiring an empty `execution_passes` argument.

Development builds exposing `EXTERNAL_CLOCK_SCHEMA_VERSION == 1` also accept
`execution_times=[...]` on both `Program.run_external` and `ExternalSession.append`.
Supply one integer millisecond execution timestamp per chart bar, distinct from
its opening timestamp. This supplies `timenow` without consulting a system clock.
Scheduled fill passes use their own `event_time_ms`; those timestamps take
precedence within each pass. Rust callers use
`append_external_bars_with_execution_times` for the same append behavior or
`append_bars_with_execution_times` after binding batch account feedback.

`EXTERNAL_CLOSING_TICK_SCHEMA_VERSION == 1` exposes the keyword-only
`closing_ticks=True` option at session construction and on batch evaluation.
It permits the `process_orders_on_close` declaration and, with scheduled fill
passes, repeated confirmed calculations at the same immutable market state.
The Host still owns scheduling and accounting. See the
[closing-tick contract](EXTERNAL_BROKER_V1.md#execution-passes-and-requested-data)
and `python/tests/test_external_closing_ticks.py`. Ordinary sessions default to
the previous fail-closed behavior.

Streaming dataset endpoints are not retrospective prefix replay. A script using
`barstate.islast`, `last_bar_index`, visible chart bounds, or initial-window
selection can produce different historical values when the whole dataset is
replayed with a new endpoint. Hosts preserving full-prefix semantics must detect
those dependencies and rebase instead of silently switching their meaning.
`Program.calculation_bars_count` exposes the declared positive initial window (or
None). It describes the declaration, not an effective window resolved from all
request contexts. The first batch supplies the complete initially known dataset;
initial-window selection slices chart bars and account feedback together once.
Later appends retain that origin. The cursor counts chart bars from successful
appends, including the initial skipped prefix, while output and intent indices
count executed bars.

Rust hosts bind an empty feedback vector with `with_external_accounts` (or
`with_external_accounts_and_passes` for fill passes), then call
`append_external_bars`. If feedback was already bound for batch execution, consume
it before appending more. Rust preflight validation failures leave state unchanged;
execution failures after binding the new feedback poison the runtime.

Validation lives in `python/tests/test_external_session.py`: batch equivalence
for prefix-stable series/TA/state and authoritative feedback, fill passes,
independent sessions, correction rejection, and poisoned-session behavior.
Rust API regressions are in `crates/pine-runtime/tests/external_session.rs`.

The Rust runtime and Python `ExternalSession` expose
`failed_request_context()` after an evaluation failure. It returns the exact
`(symbol, timeframe)` of a missing requested provider, including a miss reached
inside a nested requested expression. Other failures return `None`; a user
`runtime.error()` containing the same diagnostic text remains an ordinary script
failure. This read-only status leaves the session poisoned. A host that supplies
the missing dataset must discard its entire tentative execution and construct a
fresh evaluator against the amended immutable input plan. Reading this status
does not authorize a retry on the failed runtime or revise committed decisions.

`ExternalSession.initialize_realtime_history(bars, requested_history=None)`
reserves initially known request endpoints with observed window feedback, while
keeping chart values and chart bounds observation-driven. Supply completed
canonical rows solely for window selection and input validation. Unlike
`initialize_history`, this mode never preloads canonical chart prices into static
requests or exposes the final chart endpoint to earlier realtime observations.
`begin_bar(..., bar_state="realtime")` accepts partial tuples within the frozen
OHLCV bounds; each confirmed tuple must exactly match its planned canonical row.
Validation errors before execution leave the observation retryable. Initialization
is allowed once on a fresh scheduled session. Rebase discards tentative evaluator
state; hosts must also discard the corresponding account ledger and decisions.
Seal a complete stable pass before extending beyond the known rows. Dynamic
request contexts first encountered on realtime bars retain the existing rejection
policy; endpoint metadata does not count as historical script execution.

On 2026-10-19 the explicit-clock update passed the complete Windows
`scripts/verify.ps1` gate with Rust 1.97.1, Python 3.12.7 and Node 22.14.0:
7,901 Rust tests, 1,686 tests against a freshly built and installed Python wheel,
172 tooling tests, generated WASM/Node execution, formatting, Clippy, source
structure and binding golden checks. The focused external-broker/session Python
suite passed 41 cases. An initial gate attempt reused a syntax-fixture executable
with an embedded path to a removed checkout; clearing only that package's Cargo
artifacts and rebuilding restored all 27 fixture reads before the successful
complete rerun. These are controlled runtime and binding checks, not release,
host-deployment or TradingView differential qualification.

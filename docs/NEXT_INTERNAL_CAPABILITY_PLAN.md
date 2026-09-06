# Next Internal Capability Plan

Status: active planning document, refreshed on 2026-09-06 after mixed-family
OCA, host-neutral session window, and ordinary-chart inter-bar gap closeout.
Stage D inventoried original strategy fixtures and stopped: no independent
Tester-backed accepted-but-wrong slice is available.
Strategy broker accuracy remains the selected direction while further
source-version expansion is paused.

This document groups the next interpreter-internal work into seven large task
directions. It does not claim new compatibility. A task becomes supported only
after the matching syntax, semantic analysis, runtime behavior, fixtures,
conformance metadata, snapshots, docs, and release verification are complete.

## Selection Rules

- Pick one small slice from one direction at a time.
- Prefer work that can be proven with local fixtures and host-neutral JSON.
- Keep public output fields unchanged unless the slice explicitly designs a new
  contract.
- Keep unsupported variants rejected with diagnostics.
- Do not use roadmap wording as support evidence. Use
  `tests/fixtures/conformance.tsv`, snapshots, audit docs, and verification
  results.

## Direction 1: Strategy Maintenance

Goal: improve the accuracy and maintainability of the completed side-aware
broker subset without widening unsupported behavior ahead of executable
evidence.

Current Stage 17-22 baseline:

- one shared fill-transition path with ledger/aggregate invariant checks;
- next-tick market closes, close-command `immediately`, and historical
  `process_orders_on_close`;
- long/short generic-order signed netting and price-based entry reversal;
- fixture-backed OCA none/cancel/reduce subsets, mixed entry/order/exit
  groups, and unified cancellation;
- bounded `calc_on_order_fills`, realtime rollback, and
  `calc_on_every_tick`;
- fixture-backed entry-direction, position-size, drawdown, intraday, and
  consecutive-loss-day risk rules.

Stage 18g and Stage 23 are closed. Historical price entries, generic orders,
exits, and margin calls share one OHLC-path event loop. Named const bool
`use_bar_magnifier=true` walks host-owned lower-timeframe bars through that
same path. Mixed-family OCA, host-neutral session window input, and
ordinary-chart inter-bar gaps are closed. Remaining strategy work is
corpus-driven follow-up from legal v5/v6 samples.

Active stage order:

1. Inventory a legal, authorized, de-duplicated v5/v6 corpus and classify
   failures by root cause.
2. Close at most one evidenced accepted-but-wrong slice, or stop with an
   explicit blocker.

The closed Stage 23 record is
`docs/STRATEGY_INTERNAL_STAGE23_BAR_MAGNIFIER_FILL_WIRING_AUDIT.md`.

Keep out of scope until separately designed and fixture-backed:

- Series `oca_name`.
- Omitted `qty` for unsupported `strategy.short` order forms.
- Currency conversion, symbol precision, and richer account constraints.
- Arbitrary future binding for unmatched `from_entry` ids.
- Public pending-order, reservation, remaining-quantity, or exit-reason records.
- External strategy alert delivery before the host-owned restart-safe durable
  attempt-store, executable retry scheduling, concrete authentication secret
  store, diagnostic emission, and failure-reporting model from
  `docs/STRATEGY_EXTERNAL_ALERT_DELIVERY_ADAPTER_PLAN.md` is implemented.

Recommended next slice: an authorized v5/v6 strategy with frozen bars and
independent Tester output, then one accepted-but-wrong root cause. The Stage D
blocker record is `docs/STRATEGY_MODERN_CORPUS_BEHAVIOR_AUDIT.md`. The closed
ordinary-chart gap record is
`docs/STRATEGY_INTERBAR_GAP_BEHAVIOR_AUDIT.md`.
Omitted `from_entry` allocation remains FIFO and `strategy.close_all()`
remains independent of `close_entries_rule`. Do not add public pending-order
fields or widen conformance without runtime behavior and host-parity evidence
in the same slice. The closed mixed-family OCA record is
`docs/STRATEGY_MIXED_OCA_BEHAVIOR_AUDIT.md`. The closed session-window record
is `docs/STRATEGY_SESSION_RISK_BEHAVIOR_AUDIT.md`.

The step-by-step implementation sequence, acceptance gates, and later session,
gap, and corpus work are in
[Strategy Accuracy Next Execution Plan](STRATEGY_ACCURACY_NEXT_EXECUTION_PLAN.md).
That plan does not change current compatibility claims.

Closed maintenance slice:

- Supported `strategy.exit` while-flat no-op behavior is covered for
  single-trigger stop/limit/profit/loss, stop+limit, stop+profit, and
  loss+limit/loss+profit bracket, and trailing stop shapes by runtime fixtures,
  golden snapshots, conformance matrix entries, and Python/WASM host parity
  tests without exposing pending-order or reservation internals.
- Supported explicit wrong-entry `strategy.exit` no-op behavior is covered for
  single-trigger stop/limit/profit/loss, stop+limit, stop+profit, and
  loss+limit/loss+profit bracket, and trailing stop shapes by runtime fixtures,
  golden snapshots, conformance matrix entries, and Python/WASM host parity
  tests.

## Direction 2: Built-In Coverage

Goal: increase the number of ordinary indicator scripts that run by adding small
fixture-backed built-in subsets.

Good next slices:

- Additional common `ta.*` helpers.
- Additional `math.*` helpers and edge-case coverage for existing helpers.
- Additional `str.*` formatting and parsing helpers.
- More time, timeframe, session, symbol, and color helper subsets.
- Better diagnostics for known unsupported built-in argument forms.

Keep out of scope until separately designed:

- Built-ins that require host market data, account state, chart UI state, or
  remote services.
- Approximate behavior without fixture evidence.
- Broad claims such as "all `ta.*`" or "all string formatting".

Recommended first slice: choose one high-use built-in family from real fixture
gaps, add only the smallest documented subset, and update the matrix only for
that fixture-backed subset.

## Direction 3: Arrays And Collections

Goal: make the current typed-array support more complete while preserving clear
storage lifetime and mutation rules.

Good next slices:

- More array functions or method-call aliases for existing scalar element types.
- More array behavior inside branches, loops, and user-defined functions.
- More array/history/state interaction fixtures.
- Better diagnostics for unsupported element types and invalid mutations.

Keep out of scope until separately designed:

- UDT array behavior beyond the fixture-backed same-local and same-imported
  scalar-tree subsets, especially non-scalar, nested-collection, or recursive
  UDT arrays.
- Drawing-object and `chart.point` array behavior beyond the fixture-backed
  object-id and value-array subsets.
- Map and matrix behavior beyond the current fixture-backed typed subsets.
- Behavior that requires object identity or lifetime rules not already modeled.

Recommended first slice: add one missing array helper for already-supported
scalar arrays, with branch/loop/UDF interaction fixtures if the helper mutates
state.

## Direction 4: User-Defined Types And Methods

Goal: expand the local and imported structured-data subsets without weakening
source-scoped type identity or method resolution.

Recent closure:

- Local and imported scalar-tree UDT values now have fixture-backed
  construction, typed declarations, selected control-flow results, history,
  ordinary `var`, scalar-tree `varip`, and same-identity array subsets.
- Pure local and imported UDT methods now have fixture-backed receiver,
  parameter passthrough, alias, nested-method passthrough, constructor helper,
  and selected control-flow return coverage.

Good next slices:

- Clearer diagnostics for unsupported imported UDT and imported-method tails,
  nested mutation, and side-effecting methods.
- One additional imported-method or imported-UDT value-flow slice outside the
  current scalar-tree subset.
- Negative fixture maintenance for unsupported method parameter families or
  mismatched local UDT identity.
- Audit-only sync work when UDT/method matrix, semantic docs, and runtime
  fixtures drift.

Keep out of scope until separately designed:

- Imported UDT identity beyond the current same-imported-identity scalar-tree
  value, history, array, and method subsets.
- Imported methods beyond the current pure scalar-tree receiver/parameter/return
  subset.
- UDT arrays beyond the fixture-backed same-local and same-imported scalar-tree
  subsets, especially non-scalar, nested-collection, or recursive UDT arrays.
- Object-backed non-scalar UDT `varip` values and broader UDT history shapes.
- Side effects inside methods.

Recommended first slice: one diagnostics fixture/message improvement for an
unsupported UDT or method boundary, or one narrow imported-method value-flow
slice that keeps the current side-effect and non-scalar collection boundaries.

## Direction 5: Request Support

Goal: widen request behavior only where the runtime can stay deterministic and
host-provided data can prove the result.

Good next slices:

- More same-symbol or exact-key higher-timeframe `request.security` cases.
- More supported scalar expression shapes inside requested contexts.
- More alignment and gap-handling fixtures.
- Better host parity coverage for CLI, Python, and WASM request data injection.

Keep out of scope until separately designed:

- Lower-timeframe array-returning requests.
- Broad `request.*` families.
- Remote data lookup inside core crates.
- Requested-context strategy state or side effects.

Recommended first slice: one additional deterministic higher-timeframe
alignment case backed by request fixtures and host parity tests.

## Direction 6: Drawing Objects

Goal: add useful drawing lifecycle behavior while keeping the output
host-neutral.

Good next slices:

- Finish declaration-driven drawing object-count eviction beyond the now-backed
  label, box, line, and polyline lifecycle subsets if another drawable family is
  added.
- More `label.*`, `line.*`, `box.*`, and `table.*` methods.
- More deletion, mutation, no-op, and runtime-limit fixtures.
- More realtime rollback fixtures for already-supported drawing families.
- Better diagnostics for unsupported coordinate modes and invalid ids.

Keep out of scope until separately designed:

- General polyline arrays until the id-array behavior is deliberately designed
  and fixture-backed.
- Host-specific visual layout, drag behavior, or chart interaction.
- Drawing behavior that cannot be represented in the current JSON contract.

Recommended first slice: one missing method from an already-supported drawing
family, with runtime snapshot and realtime rollback coverage if it changes
state.

## Direction 7: Alerts

Goal: make alert events more useful while keeping alert behavior deterministic
and local to runtime output.

Closed slice:

- The local const-string alert frequency subset is fixture-backed for
  `alert.freq_once_per_bar`, `alert.freq_all`, and
  `alert.freq_once_per_bar_close`.
- Dynamic string-compatible `alert()` messages are fixture-backed through
  runtime snapshots and direct Python/WASM host parity assertions, while
  Pine-source `alert()` placeholder interpolation remains unsupported.
- Alert diagnostic fixtures cover dynamic frequency expressions, unknown
  const-string frequency values, and alertcondition placeholder rejection.
- Realtime alert-frequency rollback fixtures cover default once-per-bar
  suppression and `alert.freq_all` repeated-call emission across forming and
  confirmed updates.

Good next slices:

- Additional alert argument validation and diagnostics beyond the current
  message/title/frequency boundary.
- More branch, loop, and realtime rollback fixtures beyond the current alert
  policy and frequency rollback coverage.
- Better parity tests for alert output through CLI, Python, and WASM.
- Use `docs/STRATEGY_RUNNING_ALERT_CONFIGURATION_PLAN.md` before any external
  strategy alert delivery work.

Keep out of scope until separately designed:

- Sending webhooks, email, push notifications, or other external delivery.
- TradingView-style placeholder interpolation.
- Host scheduling, throttling, or user notification policy.

Recommended next slice: choose alert work only when it is fixture-backed and
does not require external delivery or host scheduling.

## Recommended Order

1. Integrate and review the current Stage 17-22 worktree.
2. Strategy Stage 18g true OHLC-path ordering.
3. Bar Magnifier fill wiring on the shared scheduler path.
4. Mixed-family OCA and instrument-session semantics through separate slices.
5. Strategy reporting/account gaps selected from executable fixtures.
6. Built-in, collection, UDT, request, drawing, and alert maintenance only when
   it is a prerequisite for the active strategy slice or fixes a regression.

This order applies while strategy completion is the selected project direction.
It does not make later stages supported early: each stage still closes only
through its fixtures, conformance evidence, audit, host parity, and full gate.

# Observed requested-data updates

Development foundation for phase 3, 2026-10-22. The additive capability is
`EXTERNAL_REQUEST_UPDATE_SCHEMA_VERSION = 1`. Host integration, complete gates,
resource qualification and independent reference comparison remain pending.
This is a pure runtime contract; acquisition, persistence and scheduling belong
to the embedding host.

`ObservedRequestUpdate` carries exact nonempty `symbol` and `timeframe` selectors,
an OHLCV `bar`, a `kind` (`historical`, `forming` or `confirmed`), and an inclusive
`available_at_ms`. A confirmed bar cannot be available before its actual
calendar's nominal close minus one millisecond. Forming observations cannot
precede their open. Availability must not exceed the observation clock or move
backward for a stream. Confirmed prefixes cannot be rewritten. Unknown stream
keys may be introduced by their first real update; known empty contexts can be
registered in the initial request provider.

Rust external evaluators expose `apply_external_request_updates(updates, clock)`;
Python external sessions expose `apply_request_updates(updates, observed_at_ms)`.
This is a feed-only transaction. A malformed key, future observation, invalid
OHLCV, duplicate/retroactive confirmation or invalid later element rejects the
entire transaction without changing feed, caches, account decisions or output.
These rejections do not poison a Python session. Errors during script execution
retain the established poisoned-session behavior. Observation clocks are integer
milliseconds; Python booleans and floats are rejected.

Supply one transaction before appending the next chart observation. Ordinary
closed-bar evaluation uses its execution timestamp, or its nominal chart close
minus one millisecond when no timestamp is supplied. An append cannot move
behind the accepted request observation. Existing committed outputs are never
replayed or edited by a feed-only update.

Scheduled evaluators instead attach `request_updates` to each `ExternalPass`.
The full appended transcript is preflighted before feedback is committed; updates
are applied only as that pass executes. Explicit historical schedules observe
their completed canonical chart bar, even when the interpolated fill timestamp
is earlier. Realtime and legacy observed-print passes use their event timestamp.
Closing-fill callbacks cannot introduce new requested market observations.
The older per-pass `request_data` replacement format remains supported; the two
formats cannot be mixed within one evaluator.

The initial provider and symbol metadata remain immutable. The feed stores new
confirmed bars in persistent append history and replaces only its unconfirmed
tail. Fallback expression caches are invalidated across contexts, since a nested
request may depend on the changed stream. Endpoint-independent direct-expression
checkpoints remain reusable and replay their former terminal bar plus the new
tail. Nested/endpoint-dependent expressions keep their existing full evaluator.
Skipping an initial chart calculation window first consumes the skipped request
transcript without executing skipped chart decisions. A streaming session keeps
that initial window; rebuilding a later batch would move the window and defines
a different input.

Controlled development evidence is retained in
`.local/goal-request-incremental-20261022/`. Thirty-two Rust merge/expression
configurations compare deltas against a causal full-snapshot oracle. Focused
checks cover static and dynamic nesting, calculation windows, forming
replacement/confirmation, lower-timeframe cumulative state and actual UTC month
boundaries. A first window test compared a fixed streaming window to a moving
later batch; its retained failure led to an independently replayed session
oracle with the same initial window.

The final frozen candidate passes 7,966 Rust, 1,941 installed-wheel Python and
172 tooling checks, plus strict all-target Clippy, formatting, structure/host
parity and generated WASM/Node checks. All 10,320 frozen inputs stayed unchanged.
Its wheel SHA-256 is
`f680603eb0b7da98cbd97c25d16a9d5aa713e3587a7256f844d26449d5eea6af`.
A dedicated 512-observation work-counter check replays 1,023 requested bars;
this measures bounded request work, not total throughput.

An independent audit also reproduced a preexisting generic realtime nested-cache
defect against the phase-2 wheel: growing the inner request produced
`[100,100]` instead of `[100,200]`. Generic updates now invalidate transitive
fallback snapshots while preserving direct incremental checkpoints. Rust,
Python and generated JS cover dynamic/static and nested/inline controls.
Earlier failed attempts and the initially qualified foundation remain retained.

These checks establish controlled core regression evidence. Host worker/replay,
TradingView reference, workload, end-to-end throughput and application release
qualification remain separate requirements. Tick scheduling is subsequent work.

# Realtime provider-only transactions

`REALTIME_REQUEST_FEED_SCHEMA_VERSION=1` adds atomic requested-provider updates
that do not execute Pine. Rust `RealtimeRuntime::apply_request_feed_updates`,
Python `RealtimeSession.apply_request_feed_updates` and generated JavaScript
`RealtimeSession.applyRequestFeedUpdates` accept records containing exact
`symbol`, `timeframe`, `bar` and `kind` fields. The kind is historical, forming or
confirmed. Unknown fields, invalid selectors/OHLCV and invalid stream progression
are rejected before either confirmed or forming provider state is published.

Successful updates preserve Pine output, ordinary/varip state, chart history,
revision, output cursor and last changes. Fallback request caches invalidate;
direct incremental expression checkpoints remain reusable. The caller then
submits its chart observation exactly once. Existing single-provider update
methods retain their automatic recalculation behavior.

This is an input transaction, not an observation scheduler or an availability
contract. The embedding host owns source identity, confirmed-prefix validation,
arrival times, event ordering and the chart execution clock. It must not feed
unobserved future rows before executing an earlier chart observation. Use the
separate externally observed-request contract when an external-account
transcript requires explicit per-source availability validation.

The read-only `request_bar_close` helper resolves a requested interval with the
same calendar as request merging. Python returns integer milliseconds. The
JavaScript helper accepts/returns integer JSON text, avoiding lossy floating
point timestamp conversion. Close is exclusive; an interval closing at T can
first be confirmed at T minus one milliseconds. This does not establish an
exchange acquisition or delivery time.

Controlled regressions cover one combined observation, varip retention, output
and revision stability, atomic rejection across streams, malformed input retry,
calendar-month/second units and unchanged legacy recalculation. The generated
Node path additionally verifies replica/delta coherence. Full qualification and
actual host lifecycle integration remain separate delivery gates.

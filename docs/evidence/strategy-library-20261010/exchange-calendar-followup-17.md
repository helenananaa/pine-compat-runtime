# Exchange-calendar work after the UTC lower-close qualification

Status: code audit and implementation scope only. This file does not claim the exchange-calendar capability is delivered.

Batch 17 enables the native-observed continuous UTC lower intraday close profile. The AAPL D oracle remains independent: its 6.5-hour daily close and preceding requested-minute close require actual trading boundaries. The runtime must not infer those boundaries from a symbol name or download them itself.

A shared immutable calendar registry in RequestEnvironment is the proposed neutral input. A calendar supplies ordered positive-duration open/close intervals keyed by symbol and timeframe, with deterministic coverage and validation errors. The host supplies raw intervals; core logic chooses intervals, applies Pine chart/request offsets, and clips explicit sessions. for_chart must retain the registry for same-symbol, foreign-symbol and nested requests. A host callback evaluating a full Pine time_close query would move language semantics outside the core and is excluded.

The audit found several paths that must agree before claiming exchange-calendar completion:

- builtins/time.rs::eval_bar_time_function and runtime/context.rs plain time_close need the same boundary/offset lookup.
- request/timeframe.rs::nominal_close and previous_open feed request/feed.rs alignment, builtins/requests.rs merges, builtins/static_requests.rs bounds and builtins/request_incremental.rs replay horizons.
- runtime/realtime.rs also derives an interval close through nominal_close. A partial patch of the visible clock function would leave confirmation/request scheduling on UTC duration arithmetic.
- Input exposure and capability errors must be consistent across Rust, CLI, Python and WASM; snapshots/restored execution must preserve or validate the calendar identity. No application-specific dependencies belong in these contracts.

Required controls include regular-session AAPL D/1 minute, previous/future trading intervals across weekends/holidays and DST, final partial intraday bars, explicit session clipping, missing coverage, malformed/overlapping intervals, same/foreign/nested requested contexts, realtime confirmation and snapshot/restore. Existing continuous UTC callers must retain their default results. Interval selection should use prevalidated ordered indexes rather than rebuilding/scanning a calendar every bar.

The earlier native evidence and broader boundary contract remain in stock-clock-oracle-16.json and lower-close-calendar-contract-16.md. Native full-market trade parity remains outside this implementation-scope note.

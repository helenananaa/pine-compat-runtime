# Remaining lower-timeframe close-clock contract

Status: native observations and code audit only; not implemented or qualified.

The crypto oracle (`lower-close-continuation-oracle.json`) shows that a lower
intraday `time_close` called in D/W/M context closes at the selected daily
boundary. The requested timeframe still defines `timeframe_bars_back` movement.
The stock oracle (`stock-clock-oracle-16.json`) establishes an independent
exchange-calendar requirement: AAPL D context opens/closes 23,400,000 ms apart,
and its previous requested 1-minute bar closes 63,000,000 ms before the current
daily opening. A universal UTC-midnight ceiling cannot represent both profiles.

Current runtime clock metadata is in `src/request/chart.rs`; it supplies symbol,
timeframe and price/quantity/currency grids, but no trading-calendar boundaries.
`src/builtins/time.rs::eval_bar_time_function` explicitly rejects lower
timeframe closes. `src/runtime/context.rs` sets the plain `time_close` variable
through `chart_timeframe_close`. All of these paths need coherent semantics.
The default model is explicitly continuous UTC, independent of the symbol name.

The next implementation should accept host-provided immutable ordered opening
and closing timestamps, keyed by symbol and timeframe, with positive durations
and validated non-overlapping intervals. The host obtains calendars; the core
selects intervals, performs Pine offsets, clips partial intraday bars and applies
explicit session filters. A callback that computes an entire Pine `time_close`
query would move language semantics into the host and is not the intended API.

The calendar must survive request-context changes. Same-symbol requests clone
chart metadata, while different-symbol contexts currently construct a fresh
`ChartContext`; placing a single calendar only on the root chart would therefore
miss the stock oracle's `request.security("NASDAQ:AAPL", "D", ...)` case.
Rust, Python, WASM and CLI input surfaces must expose the same neutral data
contract. Missing coverage and malformed inputs need explicit results rather
than inferred holidays or guessed exchange closing times.

Required verification cases include UTC D/W/M lower closes; independent chart
and requested offsets; the existing 500-future-bar bound; explicit sessions;
AAPL 6.5-hour days, prior trading-day offsets and final partial intraday bars;
DST, holidays, missing calendar coverage; and same/different-symbol nested
request contexts. Existing UTC-clock behavior remains a distinct compatibility
profile, and synthetic calendar tests alone cannot establish native full-market
calendar parity.

Paths above are relative to `crates/pine-runtime/`. This note records remaining
work and does not count a proposed contract as a delivered capability.

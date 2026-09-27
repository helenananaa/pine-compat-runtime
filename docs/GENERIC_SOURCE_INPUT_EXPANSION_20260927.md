# Generic source input override follow-up

Date: 2026-09-27. This is a local host-contract regression check, not a new
TradingView export or a native parity claim.

Generic series-float `input(close, "Source")` already evaluated its default
series. Hosts could not select another chart source through its call-site
override because its resolved value kind was only `Float`. `InputCall` now
also reports `is_source` from the analyzed return qualifier. The runtime uses
the same current-bar source selector as `input.source`, and Rust, CLI, Python,
and WASM accept the eight built-in chart price series. Ordinary float inputs
remain numeric; unknown names and external indicator plots fail explicitly.
The core needs only supplied chart bars and has no host application dependency.

Focused Rust and WASM executions select `hl2` across two bars with different
OHLC values and reject an unknown source. The CLI parser checks the same
admission boundary. `cargo test -p pine-cli`, `cargo test -p pine-wasm`, and
`cargo test -p pine-python` pass. The Python and WASM crates also pass
`cargo check`.

The same test run found 38 stale CLI golden snapshots from earlier broker
work. They were regenerated: 36 add the already-qualified
`strategy.close` fill order to the public order list; two reflect the
previously qualified zero-distance `strategy.exit` admission change. The
snapshot changes were inspected as structured JSON, then the CLI and WASM
test suites passed against the updated fixtures. No new TradingView evidence
is implied by these snapshot updates.

The public CLI, Python, and WASM analysis reports now expose `isSource` for
each `inputs[]` entry. This distinguishes generic `input(close)` and explicit
`input.source(close)` from numeric float inputs, so an embedding host can
present a chart-source selector. Analysis schema version is 6. Seven analysis
goldens changed only in that version and the new boolean field. The eight
accepted built-in chart sources and the explicit rejection of unsupported
external sources are unchanged.

# Delivery surfaces for the local 0.3.0-rc.1 candidate

Latest local worktree qualification: [streaming expansion](STREAMING_EXPANSION_AUDIT.md)
and [artifact identity](STREAMING_EXPANSION_ARTIFACTS.json). This includes actual
WASM streaming tests and a Windows optimized wheel; the earlier platform matrix
below is retained evidence, not a newly qualified Linux build.

This is a locally qualified prerelease for the named scope, not a stable release
or a claim of full Pine compatibility. [Current status](DELIVERY_ROADMAP.md) and
[artifact hashes](DELIVERY_ARTIFACTS.json) identify the repaired build; version
0.3.0rc1 alone does not distinguish it from older retained candidates. Consume the local artifacts built from the
matching commit; do not install the published `v0.2.0` GitHub wheels and treat
them as this checkout.

| Surface | Consumable entry | Version identity | Historical | Incremental | Realtime lifecycle | Host requirements | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Rust | path dependency on `crates/pine-runtime` plus `pine-syntax` / `pine-sema` | `CARGO_PKG_VERSION` = `0.3.0-rc.1` | `HistoricalRuntime` | per-bar `append_bar` | `RealtimeRuntime` seed/forming/confirm | `host_requirements` | Example: `crates/pine-runtime/examples/embed_runtime.rs` |
| CLI | `cargo run -p pine-cli --locked --release` → `pine-compat` | `pine-compat --version` | `run` | `run-incremental` | `run-realtime-history`, `run-realtime-forming` | `requirements` | JSON schema 8 results; analysis schema 5 |
| Python | maturin wheel `pine-compat-runtime==0.3.0rc1` (`pine_compat`) | `pine_compat.__version__` = `0.3.0-rc.1` | `compile_script` / `run_script` / `Program.run` | not a separate API; re-run batch or confirm bars on a session | `create_realtime_session` / `Program.realtime_session` | `Program.host_requirements` | Wheel version is PEP 440 `0.3.0rc1` |
| WASM | `cargo build -p pine-wasm --target wasm32-unknown-unknown` plus `generate_node_bindings` | `packageVersion()` = `0.3.0-rc.1` | `runScriptCsv` / `Program.runCsv` | `Program.realtimeSession` seed plus `applyForming` / `applyConfirmed` | `RealtimeSession` seed/forming/confirm/replay/correct, request feed, replica | `Program.hostRequirements` | JSON-string boundary; changes schema 3; runtime snapshots schema 8 |

## Streaming worktree API

Rust now exposes `RealtimeRuntime::apply_update`, `replay_historical`,
`correct_historical` and `RuntimeReplica`; Python exposes `apply_forming`,
`apply_confirmed`, `session.replay()`, `session.correct()`, `session.replica()`,
and `session.stream_snapshot()`; WASM exposes `Program.realtimeSession()`,
`applyForming` / `applyConfirmed`, `replay()`, `correct()`, `replica()`, and
`streamSnapshot()`. Changes use schema 3 with
`baseRevision`, `revision` and `retainedFrom`; runtime snapshots remain schema
8. A replica applies deltas in place and materializes a full result only on
request. Historical replay is a snapshot discontinuity, not a linear delta. See
[streaming acceptance](STREAMING_INCREMENTAL_AUDIT.md) for current validation.
These interfaces require a build of this worktree; the retained a2a1ba5fb wheels
above do not include them.

## Minimal examples

- Rust: `cargo run --locked --release -p pine-runtime --example embed_runtime`
- CLI: `pine-compat run tests/fixtures/runtime/macd.pine --bars tests/fixtures/runtime/bars.csv`
- Python: `docs/examples/python_embed.py`
- WASM/Node: `docs/examples/wasm_embed.mjs` after generating bindings

## Schema and platform matrix

- Analysis JSON schema 5, runtime JSON schema 8, host-requirements schema 1,
  render metadata 1.
- Qualified desktop targets (retained optimized wheels): Windows x86-64 (`win_amd64`) and native
  Ubuntu 22.04 (`manylinux_2_35_x86_64`), implementation a2a1ba5fb. Each passes
  717 installed tests and the named final native references. The older
  manylinux2014 (`manylinux_2_17_x86_64`) candidate at 2792a0950 passed auditwheel and 715 tests but
  does not contain the latest fixes. No current manylinux2014 qualification,
  Linux long-session budget, macOS, musllinux, ARM or free-threaded CPython
  support is established by these artifacts.
- Default chart context: synthetic `NASDAQ:AAPL`, timeframe `1`, minMove 1,
  priceScale 100, quantityPrecision 0, currency USD, unit point value, timezone
  `Etc/UTC`. These are defaults, not inferred instrument metadata.

Dynamic request arguments and broader account profiles remain explicit gaps.
Named realtime market-price references now pass; simultaneous high/low expansion
and unobserved price-condition paths remain unqualified. See [HOST_REQUIREMENTS.md](HOST_REQUIREMENTS.md) and the
candidate checklist.

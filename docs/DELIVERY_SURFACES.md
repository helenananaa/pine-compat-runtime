# Delivery surfaces for 0.3.0

The stable release uses version `0.3.0` across all surfaces. Official assets
contain Windows/Linux Python wheels, manifest and checksums. Rust, CLI and
WASM remain independently usable from source. See
[stable acceptance](STABLE_ACCEPTANCE_20261006.md) and
[migration/limits](STABLE_MIGRATION.md). Historical artifact inventories keep
their original source pins and do not substitute for current release assets.

| Surface | Consumable entry | Version identity | Historical | Incremental | Realtime lifecycle | Host requirements | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Rust | path dependency on `crates/pine-runtime` plus `pine-syntax` / `pine-sema` | `CARGO_PKG_VERSION` = `0.3.0` | `HistoricalRuntime` | per-bar `append_bar` | `RealtimeRuntime` seed/forming/confirm | `host_requirements` | Example: `crates/pine-runtime/examples/embed_runtime.rs` |
| CLI | `cargo run -p pine-cli --locked --release` → `pine-compat` | `pine-compat --version` | `run` | `run-incremental` | `run-realtime-history`, `run-realtime-forming` | `requirements` | JSON schema 9 results; analysis schema 6 |
| Python | maturin wheel `pine-compat-runtime==0.3.0` (`pine_compat`) | `pine_compat.__version__` = `0.3.0` | `compile_script` / `run_script` / `Program.run` | not a separate API; re-run batch or confirm bars on a session | `create_realtime_session` / `Program.realtime_session` | `Program.host_requirements` | Wheel version is PEP 440 `0.3.0` |
| WASM | `cargo build -p pine-wasm --target wasm32-unknown-unknown` plus `generate_node_bindings` | `packageVersion()` = `0.3.0` | `runScriptCsv` / `Program.runCsv` | `Program.realtimeSession` seed plus `applyForming` / `applyConfirmed` | `RealtimeSession` seed/forming/confirm/replay/correct, request feed, replica | `Program.hostRequirements` | JSON-string boundary; changes schema 4; runtime snapshots schema 9 |

## Integrated streaming API

Rust now exposes `RealtimeRuntime::apply_update`, `replay_historical`,
`correct_historical` and `RuntimeReplica`; Python exposes `apply_forming`,
`apply_confirmed`, `session.replay()`, `session.correct()`, `session.replica()`,
and `session.stream_snapshot()`; WASM exposes `Program.realtimeSession()`,
`applyForming` / `applyConfirmed`, `replay()`, `correct()`, `replica()`, and
`streamSnapshot()`. Changes use schema 4 with
`baseRevision`, `revision` and `retainedFrom`; runtime snapshots use schema
9. Gradient fills carry per-bar vertical stops and changes schema 4 adds
`setGradient`. Older non-gradient snapshots and changes remain readable;
gradient data labeled with old schemas is rejected. A replica applies deltas
in place and materializes a full result only on
request. Historical replay is a snapshot discontinuity, not a linear delta. See
[streaming expansion](STREAMING_EXPANSION_AUDIT.md) for the latest retained
validation. These interfaces are integrated through `f368ab96e` and included
in main by `7b70095f6`; the retained a2a1ba5fb wheels do not include them.
The streaming artifact inventory preserves its original pre-commit source
digest and does not claim a fresh build of the merged main revision.

The 0.3.0 implementation includes `RuntimeReplica::into_result()`
in Rust and `replica.intoResult()` in WASM. They finalize a completed replica
and transfer its full output; the WASM handle is consumed and must not receive
updates or another `free()` call. Read revision/origin beforehand if needed.
Use `result()` when the replica will continue receiving updates. Rust also
exposes `write_public_runtime_result_json` for a caller-owned sink and
`into_public_runtime_result_json` for an owned complete String. These additions
are covered by the [performance candidate record](PERFORMANCE_RESOURCE_CLOSURE_20261002.md);
older retained artifacts keep their own interface and source pins.

For known historical slices, `HistoricalRuntime::historical_dataset(&bars)`
executes one borrowed iterator step per bar while retaining the complete
dataset endpoint used by `append_bars`. Ordinary `append_bar` discovers a new
latest bar, which can differ for scripts that inspect the dataset endpoint.

## Minimal examples

- Rust: `cargo run --locked --release -p pine-runtime --example embed_runtime`
- CLI: `pine-compat run tests/fixtures/runtime/macd.pine --bars tests/fixtures/runtime/bars.csv`
- Python: `docs/examples/python_embed.py`
- WASM/Node: `docs/examples/wasm_embed.mjs` after generating bindings

## Schema and platform matrix

- Analysis JSON schema 6, runtime JSON schema 9, host-requirements schema 2,
  render metadata 1.
- Qualified desktop targets (retained optimized wheels): Windows x86-64 (`win_amd64`) and native
  Ubuntu 22.04 (`manylinux_2_35_x86_64`), implementation a2a1ba5fb. Each passes
  717 installed tests and the named final native references. The older
  manylinux2014 (`manylinux_2_17_x86_64`) candidate at 2792a0950 passed auditwheel and 715 tests but
  does not contain the latest fixes. These older artifacts do not establish 0.3.0 qualification; see the current
  stable acceptance record for manylinux2014 and the fresh resource matrix.
  macOS, musllinux, ARM and free-threaded CPython remain outside scope.
- Default chart context: synthetic `NASDAQ:AAPL`, timeframe `1`, minMove 1,
  priceScale 100, quantityPrecision 0, currency USD, unit point value, timezone
  `Etc/UTC`. These are defaults, not inferred instrument metadata.

Dynamic request arguments and broader account profiles remain explicit gaps.
Named realtime market-price references now pass; simultaneous high/low expansion
and unobserved price-condition paths remain unqualified. See [HOST_REQUIREMENTS.md](HOST_REQUIREMENTS.md) and the
candidate checklist.

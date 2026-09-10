# Delivery surfaces for the local 0.3.0-rc.1 candidate

This is a TV-blocked prerelease candidate, not a stable release and not a
claim of full Pine compatibility. Consume the local artifacts built from the
matching commit; do not install the published `v0.2.0` GitHub wheels and treat
them as this checkout.

| Surface | Consumable entry | Version identity | Historical | Incremental | Realtime lifecycle | Host requirements | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Rust | path dependency on `crates/pine-runtime` plus `pine-syntax` / `pine-sema` | `CARGO_PKG_VERSION` = `0.3.0-rc.1` | `HistoricalRuntime` | per-bar `append_bar` | `RealtimeRuntime` seed/forming/confirm | `host_requirements` | Example: `crates/pine-runtime/examples/embed_runtime.rs` |
| CLI | `cargo run -p pine-cli --locked --release` → `pine-compat` | `pine-compat --version` | `run` | `run-incremental` | `run-realtime-history`, `run-realtime-forming` | `requirements` | JSON schema 8 results; analysis schema 5 |
| Python | maturin wheel `pine-compat-runtime==0.3.0rc1` (`pine_compat`) | `pine_compat.__version__` = `0.3.0-rc.1` | `compile_script` / `run_script` / `Program.run` | not a separate API; re-run batch or confirm bars on a session | `create_realtime_session` / `Program.realtime_session` | `Program.host_requirements` | Wheel version is PEP 440 `0.3.0rc1` |
| WASM | `cargo build -p pine-wasm --target wasm32-unknown-unknown` plus `generate_node_bindings` | `packageVersion()` = `0.3.0-rc.1` | `runScriptCsv` / `Program.runCsv` | not exported | not exported | `Program.hostRequirements` | Historical CSV/JSON only; no forming session |

## Minimal examples

- Rust: `cargo run --locked --release -p pine-runtime --example embed_runtime`
- CLI: `pine-compat run tests/fixtures/runtime/macd.pine --bars tests/fixtures/runtime/bars.csv`
- Python: `docs/examples/python_embed.py`
- WASM/Node: `docs/examples/wasm_embed.mjs` after generating bindings

## Schema and platform matrix

- Analysis JSON schema 5, runtime JSON schema 8, host-requirements schema 1,
  render metadata 1.
- Qualified desktop targets: Windows x86-64 (`win_amd64`) and glibc Linux
  x86-64 (`manylinux_2_17_x86_64` / Ubuntu 22.04). macOS, musllinux, ARM, and
  free-threaded CPython are outside this candidate.
- Default chart context: synthetic `NASDAQ:AAPL`, timeframe `1`, minMove 1,
  priceScale 100, quantityPrecision 0, currency USD, unit point value, timezone
  `Etc/UTC`. These are defaults, not inferred instrument metadata.

Dynamic request arguments, live-tick fill prices, and broader account profiles
remain explicit gaps. See [HOST_REQUIREMENTS.md](HOST_REQUIREMENTS.md) and the
candidate checklist.

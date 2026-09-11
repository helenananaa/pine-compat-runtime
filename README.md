# Pine Compat Runtime

**Run Pine-style indicators and strategies on your own market data — locally,
deterministically, and from the host you already use.**

[![Latest release](https://img.shields.io/github/v/release/helenananaa/pine-compat-runtime?display_name=tag&sort=semver)](https://github.com/helenananaa/pine-compat-runtime/releases/latest)
[![CI](https://github.com/helenananaa/pine-compat-runtime/actions/workflows/ci.yml/badge.svg)](https://github.com/helenananaa/pine-compat-runtime/actions/workflows/ci.yml)
[![Wheels](https://github.com/helenananaa/pine-compat-runtime/actions/workflows/wheels.yml/badge.svg)](https://github.com/helenananaa/pine-compat-runtime/actions/workflows/wheels.yml)
[![Python 3.10+](https://img.shields.io/badge/Python-3.10%2B-3776AB?logo=python&logoColor=white)](https://github.com/helenananaa/pine-compat-runtime/releases/latest)
[![Rust 1.95+](https://img.shields.io/badge/Rust-1.95%2B-000000?logo=rust&logoColor=white)](Cargo.toml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Pine Compat Runtime is a clean-room, open-source runtime for an executable
Pine-compatible subset. Feed it source code and OHLCV bars; get structured,
host-neutral results for plots, drawings, alerts, orders, trades, positions,
and equity.

It is built for charting products, research tools, notebooks, backtesting
workflows, and anyone who wants Pine-style execution without coupling their
application to a charting service.

> [!NOTE]
> This project is not affiliated with, endorsed by, or sponsored by
> TradingView. It implements a tested compatibility subset and does not claim
> full Pine Script compatibility.

## Why Pine Compat Runtime?

- **Bring your own data.** Run over CSV bars or in-memory OHLCV records. The
  core does not fetch symbols, read host files, or make network requests.
- **Embed it anywhere.** Use the Rust core, CLI, Python extension, or thin WASM
  API without reimplementing Pine execution semantics in your host.
- **Get useful output, not screenshots.** Results are versioned, JSON-ready
  structures that a chart, notebook, API, or test suite can consume directly.
- **Model time series correctly.** Historical references, persistent state,
  stateful call sites, incremental append, and realtime forming-bar rollback
  are runtime concepts rather than host-side approximations.
- **Know what will run.** Unsupported features become source-spanned
  diagnostics and compatibility reports instead of silent partial execution.
- **Trust claims you can test.** The compatibility matrix is backed by
  executable fixtures, cross-host parity checks, and a single release gate.

## Quick Start

The downloads below are the published `v0.2.0` release from July 20, 2026.
This checkout is the local `0.3.0-rc.1` candidate (Python wheel `0.3.0rc1`).
It is a locally qualified prerelease for the named scope, not a stable tag or
full Pine compatibility. Current status and exact artifact identities are in
[the delivery ledger](docs/DELIVERY_ROADMAP.md); older rc1 wheels share the version
number and must not be confused with the repaired artifacts.
Build this tree for host-input discovery, source provenance, realtime clocks,
and the four-surface candidate artifacts. Do not install the published `v0.2.0`
wheels and treat them as this candidate. See
[delivery surfaces](docs/DELIVERY_SURFACES.md) and
[releasing](docs/RELEASING.md).

Version `0.2.0` ships ready-to-install Python wheels for CPython 3.10+ on
glibc Linux x86-64 and Windows x86-64. See the
[latest release](https://github.com/helenananaa/pine-compat-runtime/releases/latest)
for checksums and machine-readable release metadata.

Linux x86-64:

```bash
python -m pip install \
  "https://github.com/helenananaa/pine-compat-runtime/releases/download/v0.2.0/pine_compat_runtime-0.2.0-cp310-abi3-manylinux_2_17_x86_64.manylinux2014_x86_64.whl"
```

Windows x86-64:

```powershell
py -m pip install "https://github.com/helenananaa/pine-compat-runtime/releases/download/v0.2.0/pine_compat_runtime-0.2.0-cp310-abi3-win_amd64.whl"
```

Then run an indicator directly from Python:

```python
import pine_compat

source = """//@version=6
indicator("SMA demo")
avg = ta.sma(close, 3)
plot(avg, "SMA 3")
"""

bars = [
    {"time": i, "open": close, "high": close, "low": close,
     "close": close, "volume": 100.0}
    for i, close in enumerate([10.0, 11.0, 12.0, 13.0])
]

result = pine_compat.run_script(source, bars)
print(result["plots"][0]["values"])
# [None, None, 11.0, 12.0]
```

That same result can include chart annotations, drawing snapshots, alerts, and
partial strategy broker output — all without requiring a chart UI.

## What Works Today

The current release focuses on a broad indicator runtime and a deliberately
bounded strategy runtime.

| Area | Current executable subset |
| --- | --- |
| Language | fixture-backed v1-v6 declarations, series and history, bounded v1/v2 declaration graphs, `var`/partial `varip`, functions, tuples, `if`, `switch`, partial `for`/`while`, strings, UDTs, and host-provided pure library imports |
| Indicators | Common `ta.*`, selected `math.*`/`str.*`, inputs, plots, colors, alerts, drawing objects, tables, typed collections, fixture-backed `request.security`, and the documented executable Pine v1-v4 legacy-indicator subsets including `security` |
| Execution | Deterministic historical runs, guarded history, input overrides, explicit `timenow` execution clocks, incremental append, and realtime forming-bar rollback |
| Strategies | Partial side-aware long and short entries, orders, closes, cancellations, stop/limit/bracket/trailing exits, quantity reservations, positions, trades, and equity snapshots |
| Outputs | Versioned plots, shapes, bars, candles, fills, labels, lines, line fills, polylines, boxes, tables, alerts, diagnostics, and strategy results |
| Hosts | Rust workspace, `pine-compat` CLI, `pine_compat` Python module, and `wasm-bindgen` API |

Legacy indicator compatibility is released by source version, not as one broad
backwards-compatibility promise:

| Source profile | Maturity | Current claim |
| --- | --- | --- |
| Pine v4 indicators | Preview | Direct execution for the conformance-listed declaration, input, alias, output, expression, session, and `security` subsets |
| Pine v3 indicators | Preview | Direct execution for the conformance-listed pre-v4 declaration, input/output, alias, metadata, `na`, and same-context `security` subsets |
| Pine v2 indicators | Experimental | Direct execution for the focused declaration graph, conversion, basic indicator, and historical-lookahead `security` subsets |
| implicit Pine v1 indicators | Experimental | Direct execution for the focused v1/v2 shared `study`/input/average/plot subset |

These labels reflect evidence breadth: the fixed original seed corpus contains
only 12 v4, 7 v3, 2 v2, and 1 v1 eligible indicators. All currently pass, but
none reaches the provisional 50-script stable-profile evidence gate. Legacy
strategies are excluded from every profile.

Support is intentionally feature-specific. Before adopting a script corpus,
use the analyzer or the executable compatibility matrix rather than assuming
language-wide compatibility:

```bash
cargo run -p pine-cli -- matrix
cargo run -p pine-cli -- matrix --format json
```

The matrix in [`tests/fixtures/conformance.tsv`](tests/fixtures/conformance.tsv)
and its referenced fixtures are the source of truth. See
[Language Scope](docs/LANGUAGE_SCOPE.md) for the detailed boundary and
[Conformance](docs/CONFORMANCE.md) for how claims are accepted.

## Choose Your Integration

| Surface | Best for | Entry point |
| --- | --- | --- |
| Python | notebooks, research services, data pipelines, application plugins | `run_script(...)`, reusable `Program`, or persistent `RealtimeSession` |
| CLI | shell workflows, fixtures, compatibility checks, JSON generation | `pine-compat run`, `analyze`, `fmt-ast`, and `matrix` |
| Rust | native applications and deeper runtime embedding | workspace crates under [`crates/`](crates), [embedding walkthrough](docs/RUST_EMBEDDING.md) |
| WASM | browser, Node.js, and sandboxed JavaScript hosts | `compileScript`, `analyzeScript`, `runScriptCsv`, and `Program.runCsv` |

### Python

Compile once and reuse a program across data sets or input configurations:

```python
import pine_compat

source = '''//@version=6
indicator("Configurable SMA")
length = input.int(20, "Length")
plot(ta.sma(close, length))
'''

report = pine_compat.analyze_script(source)
length_id = report["inputs"][0]["callSiteId"]
program = pine_compat.compile_script(source)

result = program.run(bars, input_overrides={length_id: 50})
```

Current development builds can inspect a compiled program's potential host inputs
before supplying data (this API is unreleased):

```python
requirements = program.host_requirements()
```

This reports chart/account assumptions, requested contexts, execution-clock
usage and optional Magnifier/session-window fallbacks. It is a conservative
inventory, not a dataset-readiness verdict. CLI `pine-compat requirements` and
WASM `Program.hostRequirements()` expose the same versioned contract. See
[host input discovery](docs/HOST_REQUIREMENTS.md).

For a persistent realtime stream, create one session, seed its complete
confirmed history once, replace the current forming bar as ticks arrive, and
commit that same timestamp when the bar closes:

```python
session = program.realtime_session(input_overrides={length_id: 50})
snapshot = session.seed(confirmed_bars)
preview = session.update_forming(forming_bar)
preview = session.update_forming(replacement_forming_bar)
confirmed = session.update_confirmed(closed_bar)
```

`update_forming` / `update_confirmed` still return a complete snapshot. To avoid constructing a complete returned snapshot on every tick, use the same lifecycle and consume this-update changes:

```python
session = program.realtime_session(input_overrides={length_id: 50})
session.seed(confirmed_bars)
replica = session.replica()
changes = session.apply_forming(forming_bar)
replica.apply(changes)
# Request a complete Python dictionary only when needed.
visible = replica.result()
assert visible == session.result()
```

`apply_forming` / `apply_confirmed` return series append or current-bar replace, drawing add/modify/delete, order/fill/alert identity, preview vs confirmed visibility, and base/current revisions. A replica ignores an identical retransmission and rejects stale or missing revisions. Call `session.result()` when a complete snapshot is required.

Development builds also accept `seed(bars, execution_times=[...])` and
`update_forming`/`update_confirmed(..., execution_time=...)` for scripts reading
`timenow`. The clock is supplied by the host; it is never read from the machine
inside the core.

Development builds also accept `opening_update=False` on forming or confirmed
updates when the host attaches to a bar that is already open. Omitting it
retains first-observation inference. See the
[opening-context contract](docs/REALTIME_OPENING_CONTEXT_AUDIT.md).

`update_forming` rolls ordinary `var` state back to the last confirmed bar and
preserves `varip` state across replacements. The session rejects updates before
seeding, regressive confirmed timestamps, and a forming/confirmed timestamp
mismatch. `REALTIME_SESSION_SCHEMA_VERSION` versions this lifecycle contract.

Python can also inject deterministic requested bars and exact-key library
sources:

```python
execution_times = [1700000000101 + index for index in range(len(bars))]
result = pine_compat.run_script(
    source,
    bars,
    request_bars={"NYSE:IBM:5": requested_bars},
    library_sources={"user/lib/1": library_source},
    chart_symbol="NASDAQ:AAPL",
    chart_timeframe="1",
    execution_times=execution_times,
)
```

Pass one `execution_times` value per bar only when the script can reach
`timenow`; the runtime does not infer it from bar timestamps or wall-clock time.

Legacy indicators use their source version automatically; no host flag is
needed. The same analysis schema reports the selected dialect and any
translation/emulation evidence before execution:

```python
legacy = '''//@version=4
study("Legacy SMA")
length = input(20, "Length", input.integer)
plot(sma(close, length))
'''
report = pine_compat.analyze_script(legacy)
assert report["dialect"] == "v4" and report["executable"]
result = pine_compat.run_script(legacy, bars)
```

The returned `executable` flag proves that this source is admitted by the
implemented subset; it does not upgrade the whole v4 language profile beyond
preview.

### CLI

Run a script against CSV OHLCV data and receive normalized JSON:

```bash
cargo run --release -p pine-cli -- run \
  tests/fixtures/runtime/macd.pine \
  --bars tests/fixtures/runtime/bars.csv
```

Analyze without executing, or inject host-owned request and library data:

```bash
cargo run -p pine-cli -- analyze script.pine
cargo run -p pine-cli -- analyze script.pine --format json

cargo run -p pine-cli -- run script.pine --bars bars.csv \
  --execution-times execution-times.txt \
  --chart-symbol NASDAQ:AAPL --chart-timeframe 1 \
  --request-bars NYSE:IBM:5=ibm-5m.csv \
  --library-source user/lib/1=lib.pine
```

### WASM

The optional `pine-wasm` crate exposes thin, deterministic bindings for
compile, analysis, CSV execution, request-bar injection, and library-source
injection. Build and exercise the real generated JavaScript module with:

```bash
rustup target add wasm32-unknown-unknown
scripts/check_wasm_node.sh
```

For `timenow`, pass `{"$executionTimes":[...]}` in the request-host JSON used
by a `*WithRequestBars` entry point, including compiled `Program` runs.

See [Architecture](docs/ARCHITECTURE.md) for the host boundary and
[Execution Semantics](docs/EXECUTION_SEMANTICS.md) for historical and realtime
behavior.

## Honest Compatibility

The published `0.2.0` tag and this `0.3.0-rc.1` candidate are
compatibility-focused, not a full drop-in implementation of every Pine
feature. Important current boundaries include:

- the strategy broker model is still a partial side-aware long/short subset;
- `request.*` support is limited and all requested data must be supplied by the
  host;
- library resolution is exact-key and host-provided — there is no remote
  registry lookup;
- collection, drawing, alert, UDT, method, and import support is intentionally
  limited to fixture-backed shapes;
- Pine v4/v3 legacy-indicator profiles are previews and Pine v2/v1 profiles are
  experimental because the authorized release corpus is small and has no
  external reference-output oracle;
- legacy strategies, lower-timeframe legacy `security`, and non-empty or
  dynamic whole-program `study(resolution=...)` execution remain out of scope;
  the exact Pine v4 `resolution=""` form inherits the host chart context;
- unsupported syntax or semantics are rejected with diagnostics rather than
  guessed.

This explicit boundary is part of the product: hosts can inspect compatibility
before execution and decide whether to run, transform, or reject a script.

## Architecture

The runtime is a Rust core with thin host adapters:

```text
Pine source + host-provided data
              │
              ▼
 lexer → parser → semantic analysis → HIR → bar-by-bar runtime
              │                              │
              ├─ diagnostics                 ├─ plots and drawings
              └─ compatibility report        └─ alerts and strategy results
                                             │
                      Rust / CLI / Python / WASM
```

Core crates do not fetch network data, resolve remote libraries, or depend on a
specific chart renderer. Hosts own data access and presentation; the runtime
owns language semantics and normalized output.

## Documentation

- [Documentation Guide](docs/README.md) — where current contracts, roadmaps,
  and historical plans live
- [Language Scope](docs/LANGUAGE_SCOPE.md) — detailed supported and unsupported
  language shapes
- [Built-In Signatures](docs/BUILTIN_SIGNATURES.md) — accepted built-ins and
  argument subsets
- [Execution Semantics](docs/EXECUTION_SEMANTICS.md) — bar, history, state, and
  broker behavior
- [Diagnostic Codes](docs/DIAGNOSTIC_CODES.md) — stable diagnostic reference
- [Release Notes](docs/RELEASE_NOTES.md) — changes in each release
- [Releasing Binary Wheels](docs/RELEASING.md) — wheel matrix, checksums, and
  application update contract
- [Compatibility and Legal Boundaries](docs/COMPATIBILITY_AND_LEGAL.md) —
  clean-room and branding policy

## Build From Source

The workspace requires Rust 1.95+. Python bindings require Python 3.10+ and
`maturin`.

```bash
git clone https://github.com/helenananaa/pine-compat-runtime.git
cd pine-compat-runtime
cargo build --workspace
```

Build the Python module in an active virtual environment:

```bash
python -m pip install "maturin>=1.13,<2.0" pytest
maturin develop --manifest-path crates/pine-python/Cargo.toml
python -m pytest python/tests
```

Before contributing or publishing, run the canonical release gate:

```bash
scripts/verify.sh
```

On Windows, install the MSVC C++ Build Tools plus the WASM target, then run the
PowerShell equivalent. The script initializes the Visual Studio developer
environment automatically when necessary.

```powershell
rustup target add wasm32-unknown-unknown
python -m pip install "maturin>=1.13,<2.0" pytest
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify.ps1 -Python python
```

It covers Rust formatting, clippy, workspace tests, source-structure and host
parity checks, a real WASM/Node execution smoke, Python wheel build and
reinstall, and Python binding tests. See [Releasing Binary Wheels](docs/RELEASING.md)
for the supported release platforms and artifact contract.

## License

[MIT](LICENSE). Pine Script is a trademark of its respective owner. This
independent clean-room project is not affiliated with TradingView.

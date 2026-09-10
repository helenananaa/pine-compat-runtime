# Embedding the runtime in Rust

The executable example is `crates/pine-runtime/examples/embed_runtime.rs`.
Use Rust 1.95 or newer. From the repository root, run:

```text
cargo run --locked --release -p pine-runtime --example embed_runtime
```

The example supplies a complete in-memory Pine library, discovers the compiled
program's host requirements, and explicitly selects a synthetic symbol, a
one-minute timeframe, price grid and quantity precision. It compares historical
batch execution with individual appends, then seeds a realtime runtime, replaces
the open bar and confirms it. Host-provided execution timestamps satisfy
`timenow`; a deliberately missing timestamp demonstrates an unchanged runtime
after an error. Standard output is JSON; compiler diagnostics go to stderr.

All input bars and clocks are synthetic. Successful assertions demonstrate the
example's execution and API use; they are not TradingView reference evidence or
resource qualification. The embedded library is an illustrative complete source,
not a replacement for the full real-library acceptance workloads.

An embedding application needs `pine-syntax` for `SourceFile`, `pine-sema` for
analysis and library resolution, and `pine-runtime` for execution. Until the
crates are published as an accepted release, use matching workspace paths or an
exact reviewed repository revision for all three. Do not combine independent
crate revisions. This example runs without CandleScope, a database, a network
connection, credentials or a scheduler.

To copy it into a sibling application named `host-demo`, with the runtime checkout
at `../pine-interpreter`, use this application manifest and copy the example into
`src/main.rs`:

```toml
[package]
name = "pine-host-demo"
version = "0.1.0"
edition = "2024"
rust-version = "1.95"

[dependencies]
pine-syntax = { path = "../pine-interpreter/crates/pine-syntax" }
pine-sema = { path = "../pine-interpreter/crates/pine-sema" }
pine-runtime = { path = "../pine-interpreter/crates/pine-runtime" }
serde_json = "1"

[profile.release]
lto = "thin"
codegen-units = 1
```

The `serde_json` dependency is used only to format the example's combined report.
An application consuming typed results need not use that reporting code. Cargo
may need to acquire third-party build dependencies; the running example performs
no network access. In `host-demo`, first run `cargo run --release`, then preserve
the generated `Cargo.lock` and use `cargo run --locked --release` for later builds.

The application owns acquisition of bars/library sources, clock selection,
optional request data and session windows, and runtime lifetime. Consult
HOST_REQUIREMENTS.md before treating synthetic defaults as real instrument
metadata. Do not turn every potential input into an unconditional rejection:
requirements are conservative, and conditional reads are validated when reached.

`HistoricalRuntime::new(&hir)` and the environment builders borrow the compiled
HIR, which must outlive the runtime. The owned `RealtimeRuntime::from_program`
family is available when the runtime must own its compiled program. Each runtime
instance owns its execution state; returned results can be retained and modified
without mutating that state. `Forming` supplies an observed realtime update,
while `Confirmed` commits the bar; it is not interchangeable with historical
OHLC-path simulation. See REALTIME_MODEL.md for strategy behavior.

Cargo release settings in this workspace apply when building the example here.
Downstream applications control their own profiles; use ThinLTO and one
code-generation unit when reproducing the current release profile. This is not
a universal latency guarantee, and does not require CPU-specific instructions.

Windows verification passed: the repository example and the independent consumer
were built and executed in release mode, and their complete JSON output matches.
The example also passes Clippy with warnings denied. Linux execution qualification
remains separate.

The standalone manifest has been resolved offline in an independent consumer
workspace using the existing lockfile versions. Only paths were remapped to the
tested checkout, and an empty workspace section prevents accidental membership
in the enclosing evidence directory's workspace. The independent consumer then
built with `--offline --locked --release` and produced the same result as the
repository example. Logs, lockfile, source and output are retained under
`.local/delivery-20260909/rust-embedding-consumer/`.

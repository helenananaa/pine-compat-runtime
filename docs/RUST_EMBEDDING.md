# Embedding the runtime in Rust

The executable example is `crates/pine-runtime/examples/embed_runtime.rs`.
This candidate's crate identity is `0.3.0-rc.1`. Use Rust 1.95 or newer. From
the repository root, run:

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

For manually constructed HIR, reusing a `CallSiteId` for the same TA callee
identifies one logical stateful call site. Repeated execution retains that
algorithm's existing invocation and same-bar replacement rules. If different
callees share an ID, preparation assigns private, deterministic state IDs to
the TA operations declared at that ID. This separates their windows and
recursive state, including conflicts with `fixnan`. It does not rewrite HIR,
input override IDs, output IDs, request IDs or source locations. Prefer unique
IDs for distinct logical calls; this compatibility rule is not general HIR
validation or isolation for arbitrary calls outside the prepared program.

Historical execution errors disable further execution of that `HistoricalRuntime`
instance (`E_RUNTIME_POISONED`); rebuild it to retry. Validation failures before
execution, including clock counts and session-window coverage, remain retryable.
The failed historical instance can contain partial evaluator state or output;
retain previously returned owned results when a committed snapshot is needed.
`RealtimeRuntime` stages its updates and history replacements separately and
preserves the previous session after a failed operation.

For output that is immediately serialized or consumed, `result_view()` borrows
the runtime's visible histories and metadata; `confirmed_result_view()` on a
realtime runtime borrows confirmed output only. `RuntimeResult::view()` also
adapts an existing owned result. `HistoryView<T>` exposes borrowed iteration and
length without copying the historical values or exposing runtime storage.
Small per-family view headers and synthesized diagnostics are allocated.
The borrow prevents a mutable runtime update until the view is released.
Use the owned `result()` when the snapshot must outlive the runtime or be
retained independently of later updates.

`write_public_runtime_result_view_json(&view, &mut sink)` streams the same public
schema and field order to any `std::io::Write` sink and propagates I/O failures.
Buffer file or network sinks in the host. This function needs no complete JSON
buffer and leaves the runtime and revision unchanged on an I/O failure;
already-written bytes remain in the caller's sink. Drawing snapshots (including
table cells and polyline points), strategy records, alerts, and diagnostics write
their scalar fields directly to the sink. They do not allocate a temporary buffer
proportional to a record or history. Series, fill, hline and delta headers use
borrowed, type-sensitive defaults; gradient samples stream through their original
serde_json encoding rules. Persistent history iteration still uses a small tree
cursor. Convenience functions returning String still allocate the complete JSON.
The sink may short-write or fail inside a record, so discard partial output when
atomic delivery is required. The convenience function
`public_runtime_result_view_json(&view)` counts and encodes the output in two
passes to reserve one complete String buffer. The owned serializer
`public_runtime_result_json(&result)` uses the same encoding path. Python and
WASM `result`, confirmed-result, and stream-snapshot output paths consume views
while still returning independently owned dictionaries or strings.

`write_public_runtime_changes_json(&changes, &mut sink)` provides the equivalent
streaming contract for realtime deltas. It shares scalar, metadata, drawing and
strategy record rules with complete output, preserving field order, optional
sections and action names. Empty vectors are omitted where the existing delta
schema omits them; absent and present-empty strategy splices remain distinct.
`public_runtime_changes_json(&changes)` retains the existing owned String API.
A sink failure does not alter the changes or runtime state; discard already-written
partial output when the host requires atomic delivery.

For a host that will consume a view after a successful update, use
`update_without_output` or `update_with_context_without_output` instead of
constructing an intermediate owned result. They advance the revision, apply
output retention, synchronize the cursor and clear `last_changes` exactly as
the corresponding snapshot-producing update does. They do not produce a delta.
The `seed_historical_without_output`, `replay_historical_without_output` and
`correct_historical_without_output` families also provide explicit clock
variants. Replay and correction discard forming state on success and require
replicas to reset from the current result and revision. These operations share
the existing execution and failure-atomicity paths; they only omit result
materialization. Python and WASM snapshot-returning seed, replay, correction and
update methods use these operations, followed by borrowed conversion.

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

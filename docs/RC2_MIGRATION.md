# 0.3.0-rc.2 migration and release scope

This candidate has a unique identity: Cargo/CLI/WASM/Python module version
`0.3.0-rc.2`, PEP 440 wheel version `0.3.0rc2`, Git tag `v0.3.0-rc.2`.
The official release assets are two optimized Python wheels, `manifest.json`,
and `SHA256SUMS`. Rust, CLI and WASM remain available through source builds;
their local acceptance binaries are not additional official download assets.

## Resource errors

Collection allocations and copies now share a deterministic allowance of
64 MiB per bar execution. Matrix kernels have an allowance of 100,000,000 work
units per bar execution. Requested evaluators share the parent's budget;
strategy rollback does not refund work already performed. Exhaustion produces
an `E_RESOURCE_BUDGET` runtime error before an oversized allocation or expensive
matrix phase proceeds. Hosts must handle the error rather than assume every
previously accepted workload will still run.

These count logical collection payload/copy bytes and matrix work, not process
RSS, all allocator activity, elapsed time, or total retained session memory.
They do not establish a long-session memory bound.

Rust callers can use `HistoricalRuntime::with_resource_limits` or
`RealtimeRuntime::with_resource_limits` with `ResourceLimits`; `None` disables
the corresponding allowance. Python, CLI and WASM use the defaults and do not
expose configurable limits in this RC. Adapt large scripts or use a Rust
embedding when another allowance is required.

## Realtime time protocol

Seed/history bar timestamps must be strictly increasing. A new realtime bar
must be later than the last confirmed bar. Once a forming bar exists,
replacement and confirmation must use that same bar timestamp. Invalid input
is rejected before candidate execution and leaves the confirmed state and
revision intact. Validate an entire historical batch before sending it.

`time_close` now uses the selected chart/request timeframe. Fixed durations
preserve the host's bar-open offset; weekly/monthly closes use UTC calendar
buckets. Empty timeframe arguments resolve to the chart timeframe.
`timeframe.change` uses chart/request context. Supply accurate chart metadata
and requested datasets through the host-neutral interfaces. Exchange-session
and holiday-calendar fidelity beyond the named fixtures is not qualified.

## Supported delivery and remaining gaps

Python wheels target ordinary GIL-enabled CPython 3.10+ on Windows x86-64 and
glibc Linux x86-64 (`manylinux_2_17` / manylinux2014). macOS, ARM, musllinux and
free-threaded CPython are outside this release matrix.

The runtime implements a tested Pine subset. Strategy/broker behavior is
qualified for named frozen historical and realtime control cases, with explicit
unsupported capability errors. It is not blanket native live-Tick or arbitrary
strategy compatibility. Native visual geometry is also outside this scope.
Hull displayed-PnL residuals remain open (retained maximum 0.006690 USD).

The earlier fixed resource matrix remains `notPassed`: Linux Python Pivot
four-session forming/replacement/confirmation growth was
4.288 / 4.578 / 4.456 against the unchanged limit of four. That measurement
belongs to its original implementation; it is not a new measurement of rc.2.
The new per-bar budgets and sparse rollback optimizations do not close that
release qualification gap. A fresh fixed resource matrix is required for a
stable release. Ordinary WMA/variance/ALMA remain O(L) in window length, and
whole-result materialization still has a cost proportional to retained output.

See [RC2 acceptance](RC2_ACCEPTANCE_20261006.md) for source, build and comparison
evidence, and [the delivery ledger](DELIVERY_ROADMAP.md) for historical pins.

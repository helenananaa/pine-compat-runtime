# 0.3.0 stable acceptance

This release qualifies a documented executable Pine subset on Windows and
Linux x86-64. It includes the merged rc.2 runtime improvements and uses unique
`0.3.0` identities across Cargo, CLI, WASM, the Python module and wheel metadata.
See [migration and limits](STABLE_MIGRATION.md).

## Compatibility and installation

Qualified implementation: `ee9818c725e7726ec86f539fd2eafc0cc20912f6`. The final release
commit adds the acceptance and delivery documentation; qualified core/build
inputs are checked for byte equality before tagging. The published manifest
records the exact release commit. Raw receipts remain under
`.local/stable-release-0.3.0-20261006/`.

Both canonical gates pass 7,569 Rust tests, 798 installed-wheel Python tests,
166 tooling tests run with platform-dependent skips, strict formatting/Clippy,
structural/parity guards and actual Node/WASM smoke checks. Fresh optimized
CLI batch/incremental/realtime-history, direct Rust, installed Python and
actual Node/WASM rerun the unchanged fifteen-case plan on both platforms.
All 180 complete outputs agree at the original tolerances, using all 75 frozen
reference files and the original native comparators. Native data is not
recaptured. Hull's retained displayed-PnL residual remains explicit.

Official pre-tag wheels are freshly installed offline into clean Windows and
actual glibc 2.17 / CPython 3.10 environments. All fifteen complete outputs,
an independent SMA oracle and the Python embedding example pass. Official
assets are two optimized Python wheels, manifest and checksums. Rust, CLI and
WASM are available from source. The tag workflow rebuilds at the final commit;
published bytes are downloaded and independently checked after publication.
See the [machine-readable index](STABLE_ACCEPTANCE_RESULTS_20261006.json).

## Fixed resource acceptance

The original workload and budgets are unchanged: RSI default/alternate and
Pivot, histories of 1,024/16,384/100,000 bars, complete original tails up to
10,000 updates, one/four independent interleaved sessions, two repeats, three
surfaces and two platforms. All 216 trials are fresh; no older trial is reused.
Compilation finishes before measurements; the six platform/surface groups run
sequentially on the same Windows/WSL host.
The host has 24 physical / 32 logical CPUs and approximately 64 GiB physical
memory. Its pre-measurement load and host receipt hash are recorded in the
resource index; this is not a dedicated production-host throughput test.

All individual budgets and median growth checks pass. Compile is limited to
15 seconds, seed to 120 seconds per session, live phase P95 to 5 ms, replica
apply P95 to 1 ms, snapshot plus serialization to 10 seconds per session,
process peak RSS to 512 MiB for one session / 2 GiB for four, and median tail
growth from 1,024 to 100,000 bars to four. The original budget SHA-256 is
`c6d081130645cbffcd2af832a2e57beceda9a8a0f857ac78b788a037808190f6`.

All 540 complete live outputs agree across platforms/surfaces/repeats,
108 stream-zero isolation comparisons and 180 known-dataset historical
controls match. Metadata, operation counts, complete spooled output contracts,
file hashes and artifact/source identities are audited. See the
[resource index](STABLE_RESOURCE_RESULTS_20261006.json) for per-surface worst
cases, all growth ratios and receipt hashes.

The earlier Linux Python Pivot four-session growth failure remains preserved
on its earlier implementation pin. Fresh current-core evidence closes that
release gap. This finite synthetic offline plan qualifies the specified
workload; it does not establish an indefinite-session or arbitrary-workload
bound, production-host throughput, or native live-Tick fidelity.

## Scope

Named strategy/broker cases are qualified. General native live-Tick, exchange
session/holiday calendars, native visual geometry and arbitrary Pine/strategy
compatibility remain outside scope. Hull displayed-PnL residuals remain open
(retained maximum 0.006690 USD). macOS, ARM, musllinux and free-threaded CPython
are outside the wheel matrix. Ordinary WMA/variance/ALMA remain O(L), and full
result materialization remains proportional to retained output.

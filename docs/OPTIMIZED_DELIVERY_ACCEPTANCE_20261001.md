# Optimized independent runtime acceptance

Runtime source: `e73206f0c839e42811ba1375832c6a514f49ae2a`, including the provider-refresh opening-context fix
and an equivalent alert-placeholder predicate simplification required by the
Linux Rust 1.95 Clippy gate. This is a local prerelease receipt, not publication.

## Compatibility and artifacts

Fresh optimized CLI, installed Python wheel, actual Node/WASM and direct Rust
consumer probes are built on Windows x86_64 and Ubuntu 22.04 x86_64 under WSL.
Both complete canonical gates pass, including fresh installed-wheel tests.
Both optimized retained wheels separately pass all 774 Python tests.

Each platform reruns the frozen fifteen-case core set: five complete scripts,
twelve unchanged script/settings cases, a read-only Pivot observer and two fee
controls. Each case compares six complete outputs: direct Rust, CLI batch,
CLI incremental, CLI realtime-history, installed Python and actual WASM.
Thus all 180 primary outputs pass both platform-local and cross-platform full-output comparisons (absolute tolerance 1e-9, relative tolerance 1e-12). Retained native
comparators and the additional RSI/Pivot/SSL checks pass their named assertions.
No new TradingView data is captured. Hull monetary display residuals remain
open at the previously recorded maximum of 0.006690 USD.

The matrix source/reference/input pins are in
`.local/delivery-20261001-v2/plan.json`; platform build provenance and complete
results are retained under its `windows` and `linux` directories. Prior debug,
older native and intrabar receipts retain their original source pins.

CLI `run-incremental` currently loads the complete initially available input
through `HistoricalRuntime::append_bars`. Its parity receipt does not prove
seed-plus-future-one-bar append equivalence for dataset-end-dependent scripts.
The sustained resource probe exercises that latter path separately.

## Frozen resource workload

The budgets remain those in `PRODUCT_STANDARD_CANDLE_RESOURCE_PLAN.json`.
The original full RSI default, RSI SMA+BB/divergence and Pivot Traditional/Auto
daily-based sources run on synthetic minute data at 1,024/16,384/100,000 history
bars and 128/128/10,000 tail bars. Each tail bar has initial forming,
one replacement and confirmation. Pivot also receives cumulative daily forming
and confirmation events. Thirty-two preceding daily provider bars provide the
Pivot warmup required by a sub-day chart seed; no future provider seed is used.

One or four independent sessions share one compiled source and run interleaved
in a process. Exact binary-rational OHLC offsets varying with minute modulo 17
produce distinct signals without language-specific trigonometric drift. Every
case repeats in a fresh process. The measurement plan and payload hashes are
frozen under `.local/resource-20261001-final/`; correctness preflight failures
and earlier corpus attempts remain retained separately.

Updates, provider refresh, replicas, seeding, compile, snapshot and serialization
are timed separately. Direct Rust also times historical appends and retains
the batch/append control outputs. Bindings expose realtime confirmation rather
than a historical one-bar append API, so those modes are not mislabeled.
All event/sample counts and final confirmed-history counts are checked. Resource
measurement begins after all compilation and compatibility jobs finish; cases
and surfaces run sequentially on the same machine.

OS high-water marks are sampled every 20 ms. Windows uses the sum of retained
PeakWorkingSetSize counters across the worker process tree (a conservative upper
bound); Linux uses VmHWM for the direct worker. The Windows collector requires
`psutil`; the installed runtime itself does not. The initial launcher-only Python
trial was interrupted and preserved as invalid for memory acceptance, then
repeated with the repaired sampler. The frozen workloads and budgets are unchanged.
These are process peaks including input conversion, full-output
materialization, verification copies and report construction. They do not
isolate interpreter allocations or prove leak freedom. CPU/RAM and Linux WSL
environment records are retained with the workload.

The Linux native probe was rerun with a buffered JSON file reader after the
unbuffered reader caused almost one syscall per input byte on the WSL mounted
drive. The initial 1,024-bar process took 65.24 seconds despite short measured
runtime operations; the buffered rerun takes approximately 0.48 seconds.
The interrupted cohort and both probe sources/binary hashes are preserved in a
separate repair receipt. Windows native receipts retain their original probe;
both probes execute the same committed runtime and frozen workload.

## Findings and limits

All 216 planned trials were attempted: 212 complete, four Windows native/WASM
four-session long Pivot trials hit the frozen 600-second process timeout.
Completed results agree across both platforms and all surfaces, with zero
cross-surface mismatch groups. All nine session-zero singleton/interleaving
comparisons pass; the other three streams are distinct and repeatable, without
claiming separate singleton control replays for each of them.

Resource qualification is **notPassed**. The table counts completed trials with
absolute phase/memory failures; growth failures and missing controls are listed
separately in the machine-readable receipt. It does not choose the faster repeat.

| Surface | Complete | Absolute budget failures | Largest worker peak |
| --- | --- | --- | --- |
| Windows/rust | 34/36 | 10 | 9.19 GiB |
| Windows/python | 36/36 | 8 | 4.83 GiB |
| Windows/wasm | 34/36 | 8 | 2.29 GiB |
| Linux/rust | 36/36 | 12 | 9.21 GiB |
| Linux/python | 36/36 | 8 | 5.06 GiB |
| Linux/wasm | 36/36 | 8 | 2.24 GiB |

Long-history RSI exceeds the 512 MiB single-session / 2 GiB four-session process
budgets. Native Pivot also has memory/p95 failures, and selected Pivot median
latencies grow more than the frozen fourfold limit. Process timeout includes
input loading, runtime execution, historical control evaluation and report
construction; it is not a diagnosis of the core alone. Each operation/sample
count, full-output hash, timeout and growth comparison is retained.

The Linux sampler logged a process-disappearance race after some workers exited;
its captured high-water marks remain recorded. The current collector catches
that expected race and passed 40 actual short-process exit checks after all
measurements. The package preserves the measured collector revisions, not a
rewritten claim that the repaired revision ran those trials. Neither sampled
peaks nor finite runs prove leak freedom or isolate interpreter allocations.

Pivot's terminal drawing creation differs between an initially shorter seed
followed by individual appends and an initially complete batch. Both full
outputs and their dataset-end indices are retained. This is a context difference
and an unmet same-context comparison, not independent native evidence of a
Pine semantic defect. It is not resolved by dropping drawing fields or forcing
live output to equal historical lookahead output.

## Local package and installation

[Current machine-readable acceptance](OPTIMIZED_DELIVERY_ACCEPTANCE_RESULTS.json)
binds the fifteen-case matrices, resource reports, observer repairs, host records,
archive/manifest hashes and fresh installation receipts. Raw outputs and failed
trials remain under `.local/resource-20261001-final/`; the frozen budget plan is
unchanged. Its inherited debug/platform label is superseded by the packaged
`qualification-scope.json` and current build provenance, without modifying the
older receipt's hash.

The local archive is `.local/delivery-20261001-v2/release.zip` (35,920,469 bytes).
It contains source pinned to the runtime commit, both CLI/wheel/WASM variants,
75 frozen reference/corpus files, schema contracts, examples, verification tools
and 120 individually hashed payloads. Source-archive documents preserve their
commit checkpoint; packaged README/evidence define current qualification.

ZIP SHA-256:
`fe5b6f9e29972f4a3468e7464d294a87592a47c566e0bae952d633c740c3b164`.
Manifest SHA-256:
`d09b255841ff70527b32decd3cca59107bf537ea6174725ba1705982ed1f93fc`.

Both tests use the extracted archive and a fresh venv with local installation
(`--no-index --no-deps`). Windows uses Python 3.10.11 / Node 22.14.0; Ubuntu WSL
uses Python 3.10.12 / Node 24.18.0. The platform wheels are CPython 3.10+ ABI3,
`win_amd64` and `manylinux_2_35_x86_64`. Imported modules are checked to reside in
the fresh environments. CLI/Python/WASM full outputs agree on both platforms
and match the independent SMA(3) oracle: null/null/101/102 for 100/101/102/103.
The source schema versions are result 9, changes 4, realtime session 1 and
render metadata 1. The archive is local and has not been published.

The next resource task is to attribute input, core state, replicas, public output
and verification copies separately, then address measured Pivot scaling before
repeating the unchanged budget gates. Native live Tick, arbitrary scripts,
indefinite memory and all Linux distributions remain outside this qualification.

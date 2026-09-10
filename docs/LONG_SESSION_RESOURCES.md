# Sustained resource measurement

The `long_session_benchmark` example measures work after an existing history,
instead of labeling a replay from an empty runtime as sustained incremental work.
It is an offline host tool. It adds no instrumentation to the runtime API.

Build the native optimized probe first:

```text
cargo build --release -p pine-runtime --example long_session_benchmark
```

For a small exploratory run on Windows:

```text
python scripts/benchmark_long_session.py --binary target/release/examples/long_session_benchmark.exe --source tests/fixtures/benchmark/strategy_trend.pine --output pilot.json --history-bars 1000 --tail-bars 256 --repetitions 2 --replacements-per-bar 2
```

This is not a formal acceptance command. Reports always say
`qualification: notEvaluated`; numerical budgets must be frozen and evaluated
separately before claiming the delivery resource targets have passed. Those
targets remain 100,000 historical bars, 10,000 subsequent appends and 10,000
forming replacements with confirmations, plus increasing dense/collection/
Magnifier workloads. A small pilot does not substitute for them.

## Protocol

Each repetition starts fresh. Historical execution loads the complete prefix,
then appends each tail bar to that same runtime. Its complete final output must
equal a separate full-batch execution. The probe also checks that the runtime
processed the entire prefix and tail and that output stays equal across repeats.

For ordinary chart execution, a separate realtime runtime loads the prefix once.
Every tail bar receives an initial forming update at its open, the requested
number of replacements alternating its high/low close, then confirmation using
the original closed bar. Actual confirmed bar count is checked. The same live
sequence must reproduce across repetitions; it need not equal historical OHLC
execution. `formingExecutesScript` distinguishes indicators/every-tick strategies
from default strategies that return confirmed state on forming updates.

Magnifier inputs cover the entire chart sequence, including the tail. The probe
requires enabled Magnifier and supplied intrabars to agree, so absent data cannot
silently become a Magnifier performance measurement. This is a probe constraint,
not a change to the core's existing explicit fallback contract. Magnifier live
phases are excluded explicitly because that input is historical-only.

The collector validates every operation count:

| Phase | Samples |
| --- | --- |
| Compile | One per process |
| Historical/live seed | One per repetition |
| Tail append | Tail bars times repetitions |
| Forming initial/confirm | Tail bars times repetitions |
| Forming replacement | Tail bars times replacements per bar times repetitions |
| Final snapshot/serialization | One per repetition and execution mode |

Append timing excludes final snapshots and serialization. Forming/confirmation
timing includes the runtime's returned full snapshot, but excludes destruction
of that returned snapshot. Separate final snapshot and serialization timings
are also provided. P95 is emitted only with at least 100 operation samples; these
are pooled operations, not independent process samples.

Memory checkpoints are process-wide cumulative peaks, labeled by source, after
historical seed, tail append, live seed, live tail and verification. Windows
reports peak working set and peak commit charge; Linux reports VmHWM. They are
not current retained memory, not a per-runtime allocation ledger, and exclude
the final benchmark report's own JSON rendering. Unsupported measurements remain
null. Do not infer leak freedom from these peaks alone.

## Inputs and evidence

By default the collector generates the existing deterministic synthetic
regression bars. `--bars-csv` accepts an exact prefix-plus-tail dataset; no row
is trimmed to fit the requested counts. `--library-source KEY=path` supplies
complete exact library dependencies. The probe currently uses the default chart
context and has no external request provider or execution-clock input.

`--magnifier` generates two synthetic intrabars per generated chart bar. It is
rejected with a supplied real CSV, so invented intrabar paths cannot be confused
with real data. Existing runtime intrabar limits remain in force.

Reports retain source, library, input, probe/support-module, runner and binary
hashes, Git state, phase samples and final-output hashes. Any probe failure,
incomplete phase/count or failed consistency check produces a failed report.
No failure is converted to a passing empty result. These comparisons are
internal execution checks, not independent TradingView reference coverage.

Before D4 acceptance, select the real representative workload/input profiles,
freeze latency and process-memory budgets, execute the full stated scale, and
review resource-limit failure behavior. This probe supplies the measurements;
it does not by itself complete D4 or the product goal.


`verify_long_session_budget.py` evaluates a report against a separately frozen
Windows budget plan. The plan fixes source, binary, payload, commit, clean-tree
state and all execution sizes. Every timing phase and both process-memory
budgets are mandatory. The verifier recomputes statistics from operation samples,
checks counts and memory checkpoint identities, and records hashes of the plan
and report. It rejects partial/failed runs and does not trust cached percentiles.
This verifier currently requires Windows memory counters; it does not certify a
Linux run using Windows memory assumptions.

## Progress and supplementary wall-cost attribution

The sustained collector now writes live probe stderr to a progress file next to
the output (`*.progress.jsonl`), or to `--progress-log`. The file remains readable
while the process runs and is retained on timeout or failure; the report records
its path and SHA256. Never reuse an input, executable or output path for this log.
Milestones identify compilation, history/verification, live seeding and each 256
completed tail bars. They are observations, not acceptance receipts.

`diagnostics.snapshotDropTimingsMs` separately times destruction of the owned
snapshots returned by live seeding and initial/replacement/confirmed updates.
`wallBeforeReportMs` includes input handling, verification and runtime lifetimes
up to final report construction. The existing 11 timing phases, their sample
counts, memory checkpoints and frozen budgets remain unchanged. These additive
diagnostics do not relax any budget and do not include final report serialization.

A Windows debug diagnostic pilot on 10,000 history + 256 tail bars, two repeats
and one replacement recorded 9,815.65 ms wall time, 7,708.52 ms in the original
timed phases and 622.67 ms of snapshot destruction. The remainder includes
verification and other uninstrumented work. This is not the frozen 100k/10k
acceptance run and does not establish its performance. Evidence is retained in
`.local/delivery-20260909/resources/progress-pilot-10k256*`. Seven executable/
collector tests and four frozen-budget tests pass; timeout retention is tested
separately from successful real-probe progress.

## Default-strategy forming copy reduction

At clean baseline 843cd3fda, a release probe with 100,000 history bars and a
64-bar tail (two repeats, one replacement) measured forming initial/replacement/
confirmation P95 of 55.732/54.259/51.971 ms. For default strategies with both
every-tick and order-fill recalculation disabled, the implementation copied the
confirmed runtime and then copied retained user state again despite executing no
script. The optimized path clones the last visible runtime once and advances
only broker/scheduler state, with the same session validation and failure atomicity.

The same workload produced 32.940/33.347/47.151 ms P95 and 20.001 seconds total
wall time versus 25.471 seconds before. Historical/live output hashes are equal,
and six distinct workload comparisons retain identical full outputs. These are
pooled short-tail observations, not independent-process statistics or full-scale
acceptance. Full Windows gates pass 6655 Rust / 710 installed-wheel Python /
117 tool tests and actual WASM. The executing-strategy path is unchanged.

A second experiment reused the forming instance with component-level rollback.
Its replacement median improved, but total time was 20.819 seconds and confirmed
P95 was 52.544 ms in that trial. It was not selected on this evidence; its patch,
source, binary and measurements remain in `resources/broker-only-v2-*`.
The selected implementation is the simpler first variant. No original phase,
size, tolerance, memory limit or 1800-second observation window was loosened.

## Shared plot checkpoints and release profile

The 100k trend capacity inspection finds two retained series values but 200,000
plot values. Runtime plot checkpoints now share an immutable plot vector through
`Arc`; a writer detaches it with room for one new value/color per plot. The public
`RuntimeResult.plots` remains an owned `Vec<PlotSeries>`, and previously returned
values remain independent. Runtime forks, forming replacements, color changes
and caller mutation are covered by dedicated isolation tests. Capacity figures
reflect actual backing allocations and are not unique-memory measurements.

At 100k history + 64 tail, the initial shared-plot change reduced wall time from
23.864 to 19.952 seconds with identical complete historical/live outputs. Reserving
the next append during detachment reduced confirmation median from 37.678 to
31.216 ms in a subsequent trial. Six distinct workloads retain exact output parity.

The workspace release profile now uses ThinLTO and one code-generation unit. It
does not enable fast-math or host-specific CPU instructions. Embedding Rust
applications control their own Cargo profiles; workspace settings are not forced
on downstream applications. With these settings, the 100k+64 diagnostic measured
24.678/23.001/36.048 ms P95 for initial/replacement/confirmed updates. A 100k+256
diagnostic measured 28.324/24.961/39.185 ms P95, but updates plus returned-snapshot
destruction still averaged 91.929 ms per tail bar. These observations do not
establish the full 100k/10k, two-repeat, 1800-second acceptance. All original
budgets and input sizes remain required. Evidence is retained in `resources/plot-cow-*`.

Validation passes 6662 Rust / 712 installed-wheel Python / 117 tool tests and
actual WASM in the normal gate. The release profile separately passes 23 focused
Rust tests, actual release WASM smoke and 712 tests against its installed wheel.
Its complete TechnicalRating/3 + ta/9 + RelativeValue/3 graph matches all 63,399
native reference values over 21,133 bars; four CLI modes, Python and WASM have
identical complete outputs. Artifacts are in `resources/plot-cow-release-artifacts`.
These validate semantics and build settings, not the still-failing D4 full-run
window. No new full-scale acceptance was awarded from short-tail results.

The equity recorder now checks the last row through immutable access before
choosing replacement or append. Previously, `last_mut()` detached shared history
even for a new bar, bypassing the append helper's one-allocation path. Numerical
calculations and same-bar row replacement are unchanged. In a paired 100k+256
release diagnostic, update-plus-drop average changed from 66.531 to 65.852 ms per
tail bar (wall time 39.952 to 39.676 seconds). This is a small measured improvement,
not a new resource acceptance. Six workload outputs match exactly; the new equity
row isolation test and full gates pass (6663 Rust / 712 Python / 117 tools / WASM).

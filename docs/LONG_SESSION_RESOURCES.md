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

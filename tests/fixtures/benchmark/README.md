# Strategy performance workloads

Original offline benchmark sources; these are workload inputs, not new Pine
support claims or independent TradingView references.

- `strategy_trend.pine`: SMA crossover/history and orders.
- `strategy_dense.pine`: repeated order/close lifecycle.
- `strategy_recalculation.pine`: actual fill-triggered recalculation.
- `strategy_magnifier.pine`: host-provided lower bars and price exits.

The runner also uses existing matrix and calc-on-every-tick fixtures. All input
bars are deterministic synthetic data. The report records source/input/binary
hashes, per-phase raw timings, runtime profiles and per-process RSS.

Build and run from the repository root:

```bash
cargo build --release -p pine-runtime --example strategy_benchmark
python3 scripts/benchmark_modern_strategy.py \
  --binary target/release/examples/strategy_benchmark \
  --output /tmp/strategy-benchmark.json
```

Use the actual Cargo target directory if `CARGO_TARGET_DIR` is overridden. The
runner does not build inside measured phases. Persist the report and build log
outside `/tmp` for audit evidence. Each input/size runs in a fresh child process;
RSS includes compilation and verification overhead, not only retained runtime
storage. Magnifier live updates are excluded explicitly, because those inputs
are historical-only. Historical and incremental JSON must match; repeated live
sequences must match each other, without asserting historical/live equivalence.

A `partial` report means measurements exist but the baseline gate (at least 10
iterations, 100 replacements, three sizes, and RSS availability) is incomplete.
Failures return a nonzero exit and never produce a successful baseline status.
No optimization or speedup is claimed. Resource deltas describe these workloads,
not a proof of general complexity or production capacity.

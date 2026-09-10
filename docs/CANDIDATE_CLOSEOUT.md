# Local 0.3.0-rc.1 closeout

This is a TV-blocked candidate, not a stable release and not full Pine
compatibility. Nothing was pushed, tagged, or published.

## Identity

| Field | Value |
| --- | --- |
| Cargo | `0.3.0-rc.1` |
| Python wheel | `0.3.0rc1` |
| HEAD | `2792a095075d58e4a7145d3b97ce7d1286eb2800` |
| Artifact source | `b9cae7ea5343f284e6f453b6c88c678d59617056` (follow-up is `cfg(test)` only) |
| Channel | prerelease |
| Schemas | analysis 5, runtime 8, host-requirements 1, render metadata 1 |

Local artifacts: `.local/candidate-0.3.0-rc.1/`
Execution docs: `docs/DELIVERY_SURFACES.md`, `docs/CANDIDATE_ACCEPTANCE.md`, `docs/RUST_EMBEDDING.md`, `docs/HOST_REQUIREMENTS.md`
Evidence scratch: the implementer scratch directory named by the goal harness.

## Completed and verified

- Remaining D4 named workloads at 10,000 history + 1,000 tail, two repeats, one replacement: every-update, dense orders, collection, Magnifier historical-only. Pilot froze numerical budgets; a second run passed those budgets. Modes recorded per workload. Complete-output snapshot/serialization/drop costs recorded. Trend 100k/10k Windows result reused from 3ca746976.
- Resource over-limit (while-loop ceiling), failure atomicity, independence of previously returned results, and post-failure session behavior on RealtimeRuntime and Python sessions.
- Four-surface identity and embedding path: compile → requirements → historical → incremental (Rust/CLI) → realtime (Rust/CLI/Python) → error → owned result. WASM remains historical-only by capability list.
- Windows installed artifacts launched twice (CLI MACD, Rust embed JSON, installed wheel `run_script`, WASM Node).
- Linux native Ubuntu 22.04 WSL `scripts/verify.sh` and release CLI/wheel. Linux CLI launched twice on the public MACD fixture.
- Frozen independent references on the candidate CLI: TechnicalRating 63,399 values; G3 r3 65 trades / r2 41 trades; default-HTF 422,660 values; paired historical Magnifier; long/short margin 7,992 values. Original hashes and 1e-9 tolerances unchanged.
- Windows/Linux gates: 6668 rust / 715 installed-wheel Python / 122 tool tests plus actual WASM/Node.

## Implemented but not finally qualified

- manylinux2014 / `manylinux_2_17_x86_64` wheel: Docker engine was down; native Ubuntu wheel is `manylinux_2_35_x86_64`. Prior manylinux2014 qualification at da099ac11 used interim `0.2.0` labeling and is not this identity.
- D4 remaining workloads are qualified at the stated 10k+1k increasing-size scale, not at the trend 100k/10k window.
- Configured-provider completeness and dynamic request-argument resolution remain discovery-only.
- This candidate is not a GitHub Release and is not a stable product ship.

## Explicit failure

- Live-tick native comparison: **16/896** exit-price mismatches (`78246.2` vs `78246.1`), re-observed on the candidate wheel. Still a failed comparison.
- Public r1 independent-reference denominator remains **0/482**.
- B1 remains **`UNVERIFIED_INTERNAL_ORDER`**. Internal G3 r2 `b1-observable`/`b1-recalc` trade comparisons pass; that does not upgrade B1 to independent broker evidence.

## Waiting on TradingView or other external input

- New TradingView output collection (forbidden in this task).
- Exact live-tick fill pricing / hidden price events.
- Broader account/tick profiles.
- Unresolved dynamic request arguments as literal keys.
- Independent Tester oracles for public r1 and B1.

Unblocking those claims requires new independent reference captures, not a tolerance change or a deleted product goal.

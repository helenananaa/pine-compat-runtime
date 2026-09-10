# Local 0.3.0-rc.1 closeout

This is a TV-blocked candidate, not a stable release and not full Pine
compatibility. Nothing was pushed, tagged, or published.

## Identity

| Field | Value |
| --- | --- |
| Cargo | `0.3.0-rc.1` |
| Python wheel | `0.3.0rc1` |
| Candidate test baseline | `2792a095075d58e4a7145d3b97ce7d1286eb2800` |
| Reconciliation review baseline | `c03cf77bb355113909e3798b67d7d859a8b1b080`; subsequent local docs/evidence/test changes |
| Artifact source | `b9cae7ea5343f284e6f453b6c88c678d59617056` (follow-up is `cfg(test)` only) |
| Channel | prerelease |
| Schemas | analysis 5, runtime 8, host-requirements 1, render metadata 1 |

Local artifacts: `.local/candidate-0.3.0-rc.1/`
Execution docs: `docs/DELIVERY_SURFACES.md`, `docs/CANDIDATE_ACCEPTANCE.md`, `docs/RUST_EMBEDDING.md`, `docs/HOST_REQUIREMENTS.md`
Durable evidence: `.local/candidate-0.3.0-rc.1/evidence/manifest.json`.
Original goal scratch was removed. Missing original full-gate/reference logs
are disclosed in `CANDIDATE_ACCEPTANCE.md`; do not treat old summaries as
recovered raw receipts. Fresh wheel, WASM, tool and resource rechecks are separate.

## Completed and verified

- Remaining D4 named workloads at 10,000 history + 1,000 tail, two repeats, one replacement: every-update, dense orders, collection, Magnifier historical-only. Pilot froze numerical budgets; a second run passed those budgets. Modes recorded per workload. Complete-output snapshot/serialization/drop costs recorded. Trend 100k/10k Windows result reused from 3ca746976.
- Resource over-limit (while-loop ceiling), failure atomicity, independence of previously returned results, and post-failure session behavior on RealtimeRuntime and Python sessions.
- Four-surface identity and embedding path: compile → requirements → historical → incremental (Rust/CLI) → realtime (Rust/CLI/Python) → error → owned result. WASM remains historical-only by capability list.
- Windows installed artifacts launched twice (CLI MACD, Rust embed JSON, installed wheel `run_script`, WASM Node).
- Linux native Ubuntu 22.04 WSL `scripts/verify.sh` and release CLI/wheel. Linux CLI launched twice on the public MACD fixture.
- Frozen independent references on the candidate CLI: TechnicalRating 63,399 values; G3 r3 65 trades / r2 41 trades; default-HTF 422,660 values; paired historical Magnifier; long/short margin 7,992 values. Original hashes and 1e-9 tolerances unchanged.
- Windows/Linux gates: 6668 rust / 715 installed-wheel Python / 122 tool tests plus actual WASM/Node.
- Reconciliation: manylinux2014 qualified for this candidate at 2792a0950,
  with offline release build, auditwheel and 715 installed tests. The original
  Ubuntu manylinux_2_35 wheel is retained separately. Fresh Windows installed
  tests (715), delivery tool tests (123), retained WASM smoke, two launches per
  surface and five resource-budget rechecks pass. Original combined gate logs
  remain unavailable, as disclosed in CANDIDATE_ACCEPTANCE.md.
- Fresh TechnicalRating recheck preserves original reference hashes and 1e-9
  tolerances: all 63,399 values and complete CLI four-mode / Python / WASM
  outputs agree. Evidence: `evidence/technical-recheck/` in the candidate root.

## Implemented but not finally qualified

- Linux long-session resource qualification is separate from the installed-wheel checks.
- D4 remaining workloads are qualified at the stated 10k+1k increasing-size scale, not at the trend 100k/10k window.
- Configured-provider completeness and dynamic request-argument resolution remain discovery-only.
- This candidate is not a GitHub Release and is not a stable product ship.

## Explicit failure

- Live-tick native comparison: **16/896** exit-price mismatches (`78246.2` vs `78246.1`), re-observed on the candidate wheel. Still a failed comparison.

## Unverified coverage

- Public r1 independent-reference denominator remains **0/482** (missing reference coverage, not failed comparisons).
- B1 remains **`UNVERIFIED_INTERNAL_ORDER`**. Internal G3 r2 `b1-observable`/`b1-recalc` trade comparisons pass; that does not upgrade B1 to independent broker evidence.

## Waiting on TradingView or other external input

- New TradingView output collection (forbidden in this task).
- Exact live-tick fill pricing / hidden price events.
- Broader account/tick profiles.
- Independent Tester oracles for public r1 and B1.

Unblocking those claims requires new independent reference captures, not a tolerance change or a deleted product goal.

## Engineering and host requirements still outside acceptance

Dynamic request admission/resolution and provider completeness are engineering/host
contract boundaries, not automatically TV blockers. Non-trend resource budgets
are pilot-derived regression baselines; an embedding-specific product SLA and
concurrency/retention budgets are not yet established. See CANDIDATE_ACCEPTANCE.md.

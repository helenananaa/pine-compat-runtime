# Independent runtime delivery

Owner: Codex. Implementation/review support: bounded Grok tasks in isolated
worktrees. Started 2026-09-09. Status: active, no stable release accepted yet.

## Candidate scope and acceptance

First stable candidate targets deterministic standard-candle v5/v6 indicators
and strategies, host-supplied data/libraries, historical and incremental runs,
and the documented realtime lifecycle. Preserve existing legacy profiles and
their preview/experimental labels. Full Pine compatibility is not a gate.

The target workload set must include unmodified TechnicalRating/3 with exact
transitive dependencies, the separately frozen G3 strategy batches, and new
reference cases for request/security, Magnifier, and margin/account behavior.
Supporting an import does not promise every export in that library is callable.
Do not trim original dependency sources to pass admission.

Initial market scope: standard candles, explicitly supplied price grid,
same-currency linear accounting. Foreign-currency conversion, non-unit contract
multipliers and nonstandard charts remain unsupported unless demanded by the
selected workload; analysis/host contracts must disclose those limitations.

Resource qualification targets: 100,000 historical bars on the trend workload,
10,000 subsequent appends, and 10,000 forming replacements with confirmations;
also measure dense orders, collection and Magnifier workloads at increasing
sizes. Freeze numerical latency/memory budgets BEFORE acceptance runs using
host measurements and the embedding requirements. Record complete-output costs
separately; never infer bounded memory from history retention alone.

All seven product requirements remain mandatory: real complete scripts,
execution-mode correctness, independent broker evidence, explicit host
capabilities, resource qualification, four delivery surfaces, and matching
commit/artifact/docs/evidence. Internal golden agreement is not an oracle.

## Task ledger

| ID | Result required | Owner | State / evidence | Acceptance |
| --- | --- | --- | --- | --- |
| D0 | Recover and freeze prior qualified changes | Codex | HEAD 45cbd6586 plus 104-file library-arrays manifest; all hashes matched at intake; staged/unstaged patches preserved under .local/delivery-20260909 | Review provenance; full Windows gate; local commit; recheck committed source |
| D1 | Complete real TechnicalRating dependency chain | Grok implementation, Codex oracle/review | simple scalar parameter parser gap; exact RelativeValue/3 missing; series/default/local clear-push already implemented | Original library sources and dependency identities retained; reachable exports run; independent outputs; mode and host parity |
| D2 | Broaden independent execution evidence | Codex | g3-chrome-r2 and g3-real-strategy-r3 remain separate; public r1 reference is 0/482 in latest retained run | Matched closed-bar, requested/lower-bar inputs; trades, fees, positions, margin/equity compared; all differences dispositioned |
| D3 | Explicit embedding/data contract | Codex + bounded Grok audit | Price grid exists; account/context assumptions need product-level review | Required inputs, unsupported account/chart modes, source diagnostics and versioning documented and checked across hosts |
| D4 | Resource and long-session qualification | Grok measurement, Codex acceptance | Existing small benchmark and checkpoint optimization are historical baselines | Frozen workloads/budgets, repeatable release measurements, correct output, growth/limits documented |
| D5 | Deliver candidate from a known commit | Codex | No new release; working package version 0.2.0 | Windows/Linux installed wheels; Rust embedding example, CLI package, generated WASM/Node package; checksums, versions, migration docs and acceptance manifest agree |

## Evidence already available

- `LOCAL_FUNCTION_ARRAYS_AUDIT.md`: bounded arrays implementation; retained
  6,558 Rust / 676 installed-wheel Python / 101 tool tests plus real WASM.
- `STRATEGY_MODERN_NEXT_CYCLE_AUDIT.md`: two actual strategy families, three
  scenarios; 65 trades and 14,711 series values compared with no differences.
- `STRATEGY_MODERN_G3_CLOSEOUT_AUDIT.md`: separate six-case reference batch;
  price grid and commission corrections. Do not merge its denominator with r1.
- `STRATEGY_MODERN_G5_CHECKPOINT_OPTIMIZATION_AUDIT.md`: measured small-workload
  speedup; not a long-session resource guarantee.

Raw third-party sources and captures stay in ignored local evidence directories;
public regression tests must be original. Private TradingView B1 order remains
UNVERIFIED_INTERNAL_ORDER; validate observable outcomes without guessing it.

## Working protocol and next action

Grok receives an exact baseline, finite turns, explicit file/task boundaries,
minimal permissions, no subagents and no push/publish/config changes. Review
each patch and command result independently before integration. Keep current
worktree, committed verification and published status separate.

Current priority is D0 alongside D1 investigation and reference acquisition:
the prior work is substantial but not committed, and a complete library chain
exercises multiple real language features together. D2-D5 remain open, not
waived by a passing language slice. External publication requires user approval.

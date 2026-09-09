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
| D0 | Recover and freeze prior qualified changes | Codex | Complete: 85f42f20f tooling, 149d10c7a semantics, 7393379b3 roadmap; committed Windows full gate exit 0; intake 104/104 hashes match | Completed locally; Linux/new release qualification remains D5 |
| D1 | Complete real TechnicalRating dependency chain | Grok implementation, Codex oracle/review | Simple slice b8e38333d; spaced-version fix c2c028269 qualified, exact RelativeValue/3 captured; compound declarations and transitive function resolution remain blockers | Original library sources and dependency identities retained; reachable exports run; independent outputs; mode and host parity |
| D2 | Broaden independent execution evidence | Codex | EMA candidate removes the 22/1,704 initial mismatches; five separate controls now match (see EMA_INITIALIZATION_AUDIT.md); downstream regression review pending; old G3/r1 denominators unchanged | Matched closed-bar, requested/lower-bar inputs; trades, fees, positions, margin/equity compared; all differences dispositioned |
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

Current priority is D1 implementation and reference acquisition. D0 is now
committed and Windows-verified (6,558 Rust, 676 installed-wheel Python, 101 tool
tests plus generated WASM/Node). Logs and intake patches remain under
`.local/delivery-20260909/`. This is not Linux or published release acceptance.

Exact RelativeValue/3 was captured from Chrome version history (2024-11-26,
19:40; source comment v3); its missing-source blocker is resolved. Original
Chrome probes confirm simple input admission and series/const rejection.
An original two-layer library probe now proves a further blocker:
`inner.value` is unresolved despite both exact libraries being supplied.
Transitive import resolution follows simple parameter implementation; source
identity, private visibility and per-import state must survive that extension.

Full TechnicalRating/3 runs in TradingView. Its first 300-bar export was
superseded by an additional, separately retained 21,137-bar capture beginning
at bar_index=0. The frozen reference contains 21,133 closed bars with no skipped
warmup. Raw source/CSV and normalized input/output hashes are in
`.local/delivery-20260909/technical-reference-manifest.json`; this reference is
ready but local full-library execution remains blocked.

The independent standalone EMA probe discovered an actual numerical gap:
TradingView initializes EMA3 with SMA3 on bar 2 and EMA200 with SMA200 on bar
199, whereas the current runtime emits EMA from the first bar. Identical
OHLCV/timestamps were checked across both captures; SMA3 and bar_index agree.
See `.local/delivery-20260909/ema-comparison-before.json`. Before changing EMA,
verify version controls and na/length behavior, review dependent indicators
and intentional golden changes. This evidence-backed correctness task now
runs alongside library admission; it cannot be hidden by warmup skipping.

Simple slice final Windows qualification: 6,567 Rust, 677 freshly installed
wheel Python, 101 tool tests, real WASM/Node, 879 CLI snapshots and 583 required
runtime goldens. Main integration has byte-equivalent core/test/build content
to the verified isolated commit. Installation artifacts are still candidate
builds, not a stable release. D2-D5 remain open. External publication requires
user approval.

Latest continuation: c2c028269 fixes spaced version annotations, with full
Windows gate (6,575 Rust / 677 installed-wheel Python / 101 tool tests plus
real WASM/Node) and no golden refresh. The exact RelativeValue/3 no longer
raises a version conflict. Current complete-library diagnostics are retained
as `technical-after-version.json`; parser errors are not counted as independent
defects until recovery cascades are resolved.

EMA candidate remains isolated at `E:/projects/pine-interpreter-delivery-ema`.
Modern initialization, v5/v6 missing-value controls, recovery and legacy v4
reference comparisons pass without skipped warmup. 932 candidate outputs were
captured into ignored storage; nine semantic output changes plus three
last-bit math differences require disposition. No golden was overwritten.
Next: v3 control and affected loop/request review, select only justified
golden updates, run the full gate and cross-host references, then integrate.
See [EMA candidate audit](EMA_INITIALIZATION_AUDIT.md) and
[version annotation audit](VERSION_COMMENT_SPACING_AUDIT.md).

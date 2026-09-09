# Documentation Guide

This directory contains current runtime contracts, active roadmaps, design
gates, and completed phase records. Use the hierarchy below so that an older
plan is not mistaken for a current compatibility claim.

## Source-Of-Truth Order

1. [`tests/fixtures/conformance.tsv`](../tests/fixtures/conformance.tsv), its
   referenced fixtures, and generated snapshots define the executable
   compatibility claims.
2. The [root README](../README.md), semantic and execution documents,
   diagnostic references, and release notes describe the current public
   contract.
3. [Task Breakdown](TASK_BREAKDOWN.md),
   [Next Internal Capability Plan](NEXT_INTERNAL_CAPABILITY_PLAN.md), and
   [Long-Term Execution Plan](LONG_TERM_EXECUTION_PLAN.md) track current
   maintenance and future work. The
   [Modern Strategy Five-Stage Execution Plan](MODERN_STRATEGY_FIVE_STAGE_EXECUTION_PLAN.md)
   is the active step-by-step order when strategy work is selected.
4. Phase plans, phase audits, design gates, and historical review documents
   record how a slice was designed or closed. They remain useful evidence, but
   their roadmap wording does not override the conformance matrix.

Generate the current compatibility matrix with:

```text
cargo run -p pine-cli -- matrix
cargo run -p pine-cli -- matrix --format json
```

## Current Project Documents

- [Architecture](ARCHITECTURE.md): crate boundaries and host-neutral
  architecture.
- [Language Scope](LANGUAGE_SCOPE.md): supported language shape and explicit
  boundaries.
- [Execution Semantics](EXECUTION_SEMANTICS.md): historical and realtime
  execution behavior.
- [Semantic Model](SEMANTIC_MODEL.md): types, qualifiers, calls, collections,
  and imports.
- [Series Model](SERIES_MODEL.md): history, retention, and series storage
  semantics.
- [Built-In Signatures](BUILTIN_SIGNATURES.md): built-in signatures and
  supported argument subsets.
- [Conformance](CONFORMANCE.md): compatibility matrix policy and fixture
  requirements.
- [Diagnostic Codes](DIAGNOSTIC_CODES.md): stable diagnostic codes.
- [Realtime Model](REALTIME_MODEL.md): forming-bar rollback and intrabar state
  behavior.
- [Release Notes](RELEASE_NOTES.md): published release history and accumulated
  changes for the next release.
- [Releasing Binary Wheels](RELEASING.md): GitHub Actions wheel matrix, release
  contract, and application update boundary.

## Status And Roadmap Documents

- [Modern Function Default Parameters](STRATEGY_MODERN_DEFAULT_PARAMETERS_AUDIT.md): scalar optional arguments, caller scope, v5/v6 reference validation and the next TechnicalRating blockers.

- [Modern Strategy Next Cycle](STRATEGY_MODERN_NEXT_CYCLE_AUDIT.md): current G3 worktree closeout, real-strategy reference expansion, and explicit series parameter slice.

- [Strategy G5 Baseline Review](STRATEGY_MODERN_G5_BASELINE_REVIEW_AUDIT.md):
  corrected true-append/realtime timings, release probe, resource measurements
  and remaining optimization gates.
- [Remaining Modern Strategy Blockers](STRATEGY_MODERN_REMAINING_BLOCKERS_REVIEW.md):
  historical dependency/version/template probes; current G3 evidence and the next TechnicalRating blocker are linked in its dated updates.

- [Modern Strategy Five-Stage Execution Plan](MODERN_STRATEGY_FIVE_STAGE_EXECUTION_PLAN.md):
  detailed Chinese execution steps for frozen corpus measurements, language
  blockers, independent strategy-result validation, account/report contracts,
  and measured performance improvements. Includes dependencies, deliverables,
  commands, acceptance gates, and recovery from missing inputs. Planned work,
  not a claim that these five stages are complete.
- [Strategy Accuracy Next Execution Plan](STRATEGY_ACCURACY_NEXT_EXECUTION_PLAN.md):
  step-by-step Chinese plan for mixed-family OCA, host-neutral session risk
  windows, standard inter-bar gaps, and corpus-driven follow-up, with explicit
  evidence gates and stop conditions. Stages A–C are closed. The later five-stage plan completed the r1
  inventory and a separate six-case G3 reference batch; r1 itself still lacks
  independent references. Earlier blocker wording is historical.
- [Task Breakdown](TASK_BREAKDOWN.md): high-level baseline and ongoing work
  status.
- [Next Internal Capability Plan](NEXT_INTERNAL_CAPABILITY_PLAN.md): recommended
  order for the next small, fixture-backed slices.
- [Long-Term Execution Plan](LONG_TERM_EXECUTION_PLAN.md): completed phases and
  remaining broad backlog.
- [Pure Internal Roadmap](PURE_INTERNAL_ROADMAP.md): interpreter-internal design
  directions.
- [Strategy Broker Next Execution Plan](STRATEGY_BROKER_NEXT_EXECUTION_PLAN.md):
  Stage 17-23 implementation record. Stage 18g true OHLC path execution and
  Stage 23 Bar Magnifier fill wiring are closed. Later OCA, session-window,
  and ordinary-gap closeout is recorded by the strategy accuracy plan;
  current follow-up is in the five-stage plan.
- [Strategy Stage 23 Bar Magnifier Fill Wiring Execution Plan](STRATEGY_INTERNAL_STAGE23_BAR_MAGNIFIER_FILL_WIRING_EXECUTION_PLAN.md):
  step-by-step plan for behavior locking, the lower-bar sequence cursor,
  unified broker integration, recalculation, CLI/Python/WASM host inputs,
  conformance evidence, stop conditions, and closeout gates.
- [Strategy Stage 23 Bar Magnifier Fill Wiring Audit](STRATEGY_INTERNAL_STAGE23_BAR_MAGNIFIER_FILL_WIRING_AUDIT.md):
  closed Stage 23 record for host schema, fill identity, fallback diagnostics,
  host adapters, snapshots, and remaining exclusions.
- [Strategy Stage 18g True OHLC Path Execution Plan](STRATEGY_INTERNAL_STAGE18G_TRUE_OHLC_PATH_EXECUTION_PLAN.md):
  step-by-step reference lock, path model, candidate ordering, broker identity,
  fill integration, rollback, fixture matrix, and closeout gates for Stage 18g.
- [Strategy Stage 18g True OHLC Path Audit](STRATEGY_INTERNAL_STAGE18_TRUE_OHLC_PATH_AUDIT.md):
  official review, Tester evidence, sample-level A/C/D locks, B1
  `UNVERIFIED_INTERNAL_ORDER` amendment, implemented path, fixtures, snapshot
  allowlist, and Stage 18g closeout.
- [Strategy Broker Stage 17-22 Integration Audit](STRATEGY_BROKER_STAGE17_22_INTEGRATION_AUDIT.md):
  recovered worktree scope, commit boundary, final verification, Stage 18f
  correction, and remaining compatibility limits.
- [Legacy Indicator Compatibility Execution Plan](LEGACY_INDICATOR_COMPATIBILITY_EXECUTION_PLAN.md):
  indicator-only v1-v4 compatibility, corpus measurement, versioned lowering,
  execution phases, and release gates; legacy strategies are explicitly out of
  scope.
- [v0.3 Indicator Compatibility Execution Plan](V0_3_INDICATOR_COMPATIBILITY_EXECUTION_PLAN.md):
  post-v0.2.0 authorized-corpus expansion, failure-cluster prioritization, and
  indicator-only v0.3 release gates.
- [v0.3 Legacy Corpus R2 Readiness Baseline](V0_3_LEGACY_CORPUS_R2_BASELINE.md):
  private authorized-corpus intake, corpus-selected syntax measurements,
  failure ranking, and the next indicator-only decision boundary.
- [v0.3 Legacy Corpus R3 Permissive Baseline](V0_3_LEGACY_CORPUS_R3_PERMISSIVE_BASELINE.md):
  commit-pinned public permissive intake, 51-indicator v4 floor, and
  privacy-preserving local evidence boundary.
- [Legacy Indicator Phase 0 Baseline](LEGACY_INDICATOR_PHASE0_BASELINE.md):
  reproducible seed-corpus composition, stage rates, input availability, and
  ranked legacy failure clusters before compiler changes.
- [Legacy Indicator Phase 1 Audit](LEGACY_INDICATOR_PHASE1_AUDIT.md): validated
  dialect selection, script-mode gates, strategy exclusion, and public analysis
  schema synchronization.
- [Legacy Indicator Phase 2 Audit](LEGACY_INDICATOR_PHASE2_AUDIT.md): versioned
  rule catalog, scoped fallback resolution, canonical HIR lowering, deterministic
  reports, and translator cache revision.
- [Legacy Indicator Phase 3 Audit](LEGACY_INDICATOR_PHASE3_AUDIT.md): executable
  v4 `study` declarations, the first corpus-selected exact aliases, paired HIR
  and runtime equivalence, and measured corpus improvement.
- [Legacy Indicator Phase 4 Audit](LEGACY_INDICATOR_PHASE4_AUDIT.md): historical
  v4 input overloads and type constants, canonical callsite/override parity,
  strict modern negative controls, and measured corpus improvement.
- [Legacy Indicator Phase 5 Audit](LEGACY_INDICATOR_PHASE5_AUDIT.md): historical
  v4 output signatures, primitive styles, transparency normalization, expanded
  schema 8 visual data, historical/incremental/realtime parity, and measured
  corpus improvement.
- [Legacy Indicator Phase 6 Audit](LEGACY_INDICATOR_PHASE6_AUDIT.md): strict
  legacy expression evaluation, structural history lowering, type-directed RSI
  overloads, versioned session/logical defaults, and measured v4 compatibility.
- [Legacy Indicator Phase 7 Audit](LEGACY_INDICATOR_PHASE7_AUDIT.md): versioned
  legacy security signatures, provider/chart contracts, gaps/lookahead
  alignment, repaint warnings, cross-host parity, and the declaration-timeframe
  fail-closed boundary.
- [Legacy Indicator Phase 8 Audit](LEGACY_INDICATOR_PHASE8_AUDIT.md): executable
  v3 declarations, pre-v4 names/constants and chart metadata, focused untyped
  `na` inference, canonical runtime equivalence, and measured v3 compatibility.
- [Legacy Indicator Phase 9 Audit](LEGACY_INDICATOR_PHASE9_AUDIT.md): executable
  implicit-v1 and v2 declarations, bounded self/forward declaration graphs,
  historical bool/numeric conversions, canonical runtime equivalence, and the
  fully passing committed legacy seed corpus.
- [Legacy Indicator Phase 10 Audit](LEGACY_INDICATOR_PHASE10_AUDIT.md): required
  v1-v4 runtime and complete analysis goldens across CLI, Python, and WASM,
  expanded parity guardrails, automatic source-version API policy, and the
  explicit migration-preview deferral.
- [Legacy Indicator Phase 11 Release Audit](LEGACY_INDICATOR_PHASE11_RELEASE_AUDIT.md):
  final corpus, execution-mode, MTF, resource, cache, schema, license, and
  release-maturity closeout.
- [Legacy v4 Profile Closeout](LEGACY_INDICATOR_V4_PROFILE_CLOSEOUT.md): v4
  preview evidence and deferred stable gates.
- [Legacy v3 Profile Closeout](LEGACY_INDICATOR_V3_PROFILE_CLOSEOUT.md): v3
  preview evidence and known boundaries.
- [Legacy v2/v1 Profile Closeout](LEGACY_INDICATOR_V2_V1_PROFILE_CLOSEOUT.md):
  experimental declaration, conversion, lookahead, and evidence boundary.
- [Strategy Internal Gap Audit](STRATEGY_INTERNAL_GAP_AUDIT.md): historical
  strategy gap inventory; use the active broker plan below for current
  priorities and stage boundaries.
- [Strategy Internal Stage 14 Short/Reversal Plan](STRATEGY_INTERNAL_STAGE14_SHORT_REVERSAL_PLAN.md):
  boundary lock, side-aware ledger, market short entries, short closes, market
  entry reversals, and short stop/limit/profit/loss/bracket/trailing
  `strategy.exit` covers.
- [Strategy Internal Stage 15 Short Margin Plan](STRATEGY_INTERNAL_STAGE15_MARGIN_SHORT_PLAN.md):
  short `margin_short` capital held, short-entry affordability, short forced
  liquidation, and short `strategy.margin_liquidation_price`.
- [Strategy Internal Stage 16 Close-Entries-Rule Plan](STRATEGY_INTERNAL_STAGE16_CLOSE_ENTRIES_RULE_PLAN.md):
  id-specific and same-entry-id partial `close_entries_rule="ANY"` allocation for
  shorts.
- [Strategy Broker Next Execution Plan](STRATEGY_BROKER_NEXT_EXECUTION_PLAN.md):
  Stage 17-23 record for the unified fill kernel, historical order timing,
  generic netting, OCA, recalculation, broker-enforced risk rules, and Bar
  Magnifier fill wiring; use the five-stage plan for current follow-up.
- [Next Language Expansion Playbook](NEXT_LANGUAGE_EXPANSION_PLAYBOOK.md):
  process for selecting a language slice.

## Phase Records And Design Gates

Files named `PHASE_*_EXECUTION_PLAN.md`, `PHASE_*_AUDIT.md`,
`STRATEGY_INTERNAL_STAGE*_PLAN.md`, and
`STRATEGY_INTERNAL_STAGE*_AUDIT.md` are implementation and closeout records.
Files named `PURE_INTERNAL_*_DESIGN.md` are design gates for specific semantic
or broker boundaries.

Consult these records when extending the corresponding subsystem, but confirm
the current supported boundary against
[`tests/fixtures/conformance.tsv`](../tests/fixtures/conformance.tsv) and the
latest audit before changing compatibility claims.

## Release Gate

[`scripts/verify.sh`](../scripts/verify.sh) is the canonical local and CI
release gate. It checks Rust formatting and linting, all workspace tests,
source structure, host parity, the real WASM/Node path, the Python wheel, and
Python binding tests.

- [G3 参考输入接收审计](STRATEGY_MODERN_G3_REFERENCE_INTAKE_AUDIT.md)：下载导出核验与 Chrome 登录阻塞。
- [G5 checkpoint 优化审计](STRATEGY_MODERN_G5_CHECKPOINT_OPTIMIZATION_AUDIT.md)：单热点优化、交替 A/B、资源与回归证据。

- [G3 独立参考扩展与修复验收](STRATEGY_MODERN_G3_CLOSEOUT_AUDIT.md)

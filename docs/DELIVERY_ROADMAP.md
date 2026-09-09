# Independent runtime delivery

Owner: Codex, directly responsible for implementation, review and validation. Started 2026-09-09. Status: active, no stable release accepted yet.

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

| ID | Required result | Owner | Current state | Acceptance still needed |
| --- | --- | --- | --- | --- |
| D0 | Freeze recovered baseline | Codex | Complete locally: 85f42f20f tooling, 149d10c7a prior semantics; current integrated source includes later qualified fixes | Linux/release artifact qualification is tracked by D5 |
| D1 | Full TechnicalRating dependency chain | Codex implementation, semantics and oracle | Simple parameters and spaced versions closed; all three exact sources parse at 7aa527a19; transitive binding Windows gate passed (LIBRARY_CHAIN_AUDIT); RelativeValue overload/effect admission still blocks execution | Original sources preserved; complete root runs and matches frozen three-rating reference; mode/host parity |
| D2 | Independent execution correctness | Codex | EMA/SMA seed and repeated-bar correction closed at 391386d7e; 9 controls / 739,655 values match; previous G3 real-strategy references rerun unchanged | Broader strategy/account, matched MTF/Magnifier inputs and real tick reference remain open |
| D3 | Explicit embedding/data contract | Codex | Price grid exists; market/account assumptions and required capabilities need product review | Host can identify required data and unsupported profiles before execution, with useful source diagnostics and versioned contracts |
| D4 | Long-session resources | Codex measurement and acceptance | Small historical benchmark exists; no current long-session qualification | Frozen workloads/budgets for the stated historical/append/forming scales; latency, output cost, memory growth and limit behavior |
| D5 | Candidate from a known commit | Codex | Windows installed debug wheel, actual WASM/Node and CLI qualification retained for current slices; working package version remains 0.2.0 | Linux/Windows final installed artifacts, Rust embedding example, CLI/WASM packaging, synchronized versions/checksums/docs/acceptance manifest |

## Current next action

Return to D1 using the complete source/dependency graph, not one first-error
probe at a time. The bounded read-only map is retained as
`.local/delivery-20260909/grok-full-chain-map-report.md`; it separates full-source
parsing/admission from reachable calcRatingAll execution. Current-head analysis
(`technical-after-transitive.json` in the isolated library worktree evidence)
still rejects the complete root, now at RelativeValue overload/effect admission.
Collection-field parsing and statement switch arms are qualified at 7aa527a19;
source-scoped transitive imports and namespace fallback passed their final Windows gate.
Next address valid overload declarations and owned collection mutation without
removing checks for invalid source, global mutation or unsupported invocations.
The static map is not execution evidence. The target remains
unmodified TechnicalRating/3 with exact ta/9 and RelativeValue/3; unused
libraries must not be omitted or trimmed to pass admission.

The full TradingView reference is ready: 21,133 closed bars from bar_index=0,
no skipped warmup, three rating outputs. It is still a reference-only package;
local full-library comparison has not passed. Manifest:
`.local/delivery-20260909/technical-reference-manifest.json`.

## Qualified evidence and limits

- [EMA/SMA qualification](EMA_INITIALIZATION_AUDIT.md): 391386d7e; final
  synchronized Windows gate 6,588 Rust / 677 fresh installed-wheel Python /
  101 tool tests plus real WASM/Node. Nine reference controls also pass 18
  exact complete-output CLI/Python/WASM comparisons. Twelve reviewed plot-value
  goldens changed; unrelated runtime_math differences were excluded. Resource
  profiles now include undo storage; no resource ceiling was relaxed.
- [Version annotations](VERSION_COMMENT_SPACING_AUDIT.md): c2c028269; original
  RelativeValue/3 spaced version no longer misclassified as v1.
- [Simple parameters](SIMPLE_PARAMETERS_AUDIT.md): b8e38333d; explicit qualifier,
  defaults and global-input argument behavior with independent type probes.
- [Prior real strategies](STRATEGY_MODERN_NEXT_CYCLE_AUDIT.md): two actual
  strategy families / three scenarios / 65 trades. The EMA correction rerun
  passes all prior trade and series comparisons; raw batches stay separate.
- [Earlier six-case G3](STRATEGY_MODERN_G3_CLOSEOUT_AUDIT.md) remains a separate
  batch. The public r1 independent reference denominator remains 0/482 in its
  last recorded measurement, not upgraded by these other captures.
- [Performance baseline](STRATEGY_MODERN_G5_CHECKPOINT_OPTIMIZATION_AUDIT.md)
  is historical small-workload evidence, not a current long-session guarantee.

Latest local qualification receipt:
`.local/delivery-20260909/ema-final-qualification.json`; retained wheel and
WASM artifacts: `ema-final-hosts/` in the same evidence directory. These are
qualification builds, not a stable release or a completed Linux matrix.

## Work and evidence rules

Grok delegation/resume authorization was revoked by the current goal file.
Codex implements and verifies directly. Prior task logs/reports are retained as
historical leads, not current authority. At takeover no matching live Grok task
process was found; the latest four task records ended normally. Do not restart
those tasks or change shared/global agent settings. Keep bounded implementation
scope and independent verification, including review of self-authored tools.
Keep workspace, committed, installed-artifact and published states separate.

Raw third-party sources and captures stay in ignored local evidence folders;
public regressions are original. Do not make CI depend on private exports.
Never generate TradingView expectations with this runtime, skip failing inputs,
change a frozen denominator or relax tolerances to pass. A passing internal
golden is not independent correctness. B1 remains UNVERIFIED_INTERNAL_ORDER.

D1-D5 are not waived by the completed numerical slice. No external publication
or deployment is authorized; prepare a reviewable candidate before requesting
that final approval.

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
| D1 | Full TechnicalRating dependency chain | Codex implementation, semantics and oracle | Windows acceptance complete at 5920add7e: original graph, all 63,399 reference values, full CLI/installed-wheel/WASM output parity and CLI mode checks (SMA_ROLLING_SUM_AUDIT) | Linux/release matrix remains D5; long-session acceptance remains D4 |
| D2 | Independent execution correctness | Codex | Numerical controls qualified; frozen G3 batches revalidated; matched default HTF, paired historical Magnifier and long/short fractional-margin references pass with Windows host parity | Real tick reference and broader account profiles remain open; request/Magnifier/margin claims stay bounded by actual cases |
| D3 | Explicit embedding/data contract | Codex | Price grid and decimal quantity precision exist; market/account assumptions and required-capability discovery need product review | Host can identify required data and unsupported profiles before execution, with useful source diagnostics and versioned contracts |
| D4 | Long-session resources | Codex measurement and acceptance | Small historical benchmark exists; no current long-session qualification | Frozen workloads/budgets for the stated historical/append/forming scales; latency, output cost, memory growth and limit behavior |
| D5 | Candidate from a known commit | Codex | Windows installed debug wheel, actual WASM/Node and CLI qualification retained for current slices; working package version remains 0.2.0 | Linux/Windows final installed artifacts, Rust embedding example, CLI/WASM packaging, synchronized versions/checksums/docs/acceptance manifest |

## Current next action

D1 is Windows-qualified at 5920add7e: unmodified TechnicalRating/3 with exact
ta/9 and RelativeValue/3, all 21,133 closed bars from bar_index=0, and all
63,399 rating values agree with the independent reference. CLI execution modes,
installed Python wheel and generated WASM agree on complete output. Receipt:
`.local/delivery-20260909/technical-5920add7e-hosts/qualification.json`.

The rebuilt current CLI also revalidates both frozen strategy batches: r3 has
65 trades/14,711 series values; r2 has 41 trades/18,907 values. All pass with
original source/export hashes, tolerances and prior warmup policies unchanged.
Receipt: `.local/delivery-20260909/strategy-revalidation-summary.json`.
These are historical CLI reruns, not new tick or MTF/Magnifier coverage.

The independent 15-minute/60-minute default-HTF package now passes: 21,133 chart
bars, 5,286 hourly inputs and 422,660 output values, with full installed-wheel/
actual-WASM/CLI parity (MTF_REFERENCE_AUDIT.md). Paired historical Magnifier
references also pass: 148 hourly/888 intrabars, two trades and 2,960 plot values,
with full host and historical mode parity (MAGNIFIER_REFERENCE_AUDIT.md).
Long/short margin references now pass 7992 values and four trades in total,
with full installed-wheel/actual-WASM/CLI parity and CLI mode checks
(MARGIN_REFERENCE_AUDIT.md). The full Windows gate passes 6630 Rust, 690 Python
and 103 tool tests, plus actual WASM/Node. Next continue real tick evidence and D3-D5. Preserve frozen denominators and keep each reference batch
separate. The public r1 independent-reference count is not increased by these
other batches. Windows debug qualification is not final Linux/release acceptance.

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

Latest local full-library qualification receipt:
`.local/delivery-20260909/technical-5920add7e-hosts/qualification.json`; retained
wheel/WASM and complete results are in that directory. EMA-specific historical
evidence remains in `ema-final-qualification.json` and `ema-final-hosts/`. These are
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

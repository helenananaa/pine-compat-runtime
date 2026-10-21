# Native risk registration

Goal phase 1, started 2026-10-21 on `codex/pine-host-completion`, baseline
`c12e95e805de091ff75a32f15f7b3ae5cbe5139f`. This extends PR #18; the core remains
independent of CandleScope matching, accounts and persistence.

## Problem and implementation

The baseline native broker registered a risk call only when script execution
reached it, and replaced same-kind settings with later values. A 2-contract cap
inside `if false` allowed a 10-contract entry. Caps of 2 then 5 allowed 5.
The separate declarative Host configuration correctly retained both calls.

Native historical and realtime execution now resolves that same configuration
before its first market observation, after input overrides/chart context are
bound. Skipped branches and declarations below an order do not suppress a rule.
Runtime calls do not reevaluate these already registered directives. Unresolved
arguments fail closed before matching, including hidden side-effecting calls.

Numeric position/count rules use the smallest limit. Monetary rules keep the
smallest cash and percentage limit independently and check both at native risk
hooks, preserving the triggered limit's alert message. Direction declarations
intersect the allowed directions; generic `strategy.order` remains independent
of entry-direction and position-cap restrictions. Configuration is cloned with
native broker/runtime checkpoints; realtime rollback and external Host account
authority retain their existing lifecycle.

Risk discovery and native execution now use the supplied symbol, timeframe,
price grid and calendar environment, instead of synthetic metadata defaults.
The chart-only Rust helper remains available; the new environment helper also
preserves calendar context. Python discovery and external construction use the
environment helper. Pure scalar local loops use an isolated ordinary Pine
evaluator after verifying that assignments target only local bindings and that
no requests, objects, orders or user side effects are reachable. Native
registration uses the caller's execution limits; metadata probes retain their
separate bounded introspection budget.

## Validation and boundaries

The six initial regression groups failed against the baseline and passed after
the implementation. The expanded Rust module also covers daily-loss counts,
direction admission and actual halt-alert selection. Python binding regressions
cover the same boundaries plus input overrides and actual chart-grid binding.
The latest focused Rust module passes all 12 tests, including actual symbol and
timeframe values and a 20,000-iteration argument under native execution limits.
The original full-gate attempt was interrupted after its Clippy pass when a
separate probe found synthetic ticker fallback and unresolved timeframe/loop
arguments. Its logs and source freeze are retained under `before-context-fix-*`;
it is not a complete passing gate.
The final Windows gate on Rust 1.95.0 passed 7,942 Rust workspace tests,
1,886 fresh-wheel Python tests and 172 tooling tests, with zero failures or
skips. Formatting, all-target Clippy with warnings denied, structure and host
parity guards, and generated WASM/Node smoke also passed. All 950 frozen
execution/test/tooling inputs remained byte-identical. The new Rust module's
12 tests and Python module's 23 cases are included in these totals.

The installed wheel SHA256 is
`89a8621c5411c7a5e8cd500a8529ee07113090d301083de974c666902f695703`;
the actually loaded `pine_compat.pyd` SHA256 is
`ae29808a5db88a4cf9a8afa34a86603136051db1604eac62bef7f3fed2243755`.
The local receipt, full logs, source freeze, wheel and independent installation
are in `.local/goal-native-risk-20261021/receipt.json` and its directory.

The CLI capability matrix initially differed only in the updated risk-rule
description; the reviewed golden change is recorded in `matrix-diff.json` and
`matrix-snapshot-update.json`. A separate WASM metadata request failed with an
old local semantic-analysis library, although unchanged source explicitly
admitted the request. The same test passed after a byte-preserving forced
recompile. The old library and failures were retained. Final qualification
therefore rebuilt all nine workspace package roots with incremental compilation
disabled; it does not reuse the earlier incomplete gate as passing evidence.

These are controlled core candidate results. Existing CandleScope installed
worker receipts still identify their earlier binary and do not qualify this
wheel. No packaged-application, resource-matrix or release qualification is
established by this gate.

The positive finite limit and bounded immutable-expression admission remains in
force. Zero-limit semantics, arbitrary unresolved expressions, exact native
partial-fill conventions, conflicting-direction official references and broader
TradingView equality are not established by these controlled checks.

The official [current risk management documentation](https://www.tradingview.com/pine-script-docs/concepts/strategies/#risk-management)
describes branch-independent enforcement, and the [v4 risk documentation](https://www.tradingview.com/pine-script-docs/v4/essential/strategies/#risk-management)
describes same-kind strict-rule precedence. Documentation alignment is distinct
from a fresh native reference capture for this candidate.

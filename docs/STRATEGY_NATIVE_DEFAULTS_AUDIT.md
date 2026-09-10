# Native strategy capital and missing-commission defaults

Status: scoped Windows development-artifact qualification passed, 2026-09-10.
Base: `92ad6dd5d` (occupied-long stop admission slice).

## Independent reference

The original r1 `strategy_commission_cash_per_order.pine` was compiled in
TradingView without source modification (SHA-256
`5afc18c592612fa9d6147f2b729cb1f742bd06ed1340a04d8db01822804aac82`).
Its separate native scenario uses OKX:BTCUSDT one-minute standard candles.
The chart was loaded to the native first bar, 2026-08-24 00:00 UTC. The exported
first trade opens at bar 1 and closes at bar 3, matching the original source.
The initial recent-only export is retained but is not used as a bar-zero input.

The full comparison uses 24988 closed bars / 99952 plot values, with the final
forming row excluded and original 1e-9 absolute/relative tolerances. Before the
repair, 49974 values differed. Trade identity, timestamps, prices, quantity and
net trade PnL agreed; defaults in equity and missing open commission did not.

Two additional original probes independently distinguish zero from `na` using
explicit `na(...)` plots, cover negative/zero/out-of-range indices, and record
before/during/after-trade behavior. The v5 control fixes initial capital to 1000;
the v6 companion omits it and plots `strategy.initial_capital` directly.

- Native default capital: 1000000, confirmed in the original v5 strategy's
  first equity value, its Properties dialog, and the explicit v6 capital plot.
- Missing open commission: zero for negative/out-of-range integer indices
  and while flat; the explicit `na(...)` probes return false.
- Both controls retain the expected configured entry fee (1.5) and closed
  commission (3), so this is not loss of the fee input or CSV null coercion.

Evidence: `.local/tv-goal-20260910/r1-native-commission*`.
The original four-bar synthetic r1 scenario remains unchanged. This native
companion does not retroactively increase the original r1 0/482 denominator.

## Changes and compatibility impact

- Correct the shared omitted-capital default from 100000 to 1000000.
- Return zero for absent integer-index open-commission reads; keep other
  identity fields and non-integer/`na` index behavior unchanged.
- Preserve all Pine fixture sources and native exports. Update the affected
  expectations rather than inserting explicit old capital into those sources.
- Applications relying on the old starting amount can declare
  `initial_capital=100000` explicitly. Equity-percentage sizing can consequently
  change when the declaration omits initial capital.

Two targeted Rust cases failed before the repair. The broader old assertions
also exposed 19 affected runtime tests. The reviewed runtime snapshot update
changes 314 files: 2760 cash/equity offsets, 29 capital plot values, 26 missing
commission values, and 63 tiny equity-subtraction net-profit rounding changes
below 1e-9. Three unrelated platform math differences were excluded and retained
in the local rejected snapshot. No order or trade record changed. Three matrix
notes separately document the corrected boundary.

The percentage-fee cash assertion now expresses the two separately posted
cash flows (entry cost/fee, exit proceeds/fee), preserving exact arithmetic
checking instead of assuming floating-point addition is associative. Native
comparison tolerances have not changed.

## Qualification and remaining work

- Full Windows gate: 6672 Rust, 715 fresh installed-wheel Python, 130 tool
  tests, and actual WASM/Node pass.
- Retained wheel in a separate fresh environment: the original r1 native
  companion passes 99952/99952 values; the v5 and v6 controls pass 249920 and
  274934 values respectively. Total: 624806/624806, original tolerances.
- Complete CLI historical/incremental/realtime-history results agree for the
  r1 companion. All three cases agree across CLI, Python common execution
  fields, and generated WASM. Python-only metadata is not a CLI contract.
- The retained build is a development artifact. Linux and release-profile
  qualification remain separate; prior reference suites remain regression
  requirements for final delivery.

This is not a release, full Pine compatibility, or acceptance of the original
unresolved realtime-price capture. Broader v6 account-default behavior remains
outside this two-field slice.

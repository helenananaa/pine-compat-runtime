# Reversal transaction quantity receipts — 2026-09-30

The percentage/per-contract quantity finding is repaired in the current core.
Reversing entries now report the sum of closing exposure and new exposure in
both the order event and the generated entry fill-alert quantity. The trade
ledger, new-position quantity, commissions and account arithmetic are preserved.
The positive cash-per-order path already used transaction quantity.

`entries.rs` retains the closed quantity before flattening. `fill_apply.rs`
uses that quantity only when recording the order and entry fill alert; it
continues using the requested/clamped new quantity for the new trade and fee.
Both directions are covered. The core remains host-neutral.

## Current-core evidence

- Two frozen native EURUSD four-hour commission-type cases were rerun in all
  three modes. Six complete outputs now report reversal quantity 105,000,
  instead of 30,000. All other fields are exactly equal to the pre-fix output:
  464 native plotted cells and six native closed allocations remain qualified.
- Four full UT Bot GBPUSD four-hour input cases were rerun in all three modes.
  Their 7,838 historical closed trades, final historical open entries, 170,816
  Buy/Sell cells, and all non-quantity output fields remain unchanged. Native
  baseline comparisons were rechecked independently. Reversal orders now
  record two transacted units, retaining a one-unit new position. Twelve
  complete historical outputs agree across modes within each input case.
- A broker regression exercises both initial directions across no commission,
  percent, cash-per-contract, zero cash-per-order and positive cash-per-order
  settings. After a 25% close, order and entry-alert receipts report 105,000,
  while new exposure remains 30,000 and the ledger remains consistent.
- Eleven golden snapshots change only 22 order/alert quantity fields. An
  archived pre-update snapshot set and recursive JSON audit prove that no
  other snapshot fields changed. Existing market/limit/stop/stop-limit
  reversal assertions now expect transaction quantity.

The official [Pine v5 strategy documentation](https://www.tradingview.com/pine-script-docs/v5/concepts/strategies/)
defines reversal transaction size as existing position plus the requested new
entry size. Native closed allocations/position plots establish both legs;
trade CSVs do not provide a complete native order list. Fill-alert quantity
is covered by local regression, not a newly captured live TradingView alert.

## Fresh release gate

`scripts/verify.ps1` completed successfully against the changed core: rustfmt,
clippy with warnings denied, workspace and integration tests, 1,989 runtime
unit tests, 242 CLI tests, 130 tooling tests, 940 registered snapshots with
591 required runtime and five legacy-analysis Python/WASM assertions, actual
Node WASM smoke tests, and 774 tests against a freshly built/installed Python
wheel. Two preceding failed gate logs are retained: old golden receipt values,
then old direct reversal quantity assertions. Their changes were audited;
the third full gate completed with exit zero.

Evidence root: `.local/reversal-order-qty-fix-20260930/`. It retains the new
candidate executable, original native inputs/exports, complete outputs,
command receipts, audited snapshot changes, source pins, all gate logs and
the predecessor hash chain. No historical native fixture or prior candidate
was overwritten. Run
`python .local/reversal-order-qty-fix-20260930/freeze_verify.py` to verify.

## Remaining scope

This closes the specific quantity finding. Prior native receipts belong to
their recorded core/executable pins; current qualification includes the two
commission probes and four UT Bot historical cases above. The earlier UT Bot
unconfirmed opening harness was not rebuilt/rerun here and does not qualify
the changed core's opening output. Other prior scripts need current-core
requalification as appropriate. Actual live Tick behavior, non-unit point
values, pyramiding-before-reversal native controls, and the recorded Hull
monetary residual remain open. No commit or push was made. Goal remains active.

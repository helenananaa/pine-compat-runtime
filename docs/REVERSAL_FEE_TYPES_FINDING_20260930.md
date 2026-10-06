# Reversal commission types: native receipts and quantity finding

2026-09-30. Two fresh complete Pine v5 controlled probes on FX:EURUSD four-hour
bars expand the partial-close/reversal path to 0.1% commission and USD 0.0001
per contract. Both open 100,000 long units, close 25,000, reverse the remaining
75,000 into 30,000 short units, then close the short.

## What agrees

All 464 distinct plotted cells (29 fields × eight bars × two cases), null/zero
distinctions, and six native closed allocations agree. Maximum plotted error
is 2.329e-10; fill error is 2.221e-16. Maximum displayed profit error is
0.001751 USD, within the native CSV's two-decimal precision. Complete batch,
incremental and realtime-history outputs are identical within each case.

| Commission | Native allocated commissions, USD | Final net profit, USD |
| --- | --- | --- |
| 0.1% | 54.49 / 163.39 / 65.23 | -696.6063 |
| 0.0001 per contract | 5 / 15 / 6 | -439.50 |

Full-precision native commission plots agree with `(entryPrice + exitPrice) *
abs(quantity) * 0.001` for each percentage allocation. Per-contract allocations
charge both entry and exit quantity at 0.0001. This confirms fee accounting for
the named path; it does not qualify the complete order output below.

## Confirmed output gap — unresolved

Both cases serialize the reversal order with `qty=30000`, representing only
new exposure. The actual reversal transaction closes 75,000 and opens 30,000,
so transaction size is 105,000. TradingView's [official v5 strategy
documentation](https://www.tradingview.com/pine-script-docs/v5/concepts/strategies/)
specifies that a reversing entry adds the open position size to the new entry
size. Native trade allocations and position plots independently establish the
two legs here. The native trade CSV is not a full order list.

The preceding fixed-per-order path emits 105,000. In the current core,
`entries.rs` uses an atomic netting transition only for positive cash-per-order
fees. Other fee types close the old position before opening the requested new
quantity. `fill_apply.rs` records that new quantity in `StrategyOrderEvent` and
also passes it into fill-alert generation. The latter is a source-level
follow-up risk; these probes do not emit custom fill alerts.

The frozen comparator explicitly records the failed quantity field and
`full_output_qualified=false` for all six outputs. This finding is not repaired
by weakening expected quantity or claiming account agreement as full parity.
Next implementation work must correct transaction quantity in order/fill-alert
receipts, cover both directions and fee types, and requalify affected outputs.

## Evidence and boundaries

Evidence root: `.local/reversal-fee-types-native-20260930/`. It preserves both
complete sources, full native chart/trade exports, applied input/property
snapshots, screenshots, exact eight-bar OHLCV, commands, hashes, and comparisons.
Downloads originated in `I:\sys\下载`. Native properties are USD 2,000,000,
pyramiding 1, zero slippage, infinite leverage, on-bar-close evaluation and
one-tick order delay. Input direction is long first and partial close is 25%.
The time-only probes need no earlier indicator warmup; full exports are retained.

Run `python .local/reversal-fee-types-native-20260930/freeze_verify.py` to
verify this immutable pre-fix reproduction and predecessor pins. No core
change has been made in this capture. The preceding qualified CLI/full-gate
receipt is reused and does not qualify the newly discovered output gap.

The original Stoch editor draft was restored and reread. The probe was removed
through the object tree. Final DOM verifies both original strategies and
BTCUSD daily, with no temporary probe. Sampling tab closed. General live Tick,
non-unit point values, and the recorded Hull monetary residual remain open.
No commit or push occurred. Goal remains active.

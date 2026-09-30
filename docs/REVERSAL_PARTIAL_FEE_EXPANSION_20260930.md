# Partial close followed by reversal — 2026-09-30

A fresh complete Pine v5 controlled probe matches TradingView native output
on FX:EURUSD four-hour bars. It opens 100,000 long units, closes 25%, reverses
the remaining 75,000 units into a 30,000-unit short, then closes the short.
This extends the prior price-exit samples with an atomic reversal after a
partial market close. It is a controlled semantic probe, not public-strategy
coverage or general realtime qualification.

## Native result

Initial capital is USD 2,000,000, commission USD 10 per order, pyramiding 1,
zero slippage, infinite leverage, on-bar-close calculation, and one-tick
order delay. Explicit source quantities override the default quantity of 1.

| Allocation | Units | Native commission, USD | Native net profit, USD |
| --- | ---: | ---: | ---: |
| Initial partial close | 25,000 long | 12.50 | -95.25 |
| Remaining long closed by R | 75,000 long | 14.64 | -340.89 |
| R short closed finally | 30,000 short | 12.86 | -17.36 |

Native plotted commissions retain full precision. The single reversal fee
splits as `10 * 75,000 / 105,000` for closing exposure and
`10 * 30,000 / 105,000` for opening exposure. Including allocated original
entry fees and the final close fee, commissions are exactly 12.5,
14.642857142857142, and 12.857142857142858 within floating-point precision.
The runtime reversal order records a 105,000-unit transaction and leaves a
30,000-unit short; native position plots independently confirm that new size.
The native trade CSV reports allocations rather than a full order list.
Final net profit is -453.50 USD and final position is zero.

## Verification

All 29 native plotted fields across eight bars agree, including null/zero
distinctions, remaining entry fees, open profit percentages, and all three
closed commissions. Maximum absolute plot error is 1.596e-10. All three
native allocations match entry IDs, direction, quantity, times, fills and
displayed profit. Maximum fill difference is 2.221e-16; maximum displayed
profit difference is 0.002858 USD, within the native CSV's two-decimal
precision. Batch, incremental and realtime-history complete outputs are
byte-identical. This does not exercise live Tick inputs.

Evidence is retained in `.local/reversal-partial-fees-native-20260930/`:
complete source, full native chart/trade exports, exact extracted OHLCV,
applied native properties and inputs, screenshots, command receipts, and
source/input/executable/output hashes. Downloads came from `I:\sys\下载`.
The eight-bar input starts before exposure at the first time-based signal;
there is no historical indicator warmup dependency. Reproduce with
`python .local/reversal-partial-fees-native-20260930/freeze_verify.py`.

No new core change was required. The qualified CLI, core patch and predecessor
full release gate remain unchanged; the gate is reused, not rerun here.
The original Stoch editor draft was restored and reread in full. The temporary
strategy was removed through the object tree; final DOM confirms both original
strategies, BTCUSD daily, and no temporary probe. The sampling tab closed.
An intermediate cleanup snapshot is retained rather than treated as success.

Short-first reversal, other partial percentages, percentage/per-contract fees,
and pyramiding before reversal remain expansion targets. Non-unit point values,
the previously recorded Hull monetary residual, and general live Tick parity
remain open. No commit, push, or publication occurred. The Goal remains active.

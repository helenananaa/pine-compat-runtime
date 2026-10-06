# Price exit and fee expansion — 2026-09-30

Three fresh TradingView native cases pass against the current qualified CLI:
FX:EURUSD, four hours, Pine v5, long 25%, short 25%, and long 50% partial exits.
This is a complete controlled probe, not a public strategy coverage claim.

| Input case | Native exit sequence | Allocated commissions, USD | Final net profit, USD |
| --- | --- | --- | --- |
| Long, 25% | TP, SL, close remaining | 12.5 / 12.5 / 15 | -434 |
| Short, 25% | SL, TP, close remaining | 12.5 / 12.5 / 15 | 341.5 |
| Long, 50% | TP, SL | 15 / 15 | -398 |

The source submits a 100,000-unit entry, separate limit and stop exits,
then closes any remainder. Native properties are USD 2,000,000 initial
capital, fixed USD 10 per order, pyramiding 1, zero slippage, infinite
leverage, on-bar-close evaluation, and one-tick order delay.

Each case compares 29 fields over eight native OHLCV bars, with exact input
history and null/zero distinctions. All eight closed allocations match native
direction, quantity, entry/exit time, price, profit, and commission. Both price
exit order IDs, quantities, times, and prices match in each case. The short
case reverses the native execution order to SL then TP, as the historical
emulator visits the high before the low on that bar.

All nine complete outputs pass. Batch, incremental, and realtime-history
outputs are byte-identical within each input case. Across cases, there are
696 distinct plot cells; maximum absolute plot error is 2.217e-11, maximum
fill error is 2.221e-16, and maximum displayed profit error is 1.203e-11 USD.

## Evidence and reproduction

Evidence root: `.local/price-exit-fees-native-20260930/`.
It retains the complete original Pine source, unmodified native chart and
trade CSVs, applied input/property snapshots, screenshots, extracted bars,
commands, executable/source/input/output hashes, and comparison results.
Downloads came from `I:\sys\下载`. The eight-bar window begins at the first
entry signal before any exposure; this time-only probe has no historical
indicator warmup dependency. Full native exports are retained.

Run `python .local/price-exit-fees-native-20260930/freeze_verify.py` to verify
the frozen evidence and predecessor pins. The current core patch, CLI, and
previous successful full release gate are unchanged; this expansion reuses
that gate and does not claim a new full-gate run. No new core change was needed.

The original Stoch editor draft was restored and reread in full. Initial
cleanup verification found the probe still present; that intermediate state
and screenshot are retained as `cleanup-before-reopen-*`. Reopening the saved
layout verified BTCUSD daily with both original strategies present and the
temporary probe absent. The verification tab then closed. Restoration evidence
is `restored-editor.pine`, `restored-state.txt`, and `restored.png`.

## Qualification boundary

This qualifies the named historical inputs and price-exit/fee paths. General
live Tick behavior, non-unit point values, other limit/stop configurations,
and the previously recorded Hull monetary residual remain open. Core remains
host-neutral. No commit, push, or publication was performed; the expansion
Goal remains active.

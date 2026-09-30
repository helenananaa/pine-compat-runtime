# Reversal input matrix expansion — 2026-09-30

Three fresh TradingView native cases extend the preceding long-first 25%
partial-close reversal receipt. All use the same complete Pine v5 source on
FX:EURUSD four-hour bars; only the two declared inputs change.

| New input case | Initial partial / remaining units | Native closed commissions, USD | Final net profit, USD |
| --- | --- | --- | --- |
| Short first, 25% | 25,000 / 75,000 | 12.50 / 14.64 / 12.86 | 373.50 |
| Long first, 50% | 50,000 / 50,000 | 15 / 11.25 / 13.75 | -427.50 |
| Short first, 50% | 50,000 / 50,000 | 15 / 11.25 / 13.75 | 347.50 |

After the initial partial market close, the source reverses into 30,000 units
and closes that new position later. The 25% case shares the single USD 10
reversal fee across 75,000 closing and 30,000 opening units. The 50% cases
share it across 50,000 and 30,000 units. Native plots preserve the unrounded
commissions, including 14.642857142857142 and 12.857142857142858 for 25%.
Direction changes do not change this quantity-based fee allocation.

Together with the preceding receipt, both directions and both 25%/50%
partial-close inputs are now qualified for this fixed-fee reversal path.

## Verification and provenance

All 696 distinct plot cells (29 fields over eight bars in each new case),
null/zero distinctions, and nine native closed allocations pass. Entry IDs,
direction, signed size, entry/exit times, fill prices, profit, commission,
position sequence and final flat state agree. Maximum absolute plot error is
2.329e-10; fill error is 2.221e-16. Maximum displayed profit difference is
0.002858 USD, within the native CSV's two-decimal display precision.

All nine complete outputs pass; batch, incremental and realtime-history
outputs are byte-identical within each case. This is historical execution,
not live Tick qualification. Runtime reversal transaction quantities are
105,000 for 25% and 80,000 for 50%; native plotted positions independently
confirm the 30,000-unit new exposure. Native trade CSVs contain allocations,
not a full order list.

Evidence root: `.local/reversal-input-matrix-native-20260930/`. The root keeps
the unchanged complete source, full native chart/trade exports, applied
input/property snapshots, screenshots, exact extracted OHLCV, all command
receipts, hashes and comparisons. Downloads originated in `I:\sys\下载`.
Input UI snapshots are taken before applying each change. Applied chart legend
values confirm the percentage, and native trade direction and allocation sizes
independently confirm both effective inputs. Settings reopen on Properties;
those snapshots are explicitly named `*-reopened-properties.txt`.
Each input history begins at the first time-based signal before exposure;
this probe has no indicator warmup dependency. Full exports are retained.

Properties in each applied case are USD 2,000,000 initial capital, fixed
USD 10 per order, pyramiding 1, zero slippage, infinite leverage, on-bar-close
calculation and one-tick order delay. Explicit source sizes override the
default quantity of 1. Run
`python .local/reversal-input-matrix-native-20260930/freeze_verify.py` to
verify frozen evidence and the predecessor hash chain.

No new core change was needed. The core patch and qualified CLI remain
unchanged. The preceding full release gate is reused, not rerun in this
expansion. The original Stoch editor draft was restored and reread in full;
the probe was removed through the object tree. Final DOM verifies both
original strategies, BTCUSD daily, and no temporary probe. Sampling tab closed.

Percentage/per-contract commissions, pyramiding before reversal, other input
values, non-unit point values, live Tick behavior, and the recorded Hull
monetary residual remain open. No commit, push or publication occurred.
The expansion Goal remains active.

# COINBASE:ETHUSD Bitduke strategy: four-hour native comparison

Date: 2026-09-27. Baseline commit:
`6a7a2ac24bf9222cc3e77ca08363956bb3c45b48`.
This is a historical close-bar comparison for a different asset class from
the [FX four-hour slice](FX_BITDUKE_FOURHOUR_20260927.md).

TradingView ran the unchanged public Pine v4 [Bitduke Squeeze Momentum
Strategy](https://www.tradingview.com/script/5tuGpzpd-Squeeze-Momentum-Strategy-based-on-Indicator-LazyBear-Bitduke/)
on the Coinbase `COINBASE:ETHUSD` 240-minute chart. The retained source has
SHA-256 `b6aef6dbf531dac2c0a591a198060be4df554a400d98a4952549a723b77b1eb2`.
All inputs remained at their defaults, including the enabled 2019-01-01 to
2019-12-29 date filter. The chart's strategy properties showed 1,000 USD
initial capital, 1,000 USD default cash order size, 0.075% commission, zero
slippage, default four-tick historical detail, one-tick order delay, and
unlimited leverage. Native trade timestamps display in UTC+8.

The independent native chart CSV has SHA-256
`5f2db0dd5886cb17ec814e99a121b4927789a253c290c7cf690c68da35b73b24`
and 19,133 rows from 2018-01-03 08:00 UTC through a 2026-09-27 04:00 UTC
edge bar. The edge bar was excluded, leaving 19,132 confirmed OHLCV bars.
The derived local input CSV has SHA-256
`1b09d0345616ce9dedcf29bb1131e0c69d311c90c92f58fd87795168d1c610a6`.
The runtime received the bars, symbol `COINBASE:ETHUSD`, timeframe `240`,
price grid `1/100`, and quantity precision `4` through the host-neutral chart
context. The price grid is consistent with the native two-decimal OHLC/trade
prices; the four-digit quantity grid is inferred from the native report's
displayed sizes (140 of 156 rows use four decimal places). This evidence
validates the supplied grid for these trades but does not establish exchange
metadata independently. The core did not fetch market data or symbol rules.

The separately downloaded native trade CSV has SHA-256
`04485481df7800e73e72bd96ff5398de4cf0e489ab82124fd191e4f2288792eb`.
The reproducible `compare_native.py` checks every entry/exit signal, UTC+8
displayed timestamp, price, and quantity; each trade's displayed net profit,
entry notional, commission, cumulative profit, and duration; and both plotted
series after the first 30 supplied bars. The warmup exclusion accounts for
pre-export state retained by TradingView. Native prices and money amounts are
shown to two decimal places and compared within half a cent; finite plot
values use absolute and relative tolerance `1e-9`. The receipt reports **78
matching closed trades, 156 matching order rows, 38,204 matching plot cells,
zero mismatches, and zero runtime diagnostics**.

Batch, incremental, and realtime-history full JSON results are byte-identical
(SHA-256 `7a7254f97d4fc767153f14db52e8417e869d977454a509a54fba3253c7f25cb4`).
No additional runtime code change was needed. Source, native chart/trade CSVs,
input bars, three local results, comparison script and receipt, and a chart
screenshot are retained under ignored `.local/eth-fourhour-strategy-20260927/`.

The chart CSV does not validate favorable/adverse excursion fields or reveal
TradingView's private intrabar order sequence. This comparison does not
validate realtime forming-bar behavior.

# COINBASE:ETHUSD Bitduke strategy: daily exported-history comparison

Date: 2026-09-27. Baseline commit:
`6a7a2ac24bf9222cc3e77ca08363956bb3c45b48`.
This extends the [ETHUSD four-hour strategy comparison](ETH_BITDUKE_FOURHOUR_20260927.md)
to a daily chart and the script's full exported history.

TradingView ran the unchanged public Pine v4 [Bitduke Squeeze Momentum
Strategy](https://www.tradingview.com/script/5tuGpzpd-Squeeze-Momentum-Strategy-based-on-Indicator-LazyBear-Bitduke/)
on Coinbase `COINBASE:ETHUSD` 1D. Source SHA-256:
`b6aef6dbf531dac2c0a591a198060be4df554a400d98a4952549a723b77b1eb2`.
Only `Custom Backtesting Dates` changed from true to false (input call site
`34`); all other script inputs stayed at defaults. The native strategy
properties showed 1,000 USD initial capital, 1,000 USD default cash order
size, 0.075% commission, zero slippage, default four-tick historical bar
detail, one-tick order delay, and unlimited leverage. TradingView displayed
the trade dates in UTC+8, at day precision.

The independently downloaded native chart CSV has SHA-256
`45f3316bd1d73149b5ce0236c5a8090be0d40b2e8d7ec4cd995643c3806dff9e`.
Its 3,780 daily rows begin 2016-05-23 00:00 UTC. The last, 2026-09-27
edge row was excluded as a possible forming bar, leaving 3,779 confirmed
OHLCV bars through 2026-09-26. Derived local input SHA-256:
`7fb53ac7a55112361bc0323536a39c6ee052ad7064e335b7be10fd4aedbec91d`.
The runtime received these bars, symbol `COINBASE:ETHUSD`, timeframe `1D`,
price grid `1/100`, and four-digit quantity precision through its host-neutral
chart context. The quantity grid is consistent with all native trade sizes,
but was not independently obtained from exchange metadata. The core did not
fetch market data or infer symbol-specific account policy.

The native trade CSV has SHA-256
`4ca75335edfb0312937ccf4530f5a0a544787dc4a4bc2c0ca405a90f92ecb66e`.
The reproducible `compare_native.py` checks every entry/exit signal, native
displayed date, price, and quantity; every trade's displayed net profit,
entry notional, commission, cumulative profit, and duration; and both plot
columns on **every confirmed bar, with no warmup exclusion**. The native
report has 154 closed trades and 308 entry/exit rows. Its exit signals
include one `Close entry(s) order SQ_Long` and one `Close entry(s) order
SQ_Short`, both represented in the local public order list. Native money
fields use cent precision and are compared within half a cent; finite plot
values use absolute and relative tolerance `1e-9`. The receipt reports **154
matching trades, 308 matching order rows, 7,558 matching plot cells, zero
mismatches, and zero runtime diagnostics**.

Batch, incremental, and realtime-history full JSON outputs are byte-identical
(SHA-256 `13a6a0af06aa7e3eae9e9f52f0a511811392d95c3b983fc1d811d8282842ed62`).
No new runtime code change was needed. Source, native chart/trade CSVs, input
bars, three local outputs, comparison script and receipt, and screenshot are
retained under ignored `.local/eth-daily-full-strategy-20260927/`.

Follow-up on 2026-09-27: a fresh batch run with the chart-tick market-fill
change is byte-identical to the retained output. The official trade-price
comparison now requires absolute error at most `1e-9`, and the full native
receipt still has zero differences.

The native daily trade CSV exposes only displayed dates, so it does not
independently prove sub-day fill timestamps. It also does not prove private
intrabar ordering, favorable/adverse excursion parity, or realtime forming
bar behavior. “Full history” here means all confirmed rows in this exported
chart, not an independent claim about Coinbase's complete archival coverage.

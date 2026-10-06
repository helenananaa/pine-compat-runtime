# FX:EURUSD Bitduke strategy: four-hour native comparison

Date: 2026-09-27. Baseline commit: `6a7a2ac24bf9222cc3e77ca08363956bb3c45b48`.
This is a local historical comparison, not live-tick qualification.

The unchanged public Pine v4 [Bitduke Squeeze Momentum Strategy](https://www.tradingview.com/script/5tuGpzpd-Squeeze-Momentum-Strategy-based-on-Indicator-LazyBear-Bitduke/)
source has SHA-256 `b6aef6dbf531dac2c0a591a198060be4df554a400d98a4952549a723b77b1eb2`.
TradingView ran it on the FXCM `FX:EURUSD` 240-minute chart with its native
`Custom Backtesting Dates=true`, from 2019-01-01 through 2019-12-29. The
strategy uses 1,000 USD initial capital, 1,000 USD default cash order size,
0.075% commission, zero slippage, and default historical bar detail. The
reported trade timestamps are displayed in UTC+8.

The native chart-data CSV contains 13,637 rows beginning 2017-12-15 14:00 UTC.
The last chart row was excluded as a possible forming bar; the local input has
13,636 confirmed OHLCV bars. The runtime received symbol `FX:EURUSD`, timeframe
`240`, price grid `1/100000`, and integer quantity precision `0` through its
host-neutral chart context. The core did not fetch data or infer symbol metadata.

The independently downloaded native strategy report contains **64 closed
trades and 128 entry/exit rows**, from January through December 2019. The
reproducible `compare_native.py` checks every native signal, timestamp, price,
and quantity; each displayed trade profit, entry notional, commission,
cumulative profit, and duration; and both plotted series after the first 30
input bars. All compared differences are zero: **64 trades, 128 order rows,
and 27,212 plot cells**. The native money fields are rounded to cents and are
compared within half a cent; plot finite values use absolute and relative
tolerance `1e-9`. The first 30 plotted bars are excluded because TradingView's
export retains state from history before the exported chart window, whereas
the local calculation starts at its first supplied bar. The native 2019 trade
window is far beyond this warmup.

This setting exercises two `strategy.close("SQ_Short")` fills (trades 30 and
54). Before the fix, both closed trades and account results were present, but
the public `strategy.orders` list omitted their fill events. The broker now
records one `strategy.close` event per executed close order, with the native
`Close entry(s) order SQ_Short` signal. It retains the existing trade
allocation and order-fill-alert records. The 890 filtered strategy tests pass.

Batch, incremental, and realtime-history runs produce byte-identical full
JSON, SHA-256 `9e99015c4da9d8a8c8da1f908925cd101fab37a9f001eeae56c37b6e2ca49a8e`,
with no diagnostics. Evidence is in the ignored
`.local/fx-fourhour-strategy-20260927/` directory: original Pine source,
native chart and strategy CSVs, input bars, three full local results,
comparison script and receipt, and a chart screenshot. The native chart and
trade CSV hashes are respectively
`cbe87f67ba5cfd0458104534a498074a064b2942f06df82f29c56f4b4b8ff4cc`
and `e8ee7f795f6c987f1e006b915c1fdd4f5d944a95d6e23324df049fc24306db48`.

The chart CSV does not establish intrabar event ordering, favorable/adverse
excursion parity, or realtime forming-bar behavior.

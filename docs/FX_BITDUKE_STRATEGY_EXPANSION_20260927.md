# FX:EURUSD Bitduke strategy: native order and fill comparison

Date: 2026-09-27. Baseline commit: `6a7a2ac24bf9222cc3e77ca08363956bb3c45b48`.
This is a local historical strategy comparison, not a live-tick qualification.

The unchanged public Pine v4 [Bitduke Squeeze Momentum Strategy](https://www.tradingview.com/script/5tuGpzpd-Squeeze-Momentum-Strategy-based-on-Indicator-LazyBear-Bitduke/)
source is retained with SHA-256
`b6aef6dbf531dac2c0a591a198060be4df554a400d98a4952549a723b77b1eb2`.
The publication still displays its 99-line source. TradingView ran it on the
FXCM `FX:EURUSD` 1W chart with only `Custom Backtesting Dates` changed from
true to false. The UI showed initial capital 1,000 USD, default order size
1,000 USD cash, 0.075% commission, zero slippage, default four-tick historical
bar detail, one-tick order delay, and unlimited long/short leverage. The
comparison uses the chart's UTC+8 displayed trade dates.

The chart-data export contains 2,907 weekly rows starting 1971-01-03. The
last displayed row was excluded to avoid live-edge ambiguity, leaving 2,906
bars through 2026-09-13. The local runtime used exactly this strategy-chart
CSV's confirmed OHLCV, symbol `FX:EURUSD`, timeframe `1W`, price grid
`1/100000`, and integer quantity precision `0`. Compared with the separately
frozen FX indicator export, all 2,906 timestamps and OHLC values agree; one
2026-09-13 volume value was revised between exports. This script does not read
volume.

The native strategy report downloaded as an actual CSV: 116 closed trades and
232 entry/exit rows. The reproducible `compare_native.py` checks every row's
signal, chart-date, price, and quantity; each trade's displayed net profit,
entry notional, commission, cumulative profit, and duration; and both plot
columns across all confirmed bars. All differences are zero: **116 trades,
232 order rows, and 5,812 numeric plot cells**. Displayed money fields use
the native cent precision; plot finite values use absolute and relative
tolerance `1e-9`. Full local public JSON is byte-identical in batch,
incremental, and realtime-history modes, with no diagnostics.

Follow-up on 2026-09-27: the chart-tick market-fill change was rerun against
this frozen weekly case. Orders, trades, and parsed JSON remain identical to
the retained result; the new shell capture adds only a trailing newline.
The comparator now requires each reported fill price within absolute `1e-9`
and still finds zero differences. Its current-code receipt is retained as
`strict-grid-guard-comparison.json`.

The comparison exposed a broker-semantic defect. Before correction, all 116
native quantities were the integer floor of the local fractional order size,
and 13 displayed net profits differed. The runtime's
`strategy.default_entry_qty()` helper already applied an explicitly configured
quantity grid, while an omitted-quantity cash `strategy.entry` order did not.
The actual default-order path now truncates to the same grid before placement.
A focused cash-order regression test passes, as do 316 strategy unit tests.
The core still receives symbol precision from its host; it does not acquire
market data or infer a symbol-specific lot size.

Evidence is under `.local/fx-strategy-expansion-20260927/`: original source,
full native chart and trade CSVs, generated bars, complete local JSON,
`compare_native.py`, `native-comparison-receipt.json`, and chart screenshot.
The current publication source view is also captured as a screenshot.
The chart CSV does not validate favorable/adverse excursion fields, and this
historical result does not validate forming updates or non-unit point values.

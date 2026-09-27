# SSL Hybrid Strategy baseline-length expansion (2026-09-27)

The unchanged public Pine v5 [SSL Hybrid Strategy by kevinmck100](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/) was run in Chrome on `COINBASE:BTCUSD`, `1D`. `SSL1 / Baseline Type` remained `HMA`, `Source` remained `close`, and `SSL1 / Baseline Length` changed from its default 60 to 30. All other displayed script inputs and strategy properties were left at the prior chart settings. The frozen 557-line source is `.local/ssl-hybrid-length30-btcusd-daily-20260927/ssl-hybrid-original.pine`, SHA-256 `1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`. The prior [default](SSL_HYBRID_STRATEGY_V5_NATIVE_PARITY_20260925.md) and [`hl2` source](SSL_HYBRID_SOURCE_BTCUSD_DAILY_20260927.md) comparisons provide separate controls.

The original TradingView chart CSV and List of trades CSV are frozen in ignored `.local/ssl-hybrid-length30-btcusd-daily-20260927/`; their respective SHA-256 values are `5f332c8337d9c2ac603226db8b34d4244f5f07f5e6491d0ec9f25bc016f067ba` and `f8e300e8f826ce205bcaa2ecf53321761c461b24979a57a62eb1f7d4a88ff530`. `length30-chart.png` records the visible HMA 30 setting and Coinbase daily chart. The export has 4,285 rows beginning 2014-12-01 UTC. Its final 2026-09-27 bar was forming and excluded, leaving 4,284 confirmed bars through September 26. `prepare_bars.py` converts this frozen export to host-provided OHLCV. Against the preceding `hl2` export, historical OHLC is identical; seven recent volume values were revised, and this run uses its own chart export. The host-neutral CLI was given the chart price grid `1/100` explicitly.

`compare.py` checks all eight exported indicator series at `1e-8` absolute tolerance. The 29,814 nonblank exported positions have zero mismatches. Four baseline/SSL/channel series each contribute 4,251 values; the optional second moving-average column remains blank. The chart CSV flattens the false/`na` Candle Size condition to zero, which the comparator accounts for. The unchanged source analyzes and runs with zero diagnostics.

TradingView's report has **868 closed trades and two open entries**. The local result has the same counts; every closed entry ID, UTC entry/exit date, and displayed entry/exit price matches, as do the two open entry IDs, dates, and prices. The largest closed quantity difference is `0.000000996` and the largest net-PnL difference is `$0.013414`; the CSV display precision does not establish exact internal quantity or cent-by-cent PnL equality. A local default-length 60 control on the same frozen bars has 475 closed trades. Changing the baseline length therefore exercises a materially different strategy path, beyond chart-only parameter output.

Analysis schema 6 identifies `SSL1 / Baseline Length` as input callsite 13. The CLI used `--input-override 13=30`. Batch, incremental, and realtime-history JSON are byte-identical, each with SHA-256 `1b217bfbc1b741d4a3edcb226c347bc86d109e1097c691ecdee3f6059af35946`; `verify_modes.py` checks this and the native comparison receipt. No runtime semantic fix was required in this slice. This evidence covers confirmed historical daily bars for this parameter and symbol, not live forming-bar updates, other symbols, or other HMA lengths.

Reproduce from the repository root:

```powershell
python .local/ssl-hybrid-length30-btcusd-daily-20260927/prepare_bars.py
target/debug/pine-compat.exe run .local/ssl-hybrid-length30-btcusd-daily-20260927/ssl-hybrid-original.pine --bars .local/ssl-hybrid-length30-btcusd-daily-20260927/bars-confirmed.csv --chart-symbol COINBASE:BTCUSD --chart-timeframe 1D --chart-price-grid 1/100 --input-override 13=30 > .local/ssl-hybrid-length30-btcusd-daily-20260927/local-batch.json
python .local/ssl-hybrid-length30-btcusd-daily-20260927/compare.py
python .local/ssl-hybrid-length30-btcusd-daily-20260927/verify_modes.py
```

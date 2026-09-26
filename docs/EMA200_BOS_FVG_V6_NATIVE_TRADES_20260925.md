# EMA200/BOS/FVG v6 strategy: native trade comparison, 2026-09-25

The complete 3,194-line public [EMA200 REGIME + BOS/CHoCH + 2x FVG Strategy](https://www.tradingview.com/script/x3kpQqJ5/) was frozen in the ignored local evidence directory as `ema200-bos-fvg-v6.pine` (SHA-256 `054a24513560e7ae34640653da764fda02815fa80d6997fc8b6adf5ee935df69`). It analyzes without diagnostics. This receipt compares its default-input strategy trades on the logged-in TradingView `COINBASE:BTCUSD` 1D chart with the interpreter's batch and incremental runs. The script's default `MARKET` input is `NASDAQ` even on this BTCUSD chart; no input was changed in either environment.

The native Strategy Report trade CSV, copied to the ignored `ema200-bos-fvg-coinbase-native-trades-20260925.csv`, has SHA-256 `5d9eb7d5806000ddbf12faa3493813ef3435ea3c6cfc4ac501cf3f564e52a5f5`. The chart input `lazybear-volume-flow-chart-bars-v1-20260925.csv` has SHA-256 `7ae473418edd31c5a604aeea4b6cd71b923a7ee9cc783643756efc09d2d7da8f` and contains 4,282 confirmed daily bars from 2014-12-01 through 2026-09-24. The native report spans through the current 2026-09-25 chart bar, but the four closed trades all precede it. The interpreter ran with `--chart-symbol COINBASE:BTCUSD --chart-timeframe D --chart-price-grid 1/100`.

| Trade | Direction | Entry and exit date | Entry USD | Exit USD | Net PnL USD |
| --- | --- | --- | ---: | ---: | ---: |
| 1 | Long | 2024-01-03 | 43,254.68 | 43,204.68 | −50 |
| 2 | Long | 2024-07-17 | 64,923.08 | 64,873.08 | −50 |
| 3 | Long | 2025-04-25 | 93,881.45 | 93,831.45 | −50 |
| 4 | Short | 2025-11-07 | 102,578.58 | 102,628.58 | −50 |

`compare_ema200_bos_fvg_native.py` asserts the eight native CSV rows against all four local trades, checking date, direction, signal, entry and exit prices, absolute size, and net PnL. It passes. The complete batch and incremental output JSONs are equal, including order/trade records, plots, drawings, alerts, and diagnostics.

The strategy declares `use_bar_magnifier=true`, `calc_on_order_fills=true`, and `calc_on_every_tick=true`. No lower-timeframe magnifier data was supplied to the interpreter, so it reports `W_MAGNIFIER_FALLBACK` on each of the 4,282 bars and uses the standard OHLC path. The exact trade match shows that these four default-setting fills did not diverge under that fallback; it does **not** qualify magnifier execution generally, other symbols/timeframes/inputs, forming ticks, or the script's intended NQ 2-minute scenario. TradingView displayed a look-ahead caution for the strategy. The temporary chart insertion was undone and the prior layout saved after export.

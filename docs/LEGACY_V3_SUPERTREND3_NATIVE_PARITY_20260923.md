# Super Trend 3 v3 native strategy check (2026-09-23)

## Scope and inputs

- Public source: [Super Trend 3 by mhannigan](https://www.tradingview.com/script/bwF5bCek-Super-Trend-3/), unchanged Pine v3 source saved outside version control at `.local/super-trend-3-v3-20260923/original.pine` (SHA-256 `4AD651018D4D3D9CD5C28B3A1DD6B0DECDC1F3FB887D7B43C4917CA415EF8047`). Its strategy title is `Super Trend 2`.
- TradingView UI: `BINANCE:BTCUSDT`, 1D chart, `Main SuperTrend Time Frame = 1 week`; Factor 1, Pd 1, take profit 500 ticks, stop loss 400 ticks, default quantity 100. The strategy report showed 179 closed trades. Six visible report rows were captured at `.local/super-trend-3-v3-20260923/native-weekly-report-rows.json`.
- Local chart bars: 3,279 previously confirmed daily OHLCV rows in `.local/community-coverage-20260923/volatility-reversion-native-confirmed-bars.csv`, through 2026-09-22 (SHA-256 `278AE090E32E441E4CADD2BD70955DDAD650AFEEEDA21FE17651A18F5092593A`).
- Host-provided weekly bars: first six OHLCV columns from `I:\sys\下载\BINANCE_BTCUSDT, 1W_8d4bc.csv` (SHA-256 `89BD7DC4FA35D7883DAEF77D031347E862E873D448FA2FF4C1447D53E3599D56`), retaining complete historical weeks through 2026-07-27. Weeks from 2026-08-03 onward were aggregated Monday UTC from the confirmed daily OHLCV rows; this replaces the downloaded, incomplete 2026-08-03 week. The resulting 476 bars are at `.local/super-trend-3-v3-20260923/btcusdt-weekly-extended.csv` (SHA-256 `2AB6190CB4A11B539AFA08FADFC464FF0DE0B8377836ED0360AAB783CA6B3286`). The 2026-09-21 week is partial because the daily dataset ends on September 22.
- Runtime inputs: `--chart-symbol BINANCE:BTCUSDT --chart-timeframe 1D --chart-price-grid 1/100 --request-bars BINANCE:BTCUSDT:1W=... --input-override 1=1W`. The price grid matters: 500 and 400 strategy ticks mean 5.00 and 4.00 USDT for this chart.

From the repository root, the retained local run can be repeated with:

```powershell
cargo run -p pine-cli --locked -- run .local/super-trend-3-v3-20260923/original.pine --bars .local/community-coverage-20260923/volatility-reversion-native-confirmed-bars.csv --chart-symbol BINANCE:BTCUSDT --chart-timeframe 1D --chart-price-grid 1/100 --request-bars BINANCE:BTCUSDT:1W=.local/super-trend-3-v3-20260923/btcusdt-weekly-extended.csv --input-override 1=1W
```

## Result

The unchanged public script analyzes and runs locally with no diagnostics and 179 closed trades. All six visible native report rows, trade numbers 174 through 179, match local side, entry date, exit date, entry price, exit price, and quantity. The recent rows include both short and long trades; for example, trade 176 short enters and exits 2026-08-11 at 63,970.01 and 63,974.01, and trade 179 long enters and exits 2026-09-19 at 80,883.86 and 80,879.86.

The first local run also had 179 trades but short bracket exits filled one day late and at different prices. The broker discarded relative `strategy.exit` attachments placed while a short entry was pending and did not resolve deferred attachments when a market short filled. The fix retains those attachments and resolves them after the short fill, before the rest of the bar's price path. A focused v3 regression checks both profit and stop fills on the entry bar.

This check establishes the six visible recent trades and aggregate closed-trade count. It does not establish equality of every historical trade or either plotted series, because a full native chart-data or trade-list export was not obtained. The weekly series after 2026-08-03 is reconstructed from the confirmed daily chart bars rather than a fresh native weekly export. TradingView warns that some features of old Pine versions may be unavailable; that warning did not prevent this script from running.

## Complete weekly-control export, 2026-09-25

The original, unchanged public v3 source compiled and ran again in the current TradingView Pine Editor. A separate diagnostic copy then changed **only** the `Main SuperTrend Time Frame` default from `"120"` to `"1W"`, matching the earlier weekly UI setting without changing strategy logic. This copy is ignored locally as `.local/super-trend-3-v3-20260923/weekly-default-diagnostic.pine` (SHA-256 `914ba5a64bc27a9623faff989656f643ad7e35ad6e0fe106b8bcf8459f7689b2`). It is distinct from the unchanged public source and was not published.

TradingView ran the diagnostic copy on `BINANCE:BTCUSDT`, 1D, with its other default inputs. The complete strategy report downloaded to `.local/super-trend-3-v3-20260923/weekly-default-native-trades-20260925.csv` (SHA-256 `9b488e332666bf862cd022dbfde2896b72e46f51b691f014519aeae1832c0553`). The chart export is `weekly-default-native-chart-20260925.csv` (SHA-256 `adddf686cd856c91da5d8daaf423d7d14d9842d351f9ebf08a7536e43bb64755`). Both were copied from `I:\sys\下载\` and remain ignored.

The local run used the 3,324-bar full daily input beginning August 17, 2017 (`.local/community-coverage-20260923/full-daily-bars.csv`, SHA-256 `9bec5ba5960c1eb8f099d30820d728cb083e1b3a9e0305a192d85e854cc5c660`) and the previously frozen host-supplied weekly series (`btcusdt-weekly-extended.csv`, SHA-256 `2ab6190cb4a11b539afa08fadfc464ff0de0b8377836ed0360aab783ca6b3286`). The full chart export has 3,327 rows; the last three are beyond the local input and were excluded. On the 3,324 aligned rows, **all OHLC values and all 19,944 positions across two SuperTrend plots, two shape markers, and two entry arrows match**, with zero blank or numerical mismatches at `1e-8` tolerance.

The native CSV reports **181 closed trades**; the runtime also has 181. Every trade number, side, UTC entry and exit date, displayed entry and exit price, absolute quantity, and net PnL matches. Both closed-trade profit sums are **15,800 USDT**. The earlier 179-trade result began October 1, 2017; this aligned input begins at the native August 17 listing date and includes two earlier trades. The runtime result is `weekly-default-local-full-20260925.json` (SHA-256 `fe3ec940c4c88b285045abd3c46dc63f6cd38a0ad5c49c74b9fb7f38fa978148`); `compare_weekly_default_20260925.py` produces the complete ignored receipt `weekly-default-comparison-20260925.json`.

Reproduce the local comparison from the repository root:

```powershell
target\debug\pine-compat.exe run .local\super-trend-3-v3-20260923\weekly-default-diagnostic.pine --bars .local\community-coverage-20260923\full-daily-bars.csv --chart-symbol BINANCE:BTCUSDT --chart-timeframe 1D --chart-price-grid 1/100 --request-bars BINANCE:BTCUSDT:1W=.local\super-trend-3-v3-20260923\btcusdt-weekly-extended.csv > .local\super-trend-3-v3-20260923\weekly-default-local-full-20260925.json
python .local\super-trend-3-v3-20260923\compare_weekly_default_20260925.py
```

This is full historical plot and exported closed-trade parity for the diagnostic weekly-default copy on the matched input range. It does not prove exact internal accounting beyond exported precision, other input settings, or realtime tick behavior. The saved TradingView layout was reopened afterward and showed its original `COINBASE:BTCUSD` 1D chart and `Percent short market probe v6` strategy, no Super Trend 2, and `All changes saved`.

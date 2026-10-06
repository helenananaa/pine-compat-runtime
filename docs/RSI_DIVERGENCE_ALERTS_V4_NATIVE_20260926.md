# RSI Divergence Indicator (with alerts) v4 native comparison (2026-09-26)

The public [RSI Divergence Indicator (with alerts)](https://www.tradingview.com/script/NvQZc45v-RSI-Divergence-Indicator-with-alerts/) by eemani123 had 3,613 boosts when inspected in Chrome. Its unchanged Pine v4 source is retained in ignored `.local/continued-popular-20260926/rsi-divergence-alerts-original.pine` (SHA-256 `3e6f7df0db951c6e2d649e22e9c57c8b4b8c886395e43a57c5a8e4e00cb7fd93`). The 209-line source analyzes and runs with zero diagnostics and no reported unsupported features. No core change was needed.

TradingView exported 300 `COINBASE:BTCUSD` daily bars to `I:\sys\下载\COINBASE_BTCUSD, 1D (83).csv` (SHA-256 `84685aaec2b51b3ee0d3db76d9b67ea8cad7dfb3e712f42805553239c09f1b9e`), copied to ignored `.local/continued-popular-20260926/rsi-divergence-alerts-native-daily.csv`. The existing local price fixture ends on 2026-09-25, so 299 bars overlap; the 2026-09-26 native bar was forming. All overlapping OHLC and all nine exported indicator series match the local output with zero mismatches. This includes all 299 RSI values, 49 regular bullish pivot values, 49 hidden bullish pivot values, 46 regular bearish pivot values, 46 hidden bearish pivot values, five hidden bullish labels and four regular bearish labels. The other two label series are empty under the default settings in this window. The four divergence plot series use a three-bar negative display offset; the comparator aligns their confirmation-bar local values with exported display bars. The CSV preserves columns by position because another active chart study also exports unnamed `Plot` columns.

This comparison does not validate `barcolor`, alert events, or intrabar updates: TradingView's chart CSV does not export them. The receipt is `.local/continued-popular-20260926/rsi-divergence-alerts-comparison.json`.

```powershell
$b = '.local/continued-popular-20260926'
target/debug/pine-compat.exe analyze "$b/rsi-divergence-alerts-original.pine" --format json
target/debug/pine-compat.exe run "$b/rsi-divergence-alerts-original.pine" --bars "$b/chart-bars.csv" --chart-symbol COINBASE:BTCUSD --chart-timeframe D > "$b/rsi-divergence-alerts-local-daily.json"
python "$b/compare_rsi_divergence_alerts.py"
```

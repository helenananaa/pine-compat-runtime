# LuxAlgo SMC confluence filter native comparison, 2026-09-25

The complete [public Pine v5 Smart Money Concepts source](https://www.tradingview.com/script/CnB3fSph-Smart-Money-Concepts-SMC-LuxAlgo/) was used with eight transparent plot probes appended for internal and swing bias, bullish and bearish BOS events, and equal high and low events. The instrumented source is `.local/community-coverage-20260923/luxalgo-smc-oracle-v5.pine`, SHA-256 `ee0d461eb9aa768c112a4aa626767f9f78290bb493bf028e7caf11ac15d4f5ad`. It is a private derivative, not the published indicator.

TradingView compiled the script on `BINANCE:BTCUSDT`, 1D. Its settings dialog visibly showed **Confluence Filter** checked; other inputs remained at their defaults. The chart export copied from `I:\sys\下载\BINANCE_BTCUSDT, D (1).csv` to ignored `.local/community-coverage-20260923/luxalgo-smc-confluence-native-20260925.csv` has SHA-256 `62dd1c9d721df9df2e4910148a0a5f4aadd7ffa869a3a73a5678c957cef5c3d3`. It contains 3,282 daily bars from 2017-10-01 through 2026-09-25 UTC. The last bar was still forming at export time; the other 3,281 bars were confirmed.

The local run used exactly the OHLCV bars in that export, converted into `.local/community-coverage-20260923/luxalgo-smc-confluence-native-bars-20260925.csv` (SHA-256 `33aeeb93cacf0bb1f0bb1ee262fe132041d650114f28682f5275068fd83a9568`). The current-tree CLI was built with `cargo build -p pine-cli --locked --quiet`. From the repository root, reproduce the comparison with:

```powershell
target\debug\pine-compat.exe run .local\community-coverage-20260923\luxalgo-smc-oracle-v5.pine --bars .local\community-coverage-20260923\luxalgo-smc-confluence-native-bars-20260925.csv --chart-symbol BINANCE:BTCUSDT --chart-timeframe 1D --chart-price-grid 1/100 --input-override 9=true > .local\community-coverage-20260923\luxalgo-smc-confluence-native-bars-local-20260925.json
python .local\community-coverage-20260923\compare_smc_confluence_native_bars_20260925.py
```

`9` is the source call-site ID for the filter input, independently checked with `pine-compat requirements`. The comparison receipt is `.local/community-coverage-20260923/luxalgo-smc-confluence-native-bars-comparison-20260925.json`. All eight plots match the native CSV at all 3,282 time-aligned positions, with zero mismatches under the comparator's `1e-8` absolute tolerance. This includes 70 internal BOS events, 11 swing BOS events, 23 equal highs, and 21 equal lows. The filtered local run differs from the default-input local run on the same bars at 494 internal-bias positions, 26 bullish internal BOS events, and 7 bearish internal BOS events, so the setting exercises a real execution path.

An initial local run that retained 45 earlier bars outside the TradingView export had one equal-high and one equal-low mismatch in early 2018. Those event thresholds use `ta.atr(200)`; the extra earlier bars change its warmup. Starting both runs on the same first bar removes both differences. This comparison establishes the eight instrumented series for this one nondefault input and chart capture. It does not establish drawing pixels, alert delivery, other settings, other markets, or realtime updates. The forming last bar can change after the export.

The temporary TradingView changes were discarded. Reopening the saved layout showed `COINBASE:BTCUSD` 1D, its original `Percent short market probe v6` strategy, no SMC indicator, and `All changes saved`.

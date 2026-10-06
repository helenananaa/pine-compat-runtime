# Nadaraya-Watson rational quadratic kernel v5 native parity (2026-09-25)

The complete public [Nadaraya-Watson: Rational Quadratic Kernel (Non-Repainting)](https://www.tradingview.com/script/AWNvbPRM-Nadaraya-Watson-Rational-Quadratic-Kernel-Non-Repainting/) by jdehorty had 4,328 boosts in the September 25 Chrome page. Its visible source begins with `// @version=5`, which the current analyzer identifies as an explicit v5 directive. The source is retained with nonbreaking spaces normalized to ordinary spaces in ignored `.local/community-coverage-20260923/jdehorty-nw-rational-quadratic-v5-20260925.pine` (SHA-256 `b5b8f79ba25b58d36aa2e23291fa9ba203c312e1320bd213dc904d03f7e88b3f`). The original source analyzes and runs with zero diagnostics.

TradingView ran the publication with default inputs on `COINBASE:BTCUSD` 1D. `I:\sys\下载\COINBASE_BTCUSD, 1D (25).csv` is retained as ignored `jdehorty-nw-native-v5-20260925.csv` (SHA-256 `bfdfe594d0b2c4e6894a9ac2a7d2a67482cd0e1b22baefa3f8d9f308baf3f074`). The chart export has 4,283 rows, with the final forming bar excluded. The other 4,282 rows match the local input in timestamp and OHLC.

The ignored `compare_jdehorty_nw_v5_20260925.py` compares the visible rational quadratic kernel estimate. **All 4,282 positions match exactly**, including 26 paired warmup blanks. Batch, incremental, and historical realtime modes produce byte-identical full JSON output (SHA-256 `cb1a82b6e719349e7e879c3a2706a63e3f5ce34640b2eb4cb6719770b3cfa3eb`). The script also emits a hidden `Alert Stream` plot. The native chart CSV omits this plot and cannot verify alert delivery or line colors. Other settings, symbols, timeframes, and forming updates need separate native evidence.

No core semantic change was needed for this default historical path. The temporary indicator and date-range change were undone, the chart layout saved, and the research tabs closed. Reproduce from the repository root:

```powershell
target/debug/pine-compat.exe analyze .local/community-coverage-20260923/jdehorty-nw-rational-quadratic-v5-20260925.pine --format json
target/debug/pine-compat.exe run .local/community-coverage-20260923/jdehorty-nw-rational-quadratic-v5-20260925.pine --bars .local/community-coverage-20260923/lazybear-volume-flow-chart-bars-v1-20260925.csv --chart-symbol COINBASE:BTCUSD --chart-timeframe 1D > .local/community-coverage-20260923/jdehorty-nw-local-v5-20260925.json
python .local/community-coverage-20260923/compare_jdehorty_nw_v5_20260925.py
```

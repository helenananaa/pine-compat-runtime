# RSI Divergence—KT v6 native plot parity (2026-09-25)

The complete public [RSI Divergence (Pine v4)](https://www.tradingview.com/script/CaMohiJM-RSI-Divergence-Pine-v4/) by kingthies has 2,675 boosts in the September 25 Chrome page. Its title retains “Pine v4”, but the current visible source starts with `//@version=6`; this receipt covers that current v6 revision. The source is retained with nonbreaking spaces normalized to ordinary spaces in ignored `.local/community-coverage-20260923/kingthies-rsi-divergence-v6-20260925.pine` (SHA-256 `1a1531c714ddcaf9b14a7aaae553046b76609019ad697fd1573d37708b6bae5e`). It analyzes and runs unmodified with zero diagnostics.

TradingView ran the published indicator at its default settings on `COINBASE:BTCUSD` 1D. `I:\sys\下载\COINBASE_BTCUSD, 1D (21).csv` is retained as ignored `kingthies-rsi-divergence-native-v6-20260925.csv` (SHA-256 `4c1fee3612635da63094f13b46e60809c6e4b134ee1bb00a0ec3939986b5f0a7`). The export has 4,283 rows; the last is forming and excluded. The other 4,282 match the retained runtime input in timestamp and OHLC.

The ignored `compare_kingthies_rsi_divergence_v6_20260925.py` applies each plot's declared offset when comparing the local values to their positions in the native CSV. RSI and its moving average have zero offset; four divergence plots each have `offset=-5`. **All 25,692 positions across the six exported plot columns match**, including 16,005 paired blanks. The maximum absolute numeric difference is `1.71e-13`. Batch, incremental, and historical realtime modes produce byte-identical full JSON output (SHA-256 `0dcd0670428ba46ac6423695bd929ebc35cec4985dcd1b07d200d4e40ce56a14`).

The script also declares three horizontal levels, three fills (including two gradients), and a hidden midpoint plot. The native CSV does not expose the colors, gradient appearance, or alert delivery. In particular, the four divergence plots contain transparent values outside their colored signal conditions; numeric plot parity alone does not prove visual signal-color parity. Other input settings, symbols, timeframes, and forming updates need separate native evidence.

The temporary indicator and date-range change were undone, the chart layout saved, and the research tabs closed. Reproduce from the repository root:

```powershell
target/debug/pine-compat.exe analyze .local/community-coverage-20260923/kingthies-rsi-divergence-v6-20260925.pine --format json
target/debug/pine-compat.exe run .local/community-coverage-20260923/kingthies-rsi-divergence-v6-20260925.pine --bars .local/community-coverage-20260923/lazybear-volume-flow-chart-bars-v1-20260925.csv --chart-symbol COINBASE:BTCUSD --chart-timeframe 1D > .local/community-coverage-20260923/kingthies-rsi-divergence-local-v6-20260925.json
python .local/community-coverage-20260923/compare_kingthies_rsi_divergence_v6_20260925.py
```

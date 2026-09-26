# Modern Ichimoku Cloud v6 native plot comparison (2026-09-24)

The complete public [Modern Ichimoku Cloud [GBB]](https://www.tradingview.com/script/jJAqvJP5-Modern-Ichimoku-Cloud-GBB/) source was frozen as `.local/community-coverage-20260923/gbb-modern-ichimoku-v6.pine` (SHA-256 `25c5f71e33cee55f7f2c77416bbd54c31b108d8d26370dd1b4dde14d65cd4373`). TradingView ran the published default settings on `BINANCE:BTCUSDT` 1D. Its chart export is `I:\sys\下载\BINANCE_BTCUSDT, 1D (25).csv` (SHA-256 `50a4caa609a25b66b9ae705eac262388fe40d2f16fb865d84b3ed06789eae955`). The temporary indicator insertion and daily resolution change were undone afterward; the original 1h layout was saved, and TradingView reported all changes saved.

The local run used 3,324 frozen daily chart bars from `.local/community-coverage-20260923/full-daily-bars.csv` (SHA-256 `9bec5ba5960c1eb8f099d30820d728cb083e1b3a9e0305a192d85e854cc5c660`). Its `request.security` provider used `.local/community-coverage-20260923/gbb-synthetic-4d-provider-bars.csv` (SHA-256 `6da0275b0f16ff650abf88902deb1a779e6e3653d1f92a97f69100f83c5a3e88`), a UTC four-day aggregation of those daily bars. This provider is a local test fixture, not a native TradingView 4D export.

`compare_gbb_ichimoku_native.py` aligns the native CSV to local daily bars by Unix time, verifies OHLC, then compares its first 24 study columns by column position against the complete source's 16 plots and eight shape series. The two Senkou plots have a `+25` display offset, and Chikou has `-25`; their chart-export columns are compared with the corresponding shifted local values. The export clips displaced values at its own window edges. Two native Chikou values require daily bars beyond the local input and are excluded.

| Check | Result |
| --- | --- |
| Native rows | 2,283 |
| Overlapping daily OHLC bars | 2,281 / 2,281 match |
| 24 study columns | Zero mismatches on comparable rows; 2,281 comparisons per column except Chikou's 2,279 |
| Local full-source execution | 16 plots, eight shape series, 265 alert events, zero diagnostics |
| Batch, incremental, historical realtime JSON | Same SHA-256 `479d6735b2096587e24fe88daffa75fae6d4a606f4444aaa504742b9d2` |

The machine-readable receipt is `.local/community-coverage-20260923/gbb-modern-ichimoku-native-comparison.json`. This comparison verifies exported plot and shape values on one symbol, timeframe, and default settings. TradingView's chart CSV does not provide alert firing, labels, drawn levels, background colors, or tables, so those outputs are outside this parity claim.

Reproduction from the repository root:

```powershell
$b = '.local/community-coverage-20260923'
target/debug/pine-compat.exe run "$b/gbb-modern-ichimoku-v6.pine" --bars "$b/full-daily-bars.csv" --chart-symbol BINANCE:BTCUSDT --chart-timeframe 1D --request-bars "BINANCE:BTCUSDT:4D=$b/gbb-synthetic-4d-provider-bars.csv" > "$b/gbb-modern-ichimoku-current.json"
python "$b/compare_gbb_ichimoku_native.py" "$b/gbb-modern-ichimoku-current.json"
```

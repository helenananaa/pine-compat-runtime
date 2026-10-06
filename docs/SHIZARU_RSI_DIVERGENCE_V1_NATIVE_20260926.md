# RSI Divergence by Shizaru v1 native comparison (2026-09-26)

The public [RSI Divergence by Shizaru](https://www.tradingview.com/script/fH6e5TuN-RSI-Divergence/) had 19,715 boosts when inspected in Chrome. Its unchanged 16-line source has no Pine version annotation and therefore uses v1. The original is retained in ignored `.local/continued-popular-20260926/shizaru-rsi-divergence-v1-original.pine` (SHA-256 `857dbbe4debda5adad3804fd38216425547738faa546e2e0b97e1e98a91d78f5`). It analyzes and runs locally with zero diagnostics or unsupported features; no core change was needed.

TradingView exported 300 `COINBASE:BTCUSD` daily rows to `I:\sys\下载\COINBASE_BTCUSD, 1D (82).csv` (SHA-256 `82da8a5f29db5e92fb31c727c7417988b2b58237b7dbff2dc90db92c2989dfe0`), copied to ignored `.local/continued-popular-20260926/shizaru-rsi-divergence-v1-native-daily.csv`. The existing local historical input ends on 2026-09-25, leaving 299 matching timestamps; the native 2026-09-26 row was forming. All overlapping OHLC and all 299 divergence-plot values match exactly. The chart CSV contains several unnamed `Plot` columns from other active studies, so `compare_shizaru_rsi_divergence_v1.py` selects this indicator's last column by position. The receipt is `.local/continued-popular-20260926/shizaru-rsi-divergence-v1-comparison.json` with zero mismatches.

```powershell
$b = '.local/continued-popular-20260926'
target/debug/pine-compat.exe analyze "$b/shizaru-rsi-divergence-v1-original.pine" --format json
target/debug/pine-compat.exe run "$b/shizaru-rsi-divergence-v1-original.pine" --bars "$b/chart-bars.csv" --chart-symbol COINBASE:BTCUSD --chart-timeframe D > "$b/shizaru-rsi-divergence-v1-local-daily.json"
python "$b/compare_shizaru_rsi_divergence_v1.py"
```

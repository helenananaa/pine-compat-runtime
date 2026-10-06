# Waddah Attar Explosion [LazyBear] v1 native comparison (2026-09-26)

The public [Waddah Attar Explosion [LazyBear]](https://www.tradingview.com/script/iu3kKWDI-Waddah-Attar-Explosion-LazyBear/) had 3,321 boosts when inspected in Chrome. Its 41-line original source has no version annotation, so Pine v1 applies. The unchanged source is retained in ignored `.local/continued-popular-20260926/lazybear-wae-v1-original.pine` (SHA-256 `93fab5d35929bfd9e5ee4891295c714e99cc9c12c4866b746a4af7faeca4acfc`). It analyzes and executes locally with zero diagnostics or unsupported features; no core change was needed for this script.

TradingView exported 300 `COINBASE:BTCUSD` daily rows to `I:\sys\下载\COINBASE_BTCUSD, 1D (80).csv` (SHA-256 `e7f979bb2da4796bbba2adc588fca9a9ec3c65318dbf8d33b458d9abbbc024e7`), copied to ignored `.local/continued-popular-20260926/lazybear-wae-v1-native-daily.csv`. The existing local historical input ends on 2026-09-25, leaving 299 matching timestamps; the extra native bar for 2026-09-26 was forming.

All 299 overlapping OHLC rows and all 897 plot cells (`UpTrend`, `DownTrend`, `ExplosionLine`) match under `1e-9` relative / `1e-7` absolute tolerance. The largest absolute numeric difference is `3.35e-8` on `ExplosionLine`. The reproducible comparison receipt is `.local/continued-popular-20260926/lazybear-wae-v1-comparison.json` with zero mismatches. This qualifies the original indicator's default daily historical output; it does not establish other settings, symbols, or forming-bar parity.

```powershell
$b = '.local/continued-popular-20260926'
target/debug/pine-compat.exe analyze "$b/lazybear-wae-v1-original.pine" --format json
target/debug/pine-compat.exe run "$b/lazybear-wae-v1-original.pine" --bars "$b/chart-bars.csv" --chart-symbol COINBASE:BTCUSD --chart-timeframe D > "$b/lazybear-wae-v1-local-daily.json"
python "$b/compare_lazybear_wae_v1.py"
```

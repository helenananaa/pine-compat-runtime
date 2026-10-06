# WaveTrend with Crosses v1 native parity (2026-09-25)

The public open-source [WaveTrend with Crosses [LazyBear]](https://www.tradingview.com/script/jFQn4jYZ-WaveTrend-with-Crosses-LazyBear/) by lonestar108 has no version directive and therefore runs as implicit Pine v1. The visible source was copied with nonbreaking spaces normalized to ordinary spaces into ignored `.local/community-coverage-20260923/lonestar-wavetrend-crosses-v1-20260925.pine` (SHA-256 `25d5f18a459225fb31500ea1f84fda41f3e6f7d47ce06ddbefee802dc48d4d06e`). The unchanged source analyzes and executes with zero diagnostics. Compared with the earlier original WaveTrend, it adds two sparse `cross(wt1, wt2)` circle plots and conditional `barcolor` output.

TradingView ran the publication on `COINBASE:BTCUSD` 1D at its default inputs. Its chart data was downloaded to `I:\sys\下载\COINBASE_BTCUSD, 1D (18).csv` and copied to ignored `lonestar-wavetrend-crosses-native-v1-20260925.csv` (SHA-256 `cd5adc3c20ddff3de6a2129f1ef352f176b67a900a7292f6a3723fa0075d3760`). The export has 4,283 rows. The final forming bar is excluded, and the remaining 4,282 rows match the retained input's time and OHLC exactly. This script does not use volume.

The native CSV has one generic `Plot` column from an existing chart strategy followed by this script's ten plot columns in source order. The ignored `compare_wavetrend_crosses_v1_20260925.py` compares blank positions and numeric values at `1e-8` relative and absolute tolerance. **All 42,820 positions match**, including 7,440 paired blanks. Each cross plot has 622 nonblank positions, all matched. The largest absolute numeric difference is about `3.88e-12`. The local `barcolor` series has 622 nonblank colors, but TradingView chart CSV does not export candle colors, so color parity is unverified. The research study and date-range change were undone and the layout was saved.

Reproduction uses the retained ignored source and data:

```powershell
target/debug/pine-compat.exe analyze .local/community-coverage-20260923/lonestar-wavetrend-crosses-v1-20260925.pine --format json
target/debug/pine-compat.exe run .local/community-coverage-20260923/lonestar-wavetrend-crosses-v1-20260925.pine --bars .local/community-coverage-20260923/lazybear-volume-flow-chart-bars-v1-20260925.csv --chart-symbol COINBASE:BTCUSD --chart-timeframe 1D > .local/community-coverage-20260923/lonestar-wavetrend-crosses-local-v1-20260925.json
python .local/community-coverage-20260923/compare_wavetrend_crosses_v1_20260925.py
```

This qualifies the default historical plots for the named daily chart. Other inputs, symbols, intervals, forming updates, and displayed candle colors need separate native evidence.

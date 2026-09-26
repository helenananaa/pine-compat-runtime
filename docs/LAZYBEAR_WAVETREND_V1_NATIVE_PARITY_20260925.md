# LazyBear WaveTrend Oscillator v1 native parity (2026-09-25)

The original open-source [WaveTrend Oscillator [WT]](https://www.tradingview.com/script/2KE8wTuF-Indicator-WaveTrend-Oscillator-WT/) has no version directive, so the interpreter selects implicit Pine v1. Its 31-line visible source is retained with nonbreaking spaces normalized to ordinary spaces in ignored `.local/community-coverage-20260923/lazybear-wavetrend-original-v1-20260925.pine` (SHA-256 `c05c8f0909f25d1e6e356d6332e846edbf78d4a756c42109154f0c7a9d25aeff`). The source analyzes and executes with zero diagnostics. Its default path exercises `hlc3`, legacy `ema`, `sma`, `abs`, arithmetic on sparse early series, and eight plot calls.

TradingView ran the unchanged publication on `COINBASE:BTCUSD` 1D with default inputs: channel length 10, average length 21, overbought levels 60 and 53, and oversold levels -60 and -53. The native chart CSV was downloaded to `I:\sys\下载\COINBASE_BTCUSD, 1D (17).csv` and copied to ignored `lazybear-wavetrend-native-v1-20260925.csv` (SHA-256 `2c8554dc908ac4a2f6c3f3813b2882805dc7677fcc3339d7c60abca57c00feb9`). It has 4,283 chart rows; the last, forming bar is excluded. The first 4,282 rows align exactly by time and OHLC with ignored `lazybear-volume-flow-chart-bars-v1-20260925.csv`. Volume is irrelevant to this script.

The export has nine generic `Plot` columns: the first belongs to an existing chart strategy, and the next eight belong to WaveTrend in source order. The ignored `compare_wavetrend_v1_20260925.py` compares native and runtime blanks separately and numerical cells with `1e-8` relative and absolute tolerance. **All 34,256 positions match:** 34,136 numerical cells and 120 matching blanks. The maximum absolute numeric difference is about `3.88e-12`. The temporary indicator and chart range change were undone, and the chart layout showed its saved state.

Reproduce from the repository root, using the retained ignored source and data:

```powershell
target/debug/pine-compat.exe analyze .local/community-coverage-20260923/lazybear-wavetrend-original-v1-20260925.pine --format json
target/debug/pine-compat.exe run .local/community-coverage-20260923/lazybear-wavetrend-original-v1-20260925.pine --bars .local/community-coverage-20260923/lazybear-volume-flow-chart-bars-v1-20260925.csv --chart-symbol COINBASE:BTCUSD --chart-timeframe 1D > .local/community-coverage-20260923/lazybear-wavetrend-local-v1-20260925.json
python .local/community-coverage-20260923/compare_wavetrend_v1_20260925.py
```

This qualifies the original script's default historical daily output on the named symbol. Other settings, symbols, intervals, and realtime forming updates remain unverified.

# Public Bollinger Bands Strategy v5 native parity (2026-09-25)

The complete open-source [Bollinger Bands Strategy](https://www.tradingview.com/script/kE4i5MSC/) by geneZ_fi is Pine v5. Its visible 116-line source was copied with nonbreaking spaces normalized to ordinary spaces into ignored `.local/community-coverage-20260923/genez-bollinger-strategy-v5-20260925.pine` (SHA-256 `2a6384c746558690739f6418b6220fc5f95fdc80a85e8db378cf5534a3bfbbd6`). The original source analyzes and executes with zero diagnostics. It exercises SMA and standard deviation bands, explicit quantity from an evolving cash budget, long and short entries, attached stop exits, equity and open-profit reads, scheduled closes, percent commission, and slippage.

TradingView ran the publication on `COINBASE:BTCUSD` 1D with its default inputs and 1,000 USD initial capital. The chart CSV `I:\sys\下载\COINBASE_BTCUSD, 1D (19).csv` is retained as ignored `genez-bollinger-native-chart-v5-20260925.csv` (SHA-256 `ede03f5ba84f5f9a36a1661752f85ff2ad6b0c0f5b87bc6b410438d0edd66d73`). The complete native trade report `I:\sys\下载\BB_STRATEGY_COINBASE_BTCUSD_2026-09-25.csv` is retained as ignored `genez-bollinger-native-trades-v5-20260925.csv` (SHA-256 `360124fdd9f4a99ea54c2a83e2ac57f43f5cd1fb87bf7f70ffac1d69791e5144`). The chart export has 4,283 rows; the last, forming daily bar was excluded. The other 4,282 rows match the retained runtime input by time and OHLC. The source does not use volume.

The ignored `compare_genez_bollinger_v5_20260925.py` checks all three exported strategy plots and the complete trade list. **All 12,846 plot positions match**, including 347 paired warmup blanks; the maximum absolute numeric difference is about `6.79e-9`. **All 32 closed trades match** by direction, entry and exit UTC date, and displayed entry and exit price. Native quantities are printed with six decimal places; the maximum absolute quantity difference is below `1e-6`. Native net PnL is printed to cents; the maximum absolute local-to-displayed difference is about `0.0084` USD. These display precisions do not prove equality of hidden internal quantities or PnL values.

The same unmodified source and 4,282-bar input produce byte-identical JSON under `run`, `run-incremental`, and `run-realtime-history` (SHA-256 `4af302b3b54f2670bef33776605fe857827ea7c4f947d9b5b72f361b9b6389ce`), including plots, strategy state, orders, trades, and diagnostics. The research strategy and date-range change were undone and the chart layout was saved.

Reproduce the local comparison from the repository root:

```powershell
target/debug/pine-compat.exe analyze .local/community-coverage-20260923/genez-bollinger-strategy-v5-20260925.pine --format json
target/debug/pine-compat.exe run .local/community-coverage-20260923/genez-bollinger-strategy-v5-20260925.pine --bars .local/community-coverage-20260923/lazybear-volume-flow-chart-bars-v1-20260925.csv --chart-symbol COINBASE:BTCUSD --chart-timeframe 1D > .local/community-coverage-20260923/genez-bollinger-local-v5-20260925.json
python .local/community-coverage-20260923/compare_genez_bollinger_v5_20260925.py
```

This qualifies the default historical path on the named daily chart. Other settings, symbols, timeframes, realtime forming updates, and intra-bar fill behavior need separate native evidence.

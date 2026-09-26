# QQE MOD v6 native plot comparison (2026-09-25)

The public [QQE MOD by Mihkel00](https://www.tradingview.com/script/TpUW4muw-QQE-MOD/) publication showed 13,770 boosts in the September 25 UI. Its current open source declares Pine v6. The page-visible code was transcribed with whitespace normalization into ignored `.local/community-coverage-20260923/qqe-mod-v6-20260925.pine` (SHA-256 `fbf99565c94a92f69582d568970d2d6693099bcd8ac37e830eb1454e8658c7aa`). The complete source analyzes as executable with zero diagnostics. The publication's earlier Pine v4 revision was not available as a separate source in this comparison.

TradingView ran the published study with its default inputs on `COINBASE:BTCUSD` 1D. The chart CSV `I:\sys\下载\COINBASE_BTCUSD, 1D (9).csv` was copied to ignored `.local/community-coverage-20260923/qqe-mod-native-btcusd-20260925.csv` (SHA-256 `7c8a3e13f9a2e8dc654992b6076c6f55f595eafe787bdc50611ef1d082a6582d`). The export contains 4,283 rows, of which 4,280 align by time and OHLC with the frozen local input `lazybear-squeeze-native-confirmed-bars-20260925.csv` (SHA-256 `9950b7bd9c152d166375f6424a08a89ff67e04a5395ae849eeaf977a49bf950d`). Two volume values have changed between the exports; QQE MOD reads the close price and does not use volume. Three newer native rows are beyond the local input. The temporary indicator and date-range change were undone, and the original chart layout reported all changes saved.

| Study column | Aligned nonblank numeric positions | Mismatches at `1e-8` |
| --- | ---: | ---: |
| Secondary QQE Trend Line | 4,259 | 0 |
| Secondary RSI Histogram | 4,270 | 0 |
| QQE Up Signal | 1,483 | 0 |
| QQE Down Signal | 1,300 | 0 |

All 4,280 aligned OHLC/time rows match. The four columns have zero blank-position differences and a maximum absolute numeric difference of `4.27e-14`. Batch, incremental, and historical realtime output JSON is byte identical, SHA-256 `906a6a3fc8a3400b41fe001455930f6cb0e2d3e623f2d14bc68c3a779143b3b0`. The ignored `compare_qqe_mod_20260925.py` and `qqe-mod-comparison-20260925.json` retain the comparator and machine-readable receipt.

Reproduce from the repository root:

```powershell
$b = '.local/community-coverage-20260923'
target/debug/pine-compat.exe analyze "$b/qqe-mod-v6-20260925.pine" --format json
target/debug/pine-compat.exe run "$b/qqe-mod-v6-20260925.pine" --bars "$b/lazybear-squeeze-native-confirmed-bars-20260925.csv" --chart-symbol COINBASE:BTCUSD --chart-timeframe 1D > "$b/qqe-mod-local-btcusd-20260925.json"
python "$b/compare_qqe_mod_20260925.py"
```

This qualifies the four exported numeric plots for the tested symbol, timeframe, and default inputs. The chart CSV does not expose the two alert conditions' firing history or dynamic plot colors, and this receipt does not establish those behaviors or live forming-bar parity. No runtime code change was needed for this script.

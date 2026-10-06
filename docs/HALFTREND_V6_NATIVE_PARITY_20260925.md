# HalfTrend v6 native plot comparison (2026-09-25)

The public [HalfTrend by everget](https://www.tradingview.com/script/U1SJ8ubc-HalfTrend-everget/) publication showed 13,172 boosts and 416,956 views in the September 25 UI. Its current open source declares Pine v6. The page-visible source was transcribed with whitespace normalization into ignored `.local/community-coverage-20260923/halftrend-everget-v6-20260925.pine` (SHA-256 `abb81050071012d56867a64f319aca56a86d0a8f398996b0b09cc76d8a73a570`). The complete source analyzes as executable with zero diagnostics.

TradingView ran the published study with its default settings on `COINBASE:BTCUSD` 1D. Its CSV export `I:\sys\下载\COINBASE_BTCUSD, 1D (10).csv` was copied to ignored `.local/community-coverage-20260923/halftrend-native-btcusd-20260925.csv` (SHA-256 `740636a135737e81a29d591b0dccb89f34687ed357ed0e17ad1f6dfa2ade9156`). The export has 4,283 rows, 4,280 of which align by time and OHLC with the frozen local input `lazybear-squeeze-native-confirmed-bars-20260925.csv`. Three newer native rows are beyond that input.

The first local execution had 13,016 numeric or blank-position differences. `ta.highestbars()` and `ta.lowestbars()` returned `na` when the first window was incomplete. TradingView's native HalfTrend output showed that these offsets are calculated from the available bars at the start of the series. The runtime now stops at missing prehistory and skips `na` values inside the available window. The focused offset tests and dependent `request.security` expectations were updated.

| Exported column | Matched nonblank positions | Mismatches at `1e-8` |
| --- | ---: | ---: |
| HalfTrend | 4,280 | 0 |
| ATR High | 4,181 | 0 |
| ATR Low | 4,181 | 0 |
| Arrow Up | 95 | 0 |
| Arrow Down | 94 | 0 |
| Buy Label | 95 | 0 |
| Sell Label | 94 | 0 |

All 4,280 aligned OHLC/time rows match. The seven exported columns have zero blank-position differences and zero numeric differences. Batch, incremental, and historical realtime JSON are byte identical, SHA-256 `470c294986dce6e8e47c8a6f8cb2c9d81c7a5384e2bd122de67cc5c7331277c1`. The ignored comparator `compare_halftrend_20260925.py` and receipt `halftrend-comparison-20260925.json` retain the detailed evidence. `cargo test -p pine-runtime --lib --quiet` passed 1,895 tests, `cargo test -p pine-cli --quiet` passed 239 tests, and `cargo fmt --all -- --check` and `git diff --check` passed. The affected runtime and CLI expectations and golden snapshots were updated. The temporary indicator and date-range change were undone, and the original TradingView layout was saved.

Reproduce from the repository root:

```powershell
$b = '.local/community-coverage-20260923'
target/debug/pine-compat.exe analyze "$b/halftrend-everget-v6-20260925.pine" --format json
target/debug/pine-compat.exe run "$b/halftrend-everget-v6-20260925.pine" --bars "$b/lazybear-squeeze-native-confirmed-bars-20260925.csv" --chart-symbol COINBASE:BTCUSD --chart-timeframe 1D > "$b/halftrend-local-btcusd-20260925.json"
python "$b/compare_halftrend_20260925.py"
```

This qualifies the exported numeric plots and shape locations for the tested symbol, timeframe, and default inputs. The CSV does not expose fill colors, alert firing history, or forming-bar behavior. This sample is Pine v6; it does not qualify the older Pine versions by itself.

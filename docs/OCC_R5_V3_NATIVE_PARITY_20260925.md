# Open Close Cross Strategy R5.1 Pine v3 comparison (2026-09-25)

The public [Open Close Cross Strategy R5 revised by JustUncleL](https://www.tradingview.com/script/vObmEraY-Open-Close-Cross-Strategy-R5-revised-by-JustUncleL/) page showed 9,190 boosts and 296,856 views in the September 25 UI. Its current open source explicitly declares `//@version=3` and contains 180 source lines. The complete page-visible source, with blank lines removed and indentation restored, is retained in ignored `.local/community-coverage-20260923/open-close-cross-r5-v3-20260925.pine` (SHA-256 `08c3d2e7e609ec1236106b8fd71b9b966a4a486ff29258e8fc22cc2e5ddf0f64`). It analyzes as executable with no diagnostics or unsupported entries.

The original source exposed two legacy gaps: Pine v3 aliases for `alma`, `exp`, `cos`, and `tostring`, and numeric formatting with literal suffixes such as `tostring(3, "###D")`. Without the latter, its default higher-timeframe request became `3` minutes instead of `3D`. The alias catalog and string formatter now preserve the original source's `3D` request. The script's default inputs, including its 3D alternate resolution, were used for the comparison.

TradingView ran the strategy on `COINBASE:BTCUSD` 1D. The chart CSV `I:\sys\下载\COINBASE_BTCUSD, 1D (11).csv` was copied to ignored `occ-r5-v3-native-chart-20260925.csv`, and the strategy trade CSV `I:\sys\下载\OCC_Strategy_R5.1_COINBASE_BTCUSD_2026-09-25.csv` to `occ-r5-v3-native-trades-20260925.csv`. A separate `COINBASE:BTCUSD` 3D chart export supplied the host-neutral request bars in `occ-r5-v3-provider-3d-20260925.csv`. The 1D export has 4,283 rows, of which 4,280 align by time and OHLC with the frozen local input. The local run uses explicit September 25 execution timestamps because the script reads `timenow`; its default 10,000-bar limit includes the whole compared interval.

| Comparison | Result |
| --- | ---: |
| Close Series numeric matches | 4,270, zero differences |
| Open Series numeric matches | 4,270, zero differences |
| Blank-position differences | 0 |
| Closed trades with matching entry and exit UTC dates and prices | 178 of 178 |
| Open trade entry date and price | Match |

The native report displayed **100K USD** initial capital, while the unmodified local script uses the runtime's **1M USD** default. The script does not declare `initial_capital`. Consequently, quantities and PnL are not claimed as direct matches. Dividing the local closed-trade quantities by ten leaves a maximum difference of `1.99e-6` units against the native CSV's six-decimal quantity field. This report setting must not be mistaken for proof of a Pine v3 language default. The ignored `compare_occ_r5_v3_20260925.py` and `occ-r5-v3-comparison-20260925.json` retain hashes, alignment, and mismatch details.

Batch, incremental, and historical realtime outputs are byte identical (SHA-256 `d6d6315113cdf7802661f2e4f581c0f1f7c2a1b3c6f208e5bac47bbfdb524cd6`). `cargo test -p pine-runtime --lib --quiet` passed 1,896 tests, `cargo test -p pine-sema --lib --quiet` passed 1,245 tests, and `cargo test -p pine-cli --quiet` passed 239 tests. `cargo fmt --all -- --check` and `git diff --check` passed. The temporary study, date-range, and timeframe changes were undone and the original chart layout was saved.

Reproduce from the repository root:

```powershell
$b = '.local/community-coverage-20260923'
target/debug/pine-compat.exe analyze "$b/open-close-cross-r5-v3-20260925.pine" --format json
target/debug/pine-compat.exe run "$b/open-close-cross-r5-v3-20260925.pine" --bars "$b/lazybear-squeeze-native-confirmed-bars-20260925.csv" --chart-symbol COINBASE:BTCUSD --chart-timeframe 1D --request-bars "COINBASE:BTCUSD:3D=$b/occ-r5-v3-provider-3d-20260925.csv" --execution-times "$b/occ-r5-v3-execution-times-20260925.txt" > "$b/occ-r5-v3-local-default-20260925.json"
python "$b/compare_occ_r5_v3_20260925.py"
```

This qualifies the default-input historical plots and fill dates/prices on the tested symbol and timeframe. It does not qualify nondefault moving-average choices, account-property overrides, PnL, intrabar execution, or forming-bar repaint behavior. The original script enables `lookahead_on`, so its historical higher-timeframe series can repaint; the runtime emits an explicit warning for both requests.

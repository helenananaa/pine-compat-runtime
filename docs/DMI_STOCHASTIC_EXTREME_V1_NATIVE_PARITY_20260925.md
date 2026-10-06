# DMI Stochastic Extreme original-source comparison (2026-09-25)

The public [DMI Stochastic Extreme](https://www.tradingview.com/script/xUXuZ3Ki-DMI-Stochastic-Extreme/) page showed 2,268 boosts on September 25, 2026. Its 39-line source has no Pine version directive. The page's “Version 2” describes the author's indicator revision; the interpreter analyzes the source as implicit **Pine v1**. The visible source was transcribed with normalized whitespace into ignored `.local/community-coverage-20260923/dmi-stochastic-extreme-v1.pine` (SHA-256 `5a3b06859de37d5201a03f6b7eff371c95cc28298ce4ab9347fb5f812d913238`). The original source analyzes with zero diagnostics after this change.

The published study was inserted into TradingView on `COINBASE:BTCUSD` 1D with its four default inputs (DMI 10, stochastic 3, oversold 10, overbought 90). The full-range chart export `I:\sys\下载\COINBASE_BTCUSD, 1D (8).csv` is retained as ignored `.local/community-coverage-20260923/dmi-stochastic-extreme-native-20260925.csv` (SHA-256 `b98f3399a2178d47a22941d601ce72ed57665d60dd990db94903b495b3072fb4`). It has 4,283 rows, with 4,280 dates matching the earlier frozen Coinbase input. The three latest native rows are beyond the input and are excluded. The temporary indicator and date-range change were undone before the chart layout was saved.

| Check | Result |
| --- | --- |
| Source admission | Implicit Pine v1, zero diagnostics; full historical execution |
| Aligned bars | 4,280/4,280 times and OHLC match |
| Five exported study columns | `Stochastic`, `Over Bought`, `Over Sold`, `Crossing Up`, `Crossing Down`: zero blank-position or numeric mismatches at `1e-8` |
| Maximum numeric difference | `2.49e-12` |
| Volume | Four revised volume cells differ between the two chart exports; this script does not read volume |
| Execution profiles | Batch, incremental, and historical realtime outputs have the same JSON SHA-256 `2ad5ae2e578187cc2a2b0a9edc16e45fa75cf27b4ae5bb87c7f548d9af39d8b8` |

The source's `wwma` function uses its own local `wwma[1]` history. Previously this was rejected as an unknown symbol. The analyzer now predeclares a float series identity for a qualifying Pine v1/v2 function-local numeric self-history declaration, then reuses that identity for its assignment. A focused regression verifies two independent call sites and both versions. The first full-source native comparison also exposed one warmup mismatch: legacy `highest`/`lowest` skip `na` within a complete bar window, whereas the runtime required every cell to be numeric. The v1/v2 window behavior now matches the native script; a separate regression covers this case. Existing later-version behavior is unchanged.

The ignored `compare_dmi_stochastic_20260925.py` checks native prices, all five study columns, missing values, and source/data hashes. Its machine-readable receipt is `dmi-stochastic-extreme-comparison-20260925.json`. Reproduce from the repository root:

```powershell
$b = '.local/community-coverage-20260923'
target/debug/pine-compat.exe analyze "$b/dmi-stochastic-extreme-v1.pine" --format json
target/debug/pine-compat.exe run "$b/dmi-stochastic-extreme-v1.pine" --bars "$b/lazybear-squeeze-native-confirmed-bars-20260925.csv" --chart-symbol COINBASE:BTCUSD --chart-timeframe 1D > "$b/dmi-stochastic-extreme-local-20260925.json"
python "$b/compare_dmi_stochastic_20260925.py"
```

`cargo test -p pine-sema -p pine-runtime --lib --locked --quiet` passed 1,244 semantic and 1,895 runtime tests. `cargo fmt --all -- --check` and `git diff --check` passed.

This verifies the displayed numeric series and plot-character event values for the tested symbol, timeframe, and defaults. The CSV does not verify glyph appearance, colors, other input settings, or forming-bar updates.

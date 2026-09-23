# Complete-script native reference expansion, 2026-09-23

This is a local development receipt for source commit
`f249b3db45f0f145c621000422c679557a321ec9`. It expands measured
standard-candle v6 indicator coverage; it does not qualify a release artifact.
The locally built `target/debug/pine-compat.exe` had SHA-256
`68988f328364250f3c94a782d1b20ba0c8ade60758b6cc48862690e5445c09ee`.
The complete TradingView built-in sources and chart exports are retained only in
the ignored `.local/script-coverage-20260923/` directory on the qualification
machine. They are not repository fixtures or redistributable inputs.

## Frozen inputs and provenance

All three unchanged scripts came from the TradingView Pine Editor's built-in
indicator source view. The independent references are TradingView's UI chart
data exports for `BINANCE:BTCUSDT`, monthly and daily. No runtime or compiler
code was changed for this audit. SHA-256 hashes identify the exact local files:

| Local file under `.local/script-coverage-20260923/` | SHA-256 |
| --- | --- |
| `supertrend-built-in.pine` | `91cee87a701daa36a1a1bf9dfb448b6bcaf5b699f8b170560b1ec5d789483cc6` |
| `ichimoku-cloud-built-in.pine` | `0ce110aebb2585c51cdbc0a4b4971ee9e74f9cc483668e0df6616cb3ab0884b9` |
| `macd-built-in.pine` | `ee50c0501c26f667392477e473ce712324e353c67feaccfedf80a20a55c72efc` |
| `supertrend-native-monthly.csv` | `5c40490cd7f0809c9ebbb3aff2b5009f90e9430766562176782f2c754b53664f` |
| `ichimoku-native-monthly.csv` | `df6423d8b7c3a1641a5adb0ca2d56636f2f94264dde294a48b74e31ddf8c7629` |
| `daily-native-20260923.csv` | `ffe2afb530ed196d86d64dacd5761e46b1b0049e23b62d1f81087389459c1f56` |
| `macd-native-daily.csv` | `6e6d185a132abcc9aa082e9c80a7feb381ce7f1c035c6741b765279198cc85ef` |
| `macd-sma-native-daily.csv` | `adc6442db11d6bb82837e3788db6681f5517a09f7f89d23a89df52e8eee1db3f` |
| `daily-bars-with-prefix.csv` | `981bb12a47bebdf64f5f96cef9070e5fe723b0ad6726322bdd72af39cda9d468` |

The monthly comparison uses 107 confirmed bars, excluding the forming
September 2026 bar. The daily comparison uses 299 confirmed exported bars,
excluding the forming September 23 bar. The chart export only included a
limited visible daily window. `build_daily_full.py` prefixed 2,721 earlier
OHLCV bars from an older TradingView UI export; it verified 251 overlapping
confirmed bars against the fresh export and replaced the older export's last
forming bar. The resulting input has 3,020 bars. This warmup is essential for
recursive averages and trend state; direct execution on only the visible 299
bars is not equivalent to TradingView's prior history.

## Independent numerical results

The CLI compiled and executed each complete source without diagnostics.
Comparison tolerance was `abs <= 1e-8` or `rel <= 1e-10`. Counts are individual
displayed plot positions, including distinct runs of the same MACD source with
different settings. Native exports and runtime results are archived locally
alongside comparison JSON and the comparison scripts.

| Script and setting | Timeframe | Confirmed bars | Compared values | Mismatches |
| --- | --- | ---: | ---: | ---: |
| Supertrend, default | 1M | 107 | 214 | 0 |
| Ichimoku Cloud, default | 1M | 107 | 460 | 0 |
| Supertrend, default | 1D | 299 | 598 | 0 |
| Ichimoku Cloud, default | 1D | 299 | 1,420 | 0 |
| Moving Average Convergence Divergence, EMA/EMA | 1D | 299 | 897 | 0 |
| Moving Average Convergence Divergence, SMA/SMA | 1D | 299 | 897 | 0 |
| **Total** | | | **4,486** | **0** |

Ichimoku comparison aligns the script's `-25` and `+25` plot offsets with the
exported display positions. The first 25 leading-span positions in the daily
export have no source values in that export, so they are marked
`unobservableExportHead` and excluded from the 1,420 compared values. They are
not silently treated as passes. The final 25 lagging-span display positions
likewise have no confirmed native values inside the exported window.

`run`, `run-incremental`, and `run-realtime-history` produced identical full
JSON output for all three scripts on the 3,020-bar input; MACD was checked in
both parameter sets. `verify_modes.py` compares every top-level output field,
including plots, fills, alerts, and diagnostics. This is execution-mode parity,
not independent TradingView qualification of every output field.

## Reproduction and limits

After verifying the local source/export hashes, build with
`cargo build -p pine-cli --locked`. Execute the complete sources with
`target/debug/pine-compat run <source> --bars <bars.csv>
--chart-symbol BINANCE:BTCUSDT --chart-timeframe 1D` (or `1M` for monthly).
The alternate MACD run adds `--input-override 5=SMA --input-override 6=SMA`.
The retained local comparison scripts are `compare_monthly.py`,
`compare_daily_full.py`, `compare_macd_daily.py`, `compare_macd_sma.py`, and
`verify_modes.py`; their JSON reports preserve denominators and per-plot
errors. These commands only describe reproduction on the qualification
machine because the TradingView inputs are intentionally not committed.

The evidence covers numerical line positions on confirmed standard candles.
Native visual appearance, color and fill rendering, alert firing, forming-tick
paths, strategy/broker behavior, other symbols/timeframes/settings, and
installed Python/WASM artifacts from this source commit remain unverified by
this receipt. An older community v4 strategy visible in the exploratory chart
was not admitted or counted. This audit does not change the project's
host-neutral runtime boundary or stable-release classification.

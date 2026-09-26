# Pivot Points Standard native comparison (2026-09-24)

The unmodified TradingView Pivot Points Standard v6 source now analyzes without
diagnostics and executes with a host-supplied monthly `request.security` series.
The source, daily bars, and monthly provider CSV are local ignored evidence:

| File | SHA-256 |
| --- | --- |
| `.local/product-completion-20260912/corpus/pivot-points-standard.pine` | `C1592B6E15308D97B06BAED8D717EA7CC9D09F29322056356A9B79971910269B` |
| `.local/community-coverage-20260923/volatility-reversion-native-confirmed-bars.csv` | `278AE090E32E441E4CADD2BD70955DDAD650AFEEEDA21FE17651A18F5092593A` |
| `.local/pivot-20260924/btcusdt-monthly-provider.csv` | `511D00DEA39FE8DA0FBA3F1885FCDD64551D10626F4C4F50DD145AB2B6771716` |

The monthly file derives from the TradingView `BINANCE_BTCUSDT, 1M.csv` export
in `I:\sys\下载\` (SHA-256
`5BA42AED0130CBEF847AE0FFEF99B14B4306CAB0FF6C79A445FB4ED5E59CC70A`).
The in-progress September bar was replaced with the September 1-22 daily OHLC
aggregate; September pivots depend on the completed August bar. The chart was
`BINANCE:BTCUSDT` at 1D, price grid 0.01, with default `Auto` monthly pivots.

The default Traditional script emitted 1,177 historical line and label IDs,
with drawing deletion snapshots for old periods and zero diagnostics. The
September pivot prices matched TradingView's Data window for all 11 levels:

| Level | Native and local price |
| --- | ---: |
| P | 74,111.72 |
| R1 / S1 | 85,948.44 / 66,744.57 |
| R2 / S2 | 93,315.59 / 54,907.85 |
| R3 / S3 | 105,152.31 / 47,540.70 |
| R4 / S4 | 116,989.03 / 40,173.55 |
| R5 / S5 | 128,825.75 / 32,806.40 |

For Woodie, the original source ran with `--input-override 1=Woodie` and zero
diagnostics. Data window prices were displayed to two decimals, but rounding
at half-tick boundaries differed from the source's
`str.tostring(level, format.mintick)` label. Two temporary native boolean plots
established that Woodie R1 formats as `88183.23` and S3 as `49775.49`; a
negative literal `-1.235` formats as `-1.24`. The previous runtime tolerance
forced all near-half-tick ratios upward and produced `88183.24`, `49775.50`,
and `-1.23`. Removing that tolerance reproduces all three native strings.
The local Woodie P through S4 labels after the fix are `75229.12`, `88183.23`,
`68979.36`, `94432.99`, `56025.25`, `107387.10`, `49775.49`,
`126590.97`, and `30571.63` in array order. The regression fixture covers
both the negative tie and a Woodie R1 arithmetic path.

This qualifies the observed 1D monthly Traditional and Woodie values, with
the native script instrumented only by Data window plots during comparison.
Other pivot types, timeframes, drawing appearance, and realtime updates are
not covered by this comparison. The temporary study was removed, the original
Pine editor buffer restored, and the chart layout saved.

The final local gate passed `cargo test -p pine-builtins -p pine-syntax
-p pine-sema -p pine-runtime -p pine-cli -p pine-wasm --locked --quiet`.
`cargo fmt --all -- --check` and `git diff --check` also passed. These are
source-tree checks; no wheel or browser embedding artifact was rebuilt.

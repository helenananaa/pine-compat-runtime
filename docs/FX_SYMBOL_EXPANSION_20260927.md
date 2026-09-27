# FX symbol expansion: EURUSD weekly State-Dependent EMA

Date: 2026-09-27. Source baseline: `6a7a2ac24bf9222cc3e77ca08363956bb3c45b48`.
The original public v6 [State-Dependent EMA [BackQuant]](https://www.tradingview.com/script/jdVw4YmG-State-Dependent-EMA-BackQuant/)
source was unchanged, SHA-256
`a4e71b3ab6b9e706eaf934588ea6997128231eab4abab283202234275c309ccc`.
This is a historical indicator comparison, not strategy or live-tick qualification.

The TradingView chart identified the symbol as `FX:EURUSD`, 1W, from FXCM.
Successive chart zooms loaded the native export back to its 1971-01-03 first
bar. The frozen CSV contains 2,907 rows. The last displayed row was excluded
to avoid live-edge ambiguity, leaving 2,906 confirmed comparison bars through
2026-09-13. The first native State-Dependent EMA equals the first close, so
this comparison begins at the source history origin rather than an arbitrary
recursive-filter warmup point. The three numeric exports have identical
confirmed OHLCV. The CLI consumed those exact bars, symbol `FX:EURUSD`,
timeframe `1W`, and host price grid `1/100000`; this indicator does not exercise
broker rounding or quantity precision.

| Setting | Override | Native CSV SHA-256 | Numeric cells | Differences |
| --- | --- | --- | ---: | ---: |
| Published defaults, Close | none | `ec79b15254b915bbca1684710dcb84ac60c5f84613447bfcc7936002a023f749` | 23,248 | 0 |
| HLC3 calculation source | `1=hlc3` | `f07dd328eefe6334b6b0a5be497a57ff132420aa74221fcc81b1974a524a3233` | 23,248 | 0 |
| Fixed EMA comparison enabled | `23=true` | `7512c71a6940205f83c4a18695be78643e4999015ee845222f3d680f34ad1277` | 26,154 | 0 |

The first two settings have eight populated numeric series; the third adds
`Fixed EMA Comparison`. All 72,650 cells were compared from the first confirmed
bar, with missing values exact and finite values at absolute and relative
tolerance `1e-9`. Each setting produced no diagnostics, and complete public
JSON was byte-identical across batch, incremental, and realtime-history modes.

`Trend Direction=Filter Slope` was also exported. Its native CSV is byte-for-byte
identical to the default CSV because this parameter affects trend coloring and
alerts, which the chart-data CSV does not expose. The local run changes alert
count from 146 to 342 and remains identical across the three historical modes.
No native color or alert parity is claimed for this setting. The local run and
scope receipt are retained so this branch is not mistaken for an independently
qualified numeric case.

The unchanged Pine source, exact native CSV files, generated bar input, local
JSON, comparison receipts, and chart screenshot are retained in
`.local/fx-symbol-expansion-20260927/`. The current input-source implementation
accepts the built-in HLC3 selector through the public host override contract;
this comparison verifies it on a five-decimal forex chart as well as the prior
ETHUSD chart. The next evidence batch should use an independent public strategy
on a second symbol or timeframe and compare actual order/fill outcomes.

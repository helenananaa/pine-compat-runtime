# Parameter and symbol expansion: State-Dependent EMA on ETHUSD

Date: 2026-09-27. Source revision: `6a7a2ac24bf9222cc3e77ca08363956bb3c45b48`.
This is a local historical comparison, not a release or live-tick qualification.

The unchanged public v6 [State-Dependent EMA [BackQuant]](https://www.tradingview.com/script/jdVw4YmG-State-Dependent-EMA-BackQuant/) source has SHA-256
`a4e71b3ab6b9e706eaf934588ea6997128231eab4abab283202234275c309ccc`.
The earlier `COINBASE:BTCUSD` 1D default comparison is recorded in
`EXPANDED_INDICATOR_STRATEGY_NATIVE_PARITY_20260926.md`. This batch adds
`COINBASE:ETHUSD` 1D with two independently exported TradingView settings:
the default `Combined` state model and `Efficiency`, changed through the
published indicator's input dialog. All other inputs remain at their defaults.

TradingView exported 990 daily rows for each setting, starting 2024-01-12.
The final 2026-09-27 forming row was excluded. The two exports' 989 confirmed
OHLCV rows are identical. The local host used symbol `COINBASE:ETHUSD`, timeframe
`1D`, price grid `1/100`, quantity precision `6`, and the exact exported OHLCV.
The grid and precision are explicit host settings for this local run; this
indicator's eight compared numeric columns do not exercise broker sizing.
`input.string` call-site 4 was overridden with `Efficiency` for that case.
Call-site IDs belong to this compilation and must be rediscovered if the source
changes.

| State model | Native CSV SHA-256 | Confirmed comparison window | Numeric cells | Differences |
| --- | --- | ---: | ---: | ---: |
| Combined | `ca06a2879a20293576f3b33c912b0eda2b58a602a216f2e36ea8b97cfff68410` | 776 bars | 6,208 | 0 |
| Efficiency | `3fd1ac4a6b534aec362053af392de0d9a549acdd6a17d30bb2895895a9544289` | 776 bars | 6,208 | 0 |

The eight columns are Ribbon Reference, State-Dependent EMA, Adaptive Alpha,
Effective EMA Length, Efficiency Ratio, Sustained Residual State, Variance
Ratio, and Market State. Missing values are compared exactly; finite values
use absolute and relative tolerance `1e-9`. `Fixed EMA Comparison` is empty
in both settings. Visual gradient geometry, colors and alerts are outside this
numeric receipt.

The export begins well after the chart's original history. Running the
recursive filter from its first exported bar therefore gives transient
differences. The first 213 bars are explicitly excluded from native comparison;
the comparison starts at zero-based index 213 (2024-08-12). With `Combined`,
all differences end by index 210. With `Efficiency`, the final two differences
are in Sustained Residual State at indexes 211 and 212. No source or runtime
change was made to suppress them. A future capture from the original chart
history is needed before claiming full-history agreement.

The CLI was freshly rebuilt at the stated source revision with
`cargo build -p pine-cli`. For each setting, full JSON output is byte-identical
across batch, incremental, and realtime-history execution. This tests historical
mode consistency, not forming-bar updates. The source, both native exports,
generated bars, complete local JSON, exact comparison summary, and chart
screenshot are retained under `.local/parameter-symbol-expansion-20260927/`.
No runtime defect was isolated in this batch. Next expand one independent
timeframe and a parameter that changes the source series or trend direction;
capture enough history for recursive warmup and preserve separate denominators.

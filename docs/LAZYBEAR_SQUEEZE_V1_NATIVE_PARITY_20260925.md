# Original LazyBear Squeeze Momentum native comparison (2026-09-25)

The public [Squeeze Momentum Indicator [LazyBear]](https://www.tradingview.com/script/nqQ1DT5a-Squeeze-Momentum-Indicator-LazyBear/) publication shows a legacy Pine source with no version directive. Its visible source was transcribed with normalized whitespace into the ignored `.local/community-coverage-20260923/lazybear-squeeze-original-v1.pine` (SHA-256 `4b10fed9e697ad3e9ed47735c508578b4191763202fa360f88224dffbf5e9aca`). The CLI analyzes it as implicit Pine v1, executable with zero diagnostics. The page also points readers to an updated source elsewhere; this comparison covers the original source displayed on the publication page, including its `multKC * stdev(...)` expression.

The original published indicator was loaded on a TradingView `COINBASE:BTCUSD` 1D chart with default inputs. The native chart CSV was downloaded to `I:\sys\下载\COINBASE_BTCUSD, 1D (6).csv` and copied to ignored `.local/community-coverage-20260923/lazybear-squeeze-native-btcusd-20260925.csv` (SHA-256 `f44a440a7ac4c22dd53f1b525fdd1901d3b07130d096d376d0d2a1c7becae2b0`). The export has duplicate `Plot` headers. Its eighth and ninth columns are this indicator's histogram and zero line; the seventh belongs to another chart script. The temporary indicator was removed, and TradingView showed the chart's changes saved.

The runtime used the first 4,280 exported OHLCV rows, frozen as ignored `lazybear-squeeze-native-confirmed-bars-20260925.csv` (SHA-256 `9950b7bd9c152d166375f6424a08a89ff67e04a5395ae849eeaf977a49bf950d`). Three subsequent native rows are outside that input. `compare_lazybear_squeeze_20260925.py` checks the bar times and OHLCV, blank positions, and both numeric plot series against the full source execution. Its machine-readable receipt is `lazybear-squeeze-comparison-20260925.json`.

| Check | Result |
| --- | --- |
| Compared chart bars | 4,280; zero OHLCV or time mismatches |
| Histogram warmup | First nonblank at zero-based bar 38; 4,242 nonblank values |
| Histogram and zero line | Zero blank-position or numeric mismatches at `1e-8`; maximum absolute numeric difference `4.10e-11` |
| Full source execution | Two plots, zero diagnostics |

The initial runtime result delayed the histogram to bar 57 because `math.avg` returned as soon as its first argument was `na`. That skipped evaluation of a nested `sma(close, 20)` argument for 19 bars, delaying its stateful window. `math.avg`, `math.max`, and `math.min` now evaluate every argument before propagating `na`. A regression covers nested SMA calls after an early `na` for all three functions. The Pine runtime library's 1,893 tests pass.

Reproduce the local execution and comparison from the repository root:

```powershell
$b = '.local/community-coverage-20260923'
target/debug/pine-compat.exe analyze "$b/lazybear-squeeze-original-v1.pine" --format json
target/debug/pine-compat.exe run "$b/lazybear-squeeze-original-v1.pine" --bars "$b/lazybear-squeeze-native-confirmed-bars-20260925.csv" --chart-symbol COINBASE:BTCUSD --chart-timeframe 1D > "$b/lazybear-squeeze-local-native-bars-20260925.json"
python "$b/compare_lazybear_squeeze_20260925.py"
```

This establishes the exported numerical plots for one symbol, daily timeframe, and default inputs. The CSV does not expose the dynamic histogram/zero-line colors, so it cannot establish color parity. The updated source linked from the publication and other input settings were not compared.

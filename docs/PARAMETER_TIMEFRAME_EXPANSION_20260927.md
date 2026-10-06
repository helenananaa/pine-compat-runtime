# Parameter and timeframe expansion: ETHUSD weekly State-Dependent EMA

Date: 2026-09-27. Source baseline: `6a7a2ac24bf9222cc3e77ca08363956bb3c45b48`.
This is a local historical comparison, not a release or forming-tick qualification.

The unchanged public v6 [State-Dependent EMA [BackQuant]](https://www.tradingview.com/script/jdVw4YmG-State-Dependent-EMA-BackQuant/)
source has SHA-256 `a4e71b3ab6b9e706eaf934588ea6997128231eab4abab283202234275c309ccc`.
The native TradingView UI independently exported four settings on
`COINBASE:ETHUSD`, 1W. The 539 confirmed OHLCV rows, from 2016-05-23 through
2026-09-14, are identical in all four exports. The final forming 2026-09-21
week was excluded; its close changed between exports. The local CLI consumed
the exact confirmed OHLCV with chart symbol `COINBASE:ETHUSD`, timeframe `1W`,
price grid `1/100`, and quantity precision `6`.

| Setting relative to published defaults | Override | Native CSV SHA-256 | Numeric cells | Differences |
| --- | --- | --- | ---: | ---: |
| Combined | none | `4629bc38e928cb77de02b73959df2d4452233e69f8bab198ca9889e16073d78f` | 4,312 | 0 |
| Sustained Residual model | `4=Sustained Residual` | `b6211918671fffcecfcda32e9d938f7fe3443f5b538602e60f6446e32526629e` | 4,312 | 0 |
| Variance Penalty zero | `13=0` | `6e3cd0118d2ac2c1ff4280b4ffbfe0e989b469860733ac709e6ee6bc856940e5` | 4,312 | 0 |
| HLC3 source | `1=hlc3` | `8f69108d7fc9d95ce3301c03f6ac416286b889cefae9b18a451a78dd42fd93b2` | 4,312 | 0 |

Each export compares Ribbon Reference, State-Dependent EMA, Adaptive Alpha,
Effective EMA Length, Efficiency Ratio, Sustained Residual State, Variance
Ratio, and Market State. Missing values compare exactly; finite values use
absolute and relative tolerance `1e-9`. `Fixed EMA Comparison` is empty in all
settings. These 17,248 comparisons cover the full confirmed chart history,
without a warmup exclusion. Both the state-model and penalty changes alter
hundreds of output rows versus Combined. Visual gradients, colors, and alert
delivery were not compared.

The HLC3 export exposed a real parameter gap: all public hosts rejected
`input.source` overrides. The runtime now evaluates a call-site keyed source
selector against the current chart bar. Rust, CLI, Python, and WASM accept
`open`, `high`, `low`, `close`, `hl2`, `hlc3`, `ohlc4`, and `hlcc4`. Unknown names
and external indicator plot sources fail explicitly. This keeps the selector
deterministic and independent of market-data acquisition or CandleScope.

Subsequent local host coverage added the same selector for generic series-float
`input(...)` calls. That follow-up was not part of these four native exports.

For each setting, the full public JSON was byte-identical across batch,
incremental, and realtime-history modes. The HLC3 result has no diagnostics.
The source, four native CSV files, common confirmed-bar input, local JSON,
comparison receipts, execution-mode receipt, and HLC3 chart screenshot are retained under
`.local/parameter-timeframe-expansion-20260927/`. The chart source selector has
a focused eight-case runtime test and was checked by building the CLI and
compiling Python and WASM hosts. The next expansion should add a second symbol
with different price scale and an independent strategy/broker script; these
results alone cannot qualify either.

# TradingView community script qualification, 2026-09-23

The [expanded September 26 indicator/strategy comparison](EXPANDED_INDICATOR_STRATEGY_NATIVE_PARITY_20260926.md)
adds four complete v6 sources and five parameter scenarios: 154,180 exported
chart cells and 334 closed trades plus one open position pass scoped native
comparisons. Explicit entry/order quantities now honor the configured decimal
contract grid. This remains local historical evidence for the named settings.

The [September 26 popular profile indicator comparison](POPULAR_PROFILE_V6_NATIVE_PARITY_20260926.md)
adds unchanged BigBeluga and Zeiierman v6 sources, repairs drawing-enum helper
admission, skipped-call history and nested-call history-offset binding, and matches
59,962 native state cells over 4,283 confirmed bars per script. Separate v4/v6
history controls and execution-mode checks qualify the repaired semantics; native
drawing geometry and broader release qualification remain outside this evidence.

This is a local development receipt for complete public community scripts,
starting from `f249b3db45f0f145c621000422c679557a321ec9`. At the time of
this audit, the code changes described below were a local candidate. No wheel,
WASM package, or release is qualified by this receipt. TradingView's published
source text and UI exports are retained only in ignored
`.local/community-coverage-20260923/`.
The published HTML's nonbreaking spaces were normalized to spaces for local
Pine source files. No community source or TradingView market data is committed.

Later compatibility work includes the implicit-v1 original
[LazyBear Squeeze Momentum native comparison](LAZYBEAR_SQUEEZE_V1_NATIVE_PARITY_20260925.md):
4,280 `COINBASE:BTCUSD` daily bars, both exported numerical plots aligned,
and zero blank-position or numeric mismatches at `1e-8`.
The implicit-v1
[DMI Stochastic Extreme comparison](DMI_STOCHASTIC_EXTREME_V1_NATIVE_PARITY_20260925.md)
also aligns five exported study columns over 4,280 daily bars after admitting
function-local self-history and the observed legacy extreme-window warmup.
The later [QQE MOD v6 comparison](QQE_MOD_V6_NATIVE_PARITY_20260925.md)
admits the complete current public source and aligns its four exported
numerical plots over 4,280 daily bars at default inputs.
The implicit-v1 [original WaveTrend](LAZYBEAR_WAVETREND_V1_NATIVE_PARITY_20260925.md)
and [WaveTrend with Crosses](WAVETREND_CROSSES_V1_NATIVE_PARITY_20260925.md)
follow-ups align all eight and ten native plot series, respectively, over
4,282 confirmed daily bars; chart CSV does not expose the latter's candle colors.
The complete public v5 [Bollinger Bands Strategy](BOLLINGER_BANDS_STRATEGY_V5_NATIVE_PARITY_20260925.md)
further aligns three plots across 4,282 confirmed bars and all 32 closed
TradingView trades by direction, dates, and displayed prices at default inputs.
The public v4 [RSI Mean Reversion Bot Strategy](RSI_MEAN_REVERSION_V4_NATIVE_PARITY_20260925.md)
aligns nine plots across 4,282 confirmed bars and all 11 closed trades at
default inputs, including displayed quantity at the chart's six-decimal precision.
The original Pine v4 [KivancOzbilgic SuperTrend indicator and strategy](KIVANC_SUPERTREND_V4_NATIVE_PARITY_20260925.md)
each align seven exported plot and signal fields over 4,282 confirmed daily
bars. The strategy also aligns all 68 closed trades and its open long entry
by direction, dates, prices, quantity, and exported PnL at default inputs.
The original Pine v4 [DonovanWall Gaussian Channel](DONOVAN_GAUSSIAN_CHANNEL_V4_NATIVE_PARITY_20260925.md)
aligns its filter and both channel bands over all 4,282 confirmed daily bars.
Its native output exposed the v4 fractional division of an input integer and
motivated the corrected version boundary.
The current v6 [everget Chandelier Exit](EVERGET_CHANDELIER_EXIT_V6_NATIVE_PARITY_20260925.md)
aligns both stop lines and all four buy/sell marker series over 4,282
confirmed daily bars, including the positions and numerical levels of 65 buy
and 65 sell signals.
The current v6 [RSI Divergence—KT](RSI_DIVERGENCE_KT_V6_NATIVE_PARITY_20260925.md)
revision aligns six native-exported plot columns over the same 4,282 confirmed
bars, including its four five-bar offset divergence series.
The public v5 [AlphaTrend](ALPHATREND_V5_NATIVE_PARITY_20260925.md) aligns
both trend lines and BUY/SELL shape series over 4,282 confirmed bars in each
of its default MFI and no-volume RSI calculation paths.
Its public v5 [AlphaTrend Strategy](ALPHATREND_STRATEGY_V5_NATIVE_PARITY_20260925.md)
also aligns two plots and all 98 closed trades, with one matching open entry,
on the same confirmed daily chart history.
The public v5 [Nadaraya-Watson rational quadratic kernel](NADARAYA_WATSON_RATIONAL_QUADRATIC_V5_NATIVE_PARITY_20260925.md)
aligns its visible estimate exactly over 4,282 confirmed daily bars, including
the 26 warmup blanks; the exported CSV omits its hidden alert stream.
The public v5 [Nadaraya-Watson Envelope](NADARAYA_WATSON_ENVELOPE_V5_NATIVE_PARITY_20260925.md)
with the exact `KernelFunctions/2` source aligns seven numerical plots over
the same 4,282 bars; the four fill appearances remain outside CSV evidence.
The public v5 [Heiken Ashi & Super Trend strategy](RINGSCHERRY_HEIKIN_ASHI_SUPERTREND_V5_NATIVE_PARITY_20260925.md)
aligns all 67 closed native trades by direction, dates, signals, and displayed
entry/exit prices on `COINBASE:BTCUSD` daily bars using an explicit host-provided
Heikin Ashi request stream. Its fractional-tick exits and reversal-day relative
brackets exposed broker-emulator gaps that now have focused regressions.

## Script selection and admission

| Public script | Local source SHA-256 | Result at `f249b3db` | Local candidate |
| --- | --- | --- | --- |
| [Smart Money Concepts (SMC) by LuxAlgo](https://www.tradingview.com/script/CnB3fSph-Smart-Money-Concepts-SMC-LuxAlgo/), v5, 168.2K boosts | `d047cf91b0da2cbebeab5d5ac259797a20653383dc850a79019040cf2b00e8fd` | Not executable: 105 diagnostics, chiefly function mutation of global UDT fields, dynamic array index typing, and UDT branch/function-result typing. Count includes cascades, not 105 distinct root causes. | Executable with zero diagnostics on the unmodified published source; full 3,324-bar historical run and instrumented native comparison below. |
| [Trendlines with Breaks by LuxAlgo](https://www.tradingview.com/script/IYL88A1N-Trendlines-with-Breaks-LuxAlgo/), v5, 52.2K boosts | `f77236c2e5b4052df8165aa437811e581f80819bc836805cd3d5d78d5bd858b4` | Executable, zero diagnostics. | Executable, independently compared below. |
| [3Commas Bot by Bjorgum](https://www.tradingview.com/script/MvlwAzSg-3Commas-Bot/), v5, 14.9K boosts | `07e2a14a43724bb3bae2df59a149360b1efc8c6b8c71a3c292e64b61fce76f1b` | Executable, zero diagnostics. | Executable, independently compared below. |
| [Bjorgum Double Tap](https://www.tradingview.com/script/rLkjr2sQ-Bjorgum-Double-Tap/), v5, 8.9K boosts | `f45df214dbb49c70fd848f2f9998a59a0172fdd1f32398771fcebeb652f866cf` | Not executable: 109 diagnostics, including comma-separated typed declarations, multiline conditional parsing, and collection mutation inside user functions, with downstream unresolved names. Count includes repeated and cascading diagnostics. | Executable, zero diagnostics on the unmodified published source; default-input native comparison below. |

The 14.7K-boost [Hull Suite Strategy](https://www.tradingview.com/script/Q9OQye4C-Hull-Suite-Strategy/) is Pine v4; it was outside this initial v5/v6 selection and was later qualified separately in the [v4 strategy audit](LEGACY_V4_STRATEGY_AUDIT_20260923.md). Boost figures are the UI snapshot on September 23, not a stable popularity metric.

A further complete v6 candidate is [Session VWAP Profile & Candle Delta by BigBeluga](https://www.tradingview.com/script/36ik5Jpc-Session-VWAP-Profile-Candle-Delta-BigBeluga/), with 1,140 boosts in the September 23 UI snapshot. Its 440-line published source is frozen as `bigbeluga-session-vwap-profile-v6.pine` (SHA-256 `12ca8cfaa734fb2f55ebe28784958df05fa8a988481685d309f1fcf11bc654a0`). The original interpreter returned 66 diagnostics, many caused by rejection of `line[]` and `box[]` UDT field types. The parser now accepts array shorthand in UDT fields; `array.new_float` and `array.new_int` accept series lengths and evaluate those lengths on each call; `array.clear` can mutate a shared array inside a v5/v6 user function. UDT parameter field mutation now writes through the caller's object identity, and a UDT array can retain nested `line[]` and `box[]` fields. These behaviors have focused historical and realtime rollback regressions. `calc_bars_count` now restricts the initial complete batch before creating bar indices, history or state, and later appended bars extend that execution window. The unmodified published script runs without diagnostics on the frozen chart data.

The published indicator was also inserted into the logged-in TradingView chart and exported on `BINANCE:BTCUSDT`, 1D, with its default inputs. The ignored `bigbeluga-session-vwap-profile-btcusdt-1d-native.csv` (SHA-256 `e950513ea88766253160178cdedd639c6112c8ffffac420f18a1d570771e8818`) contains 2,350 displayed daily rows and 40 `Upper Reversal` plus 31 `Lower Reversal` values in the latest 2,000 executable rows. Running the **unmodified** published source on all 2,350 exported OHLCV bars yields zero diagnostics; the runtime executes the final 2,000 bars and matches the native 40 upper and 31 lower signal positions and numeric prices exactly. The ignored `compare_bigbeluga_full.py` and `bigbeluga-full-comparison.json` freeze source/data hashes and every compared value. The export also includes columns from other chart indicators, which are not attributed to BigBeluga. The temporary BigBeluga indicator was removed from the chart after export. This is admission of this script's tested default signal path, not a drawing parity claim; the runtime produced 404 lines and 392 boxes, but the chart CSV does not expose their geometry.

An independent, unsaved Pine Editor control on the same chart plotted `timeframe.change("7M")`, `time("7M")`, and `bar_index`, with `calc_bars_count=2000`. Its UI export is frozen as `bigbeluga-7m-boundary-native.csv` (SHA-256 `3402445f3be154594cee1cdf4f8005a4182efa595eb3f7ee1eb6d9e81b1e2d78`). It shows exactly 2,000 script-executed rows, from April 3, 2021 through the forming September 23, 2026 bar, with `bar_index` 0–1,999. The 7-month period resets on each January 1 and August 1; the prior implementation's continuous seven-month buckets were wrong. The runtime now matches **all 2,000 values in each of the three native columns**, including `timeframe.change("7M")=false` on the first executed bar.

The earlier `bigbeluga-no-calc-probe.pine` and `compare_bigbeluga_probe.py` remain frozen as diagnostic isolation evidence; the full-source result above supersedes that probe for admission. Other inputs, symbols, chart timeframes, visual drawings, and forming tick updates still need separate comparisons. The initial batch must contain the complete chart dataset for `calc_bars_count` to select its latest bars; a streaming host must select that window before feeding one bar at a time. The runtime has no TradingView plan limit or Settings/Inputs override for this declaration. The temporary 7M control was removed from the chart after export. [TradingView's declaration documentation](https://www.tradingview.com/pine-script-docs/language/declaration-statements/) defines `calc_bars_count` as a limit on the historical bars available to a script.

The next complete v6 strategy candidate is [Dynamic Swing Anchored VWAP STRAT by PineIndicators](https://www.tradingview.com/script/gjFSGRCo-Dynamic-Swing-Anchored-VWAP-STRAT-Zeiierman-PineIndicators/), shown with 2,624 boosts in the September 23 Chrome UI. Its 138-line published source is frozen as `dynamic-swing-anchored-vwap-strategy-v6.pine` (SHA-256 `7b99a2b56ed5d44f0f0ecaa159cb0963855becdf350cdc50e1fe08e43e412497`). Initial analysis returned three `polyline.new(points)` type errors because a UDT field held a series-qualified `array<chart.point>`. The signature now accepts point arrays by kind; a focused semantic regression and the complete original source analyze with zero diagnostics. The TradingView chart reported `Error on bar 279: Invalid qty value (-5.76595)` and did not provide a valid strategy report for the default configuration. The UI observation is frozen in `dynamic-swing-native-error.txt`. The strategy was removed from the research chart after observation.

The initial local run had only the 2,350 chart-exported bars, beginning April 18, 2020. It continued after invalid entry quantities and accumulated 751 diagnostics. TradingView's [negative quantity guidance](https://www.tradingview.com/support/solutions/43000651235-i-see-cannot-create-an-order-with-negative-quantity-error/) states that a called entry with negative quantity stops strategy calculation. The runtime now raises a fatal error at the `strategy.entry`/`strategy.order` call for invalid positive-limit quantities; zero is a no-op and `na` uses declared default sizing. Focused regressions cover these paths. To align the history origin, `build_dynamic_swing_full_history.py` retrieves 976 older BTCUSDT 1D bars from the [official Binance Spot market-data endpoint](https://developers.binance.com/en/docs/products/spot/rest-api) and appends the 2,350 TradingView-exported bars, dropping the one overlapping April 18, 2020 bar after checking its OHLCV exactly. On this 3,325-bar input the unmodified source now stops at **bar 279**, as TradingView does; it computes −5.765962053306718 versus TradingView's displayed −5.76595 (absolute difference about 0.000012). `compare_dynamic_swing_failure.py` freezes the source/data hashes and this comparison. This verifies the failure mode and bar, **not native trade admission**: the older TradingView bars have not been exported independently, the displayed quantity is rounded, and no native trade list is available from a strategy that stops with this error.

A fifth public v5 candidate is [Nadaraya-Watson Envelope by LuxAlgo](https://www.tradingview.com/script/Iko0E2kL-Nadaraya-Watson-Envelope-LuxAlgo/),
34.5K boosts in the same UI snapshot. The complete 118-line source was frozen
as `luxalgo-nwe-v5.pine` (SHA-256
`655a85c123d2ab0766c58e3865876b8902d0f8ec54229c1b2e0dd75b6ccbaf40`).
It analyzes with zero diagnostics. On 3,324 confirmed
Binance daily bars, the default repainting mode completes with 1,000 line
objects, 10 labels, and one table. With the published repainting input set to
false, the same source completes and emits 2,327 nonblank positions in each
of its two bands. A full-history native comparison for the non-repainting
setting is reported below. Its default repainting mode revises historical
drawings at the last bar; ordinary plot comparison would not test that visual
behavior.

TradingView's [v5 script-structure documentation](https://www.tradingview.com/pine-script-docs/v5/language/script-structure/) allows comma-separated one-line statements. The candidate parser admits them in global and indented-block scopes for v5/v6. [V5 arrays documentation](https://www.tradingview.com/pine-script-docs/v5/language/arrays/) permits functions to change global array contents; the candidate admits `array.set`, `array.fill`, and `array.pop` and has runtime regressions for each. These changes brought the canonical Double Tap source from 109 to 59 diagnostics.

The remaining admission fixes followed the unmodified public source and a successful native TradingView compile: v5 conditional continuation at a smaller indentation than its function body; wrapped assignments following blank lines; named `table.cell(table_id=...)` and input-selected `table.new` positions; v5 numeric-to-bool typed declarations ([v6 migration guide](https://www.tradingview.com/pine-script-docs/migration-guides/to-pine-version-6/)); and drawing constructors used as user-function arguments. Drawing `copy` and mutating calls remain subject to argument side-effect checks. `bjorgum-double-tap-executable-analysis.json` reports executable with zero diagnostics. The earlier noncanonical indentation probe and staged diagnostic reports are retained only as investigation artifacts.

The default Double Tap creates a `table.new` on every bar. An earlier core limit of 50 cumulative table objects stopped execution after bar 50. TradingView documents a limit on [displayed table positions](https://www.tradingview.com/pine-script-docs/writing/limitations/), not 50 lifetime creations; the candidate removes that artificial check. The 3,324-bar run completed with 3,324 historical table objects in 9.6 seconds. Table storage and lookup still grow with the history, so longer histories need resource profiling and a bounded representation that preserves observable table semantics.

For SMC, the next current-state analysis exposed false restrictions on
`array.remove` and `array.slice` indexes. The published v5 source uses loop and
search results as indexes; TradingView's [array documentation](https://www.tradingview.com/pine-script-docs/v5/language/arrays/)
describes these operations and the public script compiles in TradingView. The
candidate accepts series integer indexes for both and global `array.remove`
inside user functions. Runtime regressions execute changing indexes across
three bars and check the removed elements and slice contents. The canonical
SMC diagnostic count went from 101 to 95 after `array.remove`, then to 79
after `array.slice`. Further fixes admit mutation of global UDT fields inside
user functions, the value of a final UDT field assignment, UDT versus `na`
return branches, and runtime-validated series drawing styles. TradingView's
[function](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/)
and [type-system](https://www.tradingview.com/pine-script-docs/language/type-system/)
documentation permits mutation of referenced global objects. The complete
published SMC source now analyzes with zero diagnostics
(`luxalgo-smc-executable-analysis.json`). Its historical run initially failed
where `for...in` removed array items while iterating; the loop now uses the
updated array size, as specified by [TradingView's loop documentation](https://www.tradingview.com/pine-script-docs/language/loops/).
The full 3,324-bar run completes, producing 238 labels, 238 lines, 5 boxes,
and 366 alert events (`luxalgo-smc-default-run.json`). These output counts
alone do not establish native drawing or alert parity.

## Independent native comparisons

The TradingView chart exported confirmed daily OHLCV plus displayed script
positions. We excluded the forming September 23 bar. The CLI ran the complete
source on those same bars; `abs <= 1e-8` or `rel <= 1e-10` was used for plot
values. Blank markers are compared as blank, not converted to zero.

| Script / symbol | Confirmed bars | Compared native positions | Mismatches | Output basis |
| --- | ---: | ---: | ---: | --- |
| Trendlines / `BINANCE:BTCUSDT` | 299, with 2,721 earlier warmup bars | 1,168 | 0 | Upper/Lower with `-14` display offset; two breakout shapes |
| 3Commas Bot / `BINANCE:BTCUSDT` | 3,324 from exchange listing | 19,944 | 0 | Six plot series; 41 native and runtime trades have the same entry/exit dates and direction |
| 3Commas Bot / `COINBASE:BTCUSD` | 4,280 from available history | 25,680 | 0 | Six plot series; all 8 trades match entry/exit dates and prices at `1e-8`; native CSV quantity and profit display precision limits further comparison |
| Double Tap / `BINANCE:BTCUSDT` | 3,324 from exchange listing | 6,648 | 0 | Two stop plots, both blank at default settings; seven native and runtime trade events have the same dates and directions |
| Double Tap / `BINANCE:BTCUSDT`, `Use Trail Stop=true` | 3,022 native confirmed bars aligned to the 3,324-bar runtime input | 6,044 stop-plot positions, including 810 nonblank | 0 | Three closed trades match entry/exit dates and prices; the fourth open entry date matches |
| SMC diagnostic copy / `BINANCE:BTCUSDT` | 1,379 confirmed bars with prior warmup to exchange listing | 11,032 | 0 | Eight appended bias/event probes; 56 nonzero event positions across six event series |
| Nadaraya-Watson, repainting off / `BINANCE:BTCUSDT` | 3,279 confirmed bars from the native chart's first loaded day | 13,116 | 0 | Two bands with 2,282 nonblank positions each; 29 crossunder and 32 crossover markers |
| MACD Pullback Sniper / `BINANCE:BTCUSDT` | 3,085 exported confirmed bars with 194 earlier warmup bars | 24,680 | 0 | Six numerical plots and two long/short markers; 28 native and runtime trades match direction, entry/exit dates, and quantity |
| 3Commas DCA Strategy Backtesting / `BINANCE:BTCUSDT` | 3,279 replayed bars; 3,089 native chart overlap | 3,089 RSI positions and 97 closed trades | 0 RSI or trade identity/date mismatches | Full published v6 source, default inputs; entry/exit prices agree within the native CSV display precision |
| ICT 10AM First FVG Daily Strategy / `BINANCE:BTCUSDT`, 1 minute | 23,182 confirmed bars | 11 closed trades | 0 direction/date/minute mismatches | Default-input v6 source; entry/exit prices and PnL differ only by floating point noise |
| Directional Kernel Filter / `BINANCE:BTCUSDT` | 3,324 confirmed daily bars | 23,268 numerical plot positions | 0 | Full published v6 source and default inputs; all exported OHLCV cells match runtime input |
| Adaptive Decycler Supertrend / `BINANCE:BTCUSDT` | 3,089 native confirmed bars with 235 earlier runtime warmup bars | 33,979 positions at default settings and 33,979 with RMS envelope enabled | 0 in either setting | Nine plot fields and two signal fields; 35 long and 34 short signals; the enabled envelope gives 6,178 additional nonblank band values |
| Volatility Reversion Scalper / `BINANCE:BTCUSDT` | 3,279 confirmed daily bars | 9,837 numerical plot positions; 3,279 signal positions; 25 closed trades | 0 plot, signal, and trade date/price/quantity/profit mismatches at `1e-8` | Complete published v6 source and default inputs; total profit 62,843.57 USDT in both engines |
| SSL Channel Pro / `BINANCE:BTCUSDT`, 1h | 2,309 confirmed bars with 1,861 earlier warmup bars | 16,163 positions across five plots and two marker series | 0 blank or numeric mismatches | Complete published v6 source and default inputs; three unresolved alert-template diagnostics remain explicit |
| SSL Channel Pro / `BINANCE:BTCUSDT`, 1h with 4h confirmation | 299 confirmed hourly bars with earlier warmup and native 4h provider bars | 2,093 positions across five plots and two marker series | 0 blank or numeric mismatches | Complete published v6 source; one BUY and two SELL markers; batch, incremental, and historical realtime results identical; see [4h receipt](SSL_PRO_NATIVE_PARITY_20260924.md) |
| VWAP Reversal Strategy V1 / `BINANCE:BTCUSDT`, 1m | 26,124 confirmed bars | 26,124 daily VWAP and 26,060 stable-history H1 VWAP positions | 0 within these ranges | Complete published v6 source and default inputs; 37 native and local closed trades, with the six latest visible rows matching identity, time, displayed price, and size; 59 leading and four trailing H1 positions remain outside the matched window. See [native receipt](VWAP_REVERSAL_NATIVE_PARITY_20260925.md). |

The first Nadaraya-Watson comparison used the earlier 3,324-bar input. It
found 45 initial bars where the runtime bands had values and the native bands
were blank. The native chart had loaded history only from October 1, 2017,
45 bars after that earlier input began. Expanding the chart's visible range
and exporting it yielded 3,279 confirmed bars through September 22, 2026;
the forming September 23 bar was excluded. Replaying exactly those exported
OHLCV bars with input call site 4 set to `false` removed the 45 warmup
differences. Every band and marker position matches the native export, with
maximum absolute numerical difference about `2.91e-11`. This comparison
uses the complete published script and verifies the non-repainting setting;
it does not qualify the default repainting drawings or realtime behavior.
The batch, incremental, and historical realtime JSON outputs are also equal
on the same 3,279 confirmed bars. The native comparison receipt is
`luxalgo-nwe-matched-history-comparison.json`.

A later default-repainting measurement on `COINBASE:BTCUSD` 1D uses a
local-only instrumented copy of the same public source. The measurement plots
at lookback offsets 50, 100, 250, and 498 match native TradingView values;
the original and instrumented local runs have exactly equal drawings and
original plots. Values near the forming bar vary between two native exports,
and native line coordinates remain unavailable. The evidence and limits are
in [the repainting measurement](LUXALGO_NWE_REPAINT_MEASUREMENT_20260925.md).

## Additional community candidates under qualification

[Volatility Reversion Scalper by sebpageau](https://www.tradingview.com/script/IMKs0OPw/)
is a complete public v6 strategy with 56 boosts in the September 23 listing.
The frozen source has SHA-256
`cf41e6fff22f67944c6e4de1f2692e1ed8219afce853f8af651bd11e40fb6d7f`
and analyzes with zero diagnostics. Its native chart CSV, native trade CSV,
and exact 3,279-bar confirmed OHLCV input are in ignored
`.local/community-coverage-20260923/`; the final forming bar is excluded.
The comparison receipt `volatility-reversion-btcusdt-1d-comparison.json`
records all 9,837 numerical plot positions matching within `1e-7`, all 3,279
signal positions matching, and all 25 entry and exit dates, prices,
quantities, and profits matching at `1e-8` against the native report.
Initially ten signal positions differed at bars 98–107 because the
interpreter required every sample in the percentile window to be non-`na`.
The native output accepts a full bar-count window containing earlier `na`
samples and computes linear interpolation from its finite members. That
linear-interpolation path now uses this rule; nearest-rank behavior remains
separate. The native signal mismatch count is zero. The affected
`with_na` golden fixture was updated. Direct native percentile values, beyond
the full-script signal comparison, remain untested. Batch, incremental, and
historical realtime executions produce identical full JSON on the matched
3,279-bar input.

This strategy exposed an opening fill error. Its 22nd trade exits on
2024-11-07 at 75,571.99 USDT in TradingView: a resting 72,840.6105 sell
limit is already marketable at that day's open. The prior day's close equals
that open, so the interpreter's historical scheduler previously skipped its
open observation and filled at the lower limit during the intrabar path.
The scheduler now observes every host open, including a price equal to the
previous close. A focused regression test and the 25-trade native rerun pass;
`cargo test -p pine-runtime --locked --quiet` and `cargo test -p pine-cli
--locked --quiet` pass. This change is local and uncommitted.

The initial 25-trade total-profit shortfall was 0.1025 USDT. A separate
native control on the same symbol plots `syminfo.mintick = 0.01` and places
off-grid limits on January 3, 2024. A long sell limit at 45,179.551 fills at
45,179.56; a short buy limit at 43,179.549 fills at 43,179.54. Both control
trades are intrabar limit fills, not favorable opening fills. Their exact
source and native trade CSVs are retained as `limit-tick-probe-{long,short}*`
in the ignored evidence directory. The broker now snaps pure exit limits to
the host-provided price grid by order side, preserving already aligned
prices. Both controls and the full community strategy match their native
trade prices and profits within floating noise; the strategy total is
62,843.57 USDT in both engines. Bracket legs and other price-order kinds
need their own native controls before extending this rule.

[VWAP Reversal Strategy V1](https://www.tradingview.com/script/EA1AXZui-VWAP-Reversal-Strategy-V1/)
is a complete public v6 strategy (162 boosts) frozen with SHA-256
`9f095054f0410ef0491d25f76ec1d1a061df74380b6a0f0a64f5df2b243f17c1`.
It now analyzes with zero diagnostics. The prior rejection was the bare
`ta.vwap` built-in variable in `request.security(syminfo.tickerid, "60",
ta.vwap)`, distinct from a call to `ta.vwap(...)`. TradingView's
[type-system guide](https://www.tradingview.com/pine-script-docs/language/type-system/)
identifies `ta.vwap` as a series variable; a provider-backed regression now
checks requested HLC3, volume accumulation, and daily reset against an
independent chart context. The unmodified strategy executes on 23,182 frozen
BTCUSDT 1m chart bars and 388 host-supplied hourly bars aggregated from them.
It emits two VWAP plots and 33 closed trades without runtime diagnostics.
The separate `compare_vwap_reversal_local.py` oracle reproduces both 23,182
position plot series exactly. It excludes the last, incomplete 60m bar from
historical `lookahead_off` values. Its hashes and comparisons are frozen in
`vwap-reversal-local-smoke-receipt.json` under the ignored evidence directory.
The source selects a USD strategy account on a USDT-priced chart; the local
run explicitly supplies USD chart-currency compatibility to exercise the
runtime, so its trades and accounting are not native parity evidence. The
hourly series is also locally aggregated, not a native TradingView H1 export.
At this September 23 stage, no native plot or trade comparison was claimed,
and the candidate remained outside the independently admitted table. The builtins, syntax, semantic,
runtime, and CLI package suites, formatting, and diff checks passed after the
variable admission. The later [September 25 native comparison](VWAP_REVERSAL_NATIVE_PARITY_20260925.md)
supersedes that initial unverified status for the stated stable-history plot
window and six visible trades. The incomplete higher-timeframe edges, exact
profit accounting, and realtime ticks remain unverified.

[EMA200 REGIME + BOS/CHoCH + 2x FVG Strategy](https://www.tradingview.com/script/x3kpQqJ5/)
is a complete 3,194-line v6 source (68 boosts) frozen with SHA-256
`054a24513560e7ae34640653da764fda02815fa80d6997fc8b6adf5ee935df69`
and analyzes with zero diagnostics. Its declaration enables bar magnifier,
so native fill qualification needs lower-timeframe input. A later
[default-input COINBASE:BTCUSD daily native comparison](EMA200_BOS_FVG_V6_NATIVE_TRADES_20260925.md)
matches all four exported closed trades by direction, date, entry and exit
price, size, and PnL. The local run explicitly reports standard-OHLC fallback
on every bar, so general bar magnifier behavior remains unqualified.

[Directional Kernel Filter by BackQuant](https://www.tradingview.com/script/5AnxLyjj-Directional-Kernel-Filter-BackQuant/)
is a complete public v6 indicator with 424 boosts in the September 23
open-source popularity listing. Its 123-line source was frozen as
`backquant-directional-kernel-v6.pine` (SHA-256
`0a554b52e2553ef0e4c9be253a3441fe67866680545216aa4d0b5461bdce82e0`).
The initial five diagnostics were four rejected `const` scalar declarations
and an input-selected `plotcandle(display=...)`. Pine's
[type system](https://www.tradingview.com/pine-script-docs/language/type-system/)
defines `const` scalar declarations; the published source compiles on
TradingView with the input-selected candle display. The candidate now
analyzes with zero diagnostics. It enforces constant initializer values for
scalars, allows mutation of `const` reference contents, and rejects
reassignment of `const` symbols. The `plotcandle` signature admits
input display selection.

The native `BINANCE:BTCUSDT` daily chart export was expanded to the exchange
listing and frozen as `backquant-directional-kernel-native-btcusdt-1d-chart-full.csv`
(SHA-256 `9d76a12e4bb05b0d2db18073ab86067e57e912185515f57d892cad25da690554`).
The forming final bar was excluded. All 3,324 confirmed OHLCV rows became the
runtime input (SHA-256
`4fb0519dbe1f5cb8514ffa0dbee84c65e11c38121a33db72224bf77edc2f218b`).
An earlier reused bar file had one different volume cell; the final run uses
the current native export instead. `compare_backquant_kernel.py` reports zero
OHLCV mismatches and zero mismatches across seven numerical plots and 23,268
positions, with maximum absolute numerical difference `2.91e-11`. Batch,
incremental, and historical realtime JSON outputs are identical on these
bars. Native CSV does not establish plot colors, candle visuals, or alert
delivery, and no such parity is claimed.
The post-fix `cargo test -p pine-syntax -p pine-sema -p pine-runtime -p
pine-cli -p pine-wasm --locked --quiet` suite passed.

[Adaptive Decycler Supertrend by SchizoQuant](https://www.tradingview.com/script/vEWWRSv8-Adaptive-Decycler-Supertrend-SchizoQuant/)
is a complete 105-line public v6 indicator with 684 boosts in the September
23 open-source listing. The published source, normalized only for HTML
nonbreaking spaces, is frozen as
`schizoquant-adaptive-decycler-supertrend-v6.pine` (SHA-256
`483ee6e5c85770d6537098d74291ac838d7860bed4da3d8bd4a03c2928c84459`)
and analyzes with zero diagnostics. On `BINANCE:BTCUSDT` 1D, the native
chart yielded 3,089 confirmed bars matching the runtime input's OHLC exactly,
plus one excluded forming bar. The runtime had 235 earlier bars for warmup.
At default inputs, all nine numeric plot fields and both signal fields match
at 33,979 same-bar positions. The two RMS envelope plots are blank by default.
With only `Show RMS Envelope=true` (input call site 9), all 33,979 positions
again match, including 3,089 nonblank values in each RMS band. There are
35 long and 34 short native signals in each export, all at matching bars.
The full JSON result is equal across batch, incremental, and historical
realtime execution for both settings. The immutable source/input/native
export/runtime hashes and field-by-field counts are in
`schizoquant-adaptive-decycler-comparison.json`, generated by
`compare_adaptive_decycler.py` from the ignored evidence directory. Chart
CSV does not verify plot colors, gradient fill, bar colors, or alert delivery.

[Machine Learning: Lorentzian Classification by jdehorty](https://www.tradingview.com/script/WhBzgfDu-Machine-Learning-Lorentzian-Classification/)
is a complete public v6 indicator with two v5 library imports. Its 664-line
source was frozen as `jdehorty-lorentzian-v6.pine` (SHA-256
`549658208ed1a753259cab0023f0f54ee34c7c4e1139ba4d91dd2be55b9520b8`).
The public [KernelFunctions](https://www.tradingview.com/script/e0Ek9x99-KernelFunctions/)
page currently exposes the exact imported `/2` version, which was also
frozen. The [MLExtensions](https://www.tradingview.com/script/ia5ozyMF-MLExtensions/)
page exposes `/3`, whereas the indicator imports `/2`. The frozen `/3` text
is a diagnostic proxy, **not** a verified copy of `/2`.

Using those two frozen library files, the complete indicator now passes
static analysis with `executable=true` and zero diagnostics. Exported
functions may fill their own newly allocated arrays inside a stable `for`
loop; ownership tests still reject parameter arrays, aliases, and rebinding.
The library checker also accepts `table.*` drawing calls in exports, while
still rejecting nested `plot()` calls. These are consistent with the
[official library constraints](https://www.tradingview.com/pine-script-docs/concepts/libraries/)
and the interpreter's existing table runtime. A 300-bar BTCUSDT 1h smoke run
with the `/3` proxy produced two plots, four plot-shape series, 300 labels,
one table, and 36 alerts with no runtime diagnostics. Batch and incremental
JSON outputs have the same SHA-256
`5ca1e55a9bce77002a31112b5c1af256e014ee4f013ab74c35309d8aaaec9ae`.
The ignored `.local/community-coverage-20260923/lorentzian-proxy-smoke*.json`
files retain this local evidence. At this smoke-test stage, the actual
imported `MLExtensions/2` source and TradingView chart output were unverified,
so it did not establish exact-library execution or native plot parity.

A subsequent native `BINANCE:BTCUSDT` 1h export verified the kernel estimate
on all 2,349 closed chart bars after adding 200 preceding bars for warmup.
The final forming bar differed by about `0.000912` USDT; Buy/Sell still had
11 signal-cell mismatches in the 2,350-bar export. A later 1,000-hour prefix
experiment reduced this to four, while a 1,500-hour prefix increased it to
13; the native hidden history and exact `/2` library source remain unverified.
Full native parity is still open. See
[Lorentzian native audit](LORENTZIAN_NATIVE_AUDIT_20260924.md).

[TrendLock: Multi-Timeframe Supertrend Donchian Breakout by blitz_locked](https://www.tradingview.com/script/FDeprmix-TrendLock-Multi-Timeframe-Supertrend-Donchian-Breakout/)
is another complete public v6 strategy, with 132 boosts in the September 23
listing. Its 119-line source was frozen as `trendlock-mtf-v6.pine` (SHA-256
`f0f889d7e6f8282527dfc4ff0e256dbc64300e6d3c291c880b55fbdb603195ec`).
The source places `import TradingView/ta/14` before `//@version=6` and omits
an explicit alias. Both are valid per TradingView's [script structure](https://www.tradingview.com/pine-script-docs/language/script-structure/)
and [library import](https://www.tradingview.com/pine-script-docs/concepts/libraries/)
documentation. The parser now discovers the version annotation wherever it
appears and derives the library name as the alias when `as` is omitted.
Focused syntax and semantic tests cover both behaviors.
The cross-component `cargo test -p pine-syntax -p pine-sema -p pine-runtime
-p pine-cli -p pine-wasm --locked --quiet` suite passed after these changes;
`git diff --check` also passed.

The exact public [TradingView `ta` v14 library](https://www.tradingview.com/script/BICzyhq0-ta/)
was frozen as `tradingview-ta-v14.pine` (SHA-256
`8f8c503618bf213e5b0378f0559a4fe1b6e1a94af4a6f5810b7675dfda88d07c`).
The public [RelativeValue page](https://www.tradingview.com/script/cZnSLls2-RelativeValue/)
currently displays v4, but the unmodified v3 source was already retained as
`.local/delivery-20260909/RelativeValue-v3.pine` with SHA-256
`11850708783161673a00173f222a8aa823bf2aec2788c0af98c41bdd8afe4a0f`;
the same hash is recorded in the earlier TechnicalRating/3 library-chain
reference manifest. With that exact v3 source and `ta` v14 supplied, the
complete TrendLock source analyzes with `executable=true` and zero
diagnostics. A local `BINANCE:BTCUSDT` 1h smoke test over 2,985 bars, with
747 synthetic UTC 4h provider buckets, produced 3 plots, 2 shape series,
91 orders, and 49 closed trades with no runtime diagnostics. Batch and
incremental output JSON hashes match at
`9b82605541aa2e8d08c8584627b8b5e0190ccb276fa39be0c432f6440a21e411`.
The script and provider evidence are retained in the ignored
`.local/community-coverage-20260923/trendlock-*` files. The TradingView
publication's preview is `COINBASE:ETHUSD` 1D and its report shows 106
trades (59 profitable), so it cannot be compared to this BTCUSDT 1h smoke
run. The subsequent same-symbol, extended-history comparison is documented
in [TrendLock native parity](TRENDLOCK_NATIVE_PARITY_20260924.md).

A same-symbol native comparison was then captured by inserting the published
script into the `BINANCE:BTCUSDT` 1h chart. TradingView exported chart CSV
`I:\sys\下载\BINANCE_BTCUSDT, 60 (1).csv` (SHA-256
`49bd5fdc49d75a87c4d52f8c8113e355e5e4af468d7a46a57b7b46c4ad06206d`)
and strategy trades CSV `I:\sys\下载\STDB_BINANCE_BTCUSDT_2026-09-24.csv`
(SHA-256 `144028ea4c93b13ac17796de6cbe01ad650e572ac5340079e27c43c826051280`).
Across 1,109 overlapping hourly bars, `Supertrend`, `Donchian Upper`,
`Donchian Lower`, `Long Entry`, and `Short Entry` all match at every bar.
The first 1,108 OHLC bars match too; the last local bar had a different close
because its prior export captured that hour while it was still forming.
The comparison receipt is
`.local/community-coverage-20260923/trendlock-native-local-plot-comparison.json`.

The initial April-to-August local history produced partial trade agreement
because it omitted the earlier position and equity path. Extending the local
BTCUSDT hourly input to January 2024 with SHA-256-verified Binance monthly
archives, and fixing previous-pass position history plus same-bar close exit
fills, yielded **420/420 native closed trades in the local data window with
matching entry and exit times and prices**. Supplying the BTCUSDT quantity
precision of five decimal places and reserving the percentage entry commission
now also matches all 420 entry quantities exactly. The full native report has
452 trades through September 2026; the local history ends in August. All 420
net PnLs and calculated commissions match the native export to its displayed
two-decimal precision; exact internal-ledger equality is not established. The comparison receipt is
`.local/community-coverage-20260923/trendlock-long-history-trade-comparison.json`.
The temporary script insertion was undone and the original TradingView layout
saved.

[Modern Ichimoku Cloud by GBB](https://www.tradingview.com/script/jJAqvJP5-Modern-Ichimoku-Cloud-GBB/)
is a complete public v6 indicator from the open-source popularity listing
(158 boosts in the September 23 UI). Its 410-line source was frozen as
`gbb-modern-ichimoku-v6.pine` (SHA-256
`25c5f71e33cee55f7f2c77416bbd54c31b108d8d26370dd1b4dde14d65cd4373`).
Initial analysis found 18 diagnostics: four false rejections of series
transparency in `color.new`, ten repeated false rejections of `array.shift`
on global reference arrays inside a user function, and four unsupported
`{{plot("Grade")}}` alert placeholders. TradingView documents
[series transparency](https://www.tradingview.com/pine-script-docs/faq/visuals/)
and [global collection mutation in functions](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/).
The candidate now analyzes with zero diagnostics. The `Grade` value is
published as a `display.data_window` plot at source line 288 and the four
alert messages refer to that exact title. A focused runtime regression now
checks same-bar named-plot substitution. A
four-bar runtime regression checks dynamic transparency values of 0, 25, 50,
and 75; another checks the global array queue after shifts on bars 3 and 4.
The complete source now executes across 3,324 frozen BTCUSDT 1D bars with a
host-supplied 4D series generated by UTC four-day aggregation of those daily
bars. It emits 16 plots and 265 alert events with zero runtime diagnostics;
all 36 alerts containing `grade` match the `Grade` plot at the alert bar.
The hashes, counts, and comparison outcome are frozen in
`.local/community-coverage-20260923/gbb-modern-ichimoku-local-smoke-receipt.json`.
The 4D aggregation is a local smoke-test fixture, not a TradingView export;
at this stage native plot, alert, and drawing parity were unverified. After the named-plot
change, the builtins, syntax, semantic, runtime, and CLI test suites passed;
the missing-title failure path passed a separate focused runtime test.

A subsequent native `BINANCE:BTCUSDT` 1D chart export was compared against
the full-source local run. All 24 exported study columns (16 plots and eight
shape series) match on the comparable rows of 2,281 overlapping daily bars,
after applying the three Ichimoku display offsets. Two Chikou cells require
local data beyond its end and were excluded. Batch, incremental, and
historical realtime outputs have identical JSON hashes. The chart CSV does
not expose alerts or drawing objects; those remain unverified. See
[Modern Ichimoku native parity](MODERN_ICHIMOKU_NATIVE_PARITY_20260924.md).

[SSL Channel Pro by TradingFinder](https://www.tradingview.com/script/PERIWBXb-SSL-Channel-Pro-TradingFinder-Semaphore-Signal-Level-Indicator/)
is a complete public v6 indicator with 283 boosts in the same listing. Its
175-line source was frozen as `tradingfinder-ssl-channel-pro-v6.pine`
(SHA-256
`ff30bea30d8ce5ea52fbff44f9596be860f899b28a54cb0dd80656dd5e33b0c6`).
Analysis now reports zero diagnostics. The `request.security` expression
reaches a stateful user function through a `switch` based moving-average
selector; the request analyzer now checks every selector, arm, and arm-local
block, and a separate provider-backed regression verifies the selected MA and
state run on requested bars. The complete published source executes on 23,182
frozen BTCUSDT 1m chart bars with a host-supplied 60m series aggregated from
those same bars. The default run emits five plots, two shape series with 164
BUY and 162 SELL markers, and 326 labels. Setting the published `Enable HTF
Confirmation` input (callsite 4) to true yields 129 BUY and 121 SELL markers
and 250 labels; the five plot value series remain identical. The six alert
templates refer to three plot titles the source never defines. The runtime
therefore suppresses those alert events and reports three distinct
`E_UNSUPPORTED_ALERT_PLACEHOLDER` diagnostics while preserving indicator
execution. Hashes, input selection, and counts are in
`.local/community-coverage-20260923/tradingfinder-ssl-local-smoke-receipt.json`.
The 60m series is a local aggregation, and no native TradingView SSL output
had been exported for that 1m chart and input pair. A later independent
[1h default-input comparison](SSL_PRO_NATIVE_PARITY_20260924.md) matches all
16,163 exported plot and marker positions on 2,309 confirmed hours. Native
label text and alert firing remain unverified. The builtins, syntax, semantic,
runtime, and CLI package suites passed after the request and alert changes;
an additional focused test confirms an unsupported call in a different
`switch` arm is still rejected.

[MACD Pullback Sniper by blitz_locked](https://www.tradingview.com/script/V87nRt9v-MACD-Pullback-Sniper-Trend-ADX-Filtered-Strategy/)
is a complete public v6 strategy selected from TradingView's open-source
strategy popularity listing on September 23. Its 119-line source was frozen
as `blitz-macd-pullback-v6.pine` (SHA-256
`bdfa426b7a52f3a5c60ba604250def2c8d5ecf89e27f2e38d37d3415bfbd8f4c`).
It analyzes with zero diagnostics. The publication's native preview showed
170 trades, 75 profitable trades, and +5,098.79 USD total PnL when inspected;
these are native UI observations for its published preview chart, separate
from the `BINANCE:BTCUSDT` comparison below.

The complete default-input strategy was then added to the user's
`BINANCE:BTCUSDT` 1D chart. TradingView's native chart and trade CSV files
were frozen as `blitz-macd-native-btcusdt-1d-chart.csv` (SHA-256
`1c60beac8de1afeb4a26d02705ef18d920784f319aa49c1ef60f026f24e2712d`)
and `blitz-macd-native-btcusdt-1d-trades.csv` (SHA-256
`0b167cf8c44837b184e84335eba96a3c0e86a483a20f8fec5247b9ef6a52b4be`).
The native chart exported 3,085 confirmed bars and one forming bar; the
runtime replayed the already frozen 3,279-bar chart history from October 1,
2017 through September 22, 2026. All 3,085 overlapping confirmed OHLC bars
match. The native export's 3,085 positions each for Histogram, MACD, Signal,
Trend EMA, Stop, Target, Long, and Short match the runtime: 24,680/24,680
positions, including blanks and signal marks. Maximum numerical difference
among these series is below `5e-10`.

The native and runtime strategy each have 28 closed trades. All 28 directions,
entry dates, exit dates, and quantities match. Entry prices differ by at most
floating point noise; exit prices by at most `0.009` USDT against the native
CSV's two-decimal presentation; net PnL by at most `0.0051` USDT. The native
cumulative PnL is `1,229.26` USDT and runtime net profit is
`1,229.2595478495405` USDT. The independent receipts are
`blitz-macd-btcusdt-1d-chart-comparison.json` and
`blitz-macd-btcusdt-1d-comparison.json`.
Batch, incremental, and historical realtime JSON outputs are also equal on
the same 3,279 confirmed bars.

Before the broker fix, nine price exits had differences of about `0.02`
USDT. TradingView's [strategy declaration documentation](https://www.tradingview.com/pine-script-docs/language/declaration-statements/)
specifies slippage for market and stop orders, while [limit orders](https://www.tradingview.com/pine-script-docs/concepts/strategies/)
fill at their limit or a better price. The core had added slippage to
`strategy.exit` limit fills. It now distinguishes limit from stop and trailing
exits, including bracket exits, with a focused runtime regression and updated
snapshot. This native comparison verifies one chart, timeframe, and default
settings; other symbols, intrabar magnifier behavior, and realtime remain
unverified.

The local candidate passed the `pine-runtime` test suite, all 234 `pine-cli`
binary tests, and all 708 `pine-wasm` tests after refreshing the documentation
and fixtures for already admitted array indexes, table lifetimes, global UDT
field mutation, and dynamic `for...in` collection size. The previously
asserted lifetime limit of 50 table objects was removed from conformance
metadata; a runtime test now constructs 102 tables across two bars.

[3Commas DCA Strategy Backtesting by The Quant Science](https://www.tradingview.com/script/z15GLfMe-3Commas-DCA-Strategy-Backtesting-The-Quant-Science/)
is another complete public v6 strategy. Its 230-line source was frozen as
`quant-science-3commas-dca.pine` (SHA-256
`d33abe80c4d1147ab108fc5b105e1ee18a18d06a847eec99a6d463a0c291a3f6`).
It uses an explicit USDT account currency, limit base and averaging entries,
10 pyramiding levels, commission, slippage, and `process_orders_on_close`.
The unmodified source now analyzes with zero diagnostics. The host supplies
the chart quote currency (`--chart-currency USDT` in the CLI); a different
account currency fails before execution because no foreign currency rate is
available. The runtime also returns host-provided `syminfo.currency` and
`strategy.account_currency` values.

The default strategy was added to the `BINANCE:BTCUSDT` 1D chart. Native
`quant-science-dca-native-btcusdt-1d-trades.csv` and runtime
`quant-science-dca-runtime-btcusdt-1d.json` were retained locally. Both have
97 closed trades over the 3,279 confirmed bars from October 1, 2017 to
September 22, 2026. Every trade matches its entry signal, entry date, and
exit date. Maximum entry-price difference is below `0.01` USDT (native CSV
shows two decimal places); maximum exit-price difference is floating point
noise. Maximum displayed-quantity difference is below `0.00001` BTC, and
maximum per-trade PnL difference is `0.162` USDT. These quantity and PnL
comparisons are limited by the CSV's display precision.
The chart export `quant-science-dca-native-btcusdt-1d-chart.csv` (SHA-256
`1eff611a66baee2b9b3bc290345cce6433217a5c2ea3491b34b334beb8b17edc`)
contains 3,089 confirmed bars overlapping the replay, plus one forming bar.
All 12,356 overlapping OHLC values and all 3,089 RSI positions agree exactly.
The durable comparison receipt is `quant-science-dca-btcusdt-1d-comparison.json`.

The first comparison exposed a one-bar delay for base limit orders created
at a bar close. The broker now evaluates same-bar limit orders at the close
when `process_orders_on_close` is enabled. It also closes every ledger leg
in a `strategy.close_all` call; subtracting floating quantities previously
left a residual open leg and suppressed later DCA cycles. Price-based entry
slippage now applies to stop orders and excludes limit orders, including
stop-limit fills and reversals. Focused tests cover these cases and the
currency mismatch contract. This comparison qualifies the stated chart and
default inputs; it does not qualify other symbols, non-unit contracts,
intrabar magnifier behavior, or realtime execution.

[ICT 10AM First FVG Daily Strategy by mehmettopbas_](https://www.tradingview.com/script/Za5HKgrL-ICT-10AM-First-FVG-Daily-Strategy/)
is a complete public v6 strategy with 389 boosts in the September 23 listing.
The 315-line source was frozen as `ict-first-fvg-v6.pine` (SHA-256
`92aa47c09ddbe6c088efa4c6c8d0a92ae42bcf6747dc9d0aedeb5027d742ffbe`).
It analyzes with zero diagnostics. It exercises `America/New_York` calendar
functions, first-gap state, limit entries with protective bracket exits,
`calc_on_order_fills`, same-bar entry/exit, `varip`, line and box lifetimes,
and a table. Its published preview uses MNQ futures; the independent
comparison uses `BINANCE:BTCUSDT` at one minute to stay inside the runtime's
unit-point-value account profile.

TradingView exported `ict-first-fvg-native-btcusdt-1m-chart-full.csv`
(SHA-256 `56e34cce1c6757018a4ceb55b0e9949d3f8fdff358758e3accafccd2475594769`)
and `ict-first-fvg-native-btcusdt-1m-trades.csv`
(SHA-256 `5522395d4f1dfcca045c07d87c0996049a4f5690e4be14e71ffe518ce0166d62`).
The runtime replayed the chart's 23,182 confirmed bars, excluding the final
forming bar. All 92,728 OHLC positions in the frozen CSV agree with the
replay input. The native report and runtime each contain 11 closed trades.
Every entry direction and entry/exit minute matches. Maximum entry/exit
price difference is below `1.5e-11` USDT, and maximum PnL difference is
below `1e-11` USDT. The independent comparison script and receipt are
`compare_ict_first_fvg.py` and `ict-first-fvg-btcusdt-1m-comparison.json`.
Batch, incremental, and historical realtime executions produce identical
complete JSON results on these bars.
The native chart does not export line, box, or table objects, so their visual
agreement is not established by this receipt. The original MNQ contract
multiplier and bar magnifier behavior are also outside this comparison.

The published SMC indicator has no numerical plots, so its standard chart
export cannot expose the internal state. For a separate local oracle, we
appended eight transparent plots to the otherwise complete public source:
internal and swing bias, internal and swing bullish/bearish BOS, and equal
high/low event flags. This derivative is not the published script and was not
published; its SHA-256 is
`ee0d461eb9aa768c112a4aa626767f9f78290bb493bf028e7caf11ac15d4f5ad`.
TradingView compiled and executed the private copy and exported the 1D chart.
The runtime used all 3,324 confirmed daily bars as warmup and matched 1,379
exported, time-aligned confirmed bars at every one of the 11,032 probe
positions. The native export includes one later forming bar, which was
excluded. Each bias probe takes both `-1` and `1`; the six event probes
contain respectively 25, 11, 4, 1, 9, and 6 positive bars. This establishes
the probed internal-state/event agreement for default inputs on this data,
not pixel-level drawing, alert delivery, other inputs, or realtime parity.
The comparison receipt is `luxalgo-smc-oracle-comparison.json`.

The separate nondefault `Confluence Filter=true` run is recorded in
`SMC_CONFLUENCE_NATIVE_PARITY_20260925.md`. On 3,282 native daily OHLCV bars,
all eight instrumented plots match the TradingView export at every position.
The setting changes 26 bullish and 7 bearish internal BOS events compared
with the local default-input run. This extends the SMC evidence to one
meaningful alternate input, while drawings and live alerts remain untested.

The separate [FVG and empty-timeframe receipt](SMC_FVG_EMPTY_TIMEFRAME_NATIVE_PARITY_20260925.md)
records a runtime fix for `request.security(..., input.timeframe(""), ...)` on
a nondefault 1D chart. With FVG enabled, ten instrumented plots match all
3,281 confirmed native bars, including 112 bullish and 79 bearish FVG events.
The native test uses a private diagnostic copy with only the FVG input default
changed and two FVG event plots appended; it does not qualify FVG box geometry.

The [weekly FVG receipt](SMC_WEEKLY_FVG_NATIVE_PARITY_20260925.md) extends this
to a `1W` request on the 1D chart. With a host weekly stream aggregated from
the exact native daily export window, all ten SMC probes and nine additional
request/threshold diagnostics match 3,281 confirmed bars. An initial single
FVG mismatch used a full earlier week where the native study saw only one
day of its first week; the receipt preserves both inputs and that boundary.

Double Tap's plot equality is weak evidence because all 6,648 selected plot
positions are blank. The native strategy export has three closed short trades
and one open long entry; all seven entry/exit event dates and directions match
the runtime. Maximum absolute fill-price difference is `0.015 USDT`; maximum
quantity difference is `0.00000646` chart units against rounded native CSV
values. The current evidence does not establish exact fill, PnL, table, line,
label, or nondefault-setting parity. The USD account on a USDT chart also
requires TradingView's historical currency conversion for full accounting
parity. Local comparison receipt:
`bjorgum-double-tap-trade-comparison.json`.
The full JSON result is equal across batch, incremental, and historical
realtime execution on the 3,324 confirmed bars.

The nondefault `Use Trail Stop=true` Double Tap run supplies stronger plot and
broker evidence. The public source is unchanged; only input call site 26 is
overridden. Its native chart export contains 3,022 confirmed daily bars plus
one later forming bar, with OHLC matching the corresponding runtime input.
Both stop plots match at every confirmed position: 808 nonblank `Short Stop`
values, two nonblank `Long Stop` values, and all blank positions. The first
candidate had the same stop plots but only one closed trade, compared with
three in TradingView. The source passes `limit=na` alongside a finite `stop`
to `strategy.exit`; the runtime had rejected the entire bracket as a
nonfinite price. It now treats the `na` leg as absent and places the finite
stop or limit leg. A regression covers both sides and the both-`na` case.
The repaired candidate has three closed trades with native-matching entry and
exit dates and prices at `1e-8`, one matching open entry date, and no broker
diagnostics. Native CSV quantities have limited display precision (maximum
absolute difference `0.00000646`); net PnL differs by at most `0.079 USD`.
The strategy's USD account on a USDT chart still lacks a native historical
conversion feed in this host-neutral runtime, so exact account/PnL parity is
not claimed. The repaired 3,324-bar result is JSON-equivalent across batch,
incremental, and historical realtime execution; the previously qualified
Coinbase 3Commas Bot result is unchanged by this fix. The frozen native
chart/trade exports, candidate and repaired
runtime JSON, `compare_double_tap_atr.py`, and
`bjorgum-double-tap-atr-stop-comparison.json` are in the ignored local
evidence directory.

The Binance export's 3,020-bar overlap with the previous daily input matches
OHLCV exactly. Downloaded input and trade-export SHA-256 values:

| Local file under `.local/community-coverage-20260923/` | SHA-256 |
| --- | --- |
| `luxalgo-trendlines-native-daily.csv` | `84db497993b341bf3f930e986b1632f15fa3fd70d7afdacfb7db09c2bc33d1bd` |
| `bjorgum-3commas-native-full-daily.csv` | `e86fd46bc4987e7686c1844f52bf9e32e95ceef497e07c7df90a75c090a488db` |
| `bjorgum-3commas-native-trades.csv` | `27978999b07455f50dd2c47a359c30035649d675d77415f08ee0ecbff687dd2c` |
| `bjorgum-3commas-btcusd-native-daily.csv` | `b567415d1b244800015d851af4f9beba6dd5857b7497d6079fd5b36120351cb2` |
| `bjorgum-3commas-btcusd-trades.csv` | `327691130186b47c01c43a74ec6082af009423659785f935c26dd3f623baf113` |
| `full-daily-bars.csv` | `9bec5ba5960c1eb8f099d30820d728cb083e1b3a9e0305a192d85e854cc5c660` |
| `btcusd-bars.csv` | `23e2f25d5a93af010443c41c60fd32e3feac3b37d983d9c857233c6992916e08` |
| `bjorgum-double-tap-native-full-daily.csv` | `9e3506ac4fce9b4748ecebe913ee0a1cdb4bad2418bd8263787a50c1e6a5ae58` |
| `bjorgum-double-tap-native-trades.csv` | `778ced8944b619426b1ea4492438450503be8c361a1a962821d724d9bfe64286` |
| `luxalgo-smc-oracle-native-daily.csv` | `564ee6144f98001a9c92b6b1c547b90807483c1ada309c74a890f3f643cf000a` |
| `double-tap-daily-bars.csv` | `aea068200dfd7066599e15911c8ad150447fe0259aba33f21836284ea6bca645` |
| `luxalgo-nwe-native-nonrepaint-full-daily.csv` | `5ea08056fb0b57a3aee69cef114d7131c8114efe83277d973065377683735a81` |
| `blitz-macd-native-btcusdt-1d-chart.csv` | `1c60beac8de1afeb4a26d02705ef18d920784f319aa49c1ef60f026f24e2712d` |
| `blitz-macd-native-btcusdt-1d-trades.csv` | `0b167cf8c44837b184e84335eba96a3c0e86a483a20f8fec5247b9ef6a52b4be` |
| `luxalgo-nwe-native-confirmed-bars.csv` | `d0137fb2decd64f2b8826352248658050b63107d69e04c91a90fecc40c02484c` |

An initial 299-bar Binance export without older history omitted early trades.
The full daily export repaired that evidence defect; it is not a runtime fix.

## Cash market order sizing and bracket price grid

The USD-quoted Coinbase chart isolates broker semantics from currency exchange
rates. On the baseline CLI, 3Commas Bot's eight trade dates and entry prices
matched native results, but the largest quantity difference was `0.00202881`.
Native quantities imply sizing from the signal bar's close plus one tick for a
long market entry, or minus one tick for a short entry. The runtime used the
unadjusted close. The candidate now includes the configured slippage in the
estimated fill price when sizing a default `strategy.cash` market order from
`strategy.entry` or `strategy.order`. Explicit quantities and nonmarket orders
are untouched. On the rebuilt local CLI (SHA-256
`cc9775546cb8aa14cf35625cbcb5f200983b69a31ad5a2726d538c08f0fe9191`),
all eight quantities agree with the native report to its displayed precision:
maximum absolute error `0.0000009572`. Entry/exit dates still all agree, and
the 25,680 plot comparisons remain at zero mismatches. Full JSON results are
identical across batch, incremental, and historical realtime execution on the
4,280-bar USD input.

The exported Coinbase report rounds quantities and profit. A later native
`syminfo.mintick` chart export confirmed `0.01 USD`; the earlier `0.01432 USD`
maximum exit-price difference came from off-grid bracket prices. Separate
Binance BTCUSDT controls covered a long and short bracket limit fill and a
long and short bracket stop fill, each on 0.01 quote ticks. The broker now
rounds limit legs toward the favorable side and stop legs toward the adverse
side. It also uses a matching pending entry's direction when the bracket is
placed before its entry fills. On the same 4,280 Coinbase bars, all eight
entry and exit dates and prices match the native CSV at `1e-8`, and all
25,680 plot values are unchanged. Maximum quantity difference is
`0.0000009572` and maximum net-profit difference is `0.004431 USD`, below
their displayed precision. The independent receipt is
`bjorgum-3commas-btcusd-bracket-grid-comparison.json`; native control source
and trade CSVs are frozen as `bracket-{tick,stop-tick}-probe-{long,short}*`
in the ignored evidence directory. Exact unrounded TradingView quantity and
profit remain unobservable from the CSV, so full internal accounting parity
is not claimed. The complete 4,280-bar result is JSON-equivalent across
batch, incremental, and historical realtime execution after this fix.

On `BINANCE:BTCUSDT`, the script explicitly selects a USD account while the
chart is quoted in USDT. TradingView applies a historical currency rate to
cash sizing and accounting; the current interpreter's documented scope is
same-currency accounting and it has no external exchange-rate feed. The 41
matching trade dates and 19,944 matching plot values do not establish
cross-currency quantity/PnL parity. This boundary is documented in
[TradingView's strategy currency behavior](https://www.tradingview.com/pine-script-docs/concepts/strategies/#currency).

## Reproduction and next admission targets

The separate [UT Bot Alerts v4 native comparison](UT_BOT_ALERTS_V4_NATIVE_PARITY_20260924.md)
admits QuantNomad's complete 42-line public script with zero diagnostics.
Its default 128 Buy and 127 Sell markers match TradingView at every bar in a
2,350-hour `BINANCE:BTCUSDT` 1h export. With its Heikin Ashi option enabled,
all 92 Buy and 91 Sell markers match across a separate 2,351-hour export using
host-supplied Heikin Ashi bars. Batch, incremental, and historical realtime
JSON outputs are identical on both paths. Alert delivery and bar colors remain
outside these native comparisons.

The [Supertrend ATR with Trailing Stop Loss v4 audit](LEGACY_V4_SUPERTREND_ATR_AUDIT_20260924.md)
freezes a second popular v4 source and native daily trades. The complete
duplicate-`transp` normalized copy now runs with zero diagnostics on
`COINBASE:BTCUSD`: with host quantity precision six, 127 closed trades match
native entry/exit dates, prices, and displayed quantities exactly. The open
128th entry quantity also matches. Closed cumulative PnL is `8,575.373808`
USD locally versus `8,575.37` displayed natively; 126 of 127 individual
net PnLs agree within half a cent, and one differs by `0.00500176` USD. The
original published source remains distinct; the current TradingView editor
rejects its duplicate named arguments. Exact internal-ledger parity remains
unproven. A separate [v6 short market order audit](PERCENT_EQUITY_SHORT_ORDER_NATIVE_AUDIT_20260925.md)
checks the same-bar percentage-of-equity sizing rule against a visible native
strategy report, with v1–v6 repository regressions.

The ignored local directory contains source files, original UI exports, CLI
JSON, diagnostic reports, `build_full_daily.py`, `build_double_tap_daily.py`,
`compare_trendlines.py`, `compare_3commas.py`, `compare_3commas_trades.py`,
`compare_btcusd.py`, `compare_btcusd_trades.py`, `compare_double_tap.py`,
`compare_double_tap_trades.py`, `build_smc_oracle.py`,
`compare_smc_oracle.py`, `build_nwe_bars.py`, and `compare_nwe_matched.py`.
The local commands were `cargo build -p pine-cli
--locked`, then `target/debug/pine-compat run <source> --bars <bars.csv>
--chart-symbol <symbol> --chart-timeframe 1D`. The separate comparison scripts
freeze plot offsets, denominator selection, and first mismatches. `cargo test
-p pine-runtime --lib strategy --locked --quiet` passed 849 strategy tests,
including the new cash/slippage regression test. `cargo fmt --all -- --check`
and `git diff --check` passed.

After the BigBeluga parser, series array length, and UDF array clear changes,
`cargo test -p pine-builtins -p pine-syntax -p pine-sema -p pine-runtime
--locked --quiet` passed all suites, including 1,859 runtime library tests,
1,243 semantic library tests, and 2,247 semantic fixture tests. The full
test log is retained in the ignored evidence directory. `cargo fmt --all
-- --check` and `git diff --check` passed.

After UDT identity, nested drawing arrays, 7M calendar anchoring, and
`calc_bars_count`, `cargo test -p pine-builtins -p pine-syntax -p pine-sema
-p pine-runtime -p pine-cli --locked --quiet` passed all suites, including
1,860 runtime library tests, 1,243 semantic library tests, 2,247 semantic
fixtures, and 239 CLI library tests. The full test log is retained as
`bigbeluga-full-tests.log`. `cargo fmt --all -- --check` and
`git diff --check` passed. The CLI golden timeframe snapshot now reflects
the native first-bar `timeframe.change` value.
After the series chart-point array signature change, the same five-package
test command passed again, including the focused `polyline_series_points`
regression. Its log is `dynamic-swing-full-tests.log`; formatting and diff
checks passed.
After the strategy quantity execution fix, the same five-package command
passed again, including `strategy_invalid_qty` and the migrated negative
semantic fixtures. Its log is `dynamic-swing-qty-tests.log`; formatting and
diff checks passed.

For the Double Tap candidate, `cargo test -p pine-syntax -p pine-sema -p
pine-runtime --locked --quiet` passed all package tests, including 1,845
runtime library tests, 1,236 semantic library tests, and 2,247 fixture tests.
The subsequent SMC array-index candidate passed `cargo test -p pine-builtins
-p pine-syntax -p pine-sema -p pine-runtime --locked --quiet`, including 1,847
runtime library tests, 1,236 semantic library tests, and 2,247 fixture tests.
It updates the conformance descriptions and former simple-index fixture
expectations.
After the SMC UDT, style, and loop fixes, the same full package command passed
again, including 1,855 runtime library tests, 1,236 semantic library tests,
and 2,247 semantic fixture tests. The realtime array-loop fixtures now use
finite growth, which exercises the documented updated-size behavior without
creating an unbounded loop. `cargo fmt --all -- --check` and `git diff --check`
also passed on the local candidate.

DevLucem's public Pine v5 ZigZag++ and its exact public ZigLib/1 import now
analyze and run with default inputs. The runtime gained `chart.point.copy()`
method resolution and corrected the non-positive signs of
`ta.highestbars()`/`ta.lowestbars()` offsets. On the 300-bar Coinbase daily
TradingView export, the original script's direction agrees on 296 bars; the
four differences occur only at the beginning of the truncated local history.
See `docs/ZIGZAG_PLUS_V5_NATIVE_PARITY_20260925.md` for sources, evidence,
verification scope, and limits.
The rebuilt CLI's full 4,280-bar Coinbase 3Commas Bot result was previously
JSON-equivalent to the cash-sizing candidate
(`bjorgum-3commas-btcusd-admission-candidate-run.json` versus
`bjorgum-3commas-btcusd-fixed-run.json`).
The ZigZag++ original and exact library were also rerun with the full 4,282-bar
history: all 299 confirmed bars in its native export now match the local
direction series, including the four positions that needed earlier history.

The public Pine v5 SSL Hybrid Strategy now runs with zero diagnostics on
4,282 confirmed Coinbase daily bars. All eight exported plot columns match
the native default run; its 475 closed trades and five open entries align in
count and closed-trade entry ID/date/price, with all 475 exit prices exact and
at most two cents of PnL difference. The repaired behaviors are v5 commission
spelling, integer-division extreme lengths, per-entry full exit replacement,
bracket preservation across reversal, and tick snapping by the new entry's
direction. See
`docs/SSL_HYBRID_STRATEGY_V5_NATIVE_PARITY_20260925.md` for the source,
quantitative limits, and reproduction commands.

The next high-impact work is broader SMC drawing and setting comparisons,
default repainting
Nadaraya-Watson drawing comparison, additional complete v5/v6 community scripts,
and longer-history table resource checks. No compatibility
claim is made for v4 scripts, unverified visual drawings, forming ticks,
untested settings, or a distributable artifact.

LuxAlgo's complete public Pine v5 Order Blocks & Breaker Blocks source now analyzes
and runs on 4,282 confirmed Coinbase daily bars. Supporting typed receivers with
untyped later method parameters, `color` user-method receivers, and UDT identities
through function-returned tuple destructuring removed the original diagnostics.
A local-only six-plot probe matched TradingView's export on all 299 confirmed
overlapping bars (1,794 values, zero mismatches). The original local run ended
with 9 boxes and 18 lines; native drawing geometry is not present in the chart
CSV and is not claimed as verified. See
`docs/LUXALGO_ORDER_BLOCKS_BREAKER_V5_NATIVE_PARITY_20260925.md`.

The complete public Pine v4 Ichimoku Kinko Hyo Strategy by mdeous now analyzes
and runs without diagnostics. This exposed legacy `strategy.close_all(when=...)`
and a strategy-position alert placeholder. On 299 confirmed Coinbase daily chart
rows, all 1,121 available values across four offset-aware plots agree with
TradingView. The full native report and local run each have 20 closed trades with
identical entry/exit dates and prices; maximum measured quantity and profit
differences are `2.1991e-6` units and `0.0712 USD`. See
`docs/MDEOUS_ICHIMOKU_STRATEGY_V4_NATIVE_PARITY_20260925.md`.

HPotter's complete public Pine v2 FX Sniper T3-CCI strategy now analyzes and
runs without diagnostics. A v1/v2 declaration-graph fix propagates recursive
types through `iff()` and `nz()`. On the full 4,282-bar Coinbase daily run,
all 598 plot values in the 299 confirmed-bar TradingView export agree; all
311 closed trades match native entry/exit dates, prices, quantities, signals,
and profits. The related v2 SuperTrend Oscillator first exposed a
`min()`/`max()` self-reference inference gap and then three recursive
higher-timeframe `security()` admission gaps. Its unchanged source now
analyzes and executes. At default settings, all 1,495 plot cells across 299
confirmed Coinbase daily bars match the TradingView chart export. The optional
monthly request setting executes with host-provided monthly bars but has not
yet received a native numeric comparison. See
`docs/HPOTTER_T3_CCI_V2_NATIVE_PARITY_20260925.md` and
`docs/J1O9SB_SUPERTREND_OSCILLATOR_V2_REQUEST_ADMISSION_20260925.md`.

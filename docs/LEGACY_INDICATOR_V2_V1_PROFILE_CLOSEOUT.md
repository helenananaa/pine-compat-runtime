# Legacy Pine v2 and Implicit-v1 Indicator Profile Closeout

## Release Decision

Explicit Pine v2 and implicit Pine v1 indicator profiles close as
**experimental**. A missing directive selects v1; `//@version=2` selects v2.
Both use direct source-version execution and never opt into legacy strategies.

The evidence base is intentionally too small for preview/stable promotion: the
fixed original corpus has two v2 indicators and one v1 indicator. All three
pass, but there is no external reference-output oracle and both profiles are
far below the provisional 50-script stable gate.

## Closed Surface

The shared focused surface includes historical `study`, scalar `input`,
`plot`, `sma`, and `ema`. v1/v2 scalar declarations additionally support the
fixture-backed self-history and safe forward-reference family through a graph
bounded at 256 active nodes and 4096 dependency edges. Unsafe initializers,
statement barriers, current cycles, unstable types, and oversize graphs produce
focused diagnostics and no HIR.

Removed v1/v2 bool arithmetic lowers through explicit canonical `float` calls.
Pre-v6 numeric/`na` conditions lower through canonical `bool` calls; zero and
`na` are false. The runtime executes ordinary canonical HIR and has no graph or
conversion interpreter.

The focused v2 `security` profile retains the historical default lookahead-on
behavior. Historical batch execution may backfill future confirmed requested
values and emits `W_LEGACY_SECURITY_LOOKAHEAD`. Realtime forming/confirmed
execution deliberately does not expose that future value. Consequently the
release policy verifies a final realtime `na` rather than falsely requiring
equality with repainted history.

## Execution Evidence

The registry contains one v1 and three v2 rows. All four pass batch versus
incremental equality and realtime historical handoff. The v1 shared, v2 shared,
and v2 core rows also pass mutated forming/replacement/rollback/confirmed
equality. The v2 MTF row passes provider alignment plus the dedicated
future-leakage negative assertion. Retained storage peaks at 114 values under
the 4096-value ceiling.

Audit-machine end-to-end CLI analysis medians were approximately 2.511 ms for
the v1 row and 2.446-4.125 ms for the v2 rows. These timings are observational.
Cache tests prove implicit-v1 and explicit-v2 sources occupy distinct entries,
while every key also carries translator revision 8. Independent generated
tests exercise both the graph node and edge limits.

CLI/Python/WASM share runtime and complete-analysis goldens for v1 and v2,
including a v2 reference-cycle diagnostic. Public schemas remain unchanged.

## Promotion Boundary

The experimental profiles exclude legacy strategies, arbitrary declaration
graphs, side-effecting forward evaluation, unlisted built-ins/outputs/inputs,
general requested expressions, lower-timeframe security, and language-wide
v1/v2 compatibility. Promotion requires a materially broader authorized corpus,
reference oracles where available, continued no-future-leakage behavior, and
no qualifying unknown-diagnostic cluster. Passing the current three corpus
scripts is not sufficient by itself.

## Original v1 indicator follow-up (2026-09-25)

The original [LazyBear Volume Flow Indicator](LAZYBEAR_VOLUME_FLOW_V1_NATIVE_CHECK_20260925.md) now compiles and runs under implicit Pine v1. On its default `COINBASE:BTCUSD` daily chart, the corrected sparse-input `sum` behavior matches all four TradingView plot columns through 4,278 confirmed bars using the later chart export's OHLCV. Eight recent cells differ because the exported plot values still reflect earlier volumes for four revised bars; substituting those four values from a separately retained TradingView export makes all 17,128 cells agree. This native public-script check extends the original experimental corpus; it does not promote the version-wide profile.

The original [LazyBear Schaff Trend Cycle](LAZYBEAR_STC_V1_NATIVE_PARITY_20260925.md) now covers a different v1 behavior: an input inside a user function and four function-local historical recurrences. Its `Schaff_TC` plot matches all 4,282 aligned TradingView daily values at the default settings. The v3 input-in-function boundary remains rejected.

The original [LazyBear WaveTrend Oscillator](LAZYBEAR_WAVETREND_V1_NATIVE_PARITY_20260925.md) also runs as implicit v1. All eight native plot series match across 4,282 aligned confirmed daily bars at default settings, including 120 matching warmup blanks. This extends public-script evidence without promoting the version-wide experimental profile.

The public [WaveTrend with Crosses](WAVETREND_CROSSES_V1_NATIVE_PARITY_20260925.md) follows the same v1 oscillator with sparse crossover plots. Its ten plot series match all 42,820 native positions; the 622 crossover points in each of its last two plots are included. Native chart CSV does not expose its candle colors.

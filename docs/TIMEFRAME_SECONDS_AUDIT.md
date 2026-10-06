# Native timeframe conversion and Pivot drawing endpoints

The full original Pivot script computes drawing endpoints using
timeframe.in_seconds(pivotTimeframe). Native Chrome observations showed that
the runtime's 30-day month constant shortened those lines. The captured full
script body was preserved and six read-only observer columns were appended:
matrix row count, nominal duration, latest line x1/x2 and label x/y.

`corpus/pivot-drawing-native-raw.csv` comes from the authorized chart download
`BINANCE_BTCUSDT, 1M (16).csv`. The frozen manifest records its hash, source hash,
109 confirmed rows and exclusion of forming origin 109. At `9f9a24aad`, 213 of
654 values differed: the nominal yearly duration was 31104000 instead of native
31536036, and 104 line-end values inherited that duration error. After the
conversion correction, `pivot-drawing-month-fixed-comparison.json` matches all
654 values exactly. These are coordinate/count/time checks, not complete pixel
or style rendering acceptance.

Independent v5/v6 controls confirm 2628003 seconds per nominal month, multiplied
for 2M/3M/6M/12M. Day, week, minute and second controls preserve their standard
durations. Empty/omitted in_seconds arguments now resolve to the supplied chart
context rather than the universal one-minute default. A runtime test verifies
that month-close and change boundaries still follow the real calendar, including
February 2024; nominal conversion must not replace calendar bucketing.

The inverse helper also required repair because existing month round-trip
fixtures became non-executable. Native input-driven controls show canonical
1D/1W/1M strings, distinguish fixed 30D from a nominal month, select the next
available second/minute/day unit, clamp nonpositive values to 1S and long values
to 12M. In particular, 31449601 and 31535999 seconds return 365D, while 31536000
returns 12M. Exact nominal-month multiples preserve month units. Twenty named
cases agree in both v5 and v6, with source/DOM hashes recorded in
`corpus/timeframe-canonical-native-manifest.json`.

A direct nested constant probe encountered TradingView's own
`resolution.trim is not a function` compilation diagnostic. It is retained in
the rounding DOM evidence; input-driven probes avoid that constant-folding path
and provide the accepted references. It is not interpreted as a runtime
requirement to reproduce a TradingView compiler error.

The existing timeframe fixture now compares canonical strings and its snapshot
changes only the 3M/12M numeric outputs. The 23 existing runtime time tests and
two new v5/v6 conversion/calendar tests pass. Python and actual WASM smoke tests
cover supplied monthly context, nominal duration and inverse conversion.
Full verification is running in `month-conversion-full-verify.log`. Rebuilt
commit-bound artifacts, full Pivot cross-host checks and the remaining product
qualification are still pending.

The first full gate caught four legacy/modern monthly request regressions in
the pre-existing fixed-duration divisibility check. Nominal month seconds are
not an integer multiple of day seconds, so requests involving calendar months
now use their calendar merge boundaries without that fixed-ratio restriction;
the lower-timeframe guard and fixed-period ratio checks remain intact. All four
original month-boundary controls pass in `month-calendar-request-regression.log`.
The active full rerun is `month-conversion-full-verify-v2.log`.

That full Windows verifier completed successfully: 6793 Rust tests, 772 tests
against a freshly installed wheel, 130 tool tests, structure/host-parity checks
and actual generated WASM under Node. The current capability matrix now records
the supplied-chart default, nominal month duration, inverse rounding/canonical
units and clamp boundaries. This is implementation qualification; retained
same-commit artifacts and broader product scope are not yet accepted.

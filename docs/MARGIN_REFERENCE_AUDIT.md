# Margin quantity reference and candidate

Baseline fd01a580e. The candidate was implemented and qualified in the isolated
`E:/projects/pine-interpreter-delivery-account` worktree. This is a Windows debug
qualification slice; the overall stable release remains unaccepted.

An original frozen long strategy on the existing independent hourly chart data
uses capital 40450, quantity 1, 50% margin, no fees and no Magnifier. Native
TradingView reports mincontract=1e-6 and pointvalue=1. It liquidates 0.009996 at
80571.8, then closes the remaining 0.990004 at 80650. Both entries originate at
80836.6. The old runtime uses a synthetic integer quantity default and misses
the partial margin call entirely.

The [official margin calculation](https://www.tradingview.com/support/solutions/43000717375-how-to-simulate-trading-with-leverage-in-pine-script/)
truncates cover quantity to the instrument's decimal precision before multiplying
by four; displayed liquidation prices round down for longs and up for shorts to
the supplied price grid. These rules inform the candidate, not a symbol lookup.

Candidate behavior:

- Host-owned decimal quantity precision in ChartContext; default zero preserves
  the integer profile. The u32 power-of-ten scale supports precision 0 through 9.
  This is a decimal-power minimum contract profile, not arbitrary quantity steps.
- CLI --chart-quantity-precision and optional Python/WASM $chart.quantityPrecision.
  Python retains separate identity parameters and required price-grid fields.
- Candidate selection and forced-fill execution share margin quantity calculation.
- syminfo.mincontract reads host metadata. Simple/runtime metadata is no longer
  folded to synthetic defaults in numeric/history analysis or function defaults.
- Script-visible liquidation price uses the chart price grid.
- Missing open/closed size returns zero for absent integer indices. Native controls
  prove negative indices return zero and na selects index zero, including after
  two closed trades. Invalid fractional indices retain the prior rejection/result
  behavior; other field families are not inferred from size.

Raw source, original CSV/DOM, pre-fix results and subsequent control scripts live
under `.local/delivery-20260909/margin-reference/` in the main checkout. The
initial three Rust tests exposed a real constant-folding error; after correction,
metadata-dependent history and default arguments agree with runtime. The long
candidate matches both independent trades and all 3996 chart output values;
time/count/position/quantity-metadata identities compare exactly in
strict-candidate-comparison.json. Both installed host artifacts and the final Windows gate now pass.

Intermediate runtime library regressions pass 1799/1799 after updating only the
native-proven missing-size expectations and short liquidation-price rounding.
The original operands and input histories remain. Candidate golden capture is
separate from regression success; review every delta before updating snapshots.
No reference tolerances, frozen inputs, source strategy or global host settings
were changed to make a comparison pass.

The size controls distinguish three cases: absent integer indices return zero,
na selects the first record even when two closed records exist, and a fractional
non-integer value is not treated as na. The prior fractional-index behavior is
preserved. Controls are missing-size-control, active-size-control and
size-after-close-control, with retained sources and native DOM output.

Current validation: 6630 Rust tests, 690 fresh installed-wheel Python tests,
103 tool tests, structural checks, host-parity registry, and actual WASM/Node
smoke pass in full-gate-candidate-v3.log. Five targeted Rust tests cover quantity,
metadata/default/history evaluation, short partial liquidation and internal
execution-mode consistency. Python chart parsing was moved into chart_metadata.rs
after the prior full gate exposed the 800-line facade budget; validation behavior
and the structural threshold were preserved.

Ten initial candidate snapshots differed, including unrelated runtime_math low
bits. Nine reviewed existing snapshots changed only native-proven absent-size
zero values or liquidation-price tick rounding (approved-goldens.json).
runtime_math remains unchanged. The new original four-bar quantity fixture is
checked through CLI, installed Python and actual WASM. The separate internal
lifecycle test abandons a losing forming update before replacement/confirmation
against a fresh control. These lifecycle checks are not an independent tick
sequence oracle.

The separately frozen short derivative passes both trades and all 3996 values
on the same 148 hourly bars. It covers 0.023516 at 81016.5 on the entry hour,
then the remaining 0.976484 at 80650, with final net profit 177.981386000006.
Script size and runtime trade quantities are signed negative for shorts; native
trade CSV Size (qty) is an absolute magnitude. The short comparator now uses
native plotted signed size and crosschecks its magnitude against the native CSV
under the unchanged numeric tolerances. The first copied long comparator
incorrectly compared unsigned CSV quantity with signed runtime quantity; its
failure is retained as short-strict-before-comparison.json. No runtime change
was needed for short sign or margin execution. Full-source independent input
hashes and all plots/trades are retained in short-strict-candidate-comparison.json.

The first full Windows gate stopped at the new original fixture's JSON textual
spelling (1e-06 versus 0.000001); parsed values were identical. Only that new
snapshot's spelling was corrected, and CLI 228/228 then passed. The original
full-gate-candidate.log remains. Five conformance rows now describe current
minimum-contract, signed-size, missing-size and margin-rounding behavior; the
matrix schema, feature denominator and statuses are unchanged. Their notes and
fixture-link deltas were reviewed before updating the generated matrix snapshot.


Retained artifacts and complete multi-host results are under
`.local/delivery-20260909/margin-candidate-hosts/`. Both frozen full sources pass
complete CLI/installed-wheel/actual-WASM output equality: 3996 values and two
trades per direction. CLI batch, incremental, realtime-history and final-bar
forming replacement/confirmation outputs match for each source. Installed module
path, hashes and commit are recorded in qualification.json. The full original
TechnicalRating dependency graph also revalidates all 63399 independent values
on the current CLI and matches its earlier qualified complete output.

Remaining boundaries: these are historical same-currency linear-account
references with zero fees and unit point value, not arbitrary lot increments,
currency conversion, inverse contracts, all margin ratios, real tick event
ordering, sustained resource qualification or a final Linux/release matrix.

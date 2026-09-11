# TradingView goal acceptance evidence

Implementation: `a2a1ba5fb0ca566e1de01a9cb8e886322df92b52`.
This audit follows the requested realtime-price, observable B1, independent r1
reference and candidate-validation work. It does not claim full Pine parity.

Final platform receipts are checked in the local evidence index
`.local/binance-directional-20260910/qualification.json`: each optimized wheel
passes all 19 retained native scenarios, 867418 values and zero mismatches.
Both full platform gates pass 6677 Rust tests, 717 installed Python tests and
actual WASM. Windows passes 130 tool tests; Linux runs 130 with the single
Windows-only probe skipped. Each optimized wheel separately passes 717 tests.

| Requested outcome | Authoritative evidence | Disposition |
| --- | --- | --- |
| Resolve the original realtime 16/896 price differences | Original source and CSV replay on the repaired optimized wheel: 896/896, no changed tolerance or input; later OKX failure also 1856/1856 | Passed on Windows and Linux optimized wheels |
| Obtain independent native data using authorized Chrome | Both Binance sources checked against editor contents; sequential native execution records and CSV agree; continuous exchange trades reconcile to closed bars | Passed |
| Verify observable B1 behavior | Six independently captured admission/capital/creation-order scenarios, 9460 native values; margin admission repair | Passed on Windows and Linux final wheels |
| Supplement r1 independent reference | Unmodified r1 commission source on native supplied bars: 99952 values; independent v5/v6 controls expand capital/commission group to 624806 | Passed on Windows and Linux final wheels |
| Correct demonstrated additional semantics | Versioned margins, initial capital, absent commission, opening context, and realtime market prices have native controls and Rust regressions | Implemented; platform qualification below |
| Freeze provenance and comparison rules | Source SHA-256, raw exports, exchange event IDs, timestamps, separate scenarios and 1e-9 tolerances retained; earlier failed reports preserved | Passed |
| Verify candidate artifacts and regressions | Windows full gate: 6677 Rust / 717 installed Python / 130 tools / actual WASM; optimized wheel: 717 Python; Linux full gate and optimized-wheel reference replay pass | Passed |
| State unobservable and unverified boundaries | B1 private sequencing remains UNVERIFIED_INTERNAL_ORDER; original r1 synthetic denominator remains separate; range/price-condition scope below | Explicit |

## Scope of the remaining boundaries

The B1 observations establish visible admission and trade behavior; they cannot
prove a private entry/exit execution order. The original frozen r1 corpus has
not acquired references for every original synthetic scenario. The new native
companion is independent evidence for its unchanged source, with a different
market-data scenario, and is counted separately.

Realtime market orders now match the observed one-sided range expansions.
Simultaneous new high and low, price-condition orders during unobserved price
movements, and arbitrary feed/clock alignments remain unqualified. Existing
fallback behavior is documented rather than labeled as native parity.

Linux artifacts from this run target manylinux_2_35, not a newly certified
manylinux2014 distribution. No stable release, remote push or publication is
part of these local verification results. Broader product readiness and all
remaining language/data-provider capabilities are not established by this audit.

See `REALTIME_MARKET_EXTREME_AUDIT.md`, `STRATEGY_LONG_STOP_MARGIN_ADMISSION_AUDIT.md`,
`STRATEGY_NATIVE_DEFAULTS_AUDIT.md`, `STRATEGY_VERSIONED_MARGIN_AUDIT.md` and
`REALTIME_OPENING_CONTEXT_AUDIT.md` for source-level evidence and historical
before/after results. Current raw receipts are local under
`.local/binance-directional-20260910/` and `.local/tv-goal-20260910/`.

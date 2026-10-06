# USDJPY native comparison coverage

Updated 2026-09-29. These seven named SSL Hybrid Strategy v5 cases use the
unchanged public source, FXCM `FX:USDJPY`, JPY account currency, a 1/1000 price
grid, integer quantity precision, and HL2. Individual reports retain settings,
forming-bar exclusions, display tolerances, and reproduction commands.

All seven cases have now been rerun after the
[VAMA extrema repair](SSL_HYBRID_FX_AUDUSD_FOURHOUR_VAMA30_HL2_20260929.md),
in three historical modes each, with fresh native comparator results and
whole-output equality to their preceding outputs. The
[current-source requalification report](SSL_HYBRID_EXTREMA_REQUALIFICATION_20260929.md)
binds the frozen captures to core patch
`827e195c3a7db4422bbd657a5634990adb9d8d1db69658f6331df41a97cb2636`.
Original case receipts and the original broker-patch audit remain historical
records; their source identity is preserved.

| Period | Baseline | Confirmed bars | Chart positions | Closed trades | Explicit exit fills | Report |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| Four hours | HMA 20 | 21,345 | 149,285 | 1,088 | 592 | [Evidence](SSL_HYBRID_FX_USDJPY_FOURHOUR_HMA20_HL2_20260929.md) |
| Four hours | HMA 30 | 21,345 | 149,241 | 892 | 581 | [Evidence](SSL_HYBRID_FX_USDJPY_FOURHOUR_HMA30_HL2_20260929.md) |
| Four hours | EMA 30 | 21,345 | 149,255 | 470 | 319 | [Evidence](SSL_HYBRID_FX_USDJPY_FOURHOUR_EMA30_HL2_20260929.md) |
| Four hours | DEMA 30 | 21,345 | 149,141 | 658 | 480 | [Evidence](SSL_HYBRID_FX_USDJPY_FOURHOUR_DEMA30_HL2_20260929.md) |
| Four hours | TEMA 30 | 21,345 | 149,025 | 768 | 507 | [Evidence](SSL_HYBRID_FX_USDJPY_FOURHOUR_TEMA30_HL2_20260929.md) |
| Daily | TEMA 30 | 14,316 | 99,822 | 134 | 77 | [Evidence](SSL_HYBRID_FX_USDJPY_DAILY_TEMA30_HL2_20260929.md) |
| Weekly | TEMA 30 | 2,906 | 19,952 | 28 | 24 | [Evidence](SSL_HYBRID_FX_USDJPY_WEEKLY_TEMA30_HL2_20260929.md) |

All cases have native chart/trade exports from authenticated Chrome, no
diagnostics, and byte-identical batch, incremental, and realtime-history
outputs. Chart positions count nonblank observations in eight named columns,
with missing positions also compared. Counts across settings overlap and
must not be summed as independent bars or evidence for arbitrary scripts.

The original artifact audit `.local/audit_usdjpy_coverage_20260929.py` validates each
native artifact against its frozen hash, the immutable CLI and patch, current
core source hashes, mode exit codes and output hashes, comparison results,
and the full-gate receipt/log. Its machine-readable output is
`.local/usdjpy-coverage-audit-20260929.json`. The full gate is reused for verified
unchanged source; the audit does not rerun the runtime or the gate.

The original core source is base `2f9a83a0c333092167c7253571f56ee82a7ec0d0` plus
the recorded broker patch, SHA-256
`fe7c8f1ed5bcaed259a08cd284168cdf752d139bd0271e5a62022501a2cd2fc4`.
That original audit rejects the current changed core. The new read-only audit
is `.local/audit_extrema_requalification_20260929.py`; its aggregate receipt
is `.local/extrema-requalification-20260929/matrix-verification.json`.
The new runtime execution and comparator receipts are in that directory,
separately from the original captures.
This is coverage of named historical cases and selected fields. Currency
conversion, arbitrary parameter combinations, other scripts, and live
forming-tick parity are not established by this table. Local DEMA daily and
weekly sensitivity controls are not counted as native-qualified cases.

Continue expansion by crossing algorithm families with periods and symbols,
then qualifying further complete original scripts. Preserve native history
starts and settings in each case; resolve observed semantic mismatches in
the deterministic core, and keep external data supply outside the core.

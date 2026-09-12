# Pivot null barriers and plateau ownership — 2026-09-12

This repairs the two differences discovered while running the unchanged
official RSI with Bollinger smoothing and divergence enabled. It follows the
gradient-fill commit `f6339ec54`.

## Independently observed semantics

The v6 and v5 controls each export 654 values from native bar indexes 0..108,
with identical results and no tolerance. The source uses explicit constant
arrays and bar indexes rather than market prices. Original source, DOM, CSV,
hashes and the v5 cross-check are retained in
`.local/product-completion-20260912/corpus/pivot-na-*`.

The full raw left+right+1 window must exist. An `na` candidate is not a pivot.
Starting at the candidate, each side stops comparing at its nearest `na`;
values beyond that barrier do not invalidate the candidate. Equal older
values are allowed; equal newer values invalidate it, assigning a plateau to
its rightmost qualifying candidate. This is not equivalent to discarding all
nulls and comparing the remaining full window.

The previous code required a completely non-null window and required strict
inequality on both sides. It also enumerated values after flattening nulls,
which could not preserve candidate indexes once null windows were admitted.
The replacement uses the original candidate index and two directional scans.
Generic rolling-window readiness for other algorithms is unchanged.

## Verification and complete-script result

`verify-pivot-na-v2.log` passes the full Windows gate: 6,730 Rust tests, 130
tool tests, strict formatting/clippy/structure, host parity, actual WASM and
762 installed-wheel Python tests. The one earlier failure was the old
`runtime_pivots_edge_cases.json` assertion that a rightmost tied high must be
absent; precisely that one value changed from 1 to 0 in `na(tie_high)`.

Fresh retained CLI/wheel/WASM development artifacts are under `pivot-hosts/`.
`pivot-final-hosts.log` reruns actual WASM, all installed-wheel tests and the
default official RSI reference (218/218). `pivot-alternate-host-parity.log`
passes all 852 render-aligned Bollinger/divergence references, including 357
non-null values. For negative plot offsets, only output backed by confirmed
execution bars is compared; the frozen rule and denominator were not changed.
Both configurations have equal complete output through all four CLI modes,
installed Python and generated WASM, plus Python forming/confirmation replicas.

The original two-mismatch report remains in
`gradient-hosts/rsi-bb-divergence-comparison.json`. The repaired result is in
`pivot-hosts/rsi-bb-divergence-comparison.json`. These are Windows development
artifacts, not optimized final distribution or Linux qualification.

## Remaining product work

The complete Pivot Points Standard source still has parser, reference-type
matrix/UDT, function result and requested-context admission gaps. A fresh
inventory is `corpus/pivot-after-gradient-analysis.json`; the first parse error
is the chained call/field/method expression on source line 194. Real-script
coverage, broader execution/request/account capabilities, resource budgets and
unified distribution remain open in `PRODUCT_COMPLETION_EXECUTION.md`.

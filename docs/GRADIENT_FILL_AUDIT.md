# Gradient fill and complete RSI default acceptance — 2026-09-12

Implemented from `a481b4644` on the product-completion branch. The core now
retains vertical-gradient value/color stops and masking plot IDs through full
results, deltas, typed readers, replicas, realtime rollback and physical output
pruning. It does not approximate a gradient with one solid color or add a host
renderer. See `GRADIENT_FILL_CONTRACT.md` for the wire contract.

## Compatibility and native evidence

The unmodified official RSI source with default inputs passes all 218 values
on 109 confirmed monthly bars, beginning at native bar index zero. No warmup
is skipped and the frozen 1e-9 absolute/relative tolerances are unchanged.
CLI historical/incremental/realtime-history/realtime-forming, installed Python
and actual generated WASM produce equal complete output, including both
gradient fills. A Python streaming session and its replica also match.

The native v5 gradient control compiles and displays distinct upper green and
lower red stops. Source, DOM and screenshot are retained as
`.local/product-completion-20260912/gradient-native-v5.*`. The original v6 RSI
source and exports provide separate v6 admission evidence.

## Gates and artifacts

Verification completed in stages: `verify-gradient-v2.log` passes 6,729 Rust
tests, strict format/clippy/structure, 130 tool tests and host-parity checks.
Its actual-WASM stage exposed a stale expected schema getter (3 instead of 4).
`gradient-final-hosts.log` retains the corrected actual-WASM gate, all 760
tests on the installed wheel, and complete RSI cross-surface acceptance.
Earlier failure logs remain, including the initially missing diagnostic-code
documentation. No runtime comparison tolerance was weakened.

Retained development artifacts are under
`.local/product-completion-20260912/gradient-hosts/`; `qualification.json`
binds CLI, wheel and generated WASM SHA-256 values. These are Windows debug
development artifacts, not optimized release builds or Linux qualification.

Runtime schema is now 9 and changes schema is 4. Exactly 940 current output
snapshots were migrated by changing only their top-level schema number; a
machine comparison checked all other parsed values were identical. The
inventory is `schema9-snapshot-migration.json` in the same evidence root.
An unchanged schema-8 snapshot remains a legacy-read fixture. Gradient data
under an old schema and malformed gradient samples are rejected.

## Newly demonstrated remaining defect

A separate native RSI configuration (SMA + Bollinger Bands, Calculate
Divergence enabled) was frozen without source edits. Its CSV is render-aligned:
negative plot offsets are reversed only where the corresponding execution bar
is confirmed. This yields 852 comparable fields, including nulls, with the
same 1e-9 tolerances. Two initial pivot outputs differ: bullish at rendered
index 17 / execution index 22 and bearish at 14 / 19. Both runtime values are
null where the native reference reports a number. Therefore the default RSI
acceptance does not qualify all RSI configurations.

The raw alternative CSV, settings and rules are retained in
`corpus/rsi-bb-divergence-manifest.json`; the unchanged failure report is
`gradient-hosts/rsi-bb-divergence-comparison.json`. Next investigate pivot
handling near initial `na` history with independent native controls.

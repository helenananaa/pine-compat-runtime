# Modern local UDF array push — 2026-09-12

Implemented on the `cc11124f2` product-completion worktree. No publication or
new platform-distribution claim is implied.

Local v5/v6 functions now admit namespace `array.push` and receiver `.push`
using the existing typed array runtime. Caller parameters, global array
references, aliases, nested calls and copied call-result arrays preserve their
existing identities. This changes semantic admission, not array storage or
broker semantics. Imported functions retain their separate pure-library gate.
Legacy v4 push, other unqualified mutations, wrong element types, global scalar
reassignment and output side effects remain rejected.

## Independent source controls

Chrome executed the complete `tests/fixtures/runtime/udf_array_push.pine`
control in TradingView in both v6 and v5. Terminal chart legends reported
`count=1`, `alias size=3`, `nested sum=19`, `last=9`, `persistent sum=10` and
`persistent size=5`. The separate v6 global-array source reported sum 25 and
size 10. Inputs are deterministic constants and the first five bar indexes;
these observations are independent semantic controls, not a real-script
compatibility percentage or a complete per-bar export.

Original DOM captures are retained in `.local/product-completion-20260912/`:

| File | SHA-256 |
| --- | --- |
| udf-push-v6-dom.txt | 665b18722e397b6a4202638f0dc62f564ec90fca7ad1cab20197c84110b294ab |
| udf-push-v5-dom.txt | ad531bbacbf0a9a23450f3c04301c0729e93306e68ef34c41d40f1412a35da55 |
| udf-push-global-v6-dom.txt | ca1f4fb9166f5180959c969e3270cec3baad3644c482f6d117345ae389995a65 |

The unchanged formerly unsupported fixture remains at its original path and
now asserts admission. The mixed-negative call-result fixture loses precisely
the push rejection (254 to 253 diagnostics); other diagnostics remain asserted.
New tests verify copied-array independence, batch/incremental equality, `var`
rollback versus `varip` persistence, and Python replica/full-result equality.

## Verification

`verify-udf-push-v5.log` records the complete Windows gate: 6,720 Rust tests,
130 tool tests, strict format/clippy/structure and host-parity checks, actual
generated WASM/Node controls, and 757 tests against a freshly installed wheel.
Earlier failed logs remain: stale rejection assertions, one clone-on-Copy lint,
formatting, and JSON integer-vs-float test expectations were corrected without
loosening numerical tolerances or changing the original negative source.
The gate's temporary wheel is not a retained optimized distribution artifact.

## Next complete-script work

Chrome-frozen, unmodified official RSI and Pivot Points Standard sources are
under `.local/product-completion-20260912/corpus/`. RSI exposes declaration
timeframe, gradient-fill and input-controlled render metadata gaps; Pivot Points
also exposes active input controls and further syntax/reference-family gaps.
Their initial analysis reports are diagnostic inventories, not passed scripts.
The requested RSI CSV download did not yield a supported completion event;
the export is not yet counted as available independent output.

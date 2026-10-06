# Function-final tuple declarations — 2026-09-12

The unchanged official Pivot Points Standard ends one `calculatePivots`
branch with a tuple declaration from `request.security`. The runtime formerly
reported `E_FUNCTION_RETURN` for that branch even when the producer's tuple
type was known. This slice removes that diagnostic while preserving remaining
request/collection diagnostics. The complete script is still non-executable.

## Observed behavior and implementation

Native TradingView v5 and v6 accept a final tuple declaration as a function
result. All RHS slots are returned, including slots bound to `_` and even when
all destinations are `_`. A stateful producer runs once per execution. The
control covers direct, discarded, all-discarded, alternating branch and
stateful producer cases. Each native reference contains 1,090 values across
109 confirmed monthly bars from index zero; the current forming month is
excluded. OHLC and timestamps agree with the previously frozen RSI inputs.
CSV, source, DOM and frozen hashes are under
`.local/product-completion-20260912/corpus/tuple-final-*`.

Semantic analysis validates the actual declaration before using its RHS type
as the function result. Lowering returns the RHS once because the final local
destinations have no later reader. Tuple element queries and tuple UDT-array
identity propagation recognize the result form. Existing error handling for
invalid tuple producers and wrong arity remains active. Tests cover imported
library array tuples and realtime re-execution/confirmation in addition to the
native control cases. Public schemas and host boundaries are unchanged.

This is function-final declaration support. General branch/loop statement
result admission, broader UDT reference behavior and request capabilities are
separate contracts and must not be inferred from this slice.

## Verification

`verify-tuple-final.log` passes the full Windows gate: 6,748 Rust tests,
768 installed-wheel Python tests, 130 tool tests, formatting/clippy/structure,
host parity and actual generated WASM. The public Python and WASM controls
exercise discarded slots and the single stateful evaluation.

`tuple-final-hosts.log` passes on retained Windows debug CLI/wheel/WASM under
`tuple-hosts/`: v6 and v5 each match all 1,090 frozen native values. Complete
outputs agree across four CLI modes, installed Python and actual WASM, plus
Python forming/confirmation and replica paths. Both unchanged RSI reference
suites pass again (218 default, 852 Bollinger/divergence). These worktree
development receipts are not optimized final distribution or Linux acceptance.

## Next work

The fresh `corpus/pivot-tuple-analysis.json` has no parser or function-return
diagnostics. It still reports 42 diagnostics: 7 declaration types, 16 method
receivers, 7 unknown symbols, 8 unsupported features and 4 call argument
errors. Continue through reference-bearing UDT arrays/matrices, method
mutations, provider-backed request contexts and lookahead semantics. Preserve
the complete official source and frozen controls without rewriting the target
to fit the current interpreter.

Before relaxing the scalar-tree UDT-array admission guard, collect native
alias/copy/history controls: `PineValue::UserType` currently stores a value
vector, whereas drawing and matrix values use IDs. Admission alone would not
establish correct mutable-object identity. The current classifier is in
`analyzer/user_types/types.rs`; matrix declarations are currently restricted
to `matrix<float>` in `analyzer/statements/declarations.rs`.

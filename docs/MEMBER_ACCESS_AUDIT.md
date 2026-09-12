# Evaluated receivers and undefined UDT reads — 2026-09-12

This slice removes the parser obstruction in the unchanged official Pivot
Points Standard expression `.row(...).last().pivotLine.get_x2()`. It does not
qualify that complete script: reference-bearing collections, method effects,
function-final tuple declarations and requested contexts still block it.

## Implementation and compatibility boundary

The syntax tree now represents member access on evaluated receivers explicitly.
Semantic analysis and lowering preserve receiver type identity through nested
fields, function results and imported modules. Generic function invocations
resolve field indexes from their actual receiver type, including ordinary
qualified reads. Receiver evaluation happens once. Built-in method dispatch
uses the field's type and the existing argument binding rules. The AST visitors
used by imports, legacy handling and effect checks traverse the new node.
Unsupported request-expression profiles continue to reject it explicitly.

Qualified member source ranges come from lexed source tokens, preserving
comments, whitespace and Unicode boundaries in call-site provenance. Module
source text remains compiler state; no host service or network dependency was
introduced. Public runtime output schemas remain 9 and 4 for result/changes.

Pine v6 rejects history applied directly to a UDT field with
`E_UDT_FIELD_HISTORY`; object history and scalar-variable history are admitted.
Chart-point field history remains admitted. The rule is independently stated
in the [official v6 migration guide](https://www.tradingview.com/pine-script-docs/migration-guides/to-pine-version-6/).

Reading a field through an undefined UDT now raises `E_UDT_NA_FIELD`. Native
TradingView rejected the unguarded `(probe[1]).value` on bar zero with RE10041;
the observed error DOM is retained. A defined object whose scalar field is
`na` remains valid. Failed realtime reads preserve the prior result and
revision. General mutable UDT alias/history semantics and undefined-object
writes are not qualified by this member-read slice.

## Native controls and retained regressions

The v6 control has six plots: nested reads, a stateful receiver evaluated once,
scalar history, chart-point fields, a drawing-field getter and chart-point
field history. The v5 control adds direct UDT field history, evaluated UDT
field history and guarded object history. Native CSV/source/DOM manifests
were frozen before comparison under
`.local/product-completion-20260912/corpus/member-v*`.

Both controls start at native bar index zero. Each uses 109 confirmed monthly
BINANCE:BTCUSDT bars; the forming month is excluded. The v6 denominator is
654 values and v5 is 981, including initial nulls, with frozen absolute and
relative tolerances of 1e-9. Controls also verify the underlying OHLC against
the previously frozen RSI input.

The corrected undefined-object rule exposed 29 old positive fixtures. Their
original sources remain unchanged and are now executed as negative cases in
historical and incremental modes, checking the error and failure index.
`tests/fixtures/undefined_udt_access.tsv` maps each to a separate guarded
positive counterpart. CLI/Python/WASM golden tests use the positive variants;
the normal runtime inventory still executes both. Snapshot comparisons show
only generated ID changes for these 29 cases, with numeric output unchanged.
An unrelated one-ULP math snapshot regeneration was discarded.

## Verification

`verify-member-v4.log` passes the Windows gate: 6,745 Rust tests, 766
installed-wheel Python tests, 130 tool tests, formatting/clippy/structure,
host parity and actual generated WASM. Python and WASM also check that the
undefined-object diagnostic survives their public exception boundaries.
Failed gate logs are retained, including the native WASM-wrapper test that
still referenced an invalid unguarded positive fixture. That reference was
corrected rather than suppressing the runtime error.

`member-final-hosts.log` qualifies retained development artifacts under
`member-hosts/`. The frozen native controls match 654/654 v6 and 981/981 v5
values. Full output agrees across four CLI modes, installed Python and actual
WASM; Python forming/confirmation and replica output also agree. The unchanged
official RSI default 218/218 and Bollinger/divergence 852/852 references pass
again on these artifacts. Hashes and receipts are listed in
`PRODUCT_COMPLETION_ARTIFACTS.json`. These are Windows debug artifacts from the
implementation worktree, not a final optimized distribution or Linux gate.

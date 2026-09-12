# UDT identity: native evidence and current defect — 2026-09-12

Status: **unrepaired semantic defect**, following `4407159cd` and `0606a9943`.
The previously qualified member/tuple slices do not establish mutable UDT
reference semantics. This audit drives the next implementation required by
the unchanged official Pivot Points source.

## Native and local evidence

Chrome ran the unchanged eight-plot `udt-identity-control.pine` in v6 and v5.
Each frozen CSV contains 872 values on 109 confirmed monthly bars from native
index zero. Source, DOM, raw CSV, expected values and hashes are retained under
`.local/product-completion-20260912/corpus/udt-identity-*`. The current forming
month is excluded and OHLC/timestamps match the existing RSI input.

Both native versions demonstrate shared assignment and array-element aliases,
an independent outer object from `copy()`, shared nested objects after shallow
copy, and the distinction between persistent-object history (same object's
current field) and freshly allocated object history (prior object's field).

The complete control currently fails admission: two missing `copy` methods,
a resulting non-UDT field error, unsupported nested field mutation, and an
int-to-float field assignment error. The unchanged diagnostic artifact is
`corpus/udt-identity-baseline-analysis.json`.

A separate two-plot alias isolation compiles and executes but returns wrong
values. On the first confirmed bar, close is 4724.89; direct alias mutation
should expose 4725.89 and array alias mutation should expose 4728.89. Both
currently return 4724.89. The isolation report compares the two corresponding
native plots and is retained as `udt-alias-baseline-comparison.json`. It is
not an execution receipt for the complete native control.

## Required semantic migration

`PineValue::UserType(Vec<PineValue>)` clones field values on assignment.
`runtime/statements.rs` constructs a replacement vector when changing a field.
An arena owned by each runtime must instead hold object fields behind stable
references; histories and collections retain references, and explicit copy
allocates one outer object while preserving nested references. Runtime cloning
for rollback must clone arena state independently, not share mutable cells
between committed and forming states.

Integration must cover construction, member reads, direct/nested and array
field mutation, equality/identity, shallow copy, string formatting, UDT-array
ordering and public-value materialization. Preserve declared UDT identity and
field types during analysis; numeric field assignment must follow the same
promotion rules as valid constructor inputs. Do not admit arbitrary reference
collections by only removing the scalar-tree guards.

Request child runtimes need explicit transfer of returned object graphs rather
than interpreting child-local IDs in the caller arena. Preserve graph aliasing
and reject unsupported cross-context transfers until implemented. Object
lifetime/accounting must integrate with the later resource contract; output
display pruning cannot discard script-readable objects or their history.

Realtime persistence needs field-level policy. A `varip` object variable does
not by itself make its fields escape rollback; field declarations select that
behavior. This is described in the [official Objects documentation](https://www.tradingview.com/pine-script-docs/language/objects/).
The current `seed_intrabar_persistence_from` copies values/collection storage
without a UDT graph policy, so it must be updated and verified with mixed
ordinary/varip fields, aliases, nested collections and failed updates.

## Acceptance required before closing this defect

Run the complete frozen v5/v6 control unchanged with all 872 values per suite;
retain the failed baseline. Add independent realtime/varip and request-context
controls, alias mutations through methods/UDFs and array/matrix views, copy and
undefined-object error tests. Recheck original historical fixtures against
native expectations instead of preserving incorrect value-copy snapshots.
Then run the full Windows and installed CLI/Python/WASM gates plus prior RSI,
member and tuple suites. General collection admission and complete Pivot Points
qualification remain subsequent work. Goal remains active.

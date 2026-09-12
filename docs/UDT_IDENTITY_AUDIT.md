# UDT identity: native evidence and current defect — 2026-09-12

Status: **Windows-qualified for the named UDT scope; retained artifact
qualification in progress**, following `4407159cd` and `0606a9943`.
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

## In-progress arena implementation

The uncommitted worktree after `e440d0066` now allocates UDTs in a
runtime-owned `object_store` and carries `UserTypeRef` through assignments,
history and arrays. Runtime cloning clones the store independently. Field
reads/writes resolve the object; array ordering materializes keys without
replacing references, and array join materializes fields for formatting.
The existing owned `UserType` value representation is retained for materialized
values. Bare arena handles follow other collection handles in standalone
JSON/Python value conversion; public object graph transport is not qualified.

The executable alias isolation now matches 218/218 native values; the original
failure report remains unchanged. Two new `udt_identity` tests cover both
dialects, persistent-object history and forming/confirmation rollback. The
workspace compiles. A broader runtime run reaches realtime tests with two
failures: imported UDT varip transient-reference restoration and a local UDT
varip assertion based on prior field snapshots. Do not rewrite those expected
values before collecting the relevant native realtime evidence.

Request capture and return paths now explicitly reject UDT reference graphs
until transfer is implemented, preventing child-local IDs from being treated
as caller IDs. These guards are part of the unqualified worktree migration.
Next: field-level varip metadata and transient graph restoration, shallow copy,
nested mutation, constructor/field promotion, complete original control,
request graph transfer and full gates. No new distribution or acceptance
commit has been produced for this intermediate implementation.

Field-persistence continuation: native `udt-varip-control.pine` was sampled
twice during live updates in each dialect. In v6, ordinary fields on the two
existing objects stayed at 110 while varip fields moved 124 to 157; the
transient varip-retained object's ordinary and varip fields both moved 14 to
47. In v5, corresponding values were 110, 126 to 184, and 16 to 74. DOM
snapshots retain the live observations; these are update deltas, not a frozen
tick-tape replay qualification.

The worktree now carries field `varip` through AST, local/imported metadata
and HIR. Runtime rollback overlays marked fields on existing objects and
retains the fields of newly created objects kept by varip roots. Canonical
declaration names prevent import aliases from hiding field metadata. Direct
field compound assignments now parse. Four `udt_identity` tests pass,
including explicit-constructor field persistence and imported metadata.

Two additional admission gaps were found in the unmodified live control:
direct field compound assignment (now implemented) and type-field default
values (still skipped by the parser and not supplied by constructors). The
deterministic persistence test explicitly supplies constructor values to
isolate the verified rollback rules; it is not a claim that the complete
native source executes. Defaults, full original UDT identity controls, mixed
object/collection graph restoration, request transfer and distribution gates
remain open. Original realtime expectations have not been rewritten.

After this continuation, `udt-field-varip-lib.log` passes all 1,825 runtime
unit tests and `udt-field-varip-realtime.log` passes 40/41 realtime tests. The
imported transient-reference failure is repaired; the remaining failure is
`user_type_varip_fixture_persists_intrabar_value_between_forming_updates`,
which loads `tests/fixtures/realtime/user_type_varip.pine` (not the similarly
named historical runtime fixture). Its large sequence of old assertions still
needs native requalification, including shared history and alias effects.
Structure checks pass; full product acceptance remains open and changes are
still uncommitted.

Default-constructor continuation: field default expressions are now retained
in the AST and local/imported metadata. Constructors fill omitted fields with
declaration defaults, or implicit `na` (v6 bool uses false). Invalid expression
forms/types produce `E_UDT_FIELD_DEFAULT` even when overridden. UDT results
now carry the series qualifier, allowing subsequent series field assignments.
Default builtin variables are kept separate from same-named function parameters
in analysis/lowering. Six `udt_defaults` and four `udt_identity` tests pass in
`udt-default-targeted-v5.log`, including the original native field-varip source
without constructor rewrites, imported defaults/nested nulls and v5 float-field
division after integer construction/assignment.

The sema library audit currently has 1,232 passes and two old rejection tests
that expect `PrivateWrapper.new(na)` to fail. Their sources and assertions
remain for deliberate review; no full gate has been claimed. The earlier
runtime realtime snapshot-history assertion remains open too. Continue with
native default controls as needed, shallow copy/nested mutation, source-context
provenance for imported defaults, snapshot requalification and complete gates.

`udt-default-clippy-v2.log` passes strict workspace/all-target Clippy; structural
checks pass for 352 production Rust files. These are development checks only;
the ongoing object/default migration remains uncommitted with the acceptance
gaps listed above.

Copy/path continuation: the worktree now implements shallow copy through a
single evaluated temporary receiver and the existing UDT constructor HIR.
Nested field writes carry a structural AST/HIR path and mutate the referenced
child object. User-defined methods named `copy` retain precedence, including
postfix and static invocation forms. Native Chrome additionally confirmed
`Type.copy(object=receiver)` argument binding and RE10041 when copying a typed
undefined object; those source/DOM controls are retained in `corpus/udt-copy-*`.

The unchanged complete `udt-identity-control.pine` now matches 872/872 values
in each v5/v6 frozen native suite on the development CLI
(`udt-full-comparison.json`). Original 218-mismatch isolation evidence remains
unchanged. `udt-copy-targeted-final.log` passes 4 copy, 6 defaults and 4 identity
tests. `udt-copy-regression-v5.log` passes all five incremental fixture gates,
including the entire successful and expected-error runtime inventory. Strict
workspace/all-target Clippy passes in `udt-copy-clippy.log`.

Next convergence: run the full Windows gate and deliberately review old
constructor/null/history expectations, then rebuild retained wheel/WASM/CLI
and compare the complete native suites on those exact artifacts. Mixed
collection/object graph persistence, public graph transfer and resource
qualification remain open. Direct undefined-object writes and static copy of
an untyped `na` also need dedicated boundary controls. The broad product goal
and unchanged complete Pivot Points target remain outstanding; this is still
an uncommitted worktree, not distribution acceptance.

Acceptance convergence: old rejection tests for nested field writes and null
private-dependency fields now assert admission, preserving their original
sources. The 56-plot realtime fixture still checks every plot at initialization,
two forming updates, confirmation and the next bar; its expected table now
distinguishes ordinary-field rollback from replacement of a varip reference.
No fixture was skipped. A newly exposed recursion crash in `arrayLoop(...).copy()`
was repaired by stopping copy dispatch when receiver analysis fails; the original
recursive-function rejection test now passes without changing the stack limit.

The new eight-plot native array control freezes 872 values from origin and
matches the development CLI. It independently verifies reference-based includes,
indexof/lastindexof, shared `array.new` initial objects and shallow array copies.
Together with the earlier identity and live-varip controls, it explains the 27
UDT runtime snapshot changes: searches no longer match distinct equal-valued
objects, mutation is visible through array/loop aliases, and history aliases read
the referenced object's updated fields. `udt-snapshot-diff-audit.json` retains
the per-file numeric differences. An unrelated three-value last-bit math
regeneration was restored from HEAD. These snapshot changes are regression
expectations derived from the reference rules, not 27 separate native script
qualification receipts. The full gate and retained installed artifacts are
still being converged.

The completed `verify-udt-identity-v7.log` passes 6,762 Rust tests, 771
installed-wheel Python tests, 130 tool tests, strict Clippy/format/structure,
host parity and actual generated WASM. This includes new Python and WASM
identity and field-varip replica checks. All 2,247 semantic fixtures pass;
copy receiver failures no longer recurse or duplicate diagnostics. Retained
artifacts are the next step, and richer UDT graphs/requests/resources remain
explicitly outside this acceptance scope.

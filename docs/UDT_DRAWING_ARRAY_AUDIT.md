# Drawing-bearing UDT arrays — work in progress

The worktree following `ceb625d04` admits arrays whose same-identity UDT
elements contain scalar fields, drawing handles, or nested UDTs with those
fields. Drawing handle kinds are line, label, linefill, box, table and polyline.
Chart-point fields remain outside this array profile because their reference
semantics have not been migrated. Unsupported collection-valued field types
are not admitted by this change.

Array eligibility is separate from the existing scalar-tree test used for
varip. A drawing-bearing UDT array cannot acquire varip persistence merely
because ordinary array operations now accept its elements. Local/imported
declarations, typed UDF/method parameters, array result methods and imported
nested constructors use the new eligibility check. Internal classifications
are named SameLocal/SameImported rather than implying scalar-only elements.

Three targeted tests pass: local v5/v6 drawing/UDT aliases and realtime replay,
imported nested drawing objects through a typed UDF array parameter, and the
varip rejection boundary. Strict workspace/all-target Clippy and structure
checks pass. No new runtime storage implementation or host dependency was
needed; drawing IDs and UDT references retain their existing stores.

Chrome ran the unchanged `tests/fixtures/runtime/udt_drawing_array.pine` control
in v6. Its four plots verify shared line updates, label text changes, UDT field
replacement through an array copy, and size. The frozen monthly reference
contains 436 values on 109 confirmed bars from index zero, with forming month
excluded and OHLC checked against the existing input. The development CLI
matches all values. Source, DOM, CSV and manifests are retained under
`.local/product-completion-20260912/corpus/udt-drawing-array-*`; comparison is
`udt-drawing-comparison.json`. v5 was subsequently captured and also matches
436/436 values. Installed-artifact checks remain pending, and only line/label
behavior has direct native evidence.

The original Pivot Points source remains unchanged. Its latest inventory is
`corpus/pivot-after-drawing-array-analysis.json`: 39 diagnostics (previously
42). Six UDT-array declaration and three call-argument errors disappeared;
loop result and method-effect diagnostics became visible. Matrix support,
void/mutating methods, requested contexts and the rest of the product goal
remain open. This slice is uncommitted and has not passed the full gate.

The next continuation admits supported drawing builtins inside local modern
UDF/method bodies, using the existing side-effect permission helper while
retaining restrictions on declarations, plots, alerts and strategy calls.
A fourth targeted test creates, updates and deletes a line through a
UDT-returning UDF and methods. All four targeted tests pass. The 1,234 sema
unit tests passed before this method-body change; the broader gate must be
rerun against the final implementation.

`corpus/pivot-after-drawing-method-analysis.json` now has 33 diagnostics:
matrix declaration/capability errors, receiver/unknown-symbol cascades,
request restrictions and three duplicate observations of the same loop-return
gap. The exact loop gap is `affixOldPivots`, source line 144: a for-in body
ending with `if positionLabelsInput == "Right"` and a void label setter, with
no else branch. Next integrate conditional loop results through analysis,
type queries and lowering, then continue reference-bearing matrix support.

Conditional-loop continuation: loop tails now lower through one shared helper
which preserves their return values and prefixes. For/for-in/while analysis
admits conditional tails; function loops retain void drawing effects, while
value-producing loop contexts still reject void results. Three targeted tests
cover true/false drawing updates, selected scalar results, and a while prefix
that executes once with a skipped final branch returning na. They pass together
with the four drawing-array/method tests. Map type queries were moved into a
focused helper module to keep the structural guardrail intact.

`corpus/pivot-after-conditional-loop-analysis.json` now has 30 diagnostics and
no loop-return failures. Next implement reference-bearing matrix storage and
element-identity propagation for the original `matrix<pivotGraphic>` pipeline:
new/add_row/row/rows and shallow row/copy semantics must preserve UDT identity,
while numeric-only matrix operations remain type checked. Requested-context
semantics and full artifact qualification remain outstanding.

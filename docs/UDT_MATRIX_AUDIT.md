# UDT matrix implementation — work in progress

The runtime store now has `MatrixElementKind::UserType(index)`, where the
index identifies UDT metadata in the owning program. Matrix copies preserve
this tag and object references. Row/column arrays retain the element UDT name;
they have independent array slots while pointing to the same objects.
Runtime clones have independent object stores, preserving rollback isolation.

Empty 0x0 matrices now infer the first inserted row's width or column's height.
Dimension/index/cell-count checks run before state changes. Tests cover both
numeric and UDT storage and verify unchanged state after invalid insertions.
All 54 matrix-related runtime unit tests pass in
`.local/product-completion-20260912/udt-matrix-store-regression-v2.log`.

The v6 `corpus/udt-matrix-control.pine` native control independently confirms
row/object aliases, independent row-array slots, shallow matrix copy,
independent copied matrix slots and empty-matrix add_row behavior. Its frozen
reference has 872 values across 109 confirmed monthly bars from index zero;
CSV/source/DOM/manifest are under `corpus/udt-matrix-*`. This reference has not
yet been compared against a source-level local run, because UDT matrix language
admission is not implemented yet. The obsolete empty-object-copy probe was
removed from the chart after the account's 25-indicator limit blocked adding
this control; its evidence remains retained. No subscription change was made.

Next integrate a distinct UDT matrix type through semantic analysis and
lowering, with stable local/imported element identity for constructors,
declarations, aliases, get/set, row/column, add/remove and copy. Preserve
numeric-only operation restrictions rather than representing UDT matrices as
float matrices. Then run the unchanged native control and original Pivot
Points source, followed by v5 and actual installed-artifact qualification.
The broader drawing-array/method/conditional-loop changes remain uncommitted
and the full product goal remains active.

The constructor bridge now uses a distinct `ValueKind::UserTypeMatrix`.
Local/imported eligible UDT names are recognized in matrix type annotations
and constructor calls. Positional/named arguments are bound explicitly, omitted
dimensions default to zero, and initial values must have the matching UDT
identity or be na. Lowering emits the typed constructor call, and runtime
dispatch resolves the program's element metadata rather than a numeric kind.
Four targeted storage/constructor tests pass, including source-level execution
and malformed argument, non-integer dimension and mixed-identity rejection.

The UDT kind has not yet been added to generic matrix operation admission;
this keeps numeric operations from being inadvertently admitted. Next add
symbol/expression element-identity propagation and the supported structural
operation contracts (rows, get/set, row/col, add/remove/copy), then run the
unchanged native matrix control through the actual language path. This remains
an uncommitted intermediate implementation, not complete matrix support.

`udt-matrix-constructor-targeted-v2.log` passes five tests; the additional
boundary test rejects sum/determinant/average calls on a UDT matrix instead
of treating object handles as numbers. Workspace Clippy passed in
`udt-matrix-constructor-clippy.log`, and structural checks pass. No complete
native matrix-script comparison or installed-artifact gate is claimed yet.

Operation continuation: matrix symbols and expressions now carry UDT element
identity, including aliases and copy/transpose/submatrix results. Structural
matrix methods admit the UDT kind; get results and row/column arrays preserve
the element type. Writes, array insertion, matrix concatenation and typed
declarations reject mismatched identities. Operations needing element identity
reject unresolved identity instead of accepting an arbitrary UDT.

Appending a row/column supports an omitted insertion index, and removal returns
the removed values as a typed array. Context-dependent matrix argument binding
now normalizes sparse named parameters correctly; the old fallback caused an
out-of-bounds access with `add_row(array_id=...)`. The targeted test preserves
that regression. Matrix mutations in local modern UDF bodies are admitted using
the existing collection-effect helper.

The unchanged native matrix source now matches 872/872 v6 and 872/872 v5 values
on the development CLI (`udt-matrix-comparison.json`). Three source-level
operation tests pass, including full/realtime parity and negative type cases;
strict Clippy passes. `corpus/pivot-after-matrix-current.json` has 12 remaining
diagnostics: lookahead restriction, request-expression rejection and dependent
missing-symbol/for-in diagnostics. Next work is requested-context semantics,
alongside full regression and installed-artifact qualification for this batch.
General UDF matrix identity propagation, richer matrix histories/aliases and
resource/graph boundaries still require broader qualification.

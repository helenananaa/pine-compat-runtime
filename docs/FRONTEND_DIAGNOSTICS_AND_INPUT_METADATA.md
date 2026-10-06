# Frontend limits, source diagnostics, and input metadata

The 0.3 prerelease bounds recursive statement parsing at 32 levels and
expression parsing at 64 levels. Statements and inline `switch` arm expressions
share the structural depth counter. Both counters also share a weighted budget:
`2 * statement_depth + expression_depth <= 64`. Mixed nesting can therefore
reach the resource bound before either individual limit. The parser reports
`E_PARSE_STMT_DEPTH` or `E_PARSE_EXPR_DEPTH` at the entry that exceeds the budget,
and analysis produces no executable program.

Windows debug regressions run on 1 MiB threads. They cover excessive `else if`
chains, nested `if` statements and expressions, inline `switch` expressions,
parentheses, unary operators, and combined statement/expression nesting. Valid
16-level structural nesting and 32-level parentheses/unary nesting remain
accepted. These are embedding resource limits, rather than Pine syntax rules.

Analysis JSON retains schema version 6. Diagnostics from a supplied library
have three additional fields in `diagnostics[].span`: `sourceId`,
`libraryKey`, and `sourceName`. `start` and `end` are UTF-8 byte offsets in
that physical source; `line` and `column` are one-based, and columns count
Unicode scalar values. Root diagnostics retain the original span shape.

For example, an error in the first supplied library can have this span:

```json
{"start": 88, "end": 99, "line": 4, "column": 43,
 "sourceId": 1, "libraryKey": "audit/Library/1", "sourceName": "library.pine"}
```

Source IDs are local to one analysis and follow the normalized, sorted source
graph. They are not durable file identifiers. Hosts should use `libraryKey`
to select the supplied source and `sourceName` for display. CLI source names
are file names/paths; Python and WASM use `<python:key>` and `<wasm:key>`.
Compile/run error text also names the library. Clients that ignore additional
JSON fields continue to work with root and library reports.

The Rust syntax API adds `Diagnostic.source: Option<Box<DiagnosticSource>>`.
`Diagnostic::error` keeps its signature and initializes the root form. Code
constructing `Diagnostic` directly must add `source: None`; code displaying a
diagnostic should use `Diagnostic::line_col(root)` or `Diagnostic::format(root)`
so it respects library origins. This is a prerelease Rust API migration.

Compile-time input metadata resolves immutable constant symbol initializers, unary and
binary expressions, selected ternary branches, static builtin constants and
explicitly permitted pure scalar calls. These values use the same arithmetic,
casting, math, string and color semantics as execution. Input options are
atomic: if any option is unknown, analysis does not silently return a shorter
list. Source selectors and color defaults retain their string representation.

Constant evaluation has depth, symbol-cycle and step limits. It never executes
the script's blocks, loops, user functions, requests or chart-dependent calls.
Pure calls run only after every argument has become a scalar value, using an
empty program with a small execution budget. Chart-dependent formatting such
as `format.mintick`, implicit-timezone formatting, and unavailable values remain
unknown (`null` for a default/title and omitted optional constraints/options).
Reassigned aliases also remain unknown rather than exposing an obsolete
initializer. The evaluator does not simulate statement execution to resolve them.
String replacements and formatting also require a conservative output-size bound
before execution. A bound exceeding the string limit remains unknown, including
some valid large expressions whose exact result would fit.
Metadata extraction does not execute a bar or require a host data feed.

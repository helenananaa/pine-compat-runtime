# Explicit simple scalar parameters

2026-09-09. Slice verified locally, not released. Baseline 149d10c7a.

## Behavior

Modern v5/v6 local and imported functions accept explicit `simple int`,
`simple float`, `simple bool`, `simple string` and `simple color` parameters.
Their bound qualifier stays Simple, including when a constant is supplied.
Const/Input/Simple arguments are admitted; Series arguments are rejected.
Int-to-float promotion, typed na, scalar defaults, named arguments and separate
callsite history continue through the existing binding/lowering machinery.
Defaults are type/qualifier checked even if overridden. Existing explicit
Series bindings are preserved. Method/reference qualifier expansion is excluded.

A direct modern `input.*` argument at a global UDF callsite is evaluated in
the caller before inlining. The prior blanket side-effect check incorrectly
rejected this shape. Calls inside function bodies, local blocks and nested
effects retain their existing boundaries. No host/schema/broker changes.

## Independent evidence

Chrome original probes and source hashes are retained under
`.local/delivery-20260909/` in the main checkout:

- Input int through Simple into EMA compiles; simple float default 2 displays 2.
- Direct `f(input.int(3))` at global scope compiles in TradingView.
- Passing `close` to `simple float` rejects with CE10123.
- Returning `simple string` from a constant actual does not satisfy a
  const-only plot title; TradingView rejects with CE10123.
- `f(simple float x=close)` rejects with CE10170 even when called as `f(1)`.

These are compiler/type observations. They do not prove numerical equivalence
of full libraries or increase any strategy reference denominator. Official
parameter contracts: https://www.tradingview.com/pine-script-docs/language/user-defined-functions/

## Validation and remaining work

Grok produced the initial parser/sema patch in an isolated worktree with
finite turns, scoped permissions and no subagents. Its first implementation
attempt exhausted its 28-turn cap without editing. A bounded retry produced
five implementation files and three test files but ended without a final
verification report. Codex independently reviewed and tested the patch.

Initial runtime tests failed because canonical JSON writes integral numeric
values as integers; corrected original test expectations, with no changed
runtime output or widened numeric tolerance. Subsequent sema tests exposed
the direct input argument defect; corrected implementation after Chrome
confirmation, retaining those failing inputs. Added stronger stateful imported
callsite/forming rollback and negative effect controls.

Full regression then found an old negative fixture containing only
`plot(value(input.float(1.0)))`. The exact unchanged v5 source runs in
TradingView and displays 1. It is now explicitly reclassified under
`udf.direct_global_input_arguments`; its historical filename is retained.
An additional original output-call argument fixture preserves the actual
side-effect rejection. The old input fixture was not deleted or simplified.

Targeted simple/default/series regressions pass. Final non-update full Windows
gate exited 0 (`simple-full-verify-v5.log`): 6,567 Rust tests, 677 tests against
a freshly built/installed Python wheel, 101 tool tests, real WASM/Node smoke,
879 registered CLI snapshots, 583 required runtime and 5 legacy-analysis
goldens. Earlier failures and fixes are retained, including the missing WASM
assertion/manifest ordering and an initially nonexistent Node helper; the final
Node check reads the fixture/golden explicitly and compares complete JSON.
Grok's bounded read-only follow-up reported no findings in the named scope,
but did not inspect the assignment lattice/effect helper or run tests; Codex
inspected those helpers and performed the actual verification.
Existing runtime goldens
must remain unchanged; only the new simple runtime fixture and conformance
matrix may be added/updated. Matrix changes comprise two new capability rows
and the explicit fixture reassignment described above, not hidden denominator
changes to any frozen reference corpus.

Full original TechnicalRating/3 plus ta/9 and exact RelativeValue/3 remains
blocked. The new dependency source uses `// @version=6`, which the current
lexer misclassifies as implicit v1; this causes version and parser errors.
Chrome confirms that spelling selects v6. An independent original two-level
import probe also proves missing transitive function resolution. These remain
separate follow-up tasks; no library was trimmed or substituted.

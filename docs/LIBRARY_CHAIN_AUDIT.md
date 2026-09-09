# Complete library-chain qualification

Current execution owner: Codex directly. Historical external-agent reports are
leads only; delegation/resume authorization is revoked. Existing evidence is
retained under `.local/delivery-20260909/` in the main checkout.

## Grammar intake — 2026-09-09

Baseline 6e68e9db1. The exact TechnicalRating/3, ta/9 and RelativeValue/3 sources
were parsed separately, so diagnostics retain each physical source's line
numbers. TechnicalRating already parsed. ta/9 failed at inline switch-arm
reassignment; RelativeValue failed at `array<float>` / `array<int>` UDT fields,
with recovery concealing a series-qualified reference parameter in a method.

The parser now represents collection-typed fields, preserves series reference
annotations using the existing typed-parameter parser, and maps inline `name :=
expression` switch arms to ordinary one-statement blocks. Expression arms keep
their prior AST shape and newline checks. No third-party source was edited.

All three complete sources now parse with zero syntax errors. This does not
claim complete semantic admission or execution. Collection-bearing runtime UDTs
remain rejected with E_UDT_FIELD_TYPE; method capabilities, defaults, unknown
parameter types and function/global mutation checks were not removed.

Original tests cover full declaration structure, malformed syntax, existing
reference type checks, global-mutation rejection, switch/block equivalence,
state persistence, append and forming replacement/confirmation. The new
original runtime fixture is compared through CLI, installed Python wheel and
real WASM/Node. Existing runtime goldens were unchanged; only the new fixture
and two conformance entries were added.

Final non-update Windows gate exited 0: 6,596 Rust tests, 678 installed-wheel
Python tests, 101 tool tests, structural/parity checks and actual WASM/Node.
Log: library-grammar-full-verify-v2.log. The earlier gate failure came from a
test's diagnostic-prefix string being mistaken for an emitted code; the test
now asserts the actual E_UDT_FIELD_TYPE instead of adding a fictitious code.

## Remaining chain work

- Bind nested imports in their owning module, preserve builtin namespace
  fallback, private visibility and distinct source contexts.
- Admit legitimate overload identities; keep duplicate/ambiguous declarations
  and invalid invocations rejected.
- Distinguish valid-but-unimplemented export capabilities from invalid source.
  Do not simply drop declaration/body checks: an original Chrome probe proved
  that an unused function with an undeclared name still fails in TradingView
  (CE10272; unused-function-probe.pine and its DOM receipt).
- Execute the unmodified root with all three exact sources, then compare the
  frozen 21,133 closed bars and three rating outputs. That gate is still open.

Current complete-root diagnostics after grammar correction are retained in
technical-after-grammar.json: two unsupported-export effect diagnostics, one
duplicate-export diagnostic, and four unresolved nested function names with
their consequent unknown-variable cascade. These are not 42 independent bugs.

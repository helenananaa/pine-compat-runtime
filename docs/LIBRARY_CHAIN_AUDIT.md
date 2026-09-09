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

## Source-scoped transitive imports — 2026-09-09

Baseline 7aa527a19. Nested exported function calls now bind in the owning
library's import scope. Public root aliases remain distinct; dependencies use
one internal context per referenced source, avoiding path expansion for diamonds.
Internal function keys cannot be spelled by source, preventing caller aliases
from capturing a library's builtin calls. Exported namespace members take
precedence, with builtin fallback for absent members. Imported method bodies
receive the same owning-module bindings.

Original tests exercise independent state through two root aliases and a diamond,
historical/append/forming parity, builtin shadowing, method calls into a dependency,
private access, missing libraries, duplicate/missing aliases and import cycles.
The transitive runtime fixture supplies both complete original test libraries
through CLI, Python, WASM Rust tests and generated WASM/Node.

The first full gate caught missing dependency bundles in the incremental/realtime
fixture harnesses; these now supply both libraries instead of excluding the
fixture. The second caught a verifier blind spot: the host-parity scanner omitted
the CLI library-triple registry. It now reads complete dependency tuples, ignores
comment/string examples, and tests missing registration and missing host assertion
failures. Existing paired runtime_import and runtime_import_state snapshots are
now explicitly required too; no old expected outputs changed.

This is not a general nested-UDT or full library execution claim. The complete
unchanged TechnicalRating root now reaches three remaining declaration errors:
two export-effect diagnostics and the duplicate-export diagnosis for RelativeValue
overloads (technical-after-transitive.json). The previous unknown-call cascade
is gone. Final full gate exited 0: transitive-full-verify-v3.log, 6,605 Rust tests,
679 fresh installed-wheel Python tests, 103 tool tests, structural/parity checks
and actual generated WASM/Node smoke. This remains Windows debug qualification,
not the final release/Linux artifact matrix.

## Imported scalar overload qualification — 2026-09-09

Baseline 928efd0c3. Exported overload declarations now retain every distinct
signature and body. Matching includes scalar type and qualifier, named argument
binding and omitted defaults. Exact int/float kind and the weakest compatible
qualifier dominate conversions; incomparable candidates retain declaration order.
The candidate uses the same selection in analysis and scalar lowering/type queries.
Overload groups are excluded from pure-expression deduplication until its keys
can describe the selected declaration reliably. Ordinary non-overloaded functions
retain the existing path.

Independent Chrome controls (source and DOM receipts under the same evidence
directory) extend the prior intake:

- overload-const-probe and overload-precedence-probe: const float chooses simple
  float over series float in both declaration orders (11); int chooses int (101)
  and float chooses float (201), also after reversing declaration order.
- overload-ambiguous-probe and overload-order-probe: crossed int/float signatures
  called with two ints choose the first declaration (102 versus 202 after reversal).
  The initial candidate treated these incomparable costs as an error; this was
  corrected before acceptance. The probe filename is historical, not a claim that
  the native call is ambiguous.
- overload-combined-probe: combined original library/control compiles and shows
  bool=4, string named/default=5, simple=11, int=101, float=201, crossed-first=202.
  Visible live accumulation values are not a matched historical numerical oracle.

Original regressions additionally check separate persistent state, negative
duplicate signatures/unmatched calls, retained invalid-body checks on unused
overloads, and explicit rejection of reference results. The new scalar_overloads
fixture participates in historical/append/forming tests and CLI/Python/WASM/Node
golden comparisons. No existing runtime golden is updated.

This slice admits complete library declarations but only executes scalar-parameter,
scalar-result imported overloads. Reference parameters/results and root-local
overload registration remain outside the executable claim. Full original
TechnicalRating now has only the two RelativeValue export-effect diagnostics
(technical-after-overload-candidate.json); its numerical acceptance is still open.
The first full Windows gate passed runtime regressions but failed the existing
1,200-line modules.rs structural limit. Signature comparison and overload
parameter recontextualization moved into the existing function_parameters module;
the limit was unchanged. Final gate overload-full-verify-v2.log exited 0:
6,609 Rust tests, 680 installed-wheel Python tests, 103 tool tests and real
generated WASM/Node. The six constant
native controls separately match all corresponding local fixture values:
overload-constant-comparison.json. This does not qualify Linux/release artifacts
or the full TechnicalRating numerical reference.

## Owned-field admission and first complete-root execution

Candidate after ac329c10e: exact complete root first analyzed with
zero diagnostics, then ran all 21,133 frozen bars with all three exact sources
unchanged. This is a candidate result, not qualified integration.
The frozen comparison (technical-owned-fields-comparison.json) FAILS: ratingTotal
164 mismatches, ratingOther 164, ratingMA 1, across 63,399 values with unchanged
1e-9 absolute/relative tolerances and zero skipped warmup. Differences extend
beyond initialization; MA differs at index 15460. Runtime capture:
technical-owned-fields-runtime.json. Do not call the full-library oracle closed.

The ownership candidate checks fresh array-field constructors, fresh conditional
replacement and narrowly recognized local container helpers. Negative tests caught
an initial conditional-alias hole; conditional/grouped aliases, borrowed receiver
replacement and helper field/parameter replacement remain rejected. The follow-up
review and final qualification are described below; this changes declaration
admission only, not execution support for collection-bearing UDTs.

Follow-up review extended negative coverage to return/element aliases, custom
push/size methods, implicit container receivers, expression-block field writes,
helper global assignments and free reassignment of helper results. Intermediate
full Windows gates owned-fields-full-verify.log and owned-fields-full-verify-v2.log
passed, but further source review caused another tightening; neither is the final
receipt for the current candidate. owned-fields-full-verify-v3.log exited 0:
6,611 Rust tests, 680 installed-wheel Python tests, tools and real WASM/Node.
Constructor review then added exact positional/named binding, duplicate/unknown
argument rejection and rejection of unknown calls in scalar constructor fields.
Recognized scalar time/math/string calls are read-only; imported array/math/string
namespace shadows and local time-function shadows invalidate that proof. Array
fields must be freshly allocated. Reassignment only replaces owned roots with
fresh allocations; helper mutations stay on the local receiver, helper scalar
reassignment stays local, and reference aliases/escapes remain rejected.
Original regressions include both named scalar-field initialization and constructor
escape/binding failures, including an imported factory named array.new_float.
The final gate owned-fields-qualification-verify.log exited 0: 6,613 Rust tests,
680 installed-wheel Python tests, 103 tool tests and real generated WASM/Node.
A freshly rebuilt CLI also reports the complete original root executable with
zero diagnostics (technical-owned-fields-qualified-analysis.json). Existing
runtime goldens are unchanged. This qualifies Windows declaration admission,
not the failed full-library numerical comparison or Linux release artifacts.

The next numerical defect now has independent evidence: macd-seed-control.pine
uses source bar_index+1 and ta.macd(12,26,9). TradingView's retained DOM reports
first MACD index 25, first signal index 33, both seed values 7. The same local
control on 64 bars reports indices 0/0 and seeds 0/0 (macd-seed-control-before.json).
At that point no MACD implementation change had been made. This explains an initialization
defect, not all full-root discrepancies: there are 129 oscillator mismatches after
index 200, each exactly one rating contribution (+/-1/11). Their diagnostic list
is technical-oscillator-difference-indices.json; the full acceptance denominator
remains all 63,399 values.

MACD follow-up native controls are retained for implementation: repeated calls
on one bar agree with the last source sample only (macd-repeat-control, lengths
2/3/2, first indices 2/3 and seeds 5/5). Alternating missing sources seed at
indices 5/7, return na on the missing bar 10, and resume MACD=1 on bar 11
(macd-missing-control). DOM receipts accompany both original control scripts.

Next isolate the
oscillator initialization/scattered differences and the single MA decision
difference with independent component controls. Preserve original source and
frozen root reference; diagnostic probes are additional evidence only.

## Remaining chain work

MACD candidate after owned-field admission: each of its three EMA channels now
uses a non-na SMA seed and a committed per-bar base; repeated calls replace the
current sample and missing sources discard tentative samples while retaining the
previous committed value. The three native controls match through the candidate
CLI (macd-{seed,repeat,missing}-control-candidate.json). Two new runtime tests
cover seed/missing behavior plus historical/append/forming equivalence.

The original 21,133-bar root rerun reduces ratingTotal/ratingOther/ratingMA
mismatches from 164/164/1 to 130/129/1, with the first remaining difference at
index 487. All 63,399 values remain in acceptance, with unchanged 1e-9 tolerances
(technical-macd-candidate-comparison.json). This is still a failing oracle.

The initial runtime regression run exposed five old first-bar-seed expectations.
They were corrected against the independent seed evidence; short tuple tests
were extended by a bar to exercise the signal, and the higher-timeframe provider
test retains its original bars and appends two provider/four chart bars to test
actual post-warmup alignment. Runtime library tests then passed 1,797/1,797
(macd-runtime-regressions-v2.log). Golden acquisition is separate from test
acceptance (capture-macd-goldens.ps1); do not refresh all snapshots.

The MACD slice passed final Windows qualification in macd-final-verify-v2.log:
6,615 Rust tests, 680 installed-wheel Python tests, 103 tool tests, structural
checks and actual generated WASM/Node (including MACD output). Six reviewed
goldens changed only MACD values and downstream NA history; runtime_math's
unrelated last-bit differences were excluded (macd-approved-golden-deltas.json).
Native v4 and v5 seed controls agree with v6. Request-integration assertions
were corrected in all three hosts, including higher-timeframe warmup. An initial
WASM expectation used JSON float zero instead of the serializer's integer zero;
that test-format issue was corrected, without changing runtime serialization.

The next component control is technical-component-probe-487.pine, a separate
diagnostic copy, retaining the exact original imported libraries. Native and local
time/close agree at 1770342300000 / 65715.2, and native copied/original ratings
agree. Only oscillator component 7 differs (native 0, local -1): local Stochastic
RSI K=98.55303933471674, D=98.55303933471681, yielding a negative difference
of 7.105427357601002e-14 and a sell condition. Native displayed values round to
the same 12 decimal places. This locates the branch divergence but does not yet
establish the precise underlying floating-point calculation or fix. Do not add
an epsilon to Pine comparisons or relax frozen rating tolerances.

Independent overload intake at baseline 1e1638123 (Chrome, original unsaved
controls; raw source and DOM receipts in the existing local evidence directory):

- overload-declaration-probe-v2: bool versus string overloads compile and execute,
  with constant visible outputs 4 and 5. The initial probe's spaced library title
  was rejected with CE10292; its source/receipt remain preserved, not counted as
  an overload failure.
- overload-qualifier-probe: simple float versus series float declarations compile
  and a series close argument executes. This disproves treating base-kind equality
  alone as a duplicate signature. It does not establish dispatch for const/input
  arguments, which could fit both overloads.
- overload-duplicate-probe: two series float signatures are rejected with CE10110
  (same parameters). Keep this negative behavior in the implementation.

Implementation must preserve every overload rather than replacing the previous
HashMap entry. Resolve arguments/defaults against candidates before lowering and
keep declaration identity, diagnostics and inlined callsite state consistent.
Named/default arguments and qualifier-conversion preference need explicit
controls before making general dispatch claims. No overload implementation is
included in the transitive-binding commit.

- Replace the two overly broad
  export-effect rejections with a precise proof for fresh local UDT-owned arrays.
- Distinguish valid-but-unimplemented export capabilities from invalid source.
  Do not simply drop declaration/body checks: an original Chrome probe proved
  that an unused function with an undeclared name still fails in TradingView
  (CE10272; unused-function-probe.pine and its DOM receipt).
- Execute the unmodified root with all three exact sources, then compare the
  frozen 21,133 closed bars and three rating outputs. That gate is still open.

Historical complete-root diagnostics after grammar correction are retained in
technical-after-grammar.json: two unsupported-export effect diagnostics, one
duplicate-export diagnostic, and four unresolved nested function names with
their consequent unknown-variable cascade. These are not 42 independent bugs.

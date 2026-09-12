# Requested-context completion — work in progress

The unchanged Pivot Points request combines ticker modification, a local
timeframe-change dependency, two pivot-level arrays, a persistent counter and
lookahead_on. This requires more than admitting an option string.

The worktree now validates both off/on choices for modern gaps/lookahead and
actually applies those options in runtime. Historical lookahead_on aligns by
requested opening time; lookahead_off aligns by closing time. Gap modes select
new observations versus forward filling. Legacy warning behavior is retained
without labeling modern calls as legacy calls.

Realtime handling was also corrected for supplied developing requested bars.
With gaps_off an available developing value is used during both chart forming
and realtime confirmation; gaps_on excludes the unconfirmed requested sample.
A forming requested bar whose opening lies after chart time is not exposed.
Historical chart updates exclude forming requested input. This distinction is
described in the [official data-request documentation](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/).

`modern-request-merge-live-v2.log` passes the historical policy combinations in
v5/v6 and modern realtime policy tests. `modern-request-feed-regression-v3.log`
passes three feed/rollback tests. One former expectation that realtime chart
confirmation discards a developing requested value was replaced, while adding
an explicit historical-update exclusion assertion. Strict Clippy passes. Native
multi-timeframe chart capture and broad qualification remain pending.

The current original-script inventory, `corpus/pivot-after-modern-merge.json`,
has 11 diagnostics: one unsupported request expression plus dependent symbols
and loops. Next implement request result transport for scalar arrays/tuples:
snapshot requested arrays at evaluation time, preserve element kind, and allocate
caller-owned results rather than returning child-local array IDs. Then admit
the relevant local dependency graphs and persistent UDF state using requested
context evaluation, not caller-context captures. Richer object graph transfer,
execution-time availability edges, resource accounting and final artifact gates
remain open. No full-script success or distribution acceptance is claimed.

Scalar-array transport continuation: request samples now use a private owned
`RequestedValue` representation. Scalar arrays are copied at each requested
bar evaluation, preserving their element kind and contents; the caller gets
fresh arrays when a selected sample is materialized. This prevents child-handle
collisions, later requested mutations and caller writes from corrupting cached
samples. Reference-bearing arrays/objects still fail explicitly. Modern source
admission includes selected scalar-array constructors, pivot-level arrays with
admitted arguments, and supported array/scalar tuples. Legacy expression
admission remains separate. Requested bar_index is evaluated in its context.

An initial proposal to deduplicate duplicate array slots inside a returned
tuple was disproved by Chrome. `request-array-alias-control-v2.pine` requests
`[a,a]`, then changes the first returned array; explicit observer plots confirm
the first array becomes -123/-456 while the second remains the requested price,
for both same-timeframe and higher-timeframe calls. The native source/DOM are
retained in `corpus/request-array-alias-*`. The implementation now creates an
independent copy per returned slot. The earlier internal test log reflects the
rejected assumption and is not acceptance evidence.

The corrected isolation test and a source-level array/tuple request test pass;
209 existing request tests passed after transport integration. Native evidence
currently covers v6 return-slot independence, not the complete Pivot Points
request. Next implement local dependency reconstruction and stateful requested
UDF evaluation, with additional source/native checks before broad qualification.

`request-array-regression-final.log` passes 230 request-related tests, including
the corrected per-slot copy behavior, caller/child isolation and request-cache
parity; `request-array-clippy-final.log` passes strict Clippy. The native probe's
observer plots explicitly confirm that writes to the first slots took effect,
so independence is not inferred merely from an unchanged second slot.
The source profile intentionally has not admitted arbitrary array captures or
stateful UDFs yet. Before doing so, audit local declarations in requested HIR
blocks so dependency preparation cannot execute their initializers twice or
outside a conditional branch, and distinguish captured static selector/input
values from series dependencies that must be recomputed in the requested context.

Modern dependency/UDF continuation: the source analyzer now admits immutable
scalar dependency graphs, static scalar captures, simple/input selectors and
selected UDF bodies with local persistent scalar state and conditional blocks.
Requested UDF-local scalar arrays can be returned as independent tuple slots.
External mutable or series-persistent variables, collection/reference captures,
nested requests, recursion and drawing/output side effects remain rejected.
Static persistent scalars are captured only when their qualifier is below series
and no reassignment is present. This is a bounded profile, not general UDF or
object-graph request support.

The existing HIR dependency collector already excludes block-local declarations
and loop bindings from external dependencies. A new conditional initializer test
checks that a nested stateful function is not executed outside its branch or
twice by dependency preparation. Additional v5/v6 tests verify per-slot array
copies, requested-bar counter progression and recomputation of a chart series
alias against requested data. Tuple type inference now follows the expression
with four/five request arguments as well as three; otherwise a valid lookahead
argument broke tuple declarations.

`modern-request-runtime-all.log` passes 233 core request tests plus matching
integration tests. `modern-request-sema-final.log` passes all 1235 semantic tests;
`modern-request-syntax-final.log` passes all 125 parser tests. Strict Clippy passed
in `modern-request-clippy-v2.log`; the later edits only correct test expectations.
Old rejection tests for newly admitted capabilities now assert successful HIR,
while unsafe request capture/output controls remain negative. Registry coverage
also exposed missing parser chaining for matrix.remove_row/remove_col array
results; those producers are now recognized.

The unchanged original Pivot source in
`corpus/pivot-after-modern-dependencies-v3.json` now has three diagnostics, all
at label.new style on line 165. Its requested tuple is admitted. The remaining
style comes from a switch-returned tuple and requires tracing its finite string
domain. This is compile progress only: full-script runtime behavior, native
multi-timeframe parity and the full same-revision artifact gate are still pending.

Full Pivot continuation: tuple destructuring now retains analysis-only source
provenance for finite string-domain validation. The domain evaluator projects
the selected tuple element through conditional/switch expressions and proves
exhaustiveness when a bounded string selector is fully covered without a default.
Unknown, invalid and reassigned styles remain rejected. The source spans and
library context are preserved; the projected tree is never lowered or executed.
Two dedicated tests cover v5/v6 branching, repeated forming rollback, invalid
styles, incomplete selector coverage and reassignment. All 1235 semantic tests
and strict Clippy passed for that change.

The unchanged original Pivot source now analyzes with zero diagnostics
(`corpus/pivot-after-tuple-style.json`) and runs with supplied annual data. The
initial monthly-to-annual aggregation was only an execution smoke, superseded
for differential checking by native exported request values.

Chrome CSV `BINANCE_BTCUSDT, 1M (15).csv` was frozen as
`corpus/pivot-native-v6-monthly-raw.csv`. The observed script preserves the entire
original body and appends 29 read-only plot columns: eleven static pivots, eleven
developing pivots, the period counter and six annual request input fields.
`freeze-pivot-reference.py` checks 110 contiguous origin indices, excludes forming
index 109, checks annual provider snapshots are stable across historical rows,
and records hashes in `pivot-native-manifest.json`. `compare-pivot-reference.py`
matches **3161/3161** values on 109 confirmed monthly bars, with 1e-8 absolute /
1e-10 relative tolerance and matching missing values. This is v6 Traditional/Auto
historical snapshot evidence, not every Pivot option or a realtime feed oracle.

The original and observed executions have identical non-plot outputs (99 lines,
99 labels), and original batch/incremental/realtime-history results match exactly
in `pivot-native-execution-parity.json`. The first realtime-history attempt failed:
CLI had fed each historical bar as an individually complete history, triggering
the original script's insufficient-history error on bar zero. Both CLI realtime
modes now initialize their known historical segment with the existing atomic
seed API, preserving the historical end boundary and execution clocks. A CLI
regression covers early errors and final-history flags; all 234 CLI tests pass.
The failed first artifact remains retained, alongside the successful v2 output.

The comprehensive verifier stopped in `pivot-completion-full-verify.log` at the
semantic fixture gate: 2209 passed, 38 failed. Several are old unsupported
expectations for newly admitted drawing/matrix/request behavior, or old reason
strings, but they require individual review. A genuine admission regression
was also identified: qualified color constants such as color.red were omitted
from the new modern request profile. The repaired case passes the pre-existing
return-qualifier fixture in `request-color-regression.log`. No comprehensive pass is claimed;
WASM/installed-wheel gates were not reached in this run.
Visual drawing parity, broader input/dialect/feed scenarios and newly built
distribution artifacts remain unqualified until their own evidence is captured.

Fixture reconciliation: `pivot-fixtures-final.log` passes all 2247 semantic
fixtures. Reviewed matrix UDF/drawing-bearing UDT inputs now require executable
HIR; mixed invalid-call fixtures retain their argument/type/unknown-method
diagnostics. Imported drawing-bearing varip arrays still fail with the explicit
varip scalar-tree restriction. Obsolete reason strings and diagnostic counts
were updated only after comparing actual messages with the retained controls.
Another inconsistent path was fixed: direct matrix call-result mutations now
use the same UDF admission rule as named matrix receivers. A v5/v6 runtime test
checks a removed row from a copied matrix and preservation of the original
matrix, plus a temporary matrix fill (`matrix-call-result-udfs.log`). The full
verifier was restarted as `pivot-completion-full-verify-v2.log`; the prior failed
log remains retained.

Delivery metadata audit still pending: `tests/fixtures/conformance.tsv` and its
generated matrix snapshot/CONFORMANCE table retain older request and UDT wording
(non-default merges/local UDFs unsupported, UDT structural equality/value-copy
language, and pre-matrix-UDT boundaries). Update these from current evidence
before delivery; passing a snapshot-consistency test alone would not establish
that those descriptions accurately describe the new runtime. Historical fixture
filenames containing `unsupported` are retained regression inputs, not capability
claims; their revised Rust gates now establish their current expected outcomes.

The second full gate reached WASM Rust tests and stopped on two old contracts:
the unsupported-analysis snapshot differed only in the request diagnostic message
and reason, and the request-feed test expected realtime confirmation to drop the
developing sample. The snapshot was updated after field-by-field comparison.
The WASM test now retains that sample on realtime confirmation and explicitly
checks that historical replay excludes it. All 708 WASM Rust tests pass in
`pivot-wasm-rust-tests.log`. A standalone structure check found calls.rs at 1503
lines; its matrix type helpers were moved to the existing calls/matrices.rs module
without raising the threshold. Host parity passed (940 registered snapshots,
591 runtime and five legacy-analysis assertions). Full verifier attempt three
was `pivot-completion-full-verify-v3.log`; it caught two missing imports from the
helper move before running tests. Those imports were fixed and the active rerun
is `pivot-completion-full-verify-v4.log`. No full pass is claimed yet.

The fourth full Windows verifier completed successfully: **6790 Rust tests,
771 installed-wheel Python tests, 130 tool tests, structure/host-parity checks,
and actual generated WASM under Node**. The verifier installs a newly built wheel
in a separate temporary environment; these are not source-only Python results.
The temporary verifier artifacts are not retained distribution assets.

After this gate, nine capability-table descriptions were reconciled with current
request/object/matrix/function semantics, preserving the fixture indexes. The
current CLI regenerated `tests/snapshots/matrix.json` and the CONFORMANCE matrix;
`metadata-cli-tests-v2.log` passes all 234 CLI tests, including metadata/snapshot
consistency. Historical phase notes are explicitly subordinate to current matrix
entries and artifact evidence. Retained same-commit CLI/wheel/WASM builds and
full-script cross-host reference checks are the next delivery step. Broader Pivot
options, visual qualification, realtime feeds and resource/host scope remain open.

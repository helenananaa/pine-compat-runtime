# Independent runtime product completion

Started 2026-09-12 from clean `cc11124f2`. User authorized implementation and
Chrome-based TradingView reference capture. No external publication is authorized.
The delivery-status entry point remains `DELIVERY_ROADMAP.md`.

## Acceptance and scope

Complete a locally consumable, host-neutral v5/v6 standard-candle runtime for
a frozen set of complete scripts. Do not claim arbitrary Pine compatibility.
Keep prior source, corpus, reference denominators and artifact receipts intact.
New synthetic controls supplement complete scripts; they do not count as public
real-script adoption. Missing references remain unverified, not passed.

## Ordered work ledger

| Work | Acceptance | State |
| --- | --- | --- |
| Complete-script compatibility | Freeze original sources/dependencies with hashes; measure admission and execution before changes; repair high-impact combinations with independent controls | RSI configurations accepted at the retained checkpoint; original Pivot now compiles/runs and its v6 Traditional/Auto snapshot matches 3161 native values; broader options, visual output and new artifact qualification remain open |
| Execution and requests | Preserve existing native references; capture independent new cases for admitted combinations; distinguish unavailable tick path information from implementation defects | Request arrays/dependencies/local state and merge policies implemented with bounded controls; original Pivot batch/incremental/realtime-history agree; full-script forming-feed qualification remains open |
| Host capabilities | Describe supported input/account profiles and readiness checks precisely; implement deterministic capabilities demanded by frozen scripts | Modern merge-policy discovery defect repaired in schema 2; full qualification and full-script readiness checks in progress |
| Resources | Freeze finite workload, latency and memory budgets before acceptance; qualify Windows/Linux and multiple independent sessions; report full snapshots separately | PRODUCT_STANDARD_CANDLE_RESOURCE_PLAN.json frozen before measurements; acceptance not evaluated |
| Unified local distribution | Build selected Rust/CLI/Python/WASM outputs from one committed implementation; run installed-wheel, actual-WASM and retained independent reference gates | Windows/Linux debug artifacts at 4f32668a7 pass retained reference gates; current discovery repair and optimized resource/distribution acceptance remain open |
| Consumption and closeout | Match version, source identity, hashes, installation examples, schema migration and evidence; retain explicit unsupported profiles | Pending |

2026-09-23: [complete-script native reference expansion](SCRIPT_COVERAGE_20260923.md)
admits three additional unchanged built-in v6 indicators on the named monthly
and daily BTCUSDT inputs. Their six script/settings/timeframe runs match 4,486
observable TradingView plot positions with zero mismatches at the stated
tolerance. Batch, incremental, and historical realtime full outputs agree on
the daily input. This is local CLI coverage at `f249b3db`, with source/export
inputs retained outside Git; visual, forming-feed, and distribution gates remain
open.

The follow-on [community-script receipt](COMMUNITY_SCRIPT_COVERAGE_20260923.md)
separately records two admitted v5 community scripts, two failed admissions,
native plot/trade comparisons, and a cash-market-order slippage sizing repair.
Its USD-quoted strategy comparison is inside the same-currency product scope;
the USDT-quoted comparison exposes the external-FX boundary.

Account expansion and indefinite-memory claims must not be silently substituted
for the selected standard-candle scope. The core may expose deterministic
configuration and capability contracts; data feeds, persistence and scheduling
remain host responsibilities. Resource exhaustion must preserve the documented
failure/atomicity contract rather than silently dropping script-visible history.

## First implementation slice

Historical checkpoint: `PIVOT_NA_AUDIT.md` records the complete Windows gate
(6,730 Rust / 762 installed Python / 130 tools / actual WASM). The unchanged
RSI default 218 values and separate Bollinger/divergence 852 values pass with
cross-host full-output parity; the earlier 2/852 failure remains preserved.
Runtime schema 9 and changes schema 4 apply to current source; earlier artifacts
retain their original schema/source identities. Next prioritize the unchanged
Pivot Points Standard blockers instead of replacing it with an easier script.

The following slice notes preserve the chronology and are not current failures.

Investigate modern UDF mutations of caller-owned arrays. The existing runtime
supports array mutation and UDF `array.unshift`, while `array.push` on a UDF
parameter is explicitly rejected. Verify source semantics independently before
changing admission. Cover caller aliasing, nested calls, conditional execution,
historical/incremental/realtime rollback, and retained rejection boundaries.
Record broader full-script selection separately from this diagnostic control.

2026-09-12: local UDF push is implemented and Windows-qualified; see
`UDF_ARRAY_PUSH_AUDIT.md`. Complete official RSI (6,244 characters) and Pivot
Points Standard (13,218 characters) were captured without edits via Chrome.
Baseline analysis finds RSI declaration timeframe, gradient fill, input active,
and input-controlled output metadata gaps. Pivot Points adds parser/UDT/method
and requested-context gaps. Both remain failure inventory, not accepted scripts.
Next slice: admit and model the RSI metadata/declaration combinations, then
gradient fill with versioned output/streaming representation and native checks.

The versioned metadata/declaration slice passes the full Windows gate (6,724
Rust / 758 installed Python / 130 tools / actual WASM), see
`INPUT_METADATA_AUDIT.md`. Native v5 rejection of active and input-qualified
editable is preserved. RSI now has six remaining gradient-fill diagnostics.
User identified downloads under `I:\sys\下载`; the supplied one-minute CSV is
retained, with its missing initialization prefix explicitly recorded.

## Next continuation: complete official RSI

Monthly native reference is now ready, frozen by
`.local/product-completion-20260912/freeze-rsi-reference.py` and
`corpus/manifest.json`: 110 raw monthly bars with native indexes 0..109,
109 confirmed bars after excluding the current month, 218 numeric reference
values, no warmup exclusion, absolute/relative tolerance 1e-9. Keep the
one-minute incomplete-prefix CSV separate. All downloads are in the user-named
`I:\sys\下载` directory; no browser download-history access is needed.

Remaining RSI admission errors are the two gradient-fill overloads. Implement
the real vertical gradient contract (top/bottom values and colors per bar,
masked by the two plot series), not a solid-color approximation. Cover overload
binding, missing/duplicate/wrong arguments, full output, incremental deltas,
replica parsing, physical output retention and rollback. Determine explicit
schema migration before adding wire fields/actions; preserve old-format reader
behavior where supported, and do not silently ignore gradient data. Update
current consumption docs and retain historical audit source identities.

After implementation, run the unchanged official RSI against the frozen
monthly inputs, compare all 218 values including initial nulls, and exercise
alternate smoothing/divergence inputs separately. Do not call the entire RSI
capability qualified from only its default two numerical plots. Pivot Points,
broader request/account contracts, resources and unified distribution remain
open in the ledger above. Goal remains active.

2026-09-12 gradient implementation is qualified for the default RSI source:
218/218 native values and complete four-mode CLI/installed-Python/actual-WASM
parity; 6,729 Rust / 760 Python / 130 tools. See `GRADIENT_FILL_AUDIT.md`.
The separately frozen SMA+BB/divergence configuration has 2/852 mismatches at
the first RSI pivot high/low. Preserve that failed report while collecting the
new `corpus/pivot-na-reference.pine` native control. Next work is the actual
`eval_pivot` null-window behavior in `crates/pine-runtime/src/builtins/ta/pivots.rs`.

That pivot defect is now repaired and qualified; see `PIVOT_NA_AUDIT.md`.
Next continuation should inspect the original Pivot Points line 194 chained
`.row(...).last().pivotLine.get_x2()` expression and its parser/semantic lowering,
then continue through reference-bearing UDT arrays/matrices, method side effects,
function-final tuple declarations and requested-context requirements. The source
must remain unchanged; partial admission is not completion of that target.

2026-09-12: the [member access slice](MEMBER_ACCESS_AUDIT.md) now removes that
parser obstruction. The full Windows gate passes 6,745 Rust / 766 installed
Python / 130 tools / actual WASM. Retained CLI/wheel/WASM match 654 native v6
and 981 native v5 member-control values and both unchanged RSI references
(218 and 852); complete outputs and Python streaming replicas agree.
Native undefined-object errors required preserving 29 old fixtures as negative
cases alongside guarded positive variants, not discarding their regression
coverage. General mutable UDT reference semantics remain unqualified here.

Next continuation: implement and independently verify function-final tuple
declarations (including branch results and discarded bindings), then continue
the remaining original Pivot Points reference-bearing array/matrix, method
mutation and requested-context gaps. `corpus/pivot-member-analysis.json` has
zero parser errors but remains non-executable. Goal is active.

2026-09-12: [function-final tuple declarations](TUPLE_FINAL_DECLARATION_AUDIT.md)
now pass 6,748 Rust / 768 installed Python / 130 tools and actual WASM.
Retained artifacts match 1,090 native values in each v5/v6 suite, all full
host outputs and streaming replicas agree, and both RSI suites remain green.
The unchanged Pivot Points loses its function-return error but still has 42
collection/method/request-related diagnostics. Next collect native UDT
alias/copy/history controls before implementing reference-bearing arrays and
matrices; simply loosening the scalar-tree guard is insufficient. Goal remains
active and unified distribution is still outstanding.

2026-09-12: [UDT identity controls](UDT_IDENTITY_AUDIT.md) are now frozen in
v5/v6 (872 values each, identical confirmed results). The current full control
has five admission errors; an executable assignment/array alias isolation has
218/218 mismatches. This is a confirmed wrong-result baseline, not blocked
reference collection. Next implementation must replace value-copy UDT storage
with runtime-owned object identity and integrate rollback/varip, shallow copy,
field paths, public materialization and cross-request graph boundaries. Keep
the failing baseline and complete source; do not merely widen collection guards.

The UDT identity implementation now passes the full Windows gate: 6,762 Rust /
771 installed Python / 130 tools and actual WASM. It implements object-owned
storage, shallow copy, nested writes, defaults and field-level varip, with
original native identity controls and an additional array-identity reference.
Old value-copy snapshots/rejections were requalified; the failed originals and
the discovered recursive-copy regression are documented in `UDT_IDENTITY_AUDIT.md`.
Retained commit-bound artifacts are being built next. Full Pivot Points,
reference-bearing matrices/arrays, cross-request object graph transfer and
resource qualification remain open. The broad goal is still active.

Retained Windows debug artifacts now pass at implementation commit `b091d2832`:
UDT 872/872 per dialect, array identity 872/872, and all prior RSI/member/tuple
reference suites with complete cross-host and streaming parity. See
`UDT_IDENTITY_AUDIT.md` and `PRODUCT_COMPLETION_ARTIFACTS.json`. Next work returns
to the original Pivot Points reference-bearing UDT arrays/matrices and method
effects, followed by requested contexts. Its current diagnostic inventory is
`corpus/pivot-after-udt-analysis.json` (42 diagnostics, no parse/return failures).

The Pivot completion implementation now removes those original 42 diagnostics:
drawing-bearing UDT arrays/matrices, local drawing/matrix effects, conditional
loop tails, tuple-derived drawing styles and requested arrays/dependencies/state
are integrated. The unchanged full script executes, and its v6 Traditional/Auto
monthly historical observer matches 3161 native values. Batch, incremental and
realtime-history outputs agree; observer plots do not alter its 99 lines/labels.
See `REQUEST_CONTEXT_COMPLETION_AUDIT.md` for reference hashes and scope limits.
The full Windows gate passes 6790 Rust / 771 installed Python / 130 tools and
actual WASM; capability metadata reconciliation additionally passes 234 CLI tests.
Next retain fresh commit-bound artifacts and qualify both complete RSI suites,
Pivot and prior controls across them, then extend Pivot options/feed/visual checks
and complete host/resource qualification. The overall goal remains active.

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
| Complete-script compatibility | Freeze original sources/dependencies with hashes; measure admission and execution before changes; repair high-impact combinations with independent controls | In progress: inventory and function/collection boundary |
| Execution and requests | Preserve existing native references; capture independent new cases for admitted combinations; distinguish unavailable tick path information from implementation defects | Pending |
| Host capabilities | Describe supported input/account profiles and readiness checks precisely; implement deterministic capabilities demanded by frozen scripts | Pending |
| Resources | Freeze finite workload, latency and memory budgets before acceptance; qualify Windows/Linux and multiple independent sessions; report full snapshots separately | Pending |
| Unified local distribution | Build selected Rust/CLI/Python/WASM outputs from one committed implementation; run installed-wheel, actual-WASM and retained independent reference gates | Pending |
| Consumption and closeout | Match version, source identity, hashes, installation examples, schema migration and evidence; retain explicit unsupported profiles | Pending |

Account expansion and indefinite-memory claims must not be silently substituted
for the selected standard-candle scope. The core may expose deterministic
configuration and capability contracts; data feeds, persistence and scheduling
remain host responsibilities. Resource exhaustion must preserve the documented
failure/atomicity contract rather than silently dropping script-visible history.

## First implementation slice

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

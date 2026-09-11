# Streaming replica and incremental output acceptance

Current follow-up qualification: [streaming expansion](STREAMING_EXPANSION_AUDIT.md).
Earlier artifact and resource tables below remain evidence for their recorded revisions.

Date: 2026-09-10. Scope: the local worktree based on `7ffcd6524`, not the retained
pre-streaming wheels. No commit, tag, push or publication is implied.

## Behavior delivered

Rust/Python seed history once and apply forming replacements or confirmations
without constructing a complete `RuntimeResult` on every update. Pine still
executes the current bar from its confirmed checkpoint, preserving `varip` and
the established broker lifecycle. Existing full-result APIs remain compatible.

Changes schema **3** carries `baseRevision`, `revision` and `retainedFrom`.
Schema 2 payloads are still accepted and treated as `retainedFrom = 0`. A `RuntimeReplica`
owns the consumer result and revision. `apply()` mutates that native result in
place and returns true; an identical retransmission of the last applied change
returns false. Old revisions, gaps, conflicting payloads, inconsistent base
revisions and unsupported schemas fail before result/cursor mutation. A snapshot
installed without its preceding delta has no last-payload identity: replaying
that snapshot's revision is rejected, while its next revision is accepted.

Replicas are bound to one host-selected stream. Hosts route session identities;
revision numbers alone do not identify different producer sessions. After a gap,
capture a fresh producer snapshot and its revision together, reset the replica,
and continue with subsequent changes. The API does not reconstruct missing ticks
or redeliver external alerts. Historical bar corrections use
`correct_historical(from_time, suffix)` on the existing producer session;
`replay_historical` replaces the entire confirmed history. Input or script
changes still require a separately rebuilt producer session.

```python
session = program.realtime_session()
session.seed(confirmed_bars)
replica = session.replica()
changes = session.apply_forming(forming_bar)
replica.apply(changes)       # no whole-history Python conversion
replica.apply(changes)       # False: identical retransmission

# Only when a full view is needed:
visible = replica.result()

# Recovery after a missing revision:
snapshot = session.stream_snapshot()
replica.reset(snapshot["result"], revision=snapshot["revision"])
```

The unreleased dictionary helper now requires a `RuntimeReplica`:
`apply_runtime_changes(replica, changes)` returns a bool. Passing an unversioned
result dictionary is rejected. `RuntimeReplica(result, revision=...)` supports
initializing from a transported snapshot; parse/reset failures preserve the
previous replica. Existing output JSON remains schema 8.

## Copies and correctness

- Ordinary plot values/colors, remaining series families (chars, shapes, arrows,
  bars, candles, bgcolor, barcolor, fill colors), drawing snapshots, alert
  histories, and broker order/trade/position/equity/fill-alert records use a
  persistent append tree with 128-value leaves. Checkpoints share immutable
  branches. Tail modification copies at most one leaf and a logarithmic path,
  verified with clone-count tests.
- Drawing changes copy the suffix beginning at the first mutable bar snapshot,
  including multiple snapshots on the same bar. Closed snapshots are not copied
  merely to construct a delta. Runtime rollback no longer clones retained
  drawing snapshot vectors or broker record vectors in full.
- Alert differences operate on the mutable suffix and preserve the occurrence
  count of identical `alert.freq_all` events. Replica cursor checks provide
  retransmission protection without collapsing legitimate events.
- Python parses only the incoming delta and applies it to its native replica;
  full result parsing occurs at snapshot initialization/reset, and full Python
  conversion occurs only when the caller requests `result()`.
- Expanded output-family tests cover series styles, lines, line fills, boxes,
  tables, polylines, labels and fills across forming replacements/confirmation.
  Strategy controls also cover every-tick/fill-triggered execution, closing
  orders and long/short admission. They exposed and fixed explicit `None` table attributes being defaulted during
  Python parsing. Existing rollback/failure-isolation tests remain required.

## Validation and measurement

Windows verification has passed all 6,686 Rust workspace tests, 130 tool tests,
strict clippy/format/structure checks, the registered cross-host parity checks,
actual generated WASM/Node execution, and 748 installed-debug-wheel Python tests.
Checks were resumed after fixing the missing diagnostic documentation and a
retained platform-document anchor; the earlier failed logs are preserved. Rust
source checks were not repeated merely because documentation changed. Receipts:
`.local/streaming-hardening/windows-verify-final.log` and
`windows-verify-resumed.log` in the same directory.

The post-change installed-debug-wheel comparison (`after.json`) uses the same
source and sizes as `before.json`, without concurrent build/test work. At
100,000 bars the median update is 0.2920 ms and native replica merge 0.0658 ms,
versus 12.7135 ms and 195.5352 ms in the earlier wheel. At 1,024 bars the new
medians are 0.2033 / 0.04135 ms; at 16,384 bars, 0.17695 / 0.03910 ms. These
48-observation diagnostic samples show the removed history-scaling costs;
optimized-wheel budget acceptance is tracked separately below.
Retained diagnostic baseline: `.local/streaming-hardening/before.json`, measured
with the earlier installed debug wheel and the same plot source on this host.
At 100,000 bars, median producer time was 12.7135 ms and the old full-dictionary
merge took 195.5352 ms. This is a local diagnostic baseline, not a release SLA.

The [frozen resource budget](STREAMING_RESOURCE_BUDGET.json) is an explicit
engineering target frozen before optimized-wheel acceptance. It is not a budget
inferred from passing measurements or an embedding client's agreed SLA.
Reproduce using an installed optimized wheel:

```text
python scripts/benchmark_streaming.py --budget docs/STREAMING_RESOURCE_BUDGET.json --output streaming-report.json
```

Each workload/size runs in a fresh process. The three workloads are ordinary
plots, per-bar alerts, and a bounded 32-element collection. Sizes are 1,024,
16,384 and 100,000 historical bars, followed by 128 bars with three forming
observations and one confirmation each. Update, replica apply, confirmation,
explicit snapshot and peak process RSS are measured separately. Final replica
and producer snapshots must agree; ordinary series deltas contain at most one
new/replacement value. These are finite workload qualifications.

## Optimized Windows artifact and budget results

Status: **passed**, all nine workload/size combinations, with the frozen budget
unchanged. The optimized installed wheel also passes all 748 Python tests.
[STREAMING_ARTIFACTS.json](STREAMING_ARTIFACTS.json) binds this worktree's source
digest, wheel SHA-256 and the hashed local evidence index. It is a local artifact
inventory, not a published updater manifest. The retained module was checked
byte-for-byte against the wheel used for measurement.

At 100,000 historical bars (milliseconds; process RSS includes explicit final
snapshot construction and comparison):

| Workload | Update p95 | Replica apply p95 | Confirm p95 | Full snapshot | Peak RSS MiB |
| --- | --- | --- | --- | --- | --- |
| plots | 0.0369 | 0.0119 | 0.0455 | 12.3222 | 94.20 |
| alerts | 0.0458 | 0.0100 | 0.0612 | 81.8816 | 263.09 |
| collection | 0.0293 | 0.0088 | 0.0297 | 5.4940 | 77.76 |

Across all nine cases, update/confirm and replica timing ceilings, process-memory
ceilings, and the maximum 4x median-growth ratio all passed. Every final consumer
snapshot equals the producer snapshot. Explicit snapshot cost is intentionally
reported separately; calling it every tick reintroduces full-history work.

This slice does not rebuild or qualify a Linux artifact. Actual WASM testing
covers the existing historical surface, not streaming exports. Optimized output:
`.local/streaming-hardening/optimized-budget-report.json`; build/install/tests:
`optimized-qualification.log`; aggregate receipt: `qualification.json` in the
same directory. Earlier failed verification logs remain retained alongside the
corrected checks; their failures were resolved without relaxed thresholds.

## Remaining scope

No universal constant-time or indefinite bounded-memory claim is made. Retained
output still grows; requested context state, script-owned collections, and
explicit full snapshots may still incur history-dependent costs. Arbitrary
script loops inherently depend on script workload. The existing profile's plot
capacity counts allocated value slots, not append-tree headers; peak process
RSS is recorded independently.

A follow-up local slice extends the append tree to the remaining series
families, drawing snapshots and broker records. Public result JSON remains
schema 8. This follow-up is source-only until a later artifact qualification;
the optimized-wheel budget table above is the earlier plot/alert/collection
measurement, not a rerun of those nine cases.

Hosts may set `OutputRetention::keep_confirmed_bars(n)` on a realtime session.
That trims display plots, drawings, fills and alerts to a window whose first
bar index is `retainedFrom`. Series `start` values stay absolute bar indexes;
replicas drop the expired prefix, then splice at `start - retainedFrom`.
Script series history, `var`/`varip`, collections and broker records that Pine
can still read are not trimmed. `result()` is the display window; paginated
reads are host slices of that window. Strategy public order/trade lists still
grow with the session.

Realtime sessions can append, replace, and confirm `request.security` context
bars independently of the main chart. Forming request bars do not enter
confirmed chart results; failed feed updates restore prior state.

WASM `Program.realtimeSession()` now exports the same seed / forming / confirm
lifecycle, request-feed updates, display retention, replica apply/reset, and
JSON changes schema 3 as Rust and Python. Native `pine-wasm` tests compare one
event sequence against `RealtimeRuntime` result and changes JSON. Actual
generated-module Node smoke covers the same path when the wasm32 artifact is
built. Historical correction is `correct_historical(from_time, suffix)` /
`session.correct` / WASM `correct`: the session retains confirmed bars before
`from_time`, re-executes prefix plus suffix atomically, and replicas reset from
the new snapshot. `replay_historical` still replaces the whole confirmed list.
Concurrency budgets remain separate future work. Existing historical WASM
behavior must continue to pass its actual generated-module gate. Earlier native
reference receipts are not silently relabeled as validation of this new
worktree.

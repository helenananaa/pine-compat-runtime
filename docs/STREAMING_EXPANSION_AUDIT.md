# Streaming expansion qualification — 2026-09-11

Scope: the local worktree based on `5f158f59c`. This audit covers retained
display output, dense drawings/orders, live requested contexts, and the current
Rust/Python/WASM bindings. Prior streaming artifacts and timing tables retain
their original identities. No commit, tag, push or publication is implied.

## Findings and repairs

The initial full gate passed the Rust workspace but failed the structure check:
`output/json.rs` and `runtime/historical.rs` exceeded their size limits. JSON
tests and runtime profiling were moved into focused modules; limits were not
relaxed. The initial installed wheel passed 751 Python tests.

Growth probes then exposed failures that the earlier small-window tests missed:

- After physical pruning, output writers and finalizers still padded to the
  absolute Pine bar index. The display window grew back beyond its configured
  size. Writers now use storage-relative indexes; Pine's `bar_index` and public
  delta indexes remain absolute. Complete-result updates also apply retention.
- Alert and drawing cursors used positions that moved when a prefix expired.
  Cursors now find mutable suffixes using bar indexes, preserving producer/
  replica equality while the window moves. A 3-bar alert window previously
  produced 3 events but its replica fell to 1; the regression is covered.
- Prefix deletion rebuilt all surviving values. The append tree now prunes
  expired branches, shares surviving branches, retains at most a boundary leaf
  of expired values, and re-roots to keep its address space bounded. Clone-count
  and repeated-pruning tests check sharing, old checkpoint independence and
  allocated storage. Drawing pruning uses the same path.
- Live requested data invalidated a result cache and re-executed all requested
  history. Eligible expressions now reuse a checkpoint before the terminal
  requested bar. The previous terminal bar and new suffix are evaluated using
  the existing evaluator. SMA/EMA/RSI/cumulative controls agree exactly with
  forced full evaluation and assert that at most two bars are replayed for the
  tested append/replacement sequence. Feed/result histories are shared and
  alignment uses binary search instead of scanning the full series.
- Dense broker updates repeatedly summed closed-trade profits and scanned past
  fill-alert history. Realized profit is accumulated in the original trade-order
  fold when a trade closes; history iterators jump directly for `nth`/`skip`.
  The script-visible sum is checked at the corresponding execution point.

Increasing or clearing a retention limit cannot restore already discarded
output. The actual origin stays monotonic; the wider window grows prospectively.
An explicit replay can rebuild old output and supplies a snapshot for reset.
Compute history, collections, broker records and the input bars retained for
historical replay are not deleted by display retention.

## Qualification

Current source checks passed 6,715 Rust tests, strict clippy/format and structure
checks, 130 tool tests, cross-host conformance checks, and actual generated
WASM/Node execution. New Node controls cross a physical pruning boundary and
exercise stateful requested-data replacements. Installed debug-wheel Python
tests pass 755/755. A new test initially compared cash-derived equity profit
with a trade-derived sum; it was corrected to compare the script-visible sum
at the same pre-close execution point, without adding numerical tolerance.
The failed log remains retained alongside the corrected check.

Evidence: `.local/streaming-expansion/baseline-verify.log`, `baseline-python.log`,
`before.json`, `before-instrumented.json`, `intermediate.json`, `final-verify.log`
and `final-python.log`. The intermediate measurement showed that the first
checkpoint eligibility check missed symbol-form built-in references; its test
was strengthened to require an actual cache entry rather than accepting an
empty set. The repaired path is covered by the forced-full comparison.

## Resource measurement contract

[STREAMING_EXPANSION_BUDGET.json](STREAMING_EXPANSION_BUDGET.json) was frozen
before optimized qualification. It is a local engineering target, not a client
SLA. The reproducible runner is:

```text
python scripts/benchmark_streaming_expansion.py --budget docs/STREAMING_EXPANSION_BUDGET.json --output expansion-report.json
```

Each of three workloads runs in a fresh process at 1,024 / 8,192 / 32,768
historical bars with a 256-bar display window, followed by 128 appended bars
and three forming observations per bar. They cover:

- persistent line/label/table mutation plus shape/candle/color output;
- repeated strategy entries/closes and retained broker output;
- independently updated 5-minute request data on a 1-minute chart, with SMA.

Producer, consumer, confirmation and request-update timing are separate; the
runner also records peak process RSS and full snapshot cost. Window bounds and
final producer/replica equality are mandatory. Baseline diagnostic runs used
16 appended bars on the earlier debug wheel and failed window checks. At
32,768 bars their drawing update p95 was 151.996 ms and request-update p95 was
418.645 ms. These failed debug measurements are not optimized acceptance data.

## Optimized results and artifact identity

**Passed:** all nine new workload/size cases with unchanged frozen thresholds,
plus all nine earlier plot/alert/collection cases with their original budget.
The optimized installed wheel passes 755 Python tests. The native module was
checked byte-for-byte against its retained wheel. Full verification was completed
in stages after the documented structure/test-oracle corrections; old failed
logs are retained, not overwritten as successful receipts.

[STREAMING_EXPANSION_ARTIFACTS.json](STREAMING_EXPANSION_ARTIFACTS.json) binds the
working-tree source digest, optimized wheel and evidence-index SHA-256. The
local source identity is not relabeled as the base Git commit. Baseline debug
artifacts are diagnostic evidence and have their own retained hashes.

At 32,768 historical bars and a 256-bar display window, milliseconds:

| Workload | Update p95 | Replica p95 | Confirm p95 | Request update p95 | Full snapshot | Peak RSS MiB |
| --- | --- | --- | --- | --- | --- | --- |
| drawings | 0.2384 | 0.1017 | 0.2732 | n/a | 2.3791 | 107.64 |
| orders | 0.0539 | 0.0538 | 0.1390 | n/a | 102.6347 | 354.19 |
| requests | 0.0730 | 0.0196 | 0.0708 | 0.0795 | 0.0556 | 42.62 |

Every case satisfies both the display bound and final replica/producer equality.
Order/trade records remain script-readable and retained, so the dense strategy's
explicit full snapshot remains expensive; no snapshot is constructed by an
ordinary delta update. RSS includes the final full result and comparison.

Receipts: `.local/streaming-expansion/optimized.log`, `optimized-report.json`,
`prior-profile-report.json`, `source-identity.json` and `qualification.json`.
The public inventory is a local qualification record, not an updater manifest
or published release. No Linux artifact was qualified in this run.


## Explicit remaining boundaries

The incremental request optimization is conservative. Dataset-end-dependent
expressions, nested requests, complex blocks and other unclassified forms still
use full evaluation; their language admission is unchanged. The measured SMA
workload does not establish a latency promise for every request expression.

Historical correction still rebuilds the combined prefix/suffix from scratch;
it is not checkpoint-based partial replay or reconstruction of missing ticks.
Display retention does not bound all session memory: input history, request data,
script-owned collections and script-readable broker records may keep growing.
Other reporting functions can depend on trade-history length. No concurrency
or indefinite-memory SLA is claimed. This slice qualifies Windows artifacts and
the actual WASM module; it does not rebuild or certify a Linux distribution.

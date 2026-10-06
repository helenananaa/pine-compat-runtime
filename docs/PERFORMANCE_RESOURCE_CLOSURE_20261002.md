# Full resource matrix follow-up

This follows the retained array/plot-history and output-copy receipts in
`PERFORMANCE_ATTRIBUTION_20261001.md` and `PERFORMANCE_OUTPUT_20261002.md`.
The full matrix has been measured against the original frozen budgets in
`PRODUCT_STANDARD_CANDLE_RESOURCE_PLAN.json`. The previous receipts describe
their own source versions and remain unchanged.

## Known historical dataset execution

`HistoricalRuntime::historical_dataset(&bars)` returns a borrowed iterator of
execution steps for a known historical slice. It uses the same initialization,
calculation window, session validation and dataset endpoint as `append_bars`.
Hosts can time or pause individual steps while Pine still sees the endpoint of
the complete slice. `append_bars` now consumes that same implementation.
An error ends the iterator; dropping the iterator restores the previous
historical execution context. Successfully executed steps remain committed.

Ordinary `append_bar` continues to represent discovery of a new bar. A script
that reads `last_bar_index`, `last_bar_time` or historical last-bar flags can
legitimately behave differently when it initially knows only a seed slice.
The native resource probe retains that original seed/discovery control and its
full output. It additionally compares complete known-dataset step output with
batch output. Those are distinct comparisons, and the probe records both flags.
The new iterator is host-neutral: it owns no files, clocks, provider services or
host lifecycle policy.

## Worker output lifetimes

The first Windows native matrix completed all 36 trials and passed every
row-level time/memory check and known-context output comparison. Five Pivot
growth checks failed the original ratio limit of four. That candidate and its
logs remain under `.local/resource-follow-up-20261002/matrix-v2`; the run was
stopped during Windows Python to profile and repair the growth failure.

A separate instrumented source copy attributed most Pivot execution time to
complete requested-context replay. The expression's lowered scalar UDF block
and time functions previously forced the conservative fallback. Incremental
admission now supports local declarations/assignments and conditional branches,
plus `timeframe.change`, `year` and `time_tradingday`. Local state is already
part of the checkpoint; growing contexts still replay their old terminal bar.
Endpoint reads, external mutations, loops and nested requests retain the full
evaluator. Tests compare repeated forming replacements and confirmations of a
counter/array tuple against forced full replay, and check that an endpoint read
inside a UDF branch continues to fall back.

After that repair, full-tail outputs still matched, but request-update growth
remained above the limit. Drawing cursors now use a compact sorted vector and
walk it once while diffing identity-ordered drawings. A backwards identity
query uses binary search. This removes per-object tree searches and per-node
allocation during every cursor capture/clone. Cursor contents, deleted-identity
ordering, drawing tails and the full public output contract are preserved.
Reference-map lookup tests and a replica/snapshot test exercise sparse retained
identities, forming replacements and confirmations.

UDT fields and their intrabar-persistence metadata now share persistent
append-history storage between checkpoints. An object-field write copies the
affected object payload and a bounded tree path; it leaves other objects and
the checkpoint unchanged. Object identities remain dense and monotonically
allocated, and all objects remain retained. This adds sharing, not UDT garbage
collection. The shared-payload test checks one payload clone, independent old
state, trimming offsets and out-of-range writes; runtime and binding tests
cover aliasing and intrabar state.

The native probe releases every replica after spooling and validating it,
before creating live-session snapshots. It releases the live sessions before
historical controls, and each historical control before constructing the next.
Python and actual WASM/Node workers now spool complete outputs and copy them
into their final reports. Their public dict/String snapshot contracts remain.
Every session still validates its complete replica output, diagnostics and
confirmed count. Four-session streams remain independent and interleaved.

The next native candidate passed all 36 Windows trials, including the growth
checks, but Windows Python exceeded the unchanged single-session memory budget.
An instrumented default workload showed about 304 MiB after its tail, 684 MiB
during live snapshot conversion and 784 MiB during simultaneous replica
conversion. Workers now spool and release replicas before creating live
snapshots, and release processed input. Stream zero uses the original input
directly; shifted streams retain their original transformations. Complete
replica/live and stream-isolation comparisons check bytes through exact EOF.

Python owned-result conversion releases each Rust series after copying it to
the output list, using the existing direct converters and preserving Python
types. Borrowed replica conversion remains independent. WASM final replica exports consume the finished replica and encode into 64 KiB
Rust chunks while releasing each source series. They reserve the exact final
UTF-8 byte length and join the chunks after the source is gone. Ordinary
owned snapshots and borrowed replicas use the sink writer and a counting pass
with exact reservation. Ordinary seed snapshots keep one contiguous output
allocation before the runtime begins processing updates. Plot/color/fill history
fields stream individual values to the sink; drawing serializers retain one
object buffer. Both paths return a complete independently owned String.
The original borrowed core String serializer remains an independent
byte-equivalence oracle. Counting snapshots encode twice; consuming chunked
exports copy the encoded chunks once into their final complete String.

The WASM collector checks root and strategy diagnostic arrays by scanning field
boundaries, without parsing a complete output forest. It writes the complete
String in bounded UTF-8 chunks rather than creating a second result-sized Node
Buffer. The chunk writer preserves surrogate pairs, handles short writes and
compares complete output bytes. Tests cover diagnostic errors, escaped text,
missing fields and multibyte characters across chunk boundaries. The final
audit still fully parses and compares every spooled output independently.

The V7 candidate completed all 36 Windows native trials within the frozen
budgets. Windows Python completed its 36 trials within individual time and RSS
budgets, but Pivot four-session replacement and confirmation growth reached
4.293 and 4.139 against the unchanged maximum of 4. The run was stopped and
its receipts retained. Instrumented native diagnostics attributed about 9%
of large-session update time to object restoration and 17% to drawing diffs;
these instrumented timings identify work and are not acceptance measurements.

The next candidate keeps a persistent index of object identities with field
`varip` flags. Intrabar restoration visits that index, including otherwise
unreachable objects, and indexes intervening identities imported by varip
roots. Ordinary object fields still roll back. A reference test covers 2,048
ordinary objects, checkpoint independence and imported identity slots. Drawing
diffs use the final chronological bar index to recognize an entirely stable
history before falling back to boundary search; repeated-index and trimmed
histories are checked against a linear reference.

Workers now also emit a small versioned metadata sidecar. The controller checks
counts and finite nonnegative timings there, and compares repeat live outputs
with bounded byte reads. This removes duplicate parsing of complete reports
into a controller output forest (about 6.5 GiB in the observed native
four-session case). Complete reports and individual outputs remain retained,
hashed and independently audited. Controller memory remains outside worker
RSS budgets.

V8 passed all 108 Windows trials and all 36 Linux native trials, including
growth. Linux Python default RSI single-session repeats nevertheless reached
531.20 and 531.05 MiB against 512 MiB, so the matrix was stopped and retained.
Stage diagnostics showed 295.6 MiB before borrowed-replica conversion, 527.9
MiB after it, and 531.3 MiB after owned live-snapshot conversion. The next
Python converter reuses objects within consecutive runs of identical immutable
integer, float, color or string scalars. Float equality uses bits to preserve
signed zero; lists and dictionary values are never shared this way. The cache
holds one scalar per conversion and leaves the independently owned output lists
and public types unchanged.

Linux Python prechecks after scalar reuse measured 438.1, 463.4 and 195.6 MiB
for default RSI, alternate RSI and Pivot. Linux Node 24 still exceeded 512 MiB
on default RSI. Stage diagnostics showed the completed replica String and live
String overlapping in external memory. Yielding to the event loop did not
reliably reduce peak RSS. `RuntimeReplica::into_result` and WASM `intoResult`
now provide explicit finalization: transfer the complete snapshot and discard
the finished consumer cursor. WASM encodes the transferred source with the
owned serializer. `result` remains borrowed and usable for continuing streams.
Tests check full outputs after forming/confirmation and the consumed JavaScript
handle. The collector finalizes only replicas that have received their complete
tail and are no longer used.

V10 default RSI on Linux WASM reached 520.1 MiB even with finalization. The
owned encoder still reserved its entire final buffer before releasing source
histories. The subsequent chunked encoder postpones that contiguous allocation
until all transferred source fields have been dropped. It retains complete
public output and is checked against the independent borrowed String serializer,
including multibyte UTF-8 and escaped text spanning several chunks. These
diagnostic candidates remain retained separately from full matrix acceptance.

Serialization measurements include file writes; they are not pure encoding
timings. RSS includes worker input conversion, owned snapshots, verification,
allocator retention and report work. Windows records the sum of process-tree
high-water marks; Linux records process VmHWM. The orchestration process and OS
file cache are outside these worker counters.
For WASM, snapshot timing includes native encoding and diagnostic-array inspection;
serialization timing writes the already encoded public String to disk.

The V12 candidate passed all 36 Windows WASM row-level time and memory checks,
but Pivot four-session forming/replacement/confirmation growth was
4.460/4.337/4.074, exceeding the unchanged limit of four. That run stopped during
Linux native measurement; its original receipts and stop reason are retained.
The following candidate restricts chunked encoding to the final consuming
replica export and restores exact contiguous allocation for ordinary snapshots.
Allocation layout during seed is a hypothesis for the earlier growth regression;
the paired size measurements, rather than that hypothesis, decide qualification.

V13 completed all 216 trials, but one Linux Python alternate-RSI four-session
repeat exceeded the unchanged ten-second paired snapshot/serialization budget.
Its snapshot took under 0.44 seconds and serialization reached 9.74 seconds.
The interpreter, output values and memory checks passed; the Python collector's
per-token JSON encoding/writes remained too slow. That failed receipt stays
preserved. A paired full-file encoding diagnostic measured 8.51 seconds with
standard `json.dump` and 6.03 seconds with the scalar-batched standard C encoder,
with exactly identical bytes. The serializer bounds primitive batches at 8,192
items, descends nested containers, and retains standard number, escaping,
nonfinite-value and cycle rules. It keeps the public Python dict API unchanged.

This collector-only follow-up keeps all 533 core hashes and binary artifacts
identical. Both Python groups are remeasured. Four Rust/WASM groups may retain
their original 144 successful trials only after strict verification of the old
source archive, unchanged controller/dependency AST and bytes, workloads,
budgets, repetitions, artifact hashes, complete logs and outputs. The Python
worker's allowed source changes are exactly its serializer import, two snapshot
write calls and serializer-hash recording. Each new Python receipt identifies
the serializer actually used. The final record distinguishes retained from
fresh trials; the failed Python receipt is never rewritten.

## Reproducible candidate and audit

`scripts/build_resource_artifacts.py` builds optimized native, installed Python
and actual WASM candidates from explicitly hash-pinned working-tree source.
It verifies every core-file hash before and after building, records toolchains,
and runs the installed wheel tests and actual Node smoke check. This is a
resource candidate workflow. The existing clean-commit release qualification
workflow is preserved.

The matrix contains 18 cases and two fresh-process repeats on each of six
platform/surface combinations: Windows and Linux, each with Rust, Python and
WASM. It uses the original RSI default, RSI alternate and Pivot payloads;
1,024/128, 16,384/128 and 100,000/10,000 history/tail lengths; and one or four
sessions. Full outputs, event counts and the original budgets are retained.

`scripts/audit_resource_matrix.py` verifies the frozen plan, source and artifact
hashes, all 216 expected trials, logs and individual spooled output hashes.
For a Python-only collector revision, `resource_receipt_reuse.py` verifies the
restricted source proof before selecting archived Rust/WASM reports. Missing or
changed core/worker/workload identities invalidate retention. It compares full
public live outputs across surfaces, platforms and repeats,
and compares stream zero between single-session and four-session workers.
Comparison preserves keys, lengths, booleans, strings and nulls; finite numeric
values use relative tolerance 1e-12 and absolute tolerance 1e-9.
Native same-context historical comparisons are required to match. Missing
receipts and budget failures prevent a passing audit.

The audit establishes a finite synthetic offline workload on the recorded
candidate and machines. It does not establish native TradingView numerical
parity, indefinite sessions, arbitrary output sizes, production host memory or
a release qualification.

## Results

[Machine-readable result](PERFORMANCE_RESOURCE_CLOSURE_RESULTS_20261002.json)
pins the final V14 working tree, its 533 core files, source archive, unchanged
V13 binary artifacts, original frozen budget, six selected reports and independent
audit. **Resource acceptance remains `notPassed`** because three Linux Python
Pivot four-session median-growth checks exceed the original limit of four:

| Phase | Measured growth | Original limit |
| --- | ---: | ---: |
| Forming | 4.288 | 4 |
| Replacement | 4.578 | 4 |
| Confirmation | 4.456 | 4 |

All **216/216** individual process trials completed and meet their own time and
memory budgets. The failures above concern growth between the short and long
history cases; passing individual trials does not qualify these growth checks.
This record selects **144 unchanged V13 Rust/WASM trials** and **72 fresh V14
Python trials**. All 533 core hashes and binary artifacts are identical to V13.
A strict AST proof limits collector changes to Python snapshot serialization;
workloads, budgets, repeats, controller logic, Rust/WASM workers, archived logs
and complete output hashes remain verified. V13's failed Python serialization
receipt is retained separately and never rewritten.

The Python collector serialization failure is resolved on the recorded workload.
The formerly failing Linux alternate-RSI four-session case now has worst paired
snapshot/serialization times of **6.825 seconds** and **6.324 seconds** across its
two repeats, both within the unchanged ten-second budget. An independent prior
precheck also stays within time/memory budgets and reproduces the old complete
outputs byte for byte. This collector repair does not establish a cause or fix
for the remaining Pivot growth failures.

The audit reads all **540 complete live output files** (45 stream baselines and
495 other outputs), with zero cross-surface/platform/repeat mismatch groups.
All **108** single/four-session stream-zero isolation comparisons match.
Native workers retain and hash full historical controls; **180** known-dataset
step-versus-batch comparisons match. Seed-then-discovery endpoint-sensitive
controls remain distinct and retained.

Each platform's canonical gate passes **7,047 Rust tests**, **152 tooling tests**,
**776 installed-wheel tests** and the actual Node/WASM smoke check. Each retained
optimized wheel also independently passes its 776 tests. Earlier failed,
interrupted and diagnostic candidates remain preserved in the result index.

Peak RSS in MiB, taking the worst measured trial for each script/session count:

| Platform / surface | RSI default 1 / 4 | RSI alternate 1 / 4 | Pivot 1 / 4 |
| --- | ---: | ---: | ---: |
| Windows / rust | 362.4 / 914.3 | 395.5 / 1025.7 | 270.9 / 273.4 |
| Windows / python | 386.4 / 980.2 | 411.1 / 1094.5 | 202.6 / 203.9 |
| Windows / wasm | 420.4 / 900.5 | 435.2 / 954.8 | 277.1 / 314.0 |
| Linux / rust | 395.1 / 1142.7 | 409.8 / 1236.2 | 282.0 / 286.2 |
| Linux / python | 439.0 / 1176.2 | 463.9 / 1274.9 | 196.1 / 223.9 |
| Linux / wasm | 473.3 / 1045.8 | 486.1 / 1072.6 | 347.7 / 456.6 |

The original limits are 512 MiB for one session and 2,048 MiB for four.

| Platform / surface | Maximum measured growth |
| --- | ---: |
| Windows/rust | 2.954 |
| Windows/python | 2.715 |
| Windows/wasm | 1.803 |
| Linux/rust | 2.641 |
| Linux/python | 4.578 |
| Linux/wasm | 1.673 |

The worst live phase p95 across all trials is 1.107 ms against five ms.

The implementation and this measurement/audit follow-up are closed with the
three residual growth failures explicitly retained. Complete snapshots still
scale with logical output size; the sink interface lets hosts choose output
storage. The chunked owned encoder temporarily retains encoded chunks while
joining its final String. This is a source/output lifetime tradeoff, not a
universal memory bound.

These results cover the finite synthetic offline workload and recorded working
tree/artifacts. Native TradingView numerical parity, arbitrary Pine programs,
indefinite sessions, production host resources and release qualification retain
their own evidence requirements.

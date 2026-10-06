# Long-history resource attribution and optimization

The current working-tree change follows the optimized delivery receipt at
`a0184633a`. It fixes two measured storage costs without changing the public
runtime output schema or Pine values. This is a Windows native diagnostic
receipt, not a replacement for the frozen cross-platform resource acceptance.
The prior resource qualification remains `notPassed`.

## Attribution and changes

The full RSI source retains only 18 compute-series values at 100,000 historical
bars. Its drawing output nevertheless stores 200,000 logical shape points and
1,400,000 physical shape-column slots, including repeated strings and `na`.
Plot histories now compact equal leaf values. A uniform 128-point leaf retains
one value; mixed leaves keep individual values. Tail mutation expands only the
affected leaf, and checkpoints remain independent. Floating-point comparison
preserves the sign of zero and NaN payload bits. Public snapshots still contain
independent owned vectors and strings, with all logical points intact.

Pivot retains 299,486 arrays after seeding the original complete script. Repeated
request-array imports and matrix-row extraction create temporary arrays on
successive bars. Arrays now undergo reachability collection at successful
script-bar boundaries, starting after 1,024 allocations. The next scan interval
grows with the live array set. IDs remain monotonic and are never recycled.

Roots include current symbols, retained compute history, persistent variables,
call and valuewhen state, cross state, input overrides and cached request
captures. Objects, matrices and maps are conservative roots; their own storage
is not collected. Array contents and slice parents are traced before array data
and metadata are pruned. Checkpoints retain their original persistent stores.
Collection does not run inside a requested-expression evaluation or an active
function/fill callback. Dynamic history is preserved, including unbounded
history when the language requires it. This is not general object-graph garbage
collection or a claim of bounded arbitrary-script memory.

## Paired measurements

The native baseline was built before the runtime changes. Both probes use the
same release profile and instrument the existing public resource probe. The
source/helper/binary/input/output hashes and individual repeats are retained in
[the machine-readable receipt](PERFORMANCE_ATTRIBUTION_RESULTS_20261001.json).

Three original workload/settings cases run at 1,024/16,384/100,000 history bars,
with a 64-bar exact event prefix and two fresh-process repeats. One additional
four-session Pivot case uses 100,000 history bars and two repeats. The second
repeat reverses before/after order. Thus 40 short trials produce 20 paired
packages, covering 26 complete live-stream comparisons. All full outputs match.

The derived prefix stops at its terminal chart confirmation. Daily provider
events cannot be filtered by timestamp: their timestamp denotes the start of
the day, not the observation time. Earlier exploratory files directly under
`.local/performance-20261001/` used such a timestamp filter and retained excess
provider updates. Their phase measurements are excluded from this receipt;
the files remain available. Accepted diagnostic prefixes and all final raw
records are under `.local/performance-20261001/paired/`.

The following values are process working sets after initialization, including
the retained parsed input; they are not isolated core allocation sizes. Values
use the median across both repeats. The four-session peak uses the larger
sampled peak of its two trials, including report construction.

| 100,000-bar workload | Before | After |
| --- | ---: | ---: |
| RSI default, initialized single session | 230.0 MiB | 127.9 MiB |
| RSI SMA+BB/divergence, initialized single session | 230.9 MiB | 143.7 MiB |
| Pivot, initialized single session | 240.2 MiB | 97.7 MiB |
| Pivot, retained arrays after initialization | 299,486 | 1,008 |
| RSI, physical shape-column slots after initialization | 1,400,000 | 10,948 |
| Pivot, four-session process peak | 736.4 MiB | 163.9 MiB |

RSI's logical shape-point count remains 200,000. Compute-history length remains
18 values. Small Pivot latency improvements are inconsistent, so no general
speedup is claimed from the short-prefix trials.

## Original 10,000-bar tails

Two further paired cases use the original, unchanged 100,000-history/10,000-tail
payloads: Pivot and RSI default, each with one session and one before/after
repeat. Each trial includes 10,000 initial forming updates, replacements,
confirmations and historical appends. Pivot additionally receives 20,006
provider updates. The historical append and complete batch controls are retained.
Both live outputs and all four historical-control comparisons match their
baseline counterparts. Together the diagnostic set contains 44 process trials,
28 live-stream comparisons and four historical-control comparisons, with zero
before/after mismatch groups.

| Full-tail measurement | Before | After |
| --- | ---: | ---: |
| Pivot confirmation P95 | 2.8290 ms | 2.1829 ms |
| Pivot replacement P95 | 2.7167 ms | 2.1355 ms |
| Pivot provider-update P95 | 2.7454 ms | 1.9943 ms |
| Pivot process peak including controls/report | 946.1 MiB | 471.0 MiB |
| Pivot arrays after 110,000 confirmed bars | 329,497 | 246 |
| RSI confirmation P95 | 0.2948 ms | 0.1855 ms |
| RSI full snapshot | 150.2052 ms | 91.3871 ms |
| RSI process peak including controls/report | 2,450.0 MiB | 2,326.5 MiB |

Historical append is not universally faster: Pivot append P95 changes from
0.0239 to 0.0401 ms; its seed changes from 2.070 to 2.238 seconds. This change
trades some collection/storage work for reduced retention. One full-tail repeat
does not establish a stable latency distribution across machines or sessions.

Pivot's initial-seed-plus-append output still differs from its complete-batch
output in dataset-end-dependent drawings, just as in the baseline. This change
preserves both contexts and does not force them to equal each other.

## Remaining cost and verification

RSI still exceeds the frozen single-session process budget. For its full-tail
trial, parsed input uses about 129 MiB; the optimized seeded process uses about
179 MiB. Replica creation reaches 307 MiB, final snapshot 473 MiB, and parsed
public results 644 MiB. Historical controls and final report serialization then
raise the process peak to about 2.3 GiB. Owned public output and verification
copies remain expensive even after compacting internal storage. Stage RSS is
allocator- and process-dependent; subtracting stages is not an exact allocation
attribution. The sampler observes the direct native worker, not the Python
orchestrator's separate report-reading memory.

`scripts/verify.ps1` passes: 7,032 Rust tests, 133 tool tests, all 774 tests against
a freshly installed wheel, host parity, and actual WASM/Node. The new tests
exercise compact-leaf mutation/trim/snapshot isolation and float bits; sparse
store pruning; array history, persistent variables, slices, nested reference
roots, and varip through forming/replacement/confirmation with a replica.
Final formatter/Clippy checks and the three focused attribution-tool tests pass
after the offline measurement helper changes.

The new `scripts/prepare_resource_attribution.py` emits a reproducible probe,
derived input hashes and preparation receipt. For example, in an initialized
native build environment:

```powershell
python scripts/prepare_resource_attribution.py --input-root .local/resource-20261001-final --root .local/attribution-new --tail-bars 64
cargo build --offline --release --manifest-path .local/attribution-new/probe/Cargo.toml --target-dir target
target/release/attribution-probe.exe .local/attribution-new/pivot-original-100000.json 1 .local/attribution-new/result.json --no-controls
```

Stage measurements are emitted on stderr. Omitting `--no-controls` also runs the
original historical controls. The full cross-platform/binding resource matrix,
long four-session cases for every workload, indefinite sessions and general
object reclamation remain unqualified. No frozen acceptance budget was changed.

# Full-output resource follow-up

This follow-up addresses output/report duplication left after the array and
plot-history work in `PERFORMANCE_ATTRIBUTION_20261001.md`. The native baseline
is that round's optimized `after.exe`; its core already includes those changes.
Earlier receipts are preserved. Resource acceptance remains `notPassed` until
the complete frozen platform/surface/session matrix is qualified.

## Changes and ownership

The additive public Rust API `write_public_runtime_result_json(result, writer)`
writes the existing schema into a caller-supplied `std::io::Write`. The existing
String-returning API and schema remain unchanged. The core neither opens files
nor manages storage. Short writes are retried and sink errors are propagated.
The implementation reuses the existing family serializers for one item at a
time, so temporary memory still scales with the largest serialized series or
object. Strategy arrays are written separately too. Owned public result snapshots
also remain; this is not a bounded-memory arbitrary-output API.

The offline native collector owns its output files. It spools every live result
and replica, compares their complete bytes with 64 KiB buffers, and copies the
live files into the report. It releases live sessions and replicas before
historical controls, then spools all batch/incremental control results. The final
report preserves all result fields, context end indices, diagnostic checks,
confirmed counts, independent-stream checks and historical-match flags. It no
longer parses results into `serde_json::Value` or builds a cloned report tree and
a complete report String in the worker. Spool files remain next to each report
in its `.parts` directory for inspection; storage now includes the replica files
and the additional full-output files. Callers of the measurement tool own cleanup.

The collector's `serialization` metric now includes buffered file writes and
flush. The old metric encoded into a String. Those values are not directly
comparable pure-encoding timings. Snapshot and runtime update metrics retain
their previous operation boundaries.

## Verification and measurements

Writer tests cover exact equality with the existing serializer for strategy,
drawings and escaped Unicode text, successful short writes, and failure at
multiple output positions. The preparation test instruments the current native
collector and checks all streamed-report stage anchors. The canonical Windows
gate includes workspace and tooling tests, actual WASM/Node execution, and a
freshly built and installed Python wheel.

Native paired reports retain full results and historical controls and are
compared against the preceding optimized collector. RSS is the native worker's
process working-set peak, including parsed inputs, owned snapshots, allocator
retention and report work. Orchestrator parsing and OS file cache are outside
that worker metric; this does not establish a total host-memory budget.

Exploratory full-tail runs under `.local/performance-20261002/paired/` overlapped
with the canonical gate and used the first writer implementation, which
serialized one whole family at a time. That binary and writer source remain
under `.local/performance-20261002/exploratory-*`. Their logs are retained and
excluded from final performance claims. Additional exploratory `*-controlled-*`
records used a freshly resolved standalone lock (two dependency version changes);
that runner was stopped to restore the baseline lock, and those records and the
binary are retained under `pre-pin-*`. Final `*-pinned` measurements use the same
standalone dependency lock as the baseline and an offline, locked release build.
Controlled measurements run after all compilation and other
benchmark jobs complete, use unchanged original payloads, and reverse order in
the second fresh-process repeat. The receipts identify inputs, binaries,
generated collector, helper, raw outputs and stage logs by SHA-256.

The canonical gate passed after the final writer change: 7,034 Rust tests, 137
tooling tests, all 774 tests against the newly built and installed Python wheel,
structural and host-parity guards, and actual WASM/Node smoke execution. The
final gate log and all source identities are pinned in
[the machine-readable receipt](PERFORMANCE_OUTPUT_RESULTS_20261002.json).

Three original 100,000-history/10,000-tail workloads run with one session and
complete historical controls, each twice in fresh processes. Two additional
four-session workloads use 100,000 history bars and the exact 64-bar event
prefix, each twice. This totals 20 process trials and ten paired packages,
covering 22 complete live-stream comparisons and 12 complete historical-control
comparisons. All output groups, per-phase operation counts and pre-output core
logical storage counters match their baseline counterparts. The empty
`current_series` HashMap's reported spare capacity varies between fresh
processes after removals; those capacity differences are retained in the receipt
and are not treated as logical-state mismatches. Confirmed counts are 110,000 for
full tails and 100,064 for prefixes. Each full-tail trial includes 10,000
forming, replacement, confirmation and append operations; Pivot also includes
20,006 provider updates. No input tail or output values are omitted.

The table uses the larger native worker peak of the two repeats. Frozen limits
remain 512 MiB for single-session workers and 2,048 MiB for four-session workers.
An observed peak within a limit is not a full resource qualification.

| Native process and tail | Before | After |
| --- | ---: | ---: |
| Pivot, one session, original 10,000-bar tail and controls | 470.3 MiB | 269.3 MiB |
| RSI default, one session, original 10,000-bar tail and controls | 2,326.3 MiB | 493.1 MiB |
| RSI SMA+BB/divergence, one session, original 10,000-bar tail and controls | 2,344.8 MiB | 511.4 MiB |
| RSI default, four sessions, diagnostic 64-bar prefix | 3,142.5 MiB | 924.0 MiB |
| Pivot, four sessions, diagnostic 64-bar prefix | 164.2 MiB | 131.5 MiB |

RSI alternate inputs approach the single-session limit closely. Parsed JSON
inputs, owned replicas/snapshots and serialization of an individual long series
still contribute substantial memory. Arbitrary history/output lengths remain
unbounded. Pivot's dataset-end-dependent historical batch/append drawing
difference is preserved, with both original contexts recorded. This work does
not establish full compatibility or resolve that distinct semantic boundary.

The Linux/Python/WASM resource matrix, original 10,000-tail four-session cases
and indefinite sessions remain unqualified. Passing functional binding checks
does not supply those missing resource measurements. No frozen budget was
changed and no production-host integration is claimed.

## Reproduction

In an initialized native build environment, prepare a fresh directory, supply
the pinned standalone dependency lock, and build offline. The event prefixes
are diagnostic; pass the original input path to measure the complete tail.

```powershell
python scripts/prepare_resource_attribution.py --input-root .local/resource-20261001-final --root .local/output-follow-up --tail-bars 64
Copy-Item .local/performance-20261001/paired/probe/Cargo.lock .local/output-follow-up/probe/Cargo.lock
cargo build --offline --locked --release --manifest-path .local/output-follow-up/probe/Cargo.toml
.local/output-follow-up/probe/target/release/attribution-probe.exe .local/resource-20261001-final/rsi-default-100000.json 1 .local/output-follow-up/full-report.json
```

Omitting `--no-controls` retains the complete historical controls. Stage
measurements are written to stderr. Each report's `.parts` directory contains
the complete spooled runtime outputs. The final receipt pins the lock, native
binaries, generated source, immutable original payloads and all raw results.

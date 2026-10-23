# Host-interface resource measurements — 2026-10-07

The official 0.3.1 Windows and manylinux wheels completed 32 fresh-process
trials of fixed-history advance/fork and external-account feedback. All 80
retained complete outputs reproduce across repeats; all 40 Windows/Linux
output pairs have identical hashes. Historical endpoints and saved forks match
separate complete-batch controls. External positions, position history, equity,
order actions, quantities and bar indices pass explicit feedback checks.

**Resource qualification remains `notEvaluated`.** These new API workloads
have no frozen latency/memory acceptance budgets. This measurement does not
replace or extend the 0.3.0 216-trial resource qualification.

## Inputs, artifacts and operations

- Core: `14a2ab89c08a85a76d769e9fbe2f13f9c958342d`, the `v0.3.1` release commit.
- Official wheels installed offline in isolated environments; installed native
  extension hashes independently match the native files inside those wheels.
- Windows x86-64/Python 3.12.7 and Ubuntu 22.04 under WSL/Python 3.10.12.
  Both run on the local i9-13900HX machine; this is not a controlled comparison
  of platform performance. Resource trials ran sequentially, before test builds.
- Two history sizes: 1,024 and 100,000 bars, each with a subsequent 10,000-bar
  tail. The fixed dataset includes the tail from session creation so historical
  endpoint flags retain the complete horizon.
- One and four distinct deterministic input streams; two fresh-process repeats
  per interface/history/stream-count/platform case. Source text, input hashes,
  collector/support hashes and installed-artifact provenance are retained.
- History creates one session per stream, advances to the history boundary,
  retains an independent fork and its earlier Python output, then advances each
  active session in ten 1,000-bar calls. Each call includes full Python output
  materialization. Fork creation, seed, tail advance, final snapshot, batch
  control and serialization are timed separately.
- External execution makes prefix, complete and repeated-complete batch calls
  against each authoritative account transcript using one compiled Program.
  Four streams execute sequentially; they are not four persistent external
  sessions or parallel throughput. Returned native broker ledgers stay absent.

Plans were written before each platform's trials. Every planned operation and
stream output is accounted for. Timings include public API conversion costs;
they are not measurements of a bare Rust loop. Ten chunk advances per stream
do not supply a single-bar append latency distribution.

## Large-case observations

The table uses 100,000 history + 10,000 tail bars. RSS is the range of
process-wide cumulative peaks across the two repeats, in MiB. History timing
is the median of each process's operation median for a 1,000-bar advance with
full output. External timing is the same aggregation for a complete 110,000-bar
feedback batch per stream.

| Platform | Interface | Streams | Peak RSS MiB | Measured API call ms |
| --- | --- | ---: | ---: | ---: |
| Windows | History | 1 | 188.8–189.4 | 14.553 |
| Windows | History | 4 | 512.7–514.5 | 23.984 |
| Windows | External | 1 | 296.4–298.3 | 1,416.404 |
| Windows | External | 4 | 556.9–557.2 | 1,504.650 |
| Linux/WSL | History | 1 | 212.5–212.7 | 17.447 |
| Linux/WSL | History | 4 | 590.8–591.7 | 15.521 |
| Linux/WSL | External | 1 | 344.2–344.9 | 821.585 |
| Linux/WSL | External | 4 | 682.4–682.8 | 851.152 |

These peaks include Python input transcripts, runtime state, outputs, retained
fork/control data and verification. The receipts also contain current RSS and,
on Windows, private usage at named checkpoints, including after object release
and garbage collection. Neither current process RSS nor a cumulative peak is
a per-runtime allocation ledger; allocator retention does not establish a leak.

The history fork call's process-median measurements range from 0.033 to
0.075 ms in the large cases. Advancing that fork independently and comparing
its complete endpoint output are separate checks. This observation is not a
guarantee for other state families or unbounded fork retention.

## Verification and retained failures

The independent audit checks the complete trial matrix, every receipt/output
hash, installed native extension versus supplied wheel, source identity,
operation counts and fresh-process/cross-platform output agreement. Statistics
are recomputed from raw samples. Python 3.10/3.12 `statistics.pstdev` differed
by one ULP in some receipts; only derived standard deviation uses a `1e-12`
relative/absolute audit tolerance. Counts, medians and full output hashes stay
strict. Original receipts are unchanged.

An initial smoke run of the collector incorrectly required a `time` field in
external intents. That field is not in the interface; the collector now checks
the actual `bar_index` and entry quantity contract. The failed smoke logs are
retained separately and excluded from the 32 trials. No runtime repair was
required. The initial cross-Python audit rejection is also distinct from the
successful resource worker runs.

Fresh regression checks pass 172 tooling tests and all 829 installed-wheel
Python tests on each platform, with zero failures or skips. Structural
guardrails and `git diff --check` pass. This change adds offline tools and
documentation; production runtime source is unchanged. The Windows PowerShell
5 log wrapper initially rejected unittest's normal stderr progress; its log is
retained as `tooling-wrapper-rejected.log`. Re-running the wrapper under
PowerShell 7 produced a successful exit and the complete 172-test summary.

The machine index is
[HOST_INTERFACE_RESOURCE_RESULTS_20261007.json](HOST_INTERFACE_RESOURCE_RESULTS_20261007.json).
Raw plans, trial receipts, complete outputs, smoke failures and test logs are
under `.local/host-interface-resources-20261007/`. The Windows environment is
retained there; the Linux environment is `/tmp/pine-host-interface-resources-20261007`.

Reproduce a platform matrix using an environment with the desired official
wheel already installed:

```text
python scripts/benchmark_host_interfaces.py --output .local/host-interface-resources-new-run --history-bars 1024 100000 --tail-bars 10000 --chunk-bars 1000 --sessions 1 4 --repetitions 2
```

Use a fresh output directory. Audit both platform matrices:

```text
python scripts/audit_host_interface_resources.py --windows WINDOWS_RUN --linux LINUX_RUN --windows-wheel WINDOWS_WHEEL --linux-wheel LINUX_WHEEL --output audit.json
```

This scope excludes Rust/CLI/WASM resource trials, external execution-pass
groups, requested/Magnifier data in the new large workloads, native live-tick
evidence, indefinite sessions, fork accumulation and formal resource acceptance.
The next step is to choose representative host-neutral workloads and freeze
their API-specific latency, memory and growth budgets before an acceptance run.
Single-bar delta budgets from another API must not be applied to these full
snapshot or batch calls.

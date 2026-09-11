# Independent runtime delivery

Updated 2026-09-11. This is the single current delivery-status entry point.
Current classification: **locally qualified prerelease for the named scope**.
Streaming additions are integrated through `5f158f59c` and `f368ab96e`, included
in main by `7b70095f6`. No stable publication is implied by integration.

## Latest retained streaming qualification

[Streaming expansion](STREAMING_EXPANSION_AUDIT.md) records 6,715 Rust / 755
installed Python / 130 tool tests, actual generated WASM streaming, and 18
frozen-budget cases on Windows. The
[latest local artifact](STREAMING_EXPANSION_ARTIFACTS.json) retains its original
pre-commit source digest. These are retained execution receipts, not fresh
qualification of the merged main commit. No Linux streaming wheel was rebuilt.

The integrated API includes Rust/Python/WASM sessions and replicas, changes
schema 3 with `retainedFrom`, persistent append-tree histories, host-selected
display retention, and live `request.security` context append/replace/confirm.
Display retention does not bound input history, collections or script-readable
broker records. Complex request expressions can still use full evaluation;
historical correction performs full replay and requires replica snapshot reset.

The earlier [streaming acceptance](STREAMING_INCREMENTAL_AUDIT.md) records
6,686 Rust / 748 installed Python / 130 tool tests and nine budget cases for
its own [Windows artifact](STREAMING_ARTIFACTS.json). Neither streaming slice
is present in the a2a1ba5fb native-reference wheels below.

## Retained pre-streaming platform qualification

The demonstrated realtime-price failures were repaired and qualified;
"TV-blocked" describes the older candidate, not the current implementation.
The following source/artifact table and D0-D5 ledger summarize the September 10
native-reference closeout. Their receipts keep their recorded source identities;
the streaming results above supplement them without relabeling older artifacts.

### Source, artifacts and evidence

- Implementation / optimized-wheel source: `a2a1ba5fb0ca566e1de01a9cb8e886322df92b52`.
- HEAD reviewed in that closeout: `7ffcd6524`; at that point changes after the
  implementation were documentation only. Later streaming commits change runtime code.
- Cargo `0.3.0-rc.1`, Python `0.3.0rc1`. Older artifacts have the same version;
  select by source identity and SHA-256, never version alone.
- [Artifact inventory](DELIVERY_ARTIFACTS.json) identifies the two pre-streaming local
  optimized wheels and the retained evidence index. This is not an updater manifest.
- [Native acceptance](TRADINGVIEW_GOAL_ACCEPTANCE.md) records 19 scenarios and
  867,418 values per platform, zero mismatches, on Windows and Linux optimized wheels.
  Retained full gates pass 6,677 Rust and 717 installed Python tests plus actual
  WASM. Windows passes 130 tool tests; Linux passes 129 and skips one Windows-only
  probe. Each optimized wheel separately passes 717 Python tests.
- This documentation closeout rechecks all 85 indexed files by size and SHA-256;
  it does not rerun or relabel those execution receipts.
- Current Linux artifact: `manylinux_2_35_x86_64`. The earlier
  `manylinux_2_17_x86_64` candidate has older semantics and is historical evidence.

## Product scope and current ledger

The first stable scope is standard-candle v5/v6 indicators and strategies,
explicit host-supplied data/libraries and price grid, same-currency linear
accounting with unit point value, and the documented execution lifecycle.
Legacy preview/experimental labels remain unchanged. Full Pine parity is not a gate.
Market acquisition, services, persistence and concrete adapters belong to hosts.

| ID | Current result | Remaining acceptance |
| --- | --- | --- |
| D0 Identity | Current source and two optimized wheels identified; local evidence hashes verified | A future distribution must preserve this source/artifact binding |
| D1 Complete scripts | TechnicalRating/3 complete dependencies and 63,399 independent values passed in retained earlier receipts; see SMA_ROLLING_SUM_AUDIT.md | Broader real-script compatibility; do not relabel earlier workload receipts as reruns on a2a1ba5fb |
| D2 Execution | Original 896-value and later 1,856-value realtime comparisons pass; independent observable B1 controls and separate r1 native companion pass on final wheels | Simultaneous high/low expansion, unobserved price-condition paths and arbitrary feed/clock alignment remain unqualified |
| D3 Host contract | Versioned conservative requirements, source provenance and reached-input errors exist across the documented surfaces | Dynamic request admission/resolution, provider readiness, broader market/account profiles |
| D4 Resources | Retained Windows trend 100k/10k and other named workloads 10k+1k passed at their recorded commits | Linux resource budgets, concurrency, indefinite output retention and embedding-specific SLA; no new performance qualification in this closeout |
| D5 Delivery | Windows and native Linux optimized wheels locally qualified; full gates include actual WASM; four integration surfaces documented | Latest-fix manylinux2014 build if that distribution floor is retained; matching final distribution and release acceptance |

The original frozen r1 synthetic reference coverage remains 0/482; the unchanged
source's native-data companion is counted separately. This is missing coverage,
not 482 failed comparisons. B1 observable behavior has independent evidence;
private sequencing remains `UNVERIFIED_INTERNAL_ORDER`, not a requirement to
prove an inaccessible implementation.

## Next delivery work

1. Select the final distribution floor and qualify all shipped artifacts from
   one implementation revision, including retained reference workloads relevant
   to the release scope. Do not ship the older manylinux2014 wheel as the fix.
2. Define output retention, update latency and memory budgets for intended use;
   qualify those workloads and platforms before making long-session promises.
3. Expand complete-script compatibility and independent reference coverage by
   measured failures. Broader account models remain explicit future capability
   work. WASM incremental/realtime exports now pass the actual generated-module
   gate in streaming qualification; preserve that gate in final packaging.
4. Keep version/schema upgrade notes, installation examples and checksums tied
   to the selected artifacts before any separately authorized publication.

Use [delivery surfaces](DELIVERY_SURFACES.md), [closeout](CANDIDATE_CLOSEOUT.md)
and [releasing](RELEASING.md) for consumption details. The earlier candidate's
[acceptance record](CANDIDATE_ACCEPTANCE.md) remains historical and retains its
failed comparisons and missing original logs. Those results are not current failures.

## Historical audit log

Everything below is the original delivery record at its stated source identity.
Its headings, next actions and TV-blocked labels are historical, even where they
originally called themselves authoritative. The current ledger above supersedes them.

# Original delivery record

Owner: Codex, directly responsible for implementation, review and validation. Started 2026-09-09. Status: active, no stable release accepted yet.

## Candidate scope and acceptance

First stable candidate targets deterministic standard-candle v5/v6 indicators
and strategies, host-supplied data/libraries, historical and incremental runs,
and the documented realtime lifecycle. Preserve existing legacy profiles and
their preview/experimental labels. Full Pine compatibility is not a gate.

The target workload set must include unmodified TechnicalRating/3 with exact
transitive dependencies, the separately frozen G3 strategy batches, and new
reference cases for request/security, Magnifier, and margin/account behavior.
Supporting an import does not promise every export in that library is callable.
Do not trim original dependency sources to pass admission.

Initial market scope: standard candles, explicitly supplied price grid,
same-currency linear accounting. Foreign-currency conversion, non-unit contract
multipliers and nonstandard charts remain unsupported unless demanded by the
selected workload; analysis/host contracts must disclose those limitations.

Resource qualification targets: 100,000 historical bars on the trend workload,
10,000 subsequent appends, and 10,000 forming replacements with confirmations;
also measure dense orders, collection and Magnifier workloads at increasing
sizes. Freeze numerical latency/memory budgets BEFORE acceptance runs using
host measurements and the embedding requirements. Record complete-output costs
separately; never infer bounded memory from history retention alone.

All seven product requirements remain mandatory: real complete scripts,
execution-mode correctness, independent broker evidence, explicit host
capabilities, resource qualification, four delivery surfaces, and matching
commit/artifact/docs/evidence. Internal golden agreement is not an oracle.

## Task ledger

| ID | Required result | Owner | Current state | Acceptance still needed |
| --- | --- | --- | --- | --- |
| D0 | Freeze recovered baseline | Codex | Complete locally: 85f42f20f tooling, 149d10c7a prior semantics; current integrated source includes later qualified fixes | Linux/release artifact qualification is tracked by D5 |
| D1 | Full TechnicalRating dependency chain | Codex implementation, semantics and oracle | Windows acceptance complete at 5920add7e: original graph, all 63,399 reference values, full CLI/installed-wheel/WASM output parity and CLI mode checks (SMA_ROLLING_SUM_AUDIT) | Linux/release matrix remains D5; long-session acceptance remains D4 |
| D2 | Independent execution correctness | Codex | Numerical controls qualified; frozen G3 batches revalidated; matched default HTF, paired historical Magnifier and long/short fractional-margin references pass with Windows host parity | Real tick reference and broader account profiles remain open; request/Magnifier/margin claims stay bounded by actual cases |
| D3 | Explicit embedding/data contract | Codex | Price grid and decimal quantity precision exist; versioned conservative host-input inventory is Windows-qualified across Rust/CLI/installed Python/actual WASM (HOST_REQUIREMENTS.md) | Scoped D3 review complete: explicit defaults, validated profiles, source provenance and reached-input errors; final platform/distribution qualification remains D5 |
| D4 | Long-session resources | Codex measurement and acceptance | Windows trend 100k/10k reused (3ca746976). Remaining named workloads passed at 10k+1k on b9cae7ea5 with freeze-then-accept receipts; over-limit/atomicity/independence tests pass | Remaining workloads are not a 100k/10k claim; pilot-derived regression budgets are not an embedding SLA |
| D5 | Candidate from a known commit | Codex | Local `0.3.0-rc.1` / `0.3.0rc1` TV-blocked candidate. Windows+Linux native gates 6668/715/122 + WASM. Frozen refs revalidated. Not published | manylinux2014 rebuilt offline at 2792a0950: auditwheel and 715 installed tests pass; original goal raw gate logs missing; GitHub release not authorized |

## Original candidate status (superseded)

This was the delivery-status table for the original candidate. The current
ledger at the top of this file supersedes this historical table.

- Identity: Cargo `0.3.0-rc.1`, Python `0.3.0rc1`, not `0.2.0`, not a stable tag.
- Classification: TV-blocked local candidate. Not full Pine compatibility.
- Trend 100k/10k Windows resource qualification: reuse
  `.local/delivery-20260909/resources/trend-100k-acceptance-v5.json` (commit
  3ca746976). Later commits through da55025dc are docs/test-only relative to
  that runtime; this candidate adds packaging/tests/docs plus remaining D4
  measurements and does not rerun trend without cause.
- Already-passing independent references stay required: TechnicalRating
  complete graph, frozen G3 batches, matched default-HTF, paired historical
  Magnifier, long/short margin.
- Known visible failures: live-tick 16/896 exit-price mismatches, B1
  `UNVERIFIED_INTERNAL_ORDER`, public r1 0/482.
- New separate five-round live capture: 14/1856 entry-price observations differ;
  detailed browser market events are retained while finer input is investigated.
- Other worktrees (`pine-interpreter-delivery-*`) are not current main-tree
  acceptance and must not be overwritten.

Evidence reconciliation: [candidate acceptance](CANDIDATE_ACCEPTANCE.md) binds
retained reports and artifacts under `.local/candidate-0.3.0-rc.1/evidence/`.
Original goal scratch logs were removed; fresh checks and historical summaries
are distinguished. Dynamic request scope and embedding resource budgets are
engineering/host requirements, not automatically TV dependencies.

Remaining D4 named workloads and modes:

Development follow-up (2026-09-10 user-authorized TradingView goal):
`STRATEGY_LONG_STOP_MARGIN_ADMISSION_AUDIT.md` records a scoped occupied-long
stop admission repair with six independent controls (9460 values), complete
Windows checks and retained development-artifact parity. This does not relabel
the previously retained candidate binaries. `TRADINGVIEW_GOAL_PROGRESS.md`
tracks the new captures; the original live-price failure and B1/r1 evidence
boundaries remain open.
`STRATEGY_NATIVE_DEFAULTS_AUDIT.md` subsequently qualifies the native 1000000
initial-capital default and zero commission for absent open trades. Its
unmodified r1 native companion plus v5/v6 controls pass 624806 values and
complete Windows gates. This behavioral correction is later development, not
a relabeling of the earlier local candidate's artifacts.
`STRATEGY_VERSIONED_MARGIN_AUDIT.md` then qualifies v5/v6 omitted-margin
defaults and explicit-zero overrides against three native controls (225171
values), complete Windows checks, and actual CLI/Python/WASM parity.

| Workload | Source | Historical | Incremental | Realtime | Formal scale (non-trend) |
| --- | --- | --- | --- | --- | --- |
| every-update | `tests/fixtures/runtime/strategy_calc_on_every_tick.pine` | yes | yes | yes (script executes on forming) | 10k history + 1k tail, 2 reps, 1 replacement |
| dense orders | `tests/fixtures/benchmark/strategy_dense.pine` | yes | yes | yes (default strategy forming) | same |
| collection | `tests/fixtures/profile/matrix_heavy.pine` | yes | yes | yes (script executes on forming) | same |
| complete output | snapshot/serialization/drop phases of the above | yes | yes | when live is measured | recorded from the same runs |
| Magnifier | `tests/fixtures/benchmark/strategy_magnifier.pine` | yes | yes | excluded (historical-only contract) | same history/tail, no live phases |

## Historical audit log

## Current next action

D1 is Windows-qualified at 5920add7e: unmodified TechnicalRating/3 with exact
ta/9 and RelativeValue/3, all 21,133 closed bars from bar_index=0, and all
63,399 rating values agree with the independent reference. CLI execution modes,
installed Python wheel and generated WASM agree on complete output. Receipt:
`.local/delivery-20260909/technical-5920add7e-hosts/qualification.json`.

The rebuilt current CLI also revalidates both frozen strategy batches: r3 has
65 trades/14,711 series values; r2 has 41 trades/18,907 values. All pass with
original source/export hashes, tolerances and prior warmup policies unchanged.
Receipt: `.local/delivery-20260909/strategy-revalidation-summary.json`.
These are historical CLI reruns, not new tick or MTF/Magnifier coverage.

The independent 15-minute/60-minute default-HTF package now passes: 21,133 chart
bars, 5,286 hourly inputs and 422,660 output values, with full installed-wheel/
actual-WASM/CLI parity (MTF_REFERENCE_AUDIT.md). Paired historical Magnifier
references also pass: 148 hourly/888 intrabars, two trades and 2,960 plot values,
with full host and historical mode parity (MAGNIFIER_REFERENCE_AUDIT.md).
Long/short margin references now pass 7992 values and four trades in total,
with full installed-wheel/actual-WASM/CLI parity and CLI mode checks
(MARGIN_REFERENCE_AUDIT.md). The full Windows gate passes 6630 Rust, 690 Python
and 103 tool tests, plus actual WASM/Node. Next continue real tick evidence and D3-D5. Preserve frozen denominators and keep each reference batch
separate. The public r1 independent-reference count is not increased by these
other batches. Windows debug qualification is not final Linux/release acceptance.

## Host-input inventory follow-up

A versioned host-input inventory now describes executable imports, request
contexts, execution clocks, chart/account defaults and optional Magnifier/risk
session inputs. Rust/CLI/installed Python/actual WASM agree on the original
TechnicalRating graph, matched MTF, both margin sources and paired Magnifier
sources. Current Windows gate: 6640 Rust / 693 installed-wheel Python / 103 tool
tests plus actual WASM/Node. Evidence: `.local/delivery-20260909/host-contracts/`.
This is discovery, not a ready-to-run certification. Dynamic request arguments
remain unresolved explicitly; source provenance and configured-input readiness
are still D3 work. D2 real tick, D4 resources and D5 final release are unchanged.

## Qualified evidence and limits

- [EMA/SMA qualification](EMA_INITIALIZATION_AUDIT.md): 391386d7e; final
  synchronized Windows gate 6,588 Rust / 677 fresh installed-wheel Python /
  101 tool tests plus real WASM/Node. Nine reference controls also pass 18
  exact complete-output CLI/Python/WASM comparisons. Twelve reviewed plot-value
  goldens changed; unrelated runtime_math differences were excluded. Resource
  profiles now include undo storage; no resource ceiling was relaxed.
- [Version annotations](VERSION_COMMENT_SPACING_AUDIT.md): c2c028269; original
  RelativeValue/3 spaced version no longer misclassified as v1.
- [Simple parameters](SIMPLE_PARAMETERS_AUDIT.md): b8e38333d; explicit qualifier,
  defaults and global-input argument behavior with independent type probes.
- [Prior real strategies](STRATEGY_MODERN_NEXT_CYCLE_AUDIT.md): two actual
  strategy families / three scenarios / 65 trades. The EMA correction rerun
  passes all prior trade and series comparisons; raw batches stay separate.
- [Earlier six-case G3](STRATEGY_MODERN_G3_CLOSEOUT_AUDIT.md) remains a separate
  batch. The public r1 independent reference denominator remains 0/482 in its
  last recorded measurement, not upgraded by these other captures.
- [Performance baseline](STRATEGY_MODERN_G5_CHECKPOINT_OPTIMIZATION_AUDIT.md)
  is historical small-workload evidence, not a current long-session guarantee.

Latest local full-library qualification receipt:
`.local/delivery-20260909/technical-5920add7e-hosts/qualification.json`; retained
wheel/WASM and complete results are in that directory. EMA-specific historical
evidence remains in `ema-final-qualification.json` and `ema-final-hosts/`. These are
qualification builds, not a stable release or a completed Linux matrix.

## Work and evidence rules

Grok delegation/resume authorization was revoked by the current goal file.
Codex implements and verifies directly. Prior task logs/reports are retained as
historical leads, not current authority. At takeover no matching live Grok task
process was found; the latest four task records ended normally. Do not restart
those tasks or change shared/global agent settings. Keep bounded implementation
scope and independent verification, including review of self-authored tools.
Keep workspace, committed, installed-artifact and published states separate.

Raw third-party sources and captures stay in ignored local evidence folders;
public regressions are original. Do not make CI depend on private exports.
Never generate TradingView expectations with this runtime, skip failing inputs,
change a frozen denominator or relax tolerances to pass. A passing internal
golden is not independent correctness. B1 remains UNVERIFIED_INTERNAL_ORDER.

D1-D5 are not waived by the completed numerical slice. No external publication
or deployment is authorized; prepare a reviewable candidate before requesting
that final approval.


Latest point-value follow-up validates explicit unit contract metadata in Rust,
CLI and Python/WASM chart configuration; non-unit/non-finite/invalid input fails
before execution. Existing full long/short margin outputs remain identical with
pointValue=1. Windows checks pass 6643 Rust / 705 installed-wheel Python / 103
tool tests, with the updated actual WASM smoke rerun after its final test-only
change. Evidence: `.local/delivery-20260909/point-value-contract/`.

D4 next action is a small release-profile pilot to select measurement changes
and budgets, not an acceptance run. The existing probe reports Linux VmHWM only,
and its incremental phase replays the whole history rather than measuring a
sustained tail after a 100k-bar seed. Instrument those missing boundaries before
using it to claim the frozen long-session resource targets. D3 source provenance
remains required; the resource pilot does not waive it.


D4 pilot at 0d8cb0091 completed all 12 scenario/size combinations (six unchanged
synthetic benchmark workloads at 1k/10k bars, one warmup, three measured repeats,
10 forming replacements per repeat). Internal batch/incremental and repeated
live output checks pass. The overall report correctly remains partial: Windows
RSS is unavailable and this is not the frozen long-session acceptance workload.
Trend historical medians were 6.294/60.030 ms; collection forming medians were
0.359/9.592 ms, including returned snapshots. Default strategy forming timings
are not interchangeable with scripts that execute on every update. These small
observations justify adding Windows process-memory evidence and sustained-tail
measurement before freezing numerical acceptance budgets. Do not extrapolate
them into a 100k-bar or long-session pass. Files: `resources/pilot-plan.json`,
`resources/host-preflight.json`, `resources/pilot-build.log`, and
`resources/pilot.json` under `.local/delivery-20260909/`.


D4 memory instrumentation now records Windows process peak working set and
peak commit charge using GetProcessMemoryInfo. Linux retains VmHWM; unsupported
platforms and failed reads remain unavailable. This is benchmark-only host code,
not a runtime API or runtime-only memory claim. The measurement includes input,
compilation, all benchmark phases and verification before final report rendering.
Counter semantics follow [Microsoft PROCESS_MEMORY_COUNTERS](https://learn.microsoft.com/en-us/windows/win32/api/psapi/ns-psapi-process_memory_counters).

The 12-scenario pilot with memory preserves every source/input/result/live-result
hash from the initial pilot. At 10k bars, observed peak working sets range from
29260 to 89008 KiB and peak commit from 30908 to 96644 KiB. The report remains
partial because the pilot is not long-session acceptance. Probe Clippy and all
12 benchmark-tool tests pass, including real Windows counter reads and rejection
of zero, negative, boolean and non-finite memory values. Tool version is 3;
reports identify the memory source and hash supporting probe modules. Evidence:
`.local/delivery-20260909/resources/pilot-with-commit-memory.json`.

Next extend measurement to a true appended tail and repeated forming/confirmation
cycles after the fixed history, then freeze numerical budgets before acceptance.
Windows memory availability does not by itself complete D4 or the product goal.


The sustained-tail probe is implemented: prefix seeding, per-bar tail appends,
forming replacements and confirmations have distinct sample counts and timings.
A separate budget verifier freezes identity, sizes, all timing budgets and both
Windows memory budgets, and recomputes statistics from raw samples. Initial
pilots cover all six existing workload families plus the complete TechnicalRating
library graph at 1000 seed bars + 128 tail bars, two repetitions and two forming
replacements per bar. All seven pass internal consistency and counting checks;
Magnifier remains explicitly historical-only. These are measurement pilots,
not the 100k/10k acceptance. See LONG_SESSION_RESOURCES.md and
`.local/delivery-20260909/resources/sustained-*-pilot-v2.json`.


The first frozen native trend 100k-history/10k-tail attempt failed to complete
within 1800 seconds. Plan SHA256 remains
`d778839832a1db56d8128fb6119dd99b4ba6ec45217b50b03998a795d6a1d26e`.
The report and failure receipts are retained; no full-scale acceptance is claimed.
A redundant confirmed broker/scheduler/alert copy was then removed after code
review and full Windows regression validation. Seven A/B workloads preserve
complete output hashes. A 100k+64 diagnostic tail improves confirmation median
about 23%, but its 52.0943 ms P95 does not demonstrate the original 50 ms target.
See REALTIME_CHECKPOINT_COPY_AUDIT.md. The full 10k-tail run and original budgets
remain outstanding, as do the other D2-D5 requirements.


D5 release-label audit corrected the wheel manifest's unconditional stable
channel. PEP 440 version parsing now identifies alpha/beta/RC/development builds
as prereleases and accepts equivalent normalized RC tags/wheel versions. The
release workflow uses the manifest channel to add --prerelease --latest=false
for candidate tags; stable behavior is retained. Release-only tool dependency:
`scripts/requirements-release.txt`. Six manifest tests and the extracted Bash
channel selector pass locally. No workflow was dispatched, tag created, release
published or repository pushed. This fixes candidate identity handling; it does
not complete D5's multi-surface artifact or platform acceptance.


Trend full-scale v2 completed with matching historical/live outputs and all
operation counts. Frozen-budget verification fails only formingConfirm:
64.6616 ms P95 versus 50 ms. History seed max is 817.9125 ms, append P95
0.0136 ms, forming replacement P95 22.238 ms; peak process working set/commit
are 250184/359924 KiB, within their original budgets. All values are from the
complete 100k prefix + 10k tail, two-repetition report. Plan hash remains
`a741e3a40f3236717c87a5bd43adb0d3ad7f2c34a661af1dcfe5691a9326e604`.
Evidence: `resources/trend-100k-report-v2.json` and
`resources/trend-100k-acceptance-v2.json` under `.local/delivery-20260909/`.
A copy-on-write broker-history candidate is being validated to reduce copying
unchanged closed history during confirmations; public owned outputs and all
original numerical/resource budgets are preserved.


The broker-history sharing candidate passes the full Windows gate (6645 Rust /
705 installed-wheel Python / 115 tool tests plus actual WASM) and seven complete
A/B output-hash comparisons. A 100k+64 diagnostic tail reduces confirmation P95
from 57.7912 to 42.6792 ms; this is not full-scale acceptance. The next formal
trend attempt preserves the 100k/10k sizes, two repetitions, all original phase
and memory limits, and the 1800-second observation window. See
BROKER_HISTORY_SHARING_AUDIT.md. No D2/D3/D5 requirement is waived.


Trend v3 also timed out at the original 1800-second observation window; its
report and failed budget receipt are retained. No formal resource pass is claimed.
Before another long attempt, diagnose whole-run costs and add useful progress
observations instead of repeating the same opaque collection window.

Python realtime execution-clock plumbing is now Windows-qualified: 6646 Rust /
710 installed-wheel Python / 115 tool tests and actual WASM smoke pass. The new
retained wheel replays all 896 values of a frozen native real-update sample.
There are 62 position/trade-history mismatches; time, EMA/SMA and var/varip match.
Official execution-model documentation states intrabar strategy order/fill data
is not rolled back. Current confirmed-broker rebuilding is therefore a semantic
gap to repair, not an accepted preview-only substitute. Exact fill pricing also
requires care: one native fill differs from the sampled close, so no hidden
price event is invented. D2 remains open; see LIVE_TICK_REFERENCE_AUDIT.md.

An unaccepted implementation now lives on `codex/realtime-broker-ticks` from
0410bc110. Eight targeted broker tick controls pass. The first installed working
wheel reduces the frozen native comparison from 62 to 16 mismatches out of 896;
all remaining mismatches repeat one exit-price discrepancy. A second fresh
installed wheel including later targeted fixes retains the same 16/896 result.
The pre-reclassification broad runtime regression reported 1992 passes and
11 failures; those original expectations and failure logs remain in the baseline
commit and retained evidence.
Multiple-fill recalculation order and non-calculating-tick output retention also
remain under review. This work has not been integrated or release-qualified.

The 11 old failures have now been explicitly reclassified in
LIVE_TICK_REFERENCE_AUDIT.md, preserving original sources/update sequences and
historical controls. Targeted validation passes 284 strategy unit tests plus
6 quantity-precision and 8 realtime-tick tests. A separately reproduced quiet-tick
output rollback bug is fixed: without script execution, retain the latest script
state and outputs while advancing broker state. Full Windows gates now pass
6654 Rust / 710 installed-wheel Python / 115 tool tests and actual WASM smoke.
The retained v3 working wheel includes this repair and reports the same 16/896
native exit-price mismatches. Multiple-fill execution ordering still needs native
evidence; this uncommitted patch remains unaccepted for integration.

The new frozen multi-fill native control resolves duplicate execution for two
market entries with every-tick plus fill recalculation: 104/448 mismatches before
the scheduler repair, 448/448 values passing afterward in both Rust and an actual
installed wheel. Two trades additionally match eight price/quantity values.
The prior single-entry capture still fails 16/896 exit-price values. Full Windows
gates after the scheduler repair pass 6655 Rust / 710 Python / 115 tools and WASM.
Active realtime documents and conformance metadata have been corrected; final
metadata checks and known-commit artifact qualification precede integration.
See LIVE_TICK_REFERENCE_AUDIT.md; no resource or final-release requirement is waived.

Metadata synchronization passed all 231 CLI tests, including matrix equality
and unchanged historical output snapshots. The realtime repair is ready for a
local candidate commit; main-branch integration and final release claims remain
pending known-commit artifact qualification. The native single-entry price-input
uncertainty and D4 long-session failures remain visible outstanding work.

The realtime repair at `141f79513eec327a1d96af908fa6ee2887e6f413` is now
fast-forwarded into the local main worktree. Its clean-commit debug CLI, installed
wheel and generated WASM are retained under `realtime-141f79513-artifacts`.
The actual installed wheel passes 710 tests and the native multi-fill 448-value
control; the earlier 16/896 single-entry exit-price mismatches remain explicit.
All four CLI modes plus installed Python and actual WASM revalidate the complete
TechnicalRating graph: 21,133 bars, 63,399 independent values, no skipped warmup,
unchanged source/input bytes and tolerance, exact complete-output host parity.
These are candidate artifacts, not a final Windows/Linux release distribution.

D4 diagnosis now has live progress logs retained on timeout and separate snapshot
destruction measurements, without changing frozen budget phases or limits. A
10k+256 debug pilot passes internal consistency and measures 9.816 seconds wall,
including 0.623 seconds of returned-snapshot destruction. Full trend acceptance
remains failed; the next formal attempt must use the original sizes and budgets
with explicit new binary/commit identity, after reviewing diagnostic evidence.

The new 100k+64 release diagnostic found redundant copying on default-strategy
forming updates. Removing the second full-runtime copy preserves all six tested
workload outputs and passes full Windows gates. Initial/replacement/confirmation
P95 changed from 55.732/54.259/51.971 to 32.940/33.347/47.151 ms; total short-run
wall time dropped from 25.471 to 20.001 seconds. A more intrusive reuse experiment
did not improve overall time in its trial and was retained but not selected.
This supports a new full 100k/10k attempt with the original budgets and a freshly
frozen commit/binary identity; it does not itself complete D4.

While the frozen v4 resource run remains active in the delivery-hosts worktree,
D3 source-provenance work proceeds without builds in the delivery-release
worktree on `codex/host-source-provenance`, based on 6ce9c895d. The draft records
physical source identity and byte ranges at lowering, then joins discovered
request/input call IDs in the host inventory. Five tests cover Unicode root
ranges, duplicate aliases with transitive imports, legacy security, explicitly unavailable manual-HIR provenance, and caller-owned arguments
passed into imported functions. These edits are uncompiled/unqualified and not
integrated; see the working-draft section of HOST_REQUIREMENTS.md. Current main
and the resource-run source tree remain clean and unchanged.

V4 resource acceptance is terminal and failed at its unchanged 1800-second
timeout. First live repetition completed; the last retained second-repetition
milestone is 3072/10000 at 1794.454 seconds. The process is gone and the unchanged
budget verifier emitted `trend-100k-acceptance-v4.json` with a timeout failure.
No final timing distributions were produced, so this is not evidence that its
individual latency budgets passed. Preserve the progress/report/plan hashes and
diagnose full-history copying before another full run.

The source-provenance draft has now compiled and passed all 12 targeted inventory
tests. Cross-host expectations preserve the old semantic-field golden and add
independently specified source text ranges, calculated against actual UTF-8 bytes
for LF/CRLF inputs. Full regression and installed-host qualification have started;
the draft remains isolated until those checks pass.

Source provenance now passes full Windows qualification: 6660 Rust / 712
installed-wheel Python / 117 tools and actual WASM. Discovered request/input
calls carry original physical source identity and UTF-8 byte ranges, including
aliases, transitive imports and caller-owned arguments. Missing metadata is
explicitly null. Schema-1 extension and LF/CRLF/Unicode controls are validated;
configured-provider readiness review remains separate. Known-commit retained
artifact checks and local integration follow this qualified implementation.

A historical capacity inspection using the retained 141f79513 CLI on the same
100k trend source/input finds only two retained series values, versus 200,000
plot values, 100,000 equity rows and substantial order/trade/alert vectors. This
directs subsequent copy-cost investigation toward output histories, not already
trimmed series storage. These counts are not heap-size measurements or evidence
that a proposed optimization has met the unchanged D4 budgets.

Source-provenance commit 52372584375b0d12430f7e0cd4416ccccec67a37 is integrated
locally. Retained clean-commit CLI/wheel/WASM artifacts pass identical complete
requirements reports for LF and CRLF Unicode sources, with two aliases and a
transitive dependency (five verified source locations per case). Receipt:
`host-contracts/source-provenance-523725843-artifacts/qualification.json`.
No publication occurred; configured-provider readiness, D4 and final release
qualification remain incomplete.

The next D4 optimization shares immutable plot output between runtime checkpoints
and detaches with one append slot when writing, preserving public owned results.
Six workloads match baseline outputs; dedicated fork/rollback/caller-mutation
tests pass. ThinLTO plus one code-generation unit improves the release diagnostic
without adding CPU-specific flags. At 100k+256, individual update P95 values are
below 50 ms, but update-plus-snapshot-destruction average is still about 92 ms per
tail bar. Full resource acceptance remains unproven; do not infer a 30-minute pass
or restart a full run solely from these short diagnostics. Release wheel/WASM
qualification of the changed build profile is in progress in the hosts worktree.

Shared plot checkpoints, append-aware detachment and the portable ThinLTO release
profile now pass normal gates (6662 Rust / 712 Python / 117 tools / actual WASM),
plus release-mode focused tests and 712 installed release-wheel tests. Release
CLI/Python/WASM revalidate all 63,399 TechnicalRating values and full-output parity.
This is an implementation/build improvement; the measured longer short-tail
average still leaves the original complete resource window unproven. Continue
investigating output/transaction-history copying rather than loosening budgets.

Equity append now avoids an unnecessary mutable access before deciding whether
to replace the current row or append a new one. Paired 100k+256 release diagnostics
preserve full outputs and show a small reduction in average update-plus-drop
cost (66.531 to 65.852 ms/bar). Full gates pass 6663 Rust / 712 installed Python /
117 tools and actual WASM. A new complete resource run must retain all original
sizes, phase/memory budgets and the 1800-second window.

D3 configured-input review is complete for the frozen profile. Ten installed
release-wheel boundary checks supplement existing cross-host tests, without
adding unconditional gates or claiming dataset completeness. See
HOST_REQUIREMENTS.md and `host-contracts/readiness-audit.json`. Unsupported
profiles are still explicitly rejected; final platform packaging remains D5.

D5 Rust embedding work is drafted in the separate delivery-release worktree on
`codex/rust-embedding-example`, based on 3ca746976. The executable example supplies
an in-memory library and explicit chart/clock inputs, checks historical versus
incremental output, handles a failed forming update, and confirms a replacement
without changing retained results. RUST_EMBEDDING.md includes repository and
standalone-application usage. The example is not compiled or qualified yet;
validation waits for the isolated v5 resource run to finish.

The standalone Rust consumer manifest now resolves offline without changing
dependency versions; compilation remains deferred during v5. Review also found
that `verify.sh` enumerated an older subset of tool tests. It now uses the same
discovery pattern as Windows, so later long-session and budget tests are included.
POSIX shell syntax was checked with Git Bash; this is not a Linux execution gate.
These D5 changes remain isolated and unintegrated until execution checks pass.

D5 version audit confirms both local and remote v0.2.0 point to
cec39d807a469ebae199f30bc67a91d7081a3b9f, and GitHub published that release on
2026-07-20 with Windows/Linux wheels, manifest and checksums. All current workspace
packages still say 0.2.0. Final candidate packaging must use a new coordinated
version rather than overwrite or mislabel the old release. No versions, tags or
external releases were changed during this audit; README now distinguishes the
published downloads from later development functionality.

V5 completed and passed every frozen trend budget at commit
3ca74697639d7c2cb378609378d1ce04d77ff0d5. All 20,000 operations in each tail phase
were present; initial/replacement/confirmation P95 were 28.203/26.132/42.695 ms.
Peak working set/commit were 250244/359988 KiB; wall before final report was
1737.968 seconds. Plan/report hashes and percentile arithmetic were independently
rechecked in `resources/trend-100k-v5-recheck.json`. This qualifies the frozen
Windows default-strategy trend workload only. Executing-forming, collection,
dense-order, Magnifier scaling and resource-limit coverage still need final review.

The Rust embedding example now compiles and executes in Windows release mode;
an independent offline/locked consumer produces identical complete JSON. Clippy
passes. Ubuntu 22.04 WSL has Rust 1.95, Python 3.10, Node and maturin available.
Linux source and manylinux-wheel qualification are next; no Linux pass is inferred
from Windows results or the presence of tools.

Linux native verification at da099ac1134a9c04936ca01fcb77bd936ce91603 passes
6663 Rust / 712 installed-wheel Python / 117 tool tests and actual WASM on Ubuntu
22.04 WSL, with a clean independent Linux checkout. The fixed manylinux2014 image
digest is 493d2032114d757aaa761a9385ad8497f391503bf71acef9abeeb66682ca5d90;
its release wheel passes auditwheel for manylinux_2_17_x86_64. First wheel testing
found one one-ULP math.hypot discrepancy. An audit evaluated all 307 assertions in
the affected test and found only that one value (absolute difference 2.776e-17).
The original expected values now use the already-established 1e-12 float helper
for that one list; all other assertions and runtime code are unchanged. All 712
manylinux tests and the corresponding Windows test pass. Final versioned packaging
and remaining runtime/reference/resource scopes are still required.

Linux and Rust embedding evidence is retained in
`linux-da099ac11-artifacts/qualification.json` and
`rust-embedding-qualification.json`. Main now includes the test-only floating
comparison adjustment e46f7303c; no runtime code changed to fit a platform result.
There are no live v5 or Linux qualification processes from these completed runs.
Next: remaining D4 workloads/limits and a new coordinated version with complete
CLI/WASM/Rust/Python packaging and final reference checks. No release was published.

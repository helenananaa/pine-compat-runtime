# Offline intrabar and multi-timeframe acceptance

This receipt qualifies a Windows working-tree candidate based on
`5c3ab3b628b626619c34481d23f30dd002e362fe`, plus the request-refresh patch
in `crates/pine-runtime/src/runtime/realtime.rs`. The candidate is not a clean
committed release. The patch, source, tool, artifact and output hashes are in the
[machine-readable receipt](INTRABAR_MTF_ACCEPTANCE_RESULTS.json). The prior complete-script/native
matrix remains pinned to its original unchanged core; its native results are
not automatically transferred to this patch.

## Workload and host boundary

The unchanged complete SSL Hybrid strategy is exercised with short EMA/MTF
settings and five partial exit targets. Two independent small controls exercise
ordinary `var` rollback versus `varip`, and market entry with partial limit/stop
exits and fill-triggered recalculation. All inputs are synthetic standard
candles, USD, linear accounting, unit point value and fractional quantity
precision. No data subscription, scheduling, persistence or CandleScope
dependency is added.

Each script has two orderings: high before low with provider before chart,
and low before high with provider after chart. Each case seeds 192 four-hour
chart bars and 32 completed daily provider bars, then replays 12 chart bars
across two days. Its 122 events include 60 chart forming updates (including
identical repeats), 12 chart confirmations, 48 provider forming updates and
two provider confirmations. High and low expand within the same chart bar;
daily confirmation occurs on either side of chart confirmation. Provider
history contains no future seed bars.

The payloads and manifest are frozen under
`.local/intrabar-acceptance-20261001-qualified/`.

The plan's original `artifacts` field identifies the baseline used to prepare
the script/input selection. Actual replay artifact hashes are in the result
receipt, and refer to freshly rebuilt patched artifacts.

Run:

```powershell
$env:PATH = 'F:\工具箱\node.js\node-v22.14.0-win-x64;' + $env:PATH
python scripts/intrabar_acceptance.py .local/intrabar-acceptance-20261001-qualified --artifacts .local/intrabar-artifacts-20261001
python scripts/audit_intrabar_acceptance.py .local/intrabar-acceptance-20261001-qualified .local/intrabar-artifacts-20261001
```

`--generate-probe` generates the direct Rust replay harness. Python uses the
fresh installed wheel; WASM uses actual Node bindings. Every event compares
full live and confirmed results across surfaces and repeats. Every surface
also applies incremental changes to a replica and checks it against the full
result. Only chart confirmation may advance confirmed chart history.
The comparison tolerance is absolute `1e-9`, relative `1e-12`; structure,
strings, booleans and non-numeric fields are exact.

The independent audit additionally checks that the state control's persistent
counter advances once for each chart execution and each provider refresh while
the chart is forming; ordinary state equals the committed count plus the
forming slot; and the forming daily plot equals the latest supplied provider
close. These expected values come from the event stream, not another wrapper.

The patched candidate also passes the complete `scripts/verify.ps1` gate and
fresh retained-wheel tests. Named checks include 1,989 runtime unit tests, the
new opening regression, 242 CLI tests, 133 tooling tests, actual Node/WASM
smoke, and 774 installed Python tests. Build logs and generated Rust-probe
source are hash-bound by the audit receipt. These counts describe individual
components rather than a sum for the entire Rust workspace.

## Finding and correction

An explicit chart opening followed by a provider refresh reused
`opening_update=true`, rejecting a valid refresh as another chart opening.
The failing native Rust regression is preserved in
`.local/intrabar-acceptance-20261001-fractional/opening-before-v2.log`.
Provider refresh now sets the opening flag to false while retaining the latest
chart execution timestamp. The regression covers forming and confirmed provider
refresh, `barstate.isnew`, persistent execution counts, `timenow`, replicas,
unchanged chart confirmation count and atomic rejection of a genuine second
chart opening.

Earlier trial receipts remain in `.local/intrabar-acceptance-20261001/` and
`.local/intrabar-acceptance-20261001-fractional/`. Integer quantity precision
made small SSL trade legs zero and was replaced with a fractional instrument
profile. The control strategy was corrected to submit its two reserved exits only
while position size is four; it no longer tries to reserve an already allocated
quantity after a partial fill. After either partial exit, an immediate close
exercises a second fill-triggered pass within the same host event. The earlier
control produced only one pass per event; that coverage failure is preserved
under `.local/intrabar-acceptance-20261001-final/`. WASM context fields are translated to its
camelCase contract. Indicator results legitimately omit the strategy field.
These trial failures were retained, not counted as passes or fixed by relaxing
runtime error checks.

## Passed results

All six cases pass across direct Rust, installed Python and actual Node/WASM,
each repeated twice: 36 replays, 4,392 event snapshots, zero cross-surface
differences. Every final chart history contains 204 confirmed bars. Independent
state oracles check 244 events, and the source/artifact/output audit passes.

| Script | Ordering | New closed allocations after first event | Intrabar order-changing updates | Maximum executions in one host event |
| --- | --- | ---: | ---: | ---: |
| SSL Hybrid | High / provider before | 154 | 33 | Not measured |
| SSL Hybrid | Low / provider after | 103 | 21 | Not measured |
| Broker control | High / provider before | 24 | 24 | 2 |
| Broker control | Low / provider after | 24 | 24 | 2 |

Closed allocations include partial closes; they are not a count of independent
entry/exit round trips. Intrabar order-changing updates count changes in the
order-receipt array while confirmed chart history stays unchanged. Distinct
event paths legitimately produce different SSL ledgers; equality is required
for surfaces/repeats of the same path, not between high-first and low-first.

## Qualification limits

This is deterministic offline event replay. No new TradingView live Tick or
multi-timeframe reference was captured. Cross-surface agreement alone is not
independent native evidence. Native realtime qualification remains open.
The finite workload does not qualify indefinite memory, multiple long sessions,
Linux artifacts, optimized distribution or graphical geometry. Those gates
retain their own acceptance requirements.

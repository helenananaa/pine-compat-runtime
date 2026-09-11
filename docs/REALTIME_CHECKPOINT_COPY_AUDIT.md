# Confirmed checkpoint copy audit

Baseline: 4d89662cd. The frozen 100k-history/10k-tail trend run did not complete
within its 1800-second collection window. Its original report, binary and plan
are retained under `.local/delivery-20260909/resources/trend-100k-*-v1.*`.
No complete phase measurements were returned, so this failure does not identify
which phase budget would have passed or failed.

`replay_from_confirmed` cloned the complete confirmed `HistoricalRuntime`,
seeded user `varip` stores, then copied the confirmed broker, scheduler and alerts
again. The first derived clone already supplies those confirmed components.
The persistence transfer changes user variables, arrays, maps, matrices and
array type metadata; it does not change broker, scheduler or alerts.

The change removes only the second restoration. The initial confirmed clone
and user-persistence transfer remain, preserving the existing replacement,
confirmation and failure-isolation contract. Private snapshot/restore helpers
used only by tests are compiled under `cfg(test)`; no public API is removed.

Validation:

- Full Windows gate: 6643 Rust tests, 705 fresh installed-wheel Python tests,
  111 tool tests and actual WASM/Node pass without updating runtime goldens.
- Seven source/input-identical A/B workload comparisons pass exact complete
  historical/live output hashes: trend, dense, collection, recalculation,
  realtime, historical Magnifier and the complete TechnicalRating library graph.
- At 100k seed bars plus a 64-bar diagnostic tail, confirmation median changes
  from 37.4992 to 28.70735 ms (about 23% lower). P95 changes from 64.1312 to
  52.0943 ms. The latter has not demonstrated the frozen 50 ms target. Initial
  forming/replacement timings are substantially unchanged for this default
  strategy, which does not execute its script on forming updates.

This short-tail A/B result is not the original 10k-tail acceptance. The original
scope, phase budgets and memory budgets remain required. Update timing includes
the returned full snapshot but excludes destruction of that snapshot; complete
process duration also includes destruction and verification. Do not turn the
short-tail improvement into a full-run or general speedup claim.

Evidence: `checkpoint-copy-full-gate.log`, `checkpoint-copy-equivalence.json`,
`checkpoint-copy-*-100k64.json` and `confirmed-copy-audit.json` in the same resource
evidence directory. The budget verifier now preserves an incomplete probe's
original failure reason rather than replacing a timeout with a missing-phase
lookup error; its additional timeout test passes. Both failure receipts remain.

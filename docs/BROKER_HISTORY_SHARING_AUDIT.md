# Broker history sharing

The complete trend v2 run failed the frozen confirmation budget: 64.6616 ms P95
against 50 ms. Other phase and process-memory budgets passed. The original v1
timeout and v2 over-budget receipts remain unchanged.

Broker checkpoints now share orders, fill alerts, closed trades, closed-trade
metrics, position history and equity history until mutable access. Mutations
detach the vector first. Appending to shared history allocates room for the new
element immediately, avoiding a clone followed by an immediate reallocation.
This avoids copying histories that a confirmation does not modify.

Public `StrategyResult` fields remain owned vectors. No output schema, field
value or mutation API changes. Tests check both independent broker checkpoints
and independence from caller edits to returned results. Existing snapshot,
pending-order identity, risk, forming and varip tests remain in force. Test-only
container conversions were updated; expected values and runtime goldens were not.

Windows validation passes 6645 Rust tests, 705 fresh installed-wheel Python
tests, 115 tool tests and actual WASM/Node smoke. Seven source/input-identical
A/B workloads preserve exact complete historical/live output hashes, including
the full TechnicalRating dependency graph and explicit historical Magnifier.

A separate 100k-prefix/64-tail paired measurement changes confirmation median
from 38.0182 to 30.42565 ms and P95 from 57.7912 to 42.6792 ms. Initial forming and
replacement timings do not improve materially. This is a diagnostic sample,
not the required 10k-tail acceptance. It motivates a complete rerun with the
original sizes, phase budgets, memory budgets and observation window.

Sharing does not impose a bound on retained history or establish leak freedom.
The standard returned snapshots still copy their data. Larger workloads and
the rest of D2-D5 remain required before stable delivery.

Evidence is under `.local/delivery-20260909/resources/`:
`shared-history-full-gate-v2.log`, `shared-history-equivalence.json`, and
`shared-history-*-100k64.json`. Initial test compilation failures reflected
the internal container type change; no failing input or numerical expectation
was removed to obtain the final pass.

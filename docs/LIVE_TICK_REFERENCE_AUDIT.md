# Native real-update reference

> Historical capture/progress record. Later a2a1ba5fb optimized wheels pass the
> original and subsequent named realtime references on Windows and Linux.
> See [final acceptance](TRADINGVIEW_GOAL_ACCEPTANCE.md) and the
> [current delivery ledger](DELIVERY_ROADMAP.md). Failed results below remain
> evidence of earlier builds, not current unresolved failures.

This independent reference uses the unmodified original
`Realtime Four Update Reference` strategy on OKX:BTCUSDT, one-minute bars.
Its source, capture window, first-four-update policy, quantity, price grid and
comparison tolerances were frozen before capture. No live brokerage connection
or order was used; execution was in the native strategy tester.

The capture includes four warmup bars, four real updates in each of two target
bars, and both closing observations: 14 executions times 64 plot values = 896.
All 64 CSV columns are unique and present. Raw chart CSV, trade CSV, source,
initial/final DOM evidence and normalized replay inputs are retained under
`.local/delivery-20260909/live-tick-reference/`. Epoch timestamps, flags, counters,
position and closed-trade counts compare exactly; other numeric fields use
the frozen absolute/relative 1e-9 tolerance. No warmup values were skipped.

The source compiles in both engines. The native first capture bar places a
market entry on recorded update 1; update 2 observes a 0.001 position. It requests
close on update 3; update 4 observes a closed trade. The exported trade prices
are 78246.2 and 78246.1, quantity 0.001, zero commission.

At the integrated clock baseline `0410bc110`, the actual installed Python wheel
compares all 896 values and reports 62 mismatches: entryPrice 23, closed 16,
exitPrice 16 and position 7. These include repeated observations of the same
stored samples, not 62 distinct bugs. Time, EMA/SMA and var/varip values match.
That baseline's forming path rebuilds broker state from the previous confirmed bar,
discarding the order intent issued on the preceding update.

This conflicts with the [native execution model](https://www.tradingview.com/pine-script-docs/language/execution-model/),
which excludes intrabar order/fill events from rollback. The
[strategy contract](https://www.tradingview.com/pine-script-docs/concepts/strategies/)
also describes market-order execution on the next available tick. Correcting
this requires separate treatment of script rollback and broker event state;
it must not be hidden by reclassifying the requested runtime as preview-only.

One fill price differs from the sampled close. The captured sample is the first
four script updates per bar plus closed-bar continuation, not every exchange
price event. Preserve that uncertainty: do not derive a fictitious input tick
from the expected fill price merely to make a comparison pass.

Current status: independent capture complete, clock transport qualified,
realtime broker parity failed and still required. Original sources, hashes,
denominator, tolerances and failure reports remain intact. The Python comparison
returns a nonzero status on mismatch.

## Unaccepted broker tick implementation

Work continues in `codex/realtime-broker-ticks`, based on `0410bc110`, in the
delivery-release worktree. It preserves the previous successful forming broker
and scheduler while restoring ordinary user state from the confirmed checkpoint.
Pending market orders consume the next observed update at its supplied close;
price orders observe that price without replaying cumulative candle extremes.
Default strategies process broker ticks independently of normal script execution.
`process_orders_on_close` no longer treats forming as a closing event, and equity
snapshots replace the current bar's value instead of adding duplicate bar rows.

Eight targeted synthetic lifecycle tests pass: entry/close continuity and user
rollback, default-strategy broker progression, pre-creation extremes, failed
update atomicity, cancellation, close-only processing, short stop-limit activation,
and fill-only calculation. They are deterministic controls, not additional native
reference coverage.

The first actual installed working wheel compares the same 896 values with
16 mismatches, all repetitions of the native 78246.1 versus sampled 78246.2 exit.
Position, closed-trade counts and entry prices now match. This is still a failed
comparison, not 896-value acceptance. Its wheel, isolated environment, exact
comparator/input copies and results are in `live-tick-reference/broker-working/`;
the original failed comparison remains at the parent directory. The second wheel
in `broker-working-v2/` includes short stop-limit eligibility and no-fill
calculation suppression. Its fresh installed environment also reports exactly
16/896 mismatches, all exitPrice, with no input or tolerance change.

The pre-reclassification broad runtime regression has 1992 passes and 11 failures, no ignored
tests. Failures span two targets: ten
strategy unit tests and the fractional-margin realtime lifecycle test. Original
expectations and failure logs are preserved. Several assume that accepted price
events or intrabar orders can be abandoned and that arbitrary forming sequences
must equal OHLC-only history; inspect and explicitly reclassify each case with
evidence instead of silently changing snapshots. Historical margin references
remain distinct from this internal lifecycle assumption.

Still required before integration: verify multiple-fill
recalculation ordering, expand the
necessary broker controls, rerun full host gates, and resolve or explicitly bound
the native fill-price input uncertainty. This working patch is not a release or
a qualified replacement for the integrated baseline. No D2/D4/D5 gate is waived.

### Evidence-led regression reclassification

All original Pine sources and original update sequences remain in the tests.
The pre-change failures remain in `broker-ticks-regression-v2-working.log`.
No golden/reference CSV, native denominator, tolerance, or historical assertion
was changed. Additional observations were appended where needed to retain
positive fill/activation/liquidation coverage. The corrected strategy group has
284 passing tests; quantity-precision and realtime-tick integration targets have
6 and 8 passes respectively. The full Windows gate then passed 6654 Rust tests,
710 tests against a fresh installed Python wheel, 115 tool tests, and actual
generated WASM Node smoke. `broker-ticks-full-gate-working.log` records the run.
The retained rebuilt wheel and fresh venv in `broker-working-v3/` include the
quiet-tick repair and retain the same failed 16/896 native exit-price comparison.
Full gate success does not resolve the outstanding native semantics questions
or qualify a clean committed release artifact.

| Original test | Corrected contract |
| --- | --- |
| `calc_on_every_tick_false_does_not_execute_strategy_on_forming_updates` | A cumulative low is not a tick; a later observed 89 fills the pending limit without a normal script pass. |
| `calc_on_every_tick_true_executes_strategy_on_forming_and_rolls_back` | Script recalculates, but close=95 does not fill limit=90 using an earlier low. |
| `forming_bar_broker_rollback_discards_abandoned_limit_fill` | The supplied 95 observations never fill that limit; no fictitious fill is rolled back. |
| `forming_bar_broker_commit_keeps_confirmed_limit_fill` | Confirmation at 95 keeps the limit pending; a subsequent observed 89 fills it. |
| `strategy_fill_path_realtime_stop_limit_rollback` | Cumulative 11/8 extremes do not activate/fill; appended observed 11 then 8 do, and confirmation retains the fill. |
| `strategy_fill_path_realtime_margin_rollback` | Entry occurs at observed 9, not candle open 10; no inferred low is processed. A later observed 1 liquidates and recovery retains the liquidation. |
| `forming_bar_broker_rollback_discards_abandoned_order_placement` | A successfully placed order persists and fills on the next 100 observation. |
| `forming_bar_broker_rollback_discards_abandoned_cancel` | At observed 40 the resting buy limit has already filled before script cancellation. Separate new cancellation coverage cancels before a marketable price. |
| `forming_bar_broker_rollback_discards_abandoned_fill_alerts` | No observed fill means no broker fill alert; the independent ordinary script-alert assertions remain. |
| `forming_confirmed_strategy_matches_equivalent_historical_batch` | Same candle summaries with different observed price sequences need not have equal fills; historical price 90 and matching plot outputs are still checked. |
| `fractional_margin_history_streaming_and_forming_confirmation_are_consistent` | Historical mode equality remains checked. Accepted partial liquidation persists, then pending close_all fills the remaining 0.990004 at the next observed price. |

Output retention was independently reproduced as a failing test: with
`calc_on_order_fills=true`, a quiet tick after a fill removed the latest plot
sample. The runtime now retains the previous executed state when no script pass
occurs, advancing only broker/scheduler state. `quiet-output-before.log` and
`quiet-output-after.log` retain that repair evidence; the test also confirms that
the subsequent confirmation has exactly one additional bar sample.

The multiple-fill recalculation question needs a native control before changing
the scheduler again. The current [declaration documentation](https://www.tradingview.com/pine-script-docs/language/declaration-statements/)
describes at most one execution per new realtime tick, while the
[execution model](https://www.tradingview.com/pine-script-docs/language/execution-model/)
uses per-fill wording. The current strategy page also describes an extra
execution on a tick with a fill. Do not assume these descriptions prove the
ordering of two simultaneous fills, or how ordinary every-tick and fill-triggered
calculations combine. This evidence uncertainty remains explicit; the current
single-entry capture does not exercise it.

### Native multiple-fill control and duplicate execution repair

The frozen `Realtime Multi Fill Reference` was captured on OKX:BTCUSDT at
2026-09-10 01:33 UTC with both every-tick and order-fill recalculation enabled.
Its unchanged complete source SHA256 is
`0e4400d6493a25b52e877b88ca406e51f33180b6ac0b58855964d0321b731fc9`.
Seven actual executions retain nine fields each, plus the sample-count plot.
The raw chart CSV, two-trade CSV, preflight confirming 64 unique columns, DOM
evidence and comparator are in `live-tick-reference/multi-fill-v1/`.

The second recorded execution sees both 0.001 entries already filled, total
position 0.002; the fourth sees both trades closed. All seven recorded execution
timestamps differ. There is no recorded intermediate one-entry state or second
execution on the same timestamp/feed observation. The working runtime instead
ran a fill-triggered pass and a normal pass on the same input, advancing `varip`
twice and producing 104 mismatches across 448 compared plot values. Both Rust
and installed-wheel pre-fix failures are retained.

The scheduler now suppresses the ordinary realtime pass if fill-triggered
execution already occurred for the observation. Historical closing passes are
unchanged. Nine targeted tests pass, including the exact native source and seven
recorded inputs (448 values). A rebuilt, freshly installed wheel passes all
448/448 values with the original tolerance, plus a separate eight-value check
of the two trades' entry/exit prices and quantities. Native trade PnL is rounded
to cents in this CSV and is not claimed as an exact PnL reference. The replay
uses an explicitly synthetic inert historical seed; this source has no
history-dependent calculation or historical orders.

The prior single-entry reference still has exactly 16/896 exit-price mismatches
when replayed by this wheel. Its records were not overwritten. This new control
resolves duplicate ordinary/fill execution for the captured market-entry/close
scenario; it does not prove every stop/limit/close-tick/immediate-close combination.
Full Windows verification passed after the scheduler change (6655 Rust, 710
installed-wheel Python, 115 tools, actual WASM). A first gate stopped at Clippy
on a test iterator; the corrected second run is `full-gate-v2.log`.

Active realtime semantics documents and the conformance matrix metadata now
describe persistent broker state and observed-price input. Only that reviewed
description was changed in the matrix snapshot; runtime golden outputs and
independent reference values were not refreshed. Final known-commit artifact
qualification and release integration remain separate required work.

An unconditional floating-point floor to the price grid is not a valid repair
for the older exit-price discrepancy. With the supplied binary64 values and
0.1 tick size, flooring 78246.2 / 0.1 yields 78246.1, but the same operation
would also change the independently matched multi-fill exit 78105.2 to 78105.1.
That simple rule therefore contradicts the second native control. The original
price-input uncertainty remains; no such rounding patch was applied.

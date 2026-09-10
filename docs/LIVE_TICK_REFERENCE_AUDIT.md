# Native real-update reference

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

With explicit execution-clock support, the actual installed Python wheel
compares all 896 values and reports 62 mismatches: entryPrice 23, closed 16,
exitPrice 16 and position 7. These include repeated observations of the same
stored samples, not 62 distinct bugs. Time, EMA/SMA and var/varip values match.
The current forming path rebuilds broker state from the previous confirmed bar,
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

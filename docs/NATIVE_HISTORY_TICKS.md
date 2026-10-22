# Native history-tick development

2026-10-22 working checkpoint. The ordered completion Goal remains active.
Native and explicit-account implementations are under regression qualification.
The additive tick-schedule capability admits explicit historical observations;
legacy account routes reject the new declaration flag. Host clock integration,
broad reference/resource validation, wheels and packaged
application acceptance are subsequent requirements, not implied by this source.

`strategy(calc_on_every_history_tick=true)` is an optional, named constant bool
in v6. A fresh TradingView v5 compilation rejects the parameter by name. Existing
positional argument offsets remain unchanged. Host requirements expose
`calcOnEveryHistoryTick` independently of the realtime and fill flags.

The native strategy visits the standard inferred OHLC vertices, updates visible
high/low/close and cumulative quarter-volume, and commits Pine history once per
chart bar. Ordinary variables, referenced heaps, stateful TA and price-flow
builtins roll back between observations; varip values and reachable objects
survive. Supplied magnifier bars supply additional observations and their own
volume, accumulated into chart extremes. Existing explicit pass/event budgets
still bound execution. Historical scheduling never enables realtime scheduling.
Observation passes have a separate 100,000-per-bar limit and do not spend the
fill-loop guardrail. Legacy feedback groups retain their existing 65-pass limit.
Tick groups require the script's matching calculation flag. Historical tick
contexts cannot be downgraded to bar-close/fill contexts, and requested-data
updates obey each observation's event clock. Existing full historical fill
contexts retain their canonical-close clock. Both bindings expose the additive
capability and observation limit. Batch/append scheduling does not by itself
qualify a persistent same-bar streaming lifecycle.

Between observations, the historical broker walks continuous price conditions.
Market orders created at a nonfinal vertex fill at that vertex's price; their
account changes become visible on the next execution. Historical-tick and fill
flags do not add duplicate executions at one vertex. An order from the final
vertex waits for the next opening unless process_orders_on_close is enabled.
Immediate closes and final broker state retain their separate post-statement
ordering. These rules are not a model of actual exchange liquidity.

Historical requested contexts retain canonical bar values and normal merge
rules while chart prices develop. In the captured TradingView samples, even a
same-symbol/same-timeframe security close remains the final historical close at
all four chart vertices. A 30-minute request on a 10-minute chart retains its
previous confirmed value until the aligned chart bar; a lower-timeframe request
retains its ordinary historical selection. Do not replace all requested values
with developing chart values or describe this behavior as tick-causal external
data acquisition. The existing external observed-request contract is separate.

Fresh UI observations and preserved source/log/screenshot files are local under
`.local/goal-history-tick-20261022/browser/`: base-02, market-01, limit-01,
close-01/02, v5-declaration and request-01/02. The standard OHLC fixture in
`history_tick_reference.rs` locks the observed progressive values, rollback and
market/limit/closing-order observations. Additional tests cover price-flow/TA,
cloned continuation, complete-dataset parity, magnifier composition and the
historical/realtime boundary. Magnifier and realtime controls are controlled
regressions, not fresh TradingView equivalence claims. The limit viewport retains
ten rows; only complete observed bars are used for its numerical assertions.

The additional `static-realtime-01` reference records thirteen live observations
with eight distinct prices. Direct and statically nested same-context request
values follow the currently observed realtime chart tuple in those samples.
This exposed a preexisting explicit-schedule defect: nested requests retained
the outer final tuple while direct requests used the active observation. The
repair refreshes the static root on each realtime pass, truncates unobserved
future chart bars, and retains direct requested-context incremental checkpoints.
Regression controls cover all five OHLCV fields, both history-flag settings,
batch/append reconstruction and rejected-input retry. Candidate-01 remains
preserved with its confirmed failure; candidate-02 qualifies the repaired source.
These observations do not qualify arbitrary realtime orders or exchange feeds.

Official semantics were reviewed against the current [declaration statements](https://www.tradingview.com/pine-script-docs/language/declaration-statements/),
[execution model](https://www.tradingview.com/pine-script-docs/language/execution-model/)
and [strategies](https://www.tradingview.com/pine-script-docs/concepts/strategies/)
documentation. Plan restrictions and market data acquisition belong to the host.

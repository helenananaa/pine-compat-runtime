# External broker V1

`Program.run_external(bars, accounts, input_overrides=None, chart_symbol=None,
chart_timeframe=None)` evaluates normal Pine source against caller-supplied
account frames. The Rust embedding API is
`HistoricalRuntime::with_external_accounts` and `external_intents`.

The caller owns matching and accounting. Exactly one account frame is required
per bar, with matching `time`, numeric `position_size`, `equity`,
`initial_capital`, `netprofit`, `openprofit`, and optional `position_avg_price`
(null only when flat). No native broker phases execute. Output contains graphics
and ordered `external-broker/1` intents, without a native strategy ledger.

V1 supports fixed-quantity market `strategy.entry`, full `strategy.close` and
`strategy.close_all`, plus named `when`. Pyramiding must be one. Native fees,
slippage, magnifier, on-close execution, realtime recalculation and
fill-triggered recalculation are rejected. Other order APIs and account fields
fail explicitly. Hosts must separately disclose their matching model and
supported order subset; this is not native broker equivalence.

The host supplies every earlier account frame when evaluating a growing
historical prefix. It must compare earlier emitted intents with the committed
transcript before accepting the new suffix. Boundary-sensitive code that changes
past decisions is rejected. Checkpoints for this API belong to the host's input,
account and intent transcript, not to native broker snapshots.

This interface has no application dependencies. Native entrypoints and native
snapshot semantics remain unchanged. This is a local candidate extension, not a
published release.


## Additive order profile V2

The wire contract remains external-broker/1; optional fields extend intents without changing account frames.
Entry now accepts absolute limit and stop prices (both means stop-limit). Close accepts qty or qty_percent.
Exit accepts from_entry, absolute limit/stop and optional qty/qty_percent; both prices describe an OCO bracket, not a stop-limit order.
Cancel and cancel_all emit lifecycle intents. The host owns ID replacement, OCO cancellation, fills and quantity reduction.
No native broker is executed. Unsupported native settings and account fields still fail explicitly.
Pyne batch quantities and prices may be aligned series; incremental order values are scalars.
Historical prefix stability remains a host obligation. This is not a change to native incremental snapshot semantics.


## Optional execution passes

`run_external(..., execution_passes=groups)` accepts one nonempty group per supplied chart bar.
Each pass contains `bar` (visible OHLCV only), `account` (the authoritative external account frame),
`event_time_ms`, and `confirmed`. Times are ordered and a confirmed pass must be last in its group.
The last chart bar may have only unconfirmed passes. Order intents add `pass_index` in this profile.
The host must preserve prior pass inputs and reject rewritten prior decisions. Matching, event eligibility,
resource budgets, persistence and input acquisition remain host responsibilities.
This API emits no native fills or native account report.
Pine requires `calc_on_order_fills=true`; each pass runs inside the same historical bar using the existing indicator evaluation checkpoint. Unconfirmed passes expose `barstate.isconfirmed=false`; only the final series value is committed for that bar.

Pine bar timestamps, account frame timestamps and execution timestamps all use milliseconds; seconds-to-milliseconds conversion belongs to the host adapter.

## Supplied request data

Program.run_external accepts optional request_bars with the same SYMBOL:TIMEFRAME mapping as native execution (timestamps in milliseconds). The caller owns data admission and decision-time visibility. No native broker is enabled by supplying request data.

An explicitly supplied empty stream in external mode means no completed requested bars are visible yet and aligns to na. Unknown streams still fail. This opt-in external profile does not change native-mode empty-stream admission.


## Additive multi-entry and pass-data profile

The result includes the declared positive integer `pyramiding`; the embedding
host decides admission limits and owns entry allocation and fills. Exit intents
now preserve positive `profit`, `loss`, `trail_price`, `trail_points`, and
`trail_offset`. A trailing exit requires exactly one activation (price or points)
and an offset. No tick conversion or matching runs in this evaluator.

Each execution pass may supply `request_data`, a list of
`{symbol, timeframe, bars}` streams. This replaces the provider for that pass and
uses a fresh request cache. The caller must freeze every prior pass's data and
provide only data permitted by its visibility policy. Empty known streams are
distinct from absent streams. Chart metadata remains separate from the provider.
Native broker entrypoints and native incremental checkpoint semantics are
unchanged: external reconstruction uses the supplied transcript, not native
broker snapshots. This is an unpublished host-neutral candidate extension.

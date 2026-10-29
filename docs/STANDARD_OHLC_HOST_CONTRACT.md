# Standard OHLC fills on Heikin-Ashi charts

`strategy(..., fill_orders_on_standard_ohlc=true)` is a constant, named v5/v6
declaration option. On a standard chart it changes nothing. On an explicitly
declared Heikin-Ashi chart it selects standard prices for the native broker,
while Pine `open`, `high`, `low`, `close`, history and indicators keep reading
the synthetic chart series. The default is false.

This implements the price distinction described in the
[TradingView strategy properties](https://www.tradingview.com/support/solutions/43000628599-strategy-properties/).
It does not generate Heikin-Ashi candles or acquire market data. Both chart
observations and standard prices are supplied by the host.

## Rust interface

Declare the chart type with
`ChartContext::with_chart_type(ChartType::HeikinAshi)`. Bind a provider through
`RequestEnvironment::with_standard_ohlc_provider(Arc<dyn StandardOhlcProvider>)`.
The existing historical, incremental and realtime APIs use that environment.

The callback receives `StandardOhlcObservation { chart, kind, execution_time }`
and returns `Option<Bar>`. The provider is bound to one root chart's symbol and
timeframe. Its answer must be deterministic for replay and contain only the
standard prices visible at that observation. `None` means unavailable, not
permission to fill on synthetic prices. This interface supplies values only;
subscriptions, networking and provider lifecycle belong outside the core.

The standard bar must have the same opening timestamp as the chart observation,
finite OHLCV, ordered OHLC and nonnegative volume. The native broker uses it for
market/limit/stop paths, closes, margin and risk evaluation, default sizing,
trade extremes and equity. Explicit order prices still come from Pine
expressions, so a stop calculated from synthetic `high` remains that price.

`calc_on_order_fills` keeps the chart series during callbacks. Realtime
forming updates retain the existing broker persistence and Pine rollback rules;
broker-only updates also use standard prices.

## Frozen input and bindings

`InMemoryStandardOhlcProvider::from_pairs` and `standard_ohlc_from_json` support
deterministic saved observations. Schema 1 contains an `observations` array:

```json
{
  "schemaVersion": 1,
  "observations": [{
    "observation": {
      "chart": {"time": 0, "open": 1000, "high": 1002, "low": 998, "close": 1000, "volume": 10},
      "kind": "historical",
      "executionTime": null
    },
    "standard": {"time": 0, "open": 100, "high": 102, "low": 98, "close": 100, "volume": 10}
  }]
}
```

Keys include the exact chart OHLCV, update kind and execution timestamp. There
is no cross-kind fallback. A historical row cannot satisfy a forming request,
and a revised chart observation requires its own pair. Duplicate keys are
rejected. Frozen forming rows require `executionTime`; pass it through the
existing execution-time/context API. Confirmed and historical rows may omit it,
but an explicitly supplied timestamp must match the input row.

- CLI: `--chart-type heikinashi --standard-ohlc observations.json`. All four run
  commands accept these options. `run-realtime-forming` is a smoke driver that
  synthesizes two last-bar observations; the provider must explicitly contain
  those observations and timestamps. It does not invent standard prices.
- Python: add `"chartType": "heikinashi"` to `request_bars["$chart"]`, alongside
  its required `minMove` and `priceScale`, and put the schema-1 object under
  `request_bars["$standardOhlc"]`.
- WASM: supply the same `$standardOhlc` object through the existing request-host
  JSON, and set `$chart.chartType` to `heikinashi`. The chart metadata fields
  retain their existing optional defaults.

Portable fixtures are in `tests/fixtures/runtime/standard_ohlc/`. Their chart
prices are about 1000 while standard prices are about 100. Tests assert both
the chart plots and actual trade prices; accepting the declaration alone is
not an acceptance criterion.

## Failure and compatibility boundaries

Missing or invalid standard data produces `E_STANDARD_OHLC_INPUT` before native
bar execution. It does not poison the historical runtime, so corrected input
can be retried. Realtime failures leave the previously accepted state intact.
Invalid frozen JSON or duplicate observations fail while constructing the
provider. Custom providers receive the same per-observation runtime validation
for standard bar shape and timestamp.

The following combinations are explicitly unsupported:

- Heikin-Ashi standard fills plus `calc_on_every_history_tick`: a standard-price
  tick path does not define the synthetic chart values visible at each tick.
- Heikin-Ashi standard fills in external-account execution: the external broker
  would need its own dual-price enforcement contract. It is rejected at binding,
  rather than silently promising host enforcement.
- Chart types other than standard and Heikin-Ashi in the new chart-type input.

The existing historical Magnifier can supply standard-market intrabars. Its
absence/gap diagnostics and realtime limitations remain in force; input rows
must describe the standard price path, not the Heikin-Ashi path.

Host requirements schema **5** advertises the conditional standard-price
obligation and supported profile. Runtime result schema 9 and streaming changes
schema 4 are unchanged. These tests are local semantic evidence, not a new
TradingView-native equivalence certification.

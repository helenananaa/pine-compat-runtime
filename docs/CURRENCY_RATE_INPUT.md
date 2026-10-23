# Host-supplied currency rates

This contract is being qualified in local batch 31. It is separate from the
previously qualified batch 30 and is not a published release claim.

The interpreter owns conversion and deterministic broker accounting. The host
owns currency discovery, market selection, daily-rate calculation, availability
times and data acquisition. The core performs no network or symbol lookup.

## Input and lookup

Rust accepts `CurrencyRateProvider` through
`RequestEnvironment::with_currency_rates`. `InMemoryCurrencyRateProvider`
implements immutable, strictly ordered streams and as-of lookup. JSON input is
shared by the CLI, Python and WASM:

```json
{
  "schemaVersion": 1,
  "pairs": [
    {
      "from": "USDT",
      "to": "EUR",
      "rates": [{"effectiveTime": 1577923200000, "rate": 0.8874}]
    },
    {"from": "USD", "to": "ZZZ", "unavailable": true}
  ]
}
```

`effectiveTime` is a Unix millisecond availability timestamp, not an unfinished
source candle's open. Each rate is finite, positive, and measured in destination
units per source unit. Duplicate pairs, unknown fields, ambiguous `rates` plus
`unavailable`, noninteger times and unordered observations are rejected. An
explicit empty `rates` list represents known history with no available frame.
Future frames may be supplied but are excluded from earlier lookups. Inverses
and cross rates must be supplied or resolved by the host; they are not inferred.
Host pair keys and chart currency use nonempty uppercase letters, allowing
quote codes such as `USDT` without embedding a market or ISO registry.

Native historical D/W/M and realtime W/M probes sample the effective daily rate
at the chart bar's opening timestamp. `timenow` does not advance that lookup.
The host must supply daily values with their actual availability semantics;
passing unshifted, unfinished FX candles would introduce lookahead upstream.

Same-currency conversion is identity and needs no provider. An unsupplied
foreign pair is an explicit input error, including with
`ignore_invalid_currency=true`. A host-resolved unavailable pair returns `na`
when that flag is true and an error otherwise. Before the first frame of a
supplied stream, queries and conversion helpers return `na`; native broker
sizing and accounting use identity during that known prehistory. This observed
prehistory behavior does not authorize identity for an unsupplied pair.

## Embedding APIs

CLI:

```text
pine-compat run strategy.pine --bars chart.csv --chart-currency USDT --currency-rates rates.json
```

Python passes the JSON object under the reserved `$currencyRates` key of
`request_bars`. Chart currency is explicit metadata, not inferred from the name:

```python
program.run(bars, chart_symbol="BINANCE:BTCUSDT", chart_timeframe="D",
            request_bars={
                "$chart": {"minMove": 1, "priceScale": 100,
                           "quantityPrecision": 5, "currency": "USDT"},
                "$currencyRates": rates,
            })
```

WASM uses the same reserved keys in the request-bars JSON string, with
`symbol` and `timeframe` inside `$chart`. Both direct and prepared execution use
the same parser. Existing Python and WASM method signatures are unchanged.

Host requirements schema 3 adds `currencyRates`, including unresolved pair
selectors and account conversion obligations. A symbol name is not proof of a
currency, point value or available rates. `request.currency_rate` preserves
version and dynamic-request argument/scope restrictions. Pine currency-name
arguments are case insensitive and normalize to uppercase provider keys.

## Units and validation scope

Order prices remain in chart quote currency. Equity, monetary P/L, margin and
fees are in account currency. Percent commission converts fill notional at the
fill rate; cash-per-order and cash-per-contract fees are already account units.
Entry fees retain their historical account amounts. A change of FX with an
unchanged instrument price does not create notional currency exposure.
Trade extrema retain per-observation account amounts and return percentages;
later rates do not revalue closed-trade metrics.
The final exit fill belongs to the trade's path, including limit endpoints,
gaps and slippage. Trade extrema include entry fees and exclude exit fees;
account equity extrema include final exit debits. Account run-up uses the
lowest realized balance reached when opening a position after its first fee,
and retains that baseline across trades; additional pyramiding fees debit
equity. Account run-up percentages use equity at the observation, while
drawdown percentages use the preceding closed-equity peak. These are distinct
from trade-return percentages based on entry notional and entry fees.

Native evidence is under `docs/evidence/strategy-library-20261010/` in
`fx-native-*-31.json`, with unchanged probe sources. The input export contains
OHLC and effective daily conversion columns but omits volume. Volume is zero
in tests and none of the qualified probes reads it. W/M aggregated input is
qualified for query context only. Realtime probes compare queries and explicit
quantity calculations, not a complete realtime trade ledger. Multi-entry and
partial-close accounting have controlled core tests and a native two-entry
short margin/partial-liquidation segment, including six resulting trade
records and per-trade extrema. Those are distinct from complete native
strategy-library ledger qualification. Nonunit contract multipliers and
nonstandard chart broker models remain unsupported.

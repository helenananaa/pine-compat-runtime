# Requested symbol metadata

The host may freeze metadata for exact symbols evaluated by `request.security`
or `request.security_lower_tf`. The interpreter does not resolve instruments or
fetch market metadata. Rust uses `RequestEnvironment::with_symbol_contexts`;
the entry's timeframe is ignored because the request selects its timeframe.

Python `request_bars` and WASM request JSON accept a `$symbolMetadata` object:

```json
{
  "$symbolMetadata": {
    "OKX:ETH-USDT": {
      "minMove": 1,
      "priceScale": 100,
      "quantityPrecision": 3,
      "pointValue": 1,
      "currency": "USDT"
    }
  }
}
```

Every entry requires positive integer `minMove` and `priceScale`. Optional
`quantityPrecision` is an integer in 0..9; optional `pointValue` currently must
be 1. Currency codes use nonempty uppercase ASCII letters. Unknown fields,
booleans in numeric fields, invalid grids and invalid symbol keys are rejected.
Identity is the exact map key. Symbols can contain exchange prefixes.

The root chart retains its own `$chart` metadata. Requested symbols use their
registry entry at every request timeframe, including nested contexts and ordered
intrabar evaluation. Nested requests back to the root recover its metadata.
The registry is immutable within an execution session. Hosts must rebase a
session when metadata changes.

The standalone runtime preserves its existing defaults when no entry is given.
Hosts requiring actual instrument precision must validate and supply metadata
before execution instead of treating these defaults as exchange observations.
This interface does not supply descriptions, trading sessions, general contract
multipliers, protected libraries, or market-data acquisition.

Python exposes `SYMBOL_METADATA_SCHEMA_VERSION == 1` for capability negotiation.

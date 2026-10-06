# Pine v3 short exits without `from_entry` (2026-09-23)

## Reproduction

The repository control `tests/fixtures/runtime/legacy_v3_short_all_entry_sma_control.pine` (SHA-256 `E010333411049CF41F8F2BF4535BBE17EFC5F5030DAC1C4FE459AEF52DE4A622`) uses a short SMA crossunder entry and `strategy.exit("X", profit=500, loss=400)` without `from_entry`. Both the local runtime and TradingView ran it as original Pine v3 on `BINANCE:BTCUSDT` daily bars. The local input is the 3,279 confirmed daily rows at `.local/community-coverage-20260923/volatility-reversion-native-confirmed-bars.csv`, with the host-supplied 0.01 chart tick.

```powershell
cargo run -p pine-cli --locked -- run tests/fixtures/runtime/legacy_v3_short_all_entry_sma_control.pine --bars .local/community-coverage-20260923/volatility-reversion-native-confirmed-bars.csv --chart-symbol BINANCE:BTCUSDT --chart-timeframe 1D --chart-price-grid 1/100
```

TradingView's strategy report displayed 86 closed short trades. The local run also produced 86. All six visible native rows (trades 81 through 86) match local side, entry and exit date, entry and exit price, and quantity. For example, trade 86 enters and exits on 2026-09-14 at 76,842.01 and 76,837.01, quantity 1. Native visible rows are retained outside version control at `.local/super-trend-3-v3-20260923/native-short-all-entry-control.json`; the complete local JSON is beside it.

## Runtime change and limit

Relative exits with an omitted `from_entry` now use the current position direction when they derive target and stop prices. Deferred exits keep their original placement bar when resolved after a short entry fills, so the bracket can fill along that entry bar's historical price path. The same direction rule covers standalone profit and loss exits, relative trailing activation, and mixed absolute/relative brackets. A flat-position loss or trailing exit also keeps the deferred template for an eventual entry. Focused broker tests cover open and pending shorts; a v3 runtime fixture covers profit and stop fills on the entry bar.

The native comparison covers the aggregate trade count and six visible recent trades. Complete historical trade-list and plot-series exports were not obtained, so those are not claimed as matching.

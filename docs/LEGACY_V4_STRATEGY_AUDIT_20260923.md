# Pine v4 strategy compatibility slice, 2026-09-23

This local development slice admits Pine v4 `strategy(...)` sources to the
existing host-neutral strategy analyzer and broker. Separate
[v1](LEGACY_V1_STRATEGY_AUDIT_20260923.md) and
[v2/v3](LEGACY_V2_V3_STRATEGY_AUDIT_20260923.md) strategy slices are
documented independently.
This is feature-level compatibility, not arbitrary Pine v4
strategy parity or a release qualification.

The separate [UT Bot Alerts v4 comparison](UT_BOT_ALERTS_V4_NATIVE_PARITY_20260924.md)
verifies a complete popular v4 indicator's default Buy/Sell markers against
TradingView; it does not expand this strategy-specific scope.

The unchanged public [Hull Suite Strategy by DashTrader](https://www.tradingview.com/script/Q9OQye4C-Hull-Suite-Strategy/)
is the complete-script control. Its 70-line Pine v4 source is retained only in
ignored `.local/hull-v4-20260923/hull-suite-strategy-original.pine` (SHA-256
`235a18e8a69aa6479d446bebc8c023eb0ce1b4a6e293e706cec079293354037d`).
HTML nonbreaking spaces were normalized to ordinary spaces; Pine tokens and
line order were not changed. The source compiled with zero diagnostics. No
copied community source is committed.

## Native comparison

TradingView ran the original source on `BINANCE:BTCUSDT`, 1D. The chart export
contains 2,350 rows, including the forming September 23 bar. The runtime input
is the 3,279 confirmed bars from October 1, 2017 through September 22, 2026;
the 2,349 overlapping confirmed export rows are compared. The new v4 strategy's
`MHULL` and `SHULL` are chart CSV columns 5 and 6; later columns with the same
names belong to another chart indicator and are excluded.

| Reference | SHA-256 | Result |
| --- | --- | --- |
| `.local/hull-v4-20260923/hull-native-chart.csv` | `a16409c1a55f2e52a91fe950dc035d318d8fa8d797cf6ac7c4d6857f83742089` | 4,698 plotted values match within 1e-8; zero mismatches |
| `.local/hull-v4-20260923/hull-native-trades.csv` | `832095832bb743804c86b0822ada30d40e2f8068b0494c8f19527a7a5c69366a` | 75 closed trades have matching entry and exit dates and prices; one additional native trade is open on the forming bar |
| `.local/hull-v4-20260923/hull-native-trades-1m.csv` | `0802c259ee0cbb1d3e7e47dbd02ca798f8d3dbe142626e30d1676ccc397674e6` | With the UI capital matched to the runtime's 1 million default, the same 75 closed trade dates/prices match; maximum displayed quantity relative difference is 1.41e-7 |
| `.local/community-coverage-20260923/volatility-reversion-native-confirmed-bars.csv` | `278ae090e32e441e4cadd2bd70955ddad650afeeeda21fe17651a18f5092593a` | Exact historical input used by the CLI |

TradingView's chart initially used a 100,000 USDT initial-capital override.
The runtime used its 1,000,000 default because this source does not declare
`initial_capital`. A second native report was exported at 1,000,000 for the
quantity check; the chart setting was restored to 100,000 afterward. Native CSV
quantity and PnL are rounded, so they are not treated as exact floating-point
references. The maximum absolute displayed PnL difference at matched capital
was 0.39 USDT across the 75 trades.

The retained `.local/hull-v4-20260923/compare.py` compares each plotted
position and trade date/price. The local CLI output is `runtime.json` in that
directory. Full JSON from `run`, `run-incremental`, and
`run-realtime-history` is identical on the 3,279 confirmed bars, including
plots, trades, alerts and diagnostics. The committed small v4/v6 control
fixtures verify versioned default margin and legacy `wma` translation against
the existing broker path.

## Scope

This admits a measured v4 strategy subset: the original script exercises
legacy inputs and moving averages, `strategy.risk.allow_entry_in`, default
percent-of-equity entry sizing, repeated long entries, and opposite signals
that close a position under the chosen long-only direction. Other v4 order
forms, account currencies, alternative inputs, and realtime ticks require
separate native qualification. Host data and chart settings
remain external to this runtime.

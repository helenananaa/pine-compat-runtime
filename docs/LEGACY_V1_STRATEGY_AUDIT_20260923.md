# Implicit Pine v1 strategy compatibility slice, 2026-09-23

TradingView's [script structure documentation](https://www.tradingview.com/pine-script-docs/language/script-structure/)
states that a missing `//@version` annotation selects Pine v1. Its Pine Editor
confirmed the boundary: a v1 script containing `if` failed with
`no viable alternative at input 'if'` and suggested Pine v2. The tested v1
control therefore uses only top-level statements and historical named
`strategy.entry(..., when=...)` conditions.

The full source is
[`legacy_v1_strategy_sma_reversal.pine`](../tests/fixtures/runtime/legacy_v1_strategy_sma_reversal.pine)
(SHA-256 `65A0500F5841858E0108A439AB21CBDF04595A04189D35E80A6DA4518849CBF2`).
It plots two SMAs and reverses between two-unit long and short positions on
crossovers. The fixture test compares its full historical output to the
version-explicit v6 control, including plots and broker results.

TradingView compiled this exact no-version source on `BINANCE:BTCUSDT`, daily
candles. Its strategy report showed 1,231 trades, the last of which was open.
The local run on the 3,279 confirmed bars in ignored
`.local/community-coverage-20260923/volatility-reversion-native-confirmed-bars.csv`
(SHA-256 `278AE090E32E441E4CADD2BD70955DDAD650AFEEEDA21FE17651A18F5092593A`)
has 1,230 closed trades and one open order. The report date range begins on
2017-10-01, matching the local data start.

The four latest closed trades matched the native report's entry and exit
dates, prices, quantity, and profit:

| Entry UTC | Exit UTC | Side | Entry | Exit | Qty | Profit (USDT) |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| 2026-09-17 | 2026-09-18 | Short | 76,206.00 | 76,417.01 | 2 | -422.02 |
| 2026-09-15 | 2026-09-17 | Long | 78,189.20 | 76,206.00 | 2 | -3,966.40 |
| 2026-09-14 | 2026-09-15 | Short | 76,842.01 | 78,189.20 | 2 | -2,694.38 |
| 2026-09-13 | 2026-09-14 | Long | 77,278.73 | 76,842.01 | 2 | -873.44 |

Native DOM rows, source, symbol, and date range are retained in ignored
`.local/legacy-v1-20260923/native-recent-trades.json` (SHA-256
`67341D0C6EEA42A7BFD503DE9696A1EBFECBDC6C4A9D4C80FAC1F6CF927FD0B2`).
The native CSV export button did not produce a file in the configured
download directory, so the total and recent trades are independently
confirmed, while full native trade-list parity is still unproven.

This admits a measured v1 strategy shape, not every v1 order or declaration
variant. Chart settings and market data remain host inputs to the core.

## Full-history window follow-up (2026-09-25)

The same exact no-version source was compiled again in an independent TradingView layout on `BINANCE:BTCUSDT` 1D. The current chart report covers August 17, 2017 through September 25, 2026 and lists **1,248 numbered trades**, with trade 1,248 open after a September 24 short entry at `84,397.60` USDT for two units. The latest five closed trades, numbers 1,243–1,247, were read from the visible report and retained in ignored `.local/legacy-v1-20260923/native-20260925-recent-trades.json` (SHA-256 `23205447acdf050463ca7186bc1bedd0c3ca64e34a4f498005fa833895eda0d1`). The open trade's moving PnL is intentionally excluded.

The local comparison now uses the matching August 17, 2017 history start. The 3,324-bar `.local/community-coverage-20260923/full-daily-bars.csv` fixture (SHA-256 `9bec5ba5960c1eb8f099d30820d728cb083e1b3a9e0305a192d85e854cc5c660`) ends September 22. Two bars were appended from the September 24 TradingView chart export `I:\sys\下载\BINANCE_BTCUSDT, 1D (26).csv`: September 23 is confirmed and September 24 was forming at export time, but its `84,397.60` open is enough to price the final entry. The combined 3,326-bar ignored fixture is `.local/community-coverage-20260923/full-daily-through-sep24-native-bars.csv` (SHA-256 `cc2dc88aca8112159dabfacfe18263ca25c4dd5a96a424f91e47e6f319874bbf`). Its appended volume is zero because this strategy does not read volume. Reproduce with:

```powershell
target/debug/pine-compat.exe run tests/fixtures/runtime/legacy_v1_strategy_sma_reversal.pine --bars .local/community-coverage-20260923/full-daily-through-sep24-native-bars.csv --chart-symbol BINANCE:BTCUSDT --chart-timeframe 1D > .local/legacy-v1-20260923/full-history-through-sep24-20260925.json
```

The resulting JSON (SHA-256 `1515cfd1d8c62308b7c0a1a53ab3a7467ecfc526cc82bd53f69cc05d6aaf5da6`) has zero diagnostics, **1,247 closed trades and 1,248 filled orders**. All five visible native closed trades agree with the corresponding local trades on side, entry and exit UTC dates, prices, quantity, and displayed cent PnL. The local final closed trade enters September 18 at `76,417.01` and exits September 24 at `84,397.60`, with two units and `15,961.18` USDT profit; the last local order opens the two-unit short at `84,397.60` on September 24. The TradingView CSV button again produced no file in the configured download directory, so the full historical trade-list comparison remains unproven despite the aligned aggregate count and five visible rows. The editor showed an older-version caution; it did compile and run the source. [TradingView's script structure documentation](https://www.tradingview.com/pine-script-docs/language/script-structure/) states that omitted `//@version` selects v1.

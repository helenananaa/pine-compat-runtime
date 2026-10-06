# Pine v2/v3 strategy compatibility slice, 2026-09-23

This local slice admits Pine v2 and v3 `strategy(...)` declarations to the
host-neutral analyzer and broker. It also recognizes historical named
`strategy.entry(..., when=...)` calls in Pine v2. A separate implicit v1
strategy slice is documented in
[LEGACY_V1_STRATEGY_AUDIT_20260923.md](LEGACY_V1_STRATEGY_AUDIT_20260923.md).
The claim is limited to tested source shapes.

The original SMA crossover controls are
[`legacy_v2_strategy_sma_cross.pine`](../tests/fixtures/runtime/legacy_v2_strategy_sma_cross.pine)
(SHA-256 `4AD13CAC3A8BF179DF70597F31FD304C090C054B8F57A493321B338E671C7D64`)
and
[`legacy_v3_strategy_sma_cross.pine`](../tests/fixtures/runtime/legacy_v3_strategy_sma_cross.pine)
(SHA-256 `B166393C58977E81A7EF851B7FFB61BBA5AAEF8E1E71D9B558CE1779EEBABF48`).
Both calculate fast and slow SMAs, plot them, enter two long units on a
crossover, and close on a crossunder. Their full historical outputs equal the
explicit v6 control in the runtime test, including plots, orders, trades, and
equity on the test bars.

TradingView independently compiled both exact control sources on
`BINANCE:BTCUSDT`, daily candles, in the Pine Editor. Its strategy report
showed 624 trades including an open one. The three most recent **closed**
trades had these entry/exit dates and prices, quantity, and profit, all equal
to both local v2 and v3 runs on the corresponding confirmed bars:

| Entry UTC | Exit UTC | Entry | Exit | Qty | Profit (USDT) |
| --- | --- | ---: | ---: | ---: | ---: |
| 2026-09-15 | 2026-09-17 | 78,189.20 | 76,206.00 | 2 | -3,966.40 |
| 2026-09-13 | 2026-09-14 | 77,278.73 | 76,842.01 | 2 | -873.44 |
| 2026-09-07 | 2026-09-08 | 80,341.83 | 79,112.00 | 2 | -2,459.66 |

Native DOM rows for both versions are retained in ignored
`.local/legacy-v2-v3-20260923/native-recent-trades.json` (SHA-256
`903FE608DACC09DAAB583159A91A3B69385B3189F5BAC1F7280FA1B3701081C0`).
Ignored screenshots are in the same directory. The native CSV export button
did not produce a file in the configured download directory, so this is a
three-trade comparison rather than full native trade-list parity.

The local input is the 3,279 confirmed bars in ignored
`.local/community-coverage-20260923/volatility-reversion-native-confirmed-bars.csv`
(SHA-256 `278AE090E32E441E4CADD2BD70955DDAD650AFEEEDA21FE17651A18F5092593A`).
It begins on 2017-10-01; the TradingView chart begins on 2017-08-17.
The local run has 615 closed trades and an open order. Because the history
start differs, the native trade total cannot be compared directly.
The local v2 and v3 complete JSON outputs are equal on this dataset.

Broader v2/v3 strategy calls, declaration parameters, `strategy.close` named
`when`, alternative chart types and settings, realtime ticks, and other symbols
need separate qualification. This slice does not imply release readiness.

## Aligned history-origin follow-up (2026-09-25)

The earlier 615-closed-trade local comparison began October 1, 2017, whereas the native chart began August 17. Replaying both unchanged v2 and v3 controls on `.local/community-coverage-20260923/full-daily-bars.csv` (3,324 bars, August 17, 2017 through September 22, 2026; SHA-256 `9bec5ba5960c1eb8f099d30820d728cb083e1b3a9e0305a192d85e854cc5c660`) resolves that comparison boundary. Both local outputs are byte-identical, SHA-256 `00f5c63243360c2d08c71f522686e6de0af39ee54375189b729f258e2d0deb13`, retained as ignored `.local/legacy-v2-v3-20260923/full-history-v2-20260925.json` and `full-history-v3-20260925.json`.

Each version has **623 closed trades and one filled open entry**, totaling 624 numbered trades, equal to the previously observed native report's 624 including its open trade. The three latest native closed trades in the table above again match each local run's entry/exit UTC dates, prices, two-unit quantity, and cent PnL. The final local entry is September 18 at `76,417.01` USDT. The full native trade CSV is still unavailable, so aggregate count plus three visible rows remain the scope of this check. Reproduce either run from the repository root with:

```powershell
target/debug/pine-compat.exe run tests/fixtures/runtime/legacy_v2_strategy_sma_cross.pine --bars .local/community-coverage-20260923/full-daily-bars.csv --chart-symbol BINANCE:BTCUSDT --chart-timeframe 1D
```

Replace `v2` with `v3` for the other output.

## Public Pine v3 strategy follow-up (2026-09-25)

The complete public [Open Close Cross Strategy R5.1](OCC_R5_V3_NATIVE_PARITY_20260925.md) now executes in its original Pine v3 form. On 4,280 aligned `COINBASE:BTCUSD` daily bars, both exported moving-average plots match TradingView, and all 178 closed trades match native entry and exit dates and prices. Its default alternate 3D resolution uses a separately exported provider series. The native chart report's 100K account and the local default 1M account prevent a direct quantity or PnL parity claim. This is a broader original-script qualification than the two SMA controls above, while other v2/v3 scripts and settings still need separate evidence.

## Public Pine v2 strategy follow-up (2026-09-25)

The original [ANN Strategy v2](ANN_STRATEGY_V2_NATIVE_PARITY_20260925.md) compiles and runs. Both plot series match TradingView at all 4,282 aligned confirmed daily bars, and all 1,619 closed trades match direction, entry and exit dates, and prices. The final open short entry also matches. This original script exposed the missing v1/v2 `exp` alias. The comparison covers the script's default same-context `security` setting on `COINBASE:BTCUSD` 1D.

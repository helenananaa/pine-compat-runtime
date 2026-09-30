# Open-trade profit field repair

Verified 2026-09-30. Three independent official TradingView controls now match
the current source tree in nine execution-mode outputs. The repair covers
open-trade profit percentage direction and tick marking, plus zero returns for
an absent trade or out-of-range integer index. The expansion Goal remains active.

## Official captures and baseline defects

Chrome exported v5 short, v5 long and v6 short controls on FX:EURUSD / FXCM /
240 minutes directly to `I:\sys\下载`. Frozen effective Properties show
2,000,000 USD initial capital, explicit quantity 982990, zero commission and
slippage, infinite long/short leverage, bar-close script execution and one-tick
order delay. Source, native trade/chart CSVs, Properties, export settings and
screenshots are retained in `.local/profit-percent-native-20260930/`.

Each probe plots raw close, equity, total open profit and its percentage, open
trade count, and individual profit / profit percentage at indices 0, 1 and -1.
The eight exact native OHLCV rows cover the signal bar, active position, three
half-tick closes, a next-bar market close and the subsequent flat bar.
Entry is 1688029200000 ms at 1.09143; exit is 1688101200000 ms at 1.08699.
Both native trade exports and runtime trade records independently agree on
those times, prices, directions and quantities.

The preceding qualified binary reproduces the following residuals:

| Control | Maximum individual profit-percent error, percentage points | Missing versus native zero |
| --- | ---: | --- |
| v5 short | 0.8960721255600358 | Empty / flat index 0, and indices 1 / -1 |
| v5 long | 0.00045811458362310375 | Empty / flat index 0, and indices 1 / -1 |
| v6 short | 0.8960721255600358 | Empty / flat index 0, and indices 1 / -1 |

Native short profit percentages are positive while that position is profitable;
the baseline applied the long-price direction to the short position. The long
control independently isolates the raw-close versus tick-normalized mark error.
Native profit and profit-percent fields return zero for all tested absent
integer indices, both before entry and after exit.

An initial browser observation suggested a possible half-tick exit-price issue.
The executable baseline disproved it: entry and exit already match the native
prices within floating-point precision. No fill-price repair was needed.

## Implemented semantics and regression

`open_trade_profit_percent` derives its amount from the existing direction-aware,
tick-normalized `open_trade_profit`, then uses entry price times absolute
quantity as the denominator. The two builtins return numeric zero when an
integer index has no associated open trade.

A new pure Rust regression checks the independently exported short and long
percentages on all eight marks, integer indices 0 / 1 / -1, empty and closed
position states, and the existing correct exit fill. The older field regression
and exactly two golden snapshots were corrected for the native zero behavior.
The snapshot audit records each changed plot and verifies the updates only
replace absent profit/profit-percent values; other fields retain their prior
expectations. Fractional and missing-index arguments remain outside this native
control's qualification.

All three actual source files pass batch, incremental and realtime-history
execution. Complete outputs are byte-identical within each case. All eleven
plots match every one of the eight native rows, including explicit zero versus
missing-value checks; no missing-value exception is used in this receipt.
Maximum monetary error is below 2.4e-10 USD. Maximum profit-percent error is
below 2.6e-14 percentage points. Entry and exit price error is below 2.3e-16.
Fee-bearing positions and pyramiding have not received new native controls here.

## Complete-script guards and release gate

Twelve complete outputs were replayed against the previous marking candidate:
nine Hull outputs for Close / HL2 / HLC3 in all three modes, the full-history
equity diagnostic, and the BTCUSD weekly second-Hull55 / AUDUSD four-hour VAMA
SSL guards. Every output retains its previous SHA-256. This preserves the named
complete-script plot, trade, position and equity qualification.

The release gate completed with exit code 0 after correcting the two stale
snapshots: rustfmt, clippy with warnings denied, workspace / integration tests,
1,987 runtime and 242 CLI unit tests, 130 tooling tests, structural checks,
940 registered snapshots / 591 required runtime host assertions plus five
legacy-analysis assertions, actual Node WASM smoke, and 774 tests against the
newly built and installed Python wheel. One interrupted gate attempt stopped
after wheel construction; its partial log is retained separately and is not
treated as a pass. Completed replay outputs were verified before resuming the
one remaining SSL replay.

The immutable verifier checks current source and snapshot pins, exact native
fixture extraction, all plot values and missing states, trades, executable
hashes, every completed mode receipt, all twelve guards and the final gate log.

- HEAD: `2f9a83a0c333092167c7253571f56ee82a7ec0d0`
- Dirty core patch SHA-256: `e37a2ad566f57fa290db22a710c7ab9e9819ba38210bbc7c5699b5cab0ce887c`
- Candidate CLI SHA-256: `2c75149b6e0cbe3f3ff27e2e9bdd240857b07949f114e4c77e2ff68e0ccf6c6a`
- Final release log SHA-256: `29c74cac694420fdd7162f2bce41652b349969803da4fe97e62e0fc896d8143a`

## Browser restoration and remaining work

After the browser-plugin update, the old sampling tab could no longer be
inspected or closed through its supported control handle. A fresh tab on the
same saved layout confirms Coinbase BTCUSD daily with no temporary strategy.
The prior v5 mark-probe editor draft was restored and its complete contents
verified through the browser clipboard. The old EURUSD sampling tab remains
open; its cleanup is unverified. Restoration screenshots and draft bytes are
retained separately.

Earlier source pins remain historical receipts. Full Hull compatibility is
still incomplete: native displayed closed-trade monetary differences remain
approximately 0.00669 / 0.00560 / 0.00614 USD for Close / HL2 / HLC3. Next native
controls should extend profit fields to commissions and pyramiding, alongside
further parameter, symbol and timeframe coverage. No commit or push was made.

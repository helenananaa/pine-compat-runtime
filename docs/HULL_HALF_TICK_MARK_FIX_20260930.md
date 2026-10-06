# Half-tick position marking repair

Subsequent evidence: [open-trade profit field repair](OPEN_TRADE_PROFIT_FIELDS_FIX_20260930.md)
qualifies profit-percent direction and tick marking, and fixes the absent-trade
zero/null discrepancy retained below. That receipt preserves twelve complete
outputs from this candidate and records the browser restoration boundary.
This earlier receipt's editor-restoration claim used a DOM snapshot; the later
round verifies the complete restored draft through the browser clipboard.

Verified 2026-09-30 against the current dirty source tree. Independent native
long and short controls establish the mark-price rule for the three residual
FX:EURUSD four-hour closes. The repair removes the original full-history HL2
equity residuals and passes the complete release gate. Native displayed trade
amounts still differ, so full Hull compatibility remains incomplete.

## Independent official controls

Chrome exported two temporary v5 strategies directly from TradingView to
`I:\sys\下载`. Both use initial capital 2,000,000 USD, explicit quantity
982990, no commission or slippage, infinite leverage, execution on bar close,
and one-tick order delay. Effective Properties and export settings are frozen.
The entry signal is 1688014800000 ms; the next-bar fill is 1688029200000 ms at
1.09143. Native trade exports independently confirm direction, quantity, ID,
price and entry time.

Each source plots raw close, equity, total open profit, realized net profit,
position size, average price and individual open-trade profit. Six exact native
OHLCV rows from the signal through the third residual mark form the executable
fixture. Raw full CSV downloads remain frozen separately; the live final bar
and later historical rows are outside this isolated six-bar comparison.

| Raw close | Native short open profit USD | Native long open profit USD | Effective mark |
| ---: | ---: | ---: | ---: |
| 1.0881150000000002 | 3253.6968999999253 | -3253.6968999999253 | 1.08812 |
| 1.087225 | 4128.557999999982 | -4128.557999999982 | 1.08723 |
| 1.086995 | 4364.4756 | -4364.4756 | 1.08699 |

The baseline binary reproduces approximately 4.91495 USD errors in equity,
total open profit and individual trade profit at these three marks in both
directions. Both sources produce identical complete outputs across batch,
incremental and realtime-history execution.

One independent discrepancy is retained explicitly: before a trade exists,
TradingView exports individual trade profit as zero, while the runtime returns
null. That signal-bar field is excluded from the monetary comparison and is
not claimed repaired. Missing average-price values are checked as missing.

## Core repair

The broker normalizes the mark to the configured host-provided minimum tick
before calculating equity snapshots, current equity, total open profit and
individual trade profit. Finite prices already on the grid are preserved.
For off-grid values, the calculation is `(price / tick).round() * tick`.
On the third residual, division gives 108699.49999999999; multiplication by
100000 or adding 0.5 before flooring produces a different midpoint result.
The independent native observations determine the arithmetic sequence.

A Rust regression checks both directions, all three half-tick closes, normal
closes, raw-close plots and the recorded equity snapshots. The two frozen
native fixtures are also replayed in all three execution modes with the actual
new CLI. Maximum isolated monetary error is below 2.4e-10 USD.

## Full-history requalification

The unchanged published v4 Hull source, all 21,338 frozen bars, and the original
Close / HL2 / HLC3 input settings each pass three complete mode executions.
Both indicator plots and all closed-trade times, directions, prices and
quantities match. Closed-trade counts are 584 / 578 / 578 respectively. All
three surviving positions match their native entry time, price, ID and size.

Compared with the preceding quantity-repair candidate, each source changes
only its equity snapshots at bars 16302, 16305 and 16306. Complete plot,
order, trade, position and alert arrays are byte-equivalent as parsed values.
The native HL2 diagnostic probe now has no equity residual over 1e-7 USD:

| Full-history field | Maximum absolute error |
| --- | ---: |
| Equity | 6.51925802230835e-09 USD |
| Realized net profit | 3.892637323588133e-09 USD |
| Position size | 0 |

The nine independent budget-sizing controls pass another 27 mode executions.
The BTCUSD weekly second-Hull55 and AUDUSD four-hour VAMA SSL guards remain
byte-identical to their original receipts. Earlier captures retain their
original source pins.

## Release gate and evidence

`scripts/verify.ps1` completed with exit code 0: rustfmt, clippy with warnings
denied, workspace and integration tests, 1,986 runtime and 242 CLI unit tests,
130 tooling tests, structure checks, 940 registered snapshots / 591 required
host assertions plus five legacy-analysis assertions, actual Node WASM smoke,
and 774 tests against the newly built and installed Python wheel.

Evidence is in `.local/mark-price-native-20260930/`. The frozen verifier checks
source pins, native fixtures, per-mode receipts, candidate outputs and gate log.
The browser's temporary strategy was removed, the prior Hull editor draft was
restored, and the chart returned to Coinbase BTCUSD daily. Native screenshots
and restoration evidence are retained.

- HEAD: `2f9a83a0c333092167c7253571f56ee82a7ec0d0`
- Dirty core patch SHA-256: `68fdb700704b2f174d71298a6f4af573fd8f1acd6ab2304e04655542e8972160`
- Candidate CLI SHA-256: `c1b9346b8e7df13c0355395118519cf5a06e9fcdc4dfd24cb610d8b66e38e34e`
- Release log SHA-256: `23e25fd61890360ba45f143f6327dda8cfd445e9654f86c22436790ac27f7c21`

Qualification applies to this source tree and the named inputs and windows.
Native displayed closed-trade PnL differences remain 0.00669 / 0.00560 /
0.00614 USD for Close / HL2 / HLC3. Per-trade profit-percent marking has not
received an independent native control in this round. The unopened-trade
zero/null discrepancy also remains. No commit, push or full compatibility
qualification is claimed. The expansion Goal remains active.

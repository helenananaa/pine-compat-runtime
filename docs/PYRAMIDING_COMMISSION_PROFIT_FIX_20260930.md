# Pyramiding commission and profit-field repair

Native controls captured 2026-09-30 on FX:EURUSD / FXCM / 240 minutes.
The expansion Goal remains active. This receipt qualifies the named controls,
not arbitrary Pine scripts, live ticks or every fee allocation policy.

## Independent official evidence

Chrome exported three complete v5 probes directly to `I:\sys\下载`:

| Case | Direction | Commission | Entries |
| --- | --- | --- | --- |
| cash-short | Short | 10 USD per order | E1 100000, E2 300000 |
| cash-long | Long | 10 USD per order | E1 100000, E2 300000 |
| percent-short | Short | 0.1 percent | E1 100000, E2 300000 |

All use 2,000,000 USD initial capital, pyramiding=2, zero slippage and
zero margin requirements, bar-close execution and next-tick market fills.
E1 enters at 1688029200000 ms / 1.09143 and exits at 1688086800000 / 1.08723;
E2 enters at 1688043600000 / 1.08812 and exits at 1688101200000 / 1.08699.
The entries close separately; after E1 closes, open index 0 refers to E2.

The original full chart CSVs, trade CSVs, submitted sources, native UI states,
effective cash and percent Properties, export settings and screenshots are
frozen in `.local/pyramiding-fees-native-20260930/`. Cash-long has the same
explicit settings as cash-short; its direction is independently confirmed by
the native long trade export. Each local replay uses eight exact native OHLCV
rows, including three off-grid half-tick closes, plus host-neutral symbol,
currency, timeframe, point-value and price/quantity-grid metadata.

Each probe exports 21 fields: raw close, equity, net profit, total open profit
and percent, position size and average price, open/closed trade counts, and
profit / profit percent / commission for open and closed indices 0 and 1.
Native blanks and zeros are checked explicitly; none is excluded from comparison.

## Reproduced defects and implemented semantics

The previous qualified executable
`2c75149b6e0cbe3f3ff27e2e9bdd240857b07949f114e4c77e2ff68e0ccf6c6a`
reproduces the defects in all three execution modes:

| Observable | Baseline residual | Native-supported repair |
| --- | --- | --- |
| Individual open profit | Missing 20 USD round-trip cash fee, or up to 652.605 USD percent fee | Deduct paid entry fee and estimated exit fee at the tick-normalized mark |
| Individual open profit percent | Up to 0.1998002 percentage points | Divide net individual profit by entry notional plus allocated entry commission |
| Closed profit percent | Up to 0.000185017 percentage points | Use allocated entry commission in the closed trade's capital denominator |
| Total open profit percent | Up to 0.000010489 percentage points | Use initial capital plus net profit, including paid fees of still-open trades |
| Absent closed profit percent | Missing values before a close and at index 1 | Return numeric zero for absent integer indices |

For example, percent-short E1 has entry fee 109.143 USD. At the first mark,
gross open profit is 331 USD, estimated exit fee is 108.812 USD, individual
profit is 113.045 USD, and individual percent is 0.10347165455601175.
This distinguishes an exit fee valued at the current mark from twice the
entry fee. After E1 closes, E2's native individual profit is -313.533 USD,
including its entry fee 326.436 and estimated exit fee 326.097 USD.

Total `strategy.openprofit` remains gross price-difference profit. Paid entry
commission is already accounted for in `strategy.netprofit`; individual
`opentrades.commission` reports the paid entry fee, and closed commission
includes entry and exit fees. The executable baseline independently confirms
these existing amounts and correct trade fills, so those paths are preserved.

Closed percentage uses the allocated entry fee recorded at the actual close;
the implementation does not reconstruct a full cash-per-order fee for a
partially closed allocation. The cached percentage and the public closed
percentage now agree. Percentage excursion fields retain their existing
denominators, since this capture does not qualify those fields.

One permanent pure Rust regression freezes seven changed native fields for
all eight bars and all three cases. Three existing unit expectations were
updated for the fee-inclusive individual profit and native closed zero.
Exactly four plot arrays in two closed-trade golden snapshots replace absent
percent values with zero; the before files and exact change audit are frozen.
No broad snapshot regeneration is used.

## Verification and remaining scope

All three sources pass batch, incremental and realtime-history execution:
nine complete outputs, byte-identical within each case. Every one of the
21 plots agrees with every native row. Maximum monetary error is below
2.4e-10 USD, percentage error below 2.8e-14 percentage points, and fill-price
error below 2.3e-16. Position, quantity, direction, times and missing states
match. Native displayed percent-fee trade PnL differs by 0.004 / 0.003 USD,
within the two-decimal display rounding interval; unrounded native plot
profits agree within floating-point precision.

Twelve complete outputs retain their previous SHA-256: nine full-history
Hull Close / HL2 / HLC3 outputs across the three modes, the full-history
equity diagnostic, and BTCUSD weekly second-Hull55 / AUDUSD four-hour VAMA
SSL guards. Those complete scripts preserve prior plot, order, trade,
position and equity qualification.

The full release gate completed with exit code 0: rustfmt, clippy with warnings
denied, workspace/integration tests, 1,988 runtime and 242 CLI unit tests,
130 tooling tests, structural checks, 940 registered snapshots / 591 required
host runtime assertions and five legacy-analysis assertions, actual Node
WASM smoke, and 774 tests against the newly built and installed Python wheel.
The initial stale-snapshot failure is retained separately.

`freeze_verify.py` pins the current source tree, two changed goldens, exact
native fixtures, source submissions, executable, every completed mode,
complete-script guards, browser restoration evidence and the successful gate.
It verifies all native values and missing states again without changing the
frozen evidence. Re-run from the repository root:

```powershell
python .local/pyramiding-fees-native-20260930/freeze_verify.py
```

- HEAD: `2f9a83a0c333092167c7253571f56ee82a7ec0d0`
- Dirty core patch SHA-256: `be5cf60745bc497967b9b1f7499de3b0b35ba62436443d76872e88e873076e0e`
- Candidate CLI SHA-256: `b714dd49ff77f588bbc49e387b2d36559d3f9b50f155cbb018f9d71dce33bc66`
- Completed gate log SHA-256: `0a07bdc487e6b5455ee7ca80d071809d612c5cc3808a28901abf05d9dfe5f82a`

The original editor draft was restored and its full clipboard contents
compared with the captured original. The chart was restored to Coinbase
BTCUSD daily, the temporary strategy removed, and this round's sampling tab
closed successfully. This does not establish cleanup of the older stale tab
documented in the preceding receipt.

Next native coverage: partial closes and fee allocation, cash-per-contract
fees, v6 fee controls and non-unit point values. Full Hull trade monetary
display remains open: its prior maximum 0.00669 / 0.00560 / 0.00614 USD
residuals are outside a uniform two-decimal display qualification. No new
arbitrary-script or live-tick parity claim, local commit, push or release is made.

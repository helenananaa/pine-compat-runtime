# SSL Hybrid: BTCUSD weekly second Hull lengths 55 / 57

Verified 2026-09-29 with two fresh authenticated Chrome captures of unchanged
Pine v5 [SSL Hybrid Strategy](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/).
Source SHA-256: `1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.
This follows [weekly second T3 and cross dots](SSL_HYBRID_BTCUSD_WEEKLY_CROSSDOTS_20260929.md)
and qualifies the original optional second Hull branch at two odd lengths.

## Controlled settings

Coinbase `COINBASE:BTCUSD`, `1W`: Kijun v2 baseline length 30, divider 3,
HL2; JMA SSL2 length 5 / Phase 3 / Power 2; HMA exit length 15. Primary CF
is enabled RMA type 6 / length 20 / direction interval 2. Optional second MA
is enabled Hull type 4, length 55 / 57, with cross dots enabled.
Current chart resolution, TP/SL and Move SL on TP1 remain enabled.
Both CF averages read close separately from baseline HL2. Second factor input
7 remains recorded but is not consumed by the Hull branch.

Only the second MA length differs. Full native Inputs panels agree after
normalizing focus markers and numeric stepper controls and applying 55 to 57.
The pinned original expression is:

```pine
hullma2 = ta.wma(2 * ta.wma(source, length2 / 2) - ta.wma(source, length2), math.round(math.sqrt(length2)))
```

The source rounds its outer WMA length: sqrt(55) rounds to 7; sqrt(57)
rounds to 8. Both cases also exercise the original division of odd lengths.
The full numeric outputs and initial missing positions qualify this source
behavior; the length change affects both inner and outer windows, so no
separate causal attribution to the outer rounding alone is made.

Original call-site overrides: `12=Kijun v2`, `13=30`, `18=hl2`, `20=3`,
`21=2`, `19=3`, `587=true`, `589=20`, `590=7`, `591=6`, `594=2`,
`595=true`, `597=55` / `597=57`, `598=7`, `599=4`, `603=true`,
`652=true`, `669=true`, `671=true`.

Properties: USD 5,000 capital, 10% equity sizing, pyramiding 10, 0.04%
commission, zero slippage, default four historical ticks, bar-close / realtime
execution, requested limit price, one-tick delay and infinite leverage.
Host-neutral metadata: USD, price grid `1/100`, quantity precision 6, point
value 1. Market data remains in fixtures outside the runtime core.

## Capture provenance

Each raw chart has 616 rows. Confirmed input is 615 weekly bars from
2014-12-01 through 2026-09-21 UTC. Forming September 28 is retained in raw
evidence and excluded from execution. Native history jumps from December 15,
2014 to January 5, 2015; absent timestamps remain absent. Complete OHLCV
is copied exactly with seconds converted to milliseconds, with no synthetic
bars. Date-only trades use UTC session dates. Neither trade capture contains
forming-week entry or exit records; five older long entries survive.

Confirmed OHLCV is byte-identical across captures, SHA-256:
`66bd686a4938db3aefb9c7d933c53a82705cae907c75f7866b6e256010e85d67`.
All seven other named native series agree. Second-MA cells differ at 555
positions; cross-dot cells differ at 16 positions, including missing values.

Downloads in `I:\sys\下载`, September 29, UTC+8:

| Length | Chart CSV | Time | Trades CSV | Time |
| --- | --- | --- | --- | --- |
| 55 | `COINBASE_BTCUSD, 1W (3).csv` | 21:44:23 | `SSL_Hybrid_Strategy_COINBASE_BTCUSD_2026-09-29 (3).csv` | 21:44:09 |
| 57 | `COINBASE_BTCUSD, 1W (4).csv` | 21:45:44 | `SSL_Hybrid_Strategy_COINBASE_BTCUSD_2026-09-29 (4).csv` | 21:45:29 |

Frozen directories:

- `.local/ssl-hybrid-coinbase-btcusd-weekly-kijun30-cf-rma20-second-hull55-dots-on-hl2-20260929/`
- `.local/ssl-hybrid-coinbase-btcusd-weekly-kijun30-cf-rma20-second-hull57-dots-on-hl2-20260929/`

## Results

| Length | Named observations | Second-MA values | Second startup NA | Cross dots | Dot blanks | Total observations |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 55 | 4,700 | 555 | 60 | 9 | 606 | 4,709 |
| 57 | 4,697 | 552 | 63 | 7 | 608 | 4,704 |

Every native value and missing position matches at `1e-8` numeric tolerance.
Maximum second Hull and cross-dot absolute errors are zero in both cases.
Baseline / SSL1 startup 29, channels 30 and primary direction 21 are checked.
Candle Size false/NA exports as zero. No initialization bars are discarded.

Duplicate native Plot columns are checked against the complete 16-column
header. csv.reader column index 13 is compared to untitled runtime plot
ID 656; column 15 belongs to the unrelated Percent short market probe.
Both columns remain intact. The second trade-download button lost its label
after the UI update; visible DOM and screenshot identified the native button
before clicking it. Both downloads used the authenticated native interface.

Each case matches 60 closed trades, 34 explicit exit fills and five surviving
long entries. IDs, directions, dates, entry/exit prices, quantities, durations,
entry values, displayed closed PnL/commission and exit multisets match.
Quantity difference is zero at `1e-10` tolerance; prices use `1e-8`.
Maximum closed PnL / commission display deltas are `0.004541048640000156`
/ `0.0049194862599999944`, within 0.005. Native Size (value) is checked at
ten-significant-digit half quantum plus four ULP, maximum difference
`4.999998282073648e-08`. Open IDs, directions, dates, prices and quantities
match. Final long position is `0.00715 BTC`, average price `77672.10`.

Native closed records agree in every field between captures. The raw trade
CSVs differ only in the five open trades' displayed Net PnL, Return and
Cumulative PnL fields, consistent with changing current quotes. The verifier
initially rejected byte equality, then inspected every differing cell. It now
requires all closed rows equal and permits only these named display fields on
paired Open records; all differences are retained in the sensitivity receipt.
These current-market valuation fields are outside confirmed-history comparison.
All open execution fields agree and complete local strategy objects are equal.
No runtime repair was needed.

## Execution and reproducibility

All six runs terminate with exit zero, empty stderr and no diagnostics.
Batch, incremental and historical realtime complete JSON is byte-identical
within each case:

| Length | Complete output SHA-256 |
| --- | --- |
| 55 | `68bb5b14347eccddce50219bd3376890243c9142973c925fd9b29db6deb74b25` |
| 57 | `be345f1a2bd5bad118f8b5f45ee4a828794f1221944f346369f81db20b6a7cb8` |

Base HEAD `2f9a83a0c333092167c7253571f56ee82a7ec0d0`, core patch
`827e195c3a7db4422bbd657a5634990adb9d8d1db69658f6331df41a97cb2636`,
CLI `584ae27bb25fed15c038c5c55a9356720c1bff90b8db29ce8b403e8a08010f36`.
Source/core/golden hashes match the previously passing VAMA full gate, reused
without another full-gate execution. No new core change.

Each directory freezes original source, executable, patch, native CSVs, full
input/property/export/chart snapshots, screenshot, analysis, mode receipts,
comparison and verification receipt. Shared verifier checks exact commands,
metadata, hashes, native copies, full OHLCV, native gaps, all comparisons,
input isolation and cross-case strategy equality. Original HMA60/Close/divider1/
Power1, both MA types 1, second length 50, CF/second MA/cross dots disabled
and both exits enabled were restored on BTCUSD daily and recorded.

Run each case's run_modes.py, then from the repository root:

```powershell
python .local/verify_btc_weekly_second_hull_20260929.py
```

Aggregate `.local/btc-weekly-second-hull-matrix-20260929.json`; sensitivity
`.local/btc-weekly-second-hull-sensitivity-20260929.json`; terminal verifier log
`.local/btc-weekly-second-hull-verifier-20260929.log`. Verifier exits zero.
Qualification applies to this original source, settings, metadata and histories.
Live ticks, forming bars, different-timeframe requests, arbitrary scripts and
the unresolved Hull v4 strategy discrepancy remain outside the proven scope.
No commit or push; the expansion goal remains active.

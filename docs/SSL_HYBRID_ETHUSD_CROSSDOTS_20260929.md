# SSL Hybrid: ETHUSD native cross-dot output

Verified 2026-09-29 using two fresh authenticated Chrome captures of unchanged
Pine v5 [SSL Hybrid Strategy](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/).
Source SHA-256:
`1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.
This follows [secondary TEMA / T3 qualification](SSL_HYBRID_ETHUSD_SECOND_MA_TEMA_T3_20260929.md)
and adds direct native qualification of the original conditional cross-dot plot.

## Controlled settings

Coinbase `COINBASE:ETHUSD`, daily `1D`: Kijun v2 baseline length 30,
divider 3, HL2; JMA SSL2 length 5 / Phase 3 / Power 2; HMA exit length 15.
CF enabled with primary RMA type 6, length 20, direction interval 2; optional
second MA enabled with T3 type 8, length 50, factor input 7 (0.7). Current
chart resolution, TP/SL and Move SL on TP1 remain enabled.

Only Show Dots on Cross of Both MA's changes: off / on. Full native Inputs
panels agree after normalizing focus markers and numeric stepper controls and
applying this one checkbox difference. Both averages read chart close separately
from baseline HL2. The original plot emits the second MA value where the
enabled primary/secondary cross condition holds. Crossing bar highlights remain
disabled. Same-symbol, same-period request.security is exercised.

Exact original call-site overrides: `12=Kijun v2`, `13=30`, `18=hl2`, `20=3`,
`21=2`, `19=3`, `587=true`, `589=20`, `590=7`, `591=6`, `594=2`,
`595=true`, `597=50`, `598=7`, `599=8`, `603=false` / `603=true`,
`652=true`, `669=true`, `671=true`.

Native properties: USD 5,000 capital, 10% equity sizing, pyramiding 10,
0.04% commission, zero slippage, default four historical ticks, bar-close /
realtime execution, requested limit price, one-tick delay and infinite leverage.
Host-neutral metadata: USD, price grid `1/100`, quantity precision 4, point
value 1. External market data remains in fixtures outside the runtime core.

## Duplicate native plot columns

The chart CSV has two columns named Plot. Column index 13 (zero-based) is
the SSL cross-dot plot; column 15 is the unrelated Percent short market probe.
The cross-dot comparator uses `csv.reader`, asserts the complete 16-column
header, and compares column 13 to runtime plot ID 656, which has no title.
It does not use `DictReader` for this duplicate column, since that would retain
the later probe value. Both native columns remain intact in the frozen CSV.

The eight unique named columns retain their existing comparison. Native and
local missing positions are compared on every bar; numeric dots use absolute
tolerance `1e-8`. A comparator assumption that unnamed plots include a null
title initially raised KeyError; the helper now handles an omitted title field
while requiring the pinned source's plot ID. No runtime change was needed.

## Capture provenance

Each raw chart has 3,782 rows. Confirmed input is 3,781 daily bars from
2016-05-23 through 2026-09-28 UTC. The forming September 29 bar is retained
in raw evidence and excluded from input; neither trade capture contains
forming-period entry or exit records. Confirmed OHLCV is copied exactly,
converting seconds to milliseconds only; date-only trades use UTC session dates.

Confirmed OHLCV is byte-identical across the two captures, SHA-256
`fd06518bdd115dec2252f6beb108660f7a8595689b8f5feff25005fc6c5c7ee1`.
All eight named native columns are equal between captures. Native trade CSV
bytes and complete local strategy objects are also equal. No equality with
older batches' provider history is assumed.

Downloads in `I:\sys\下载`, September 29, UTC+8:

| Dots | Chart CSV | Time | Trade CSV | Time |
| --- | --- | --- | --- | --- |
| Off | `COINBASE_ETHUSD, 1D (14).csv` | 21:18:07 | `SSL_Hybrid_Strategy_COINBASE_ETHUSD_2026-09-29 (8).csv` | 21:17:51 |
| On | `COINBASE_ETHUSD, 1D (15).csv` | 21:19:52 | `SSL_Hybrid_Strategy_COINBASE_ETHUSD_2026-09-29 (9).csv` | 21:19:32 |

Frozen evidence directories:

- `.local/ssl-hybrid-coinbase-ethusd-daily-kijun30-cf-rma20-second-t350-dots-off-hl2-20260929/`
- `.local/ssl-hybrid-coinbase-ethusd-daily-kijun30-cf-rma20-second-t350-dots-on-hl2-20260929/`

The initial Chrome operation timed out. Browser-client setup and the documented
lightweight retry recovered communication; no additional installation or blank
window was needed. Both captures then used authenticated native downloads.

## Results

| Dots | Named observations | Cross dots | Cross-plot blank positions | Total nonblank observations | Closed trades | Explicit exit fills | Boundary |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| Off | 29,794 | 0 | 3,781 | 29,794 | 460 | 263 | Flat |
| On | 29,794 | 54 | 3,727 | 29,848 | 460 | 263 | Flat |

Every native dot and blank position matches. Maximum dot absolute error:
`5.002220859751105e-12`. The secondary T3 also retains 294 initial missing
bars and 3,487 numeric values, max error `1.2732925824820995e-11`. Baseline /
SSL1 startup 29, channels 30 and primary direction 21 remain checked. Candle
Size false/NA exports as zero. No initialization bars are dropped.

Trades match IDs, directions, entry/exit dates, prices, quantities, duration,
entry value and displayed PnL/commission; exit multisets and confirmed boundary
positions match. Both are flat, average price null, with no surviving entries.
Quantity tolerance is `1e-10`, observed difference zero; price tolerance `1e-8`.
Entry-value tolerance remains `1e-8`, max difference `2.842170943040401e-14`.
Maximum net PnL / commission display differences are
`0.004999323999999916` / `0.004966256000000002`, within 0.005 in both cases.

## Execution and reproducibility

All six runs terminate with exit zero, empty stderr and no diagnostics. Batch,
incremental and historical realtime complete JSON is byte-identical per case:

| Dots | Complete output SHA-256 |
| --- | --- |
| Off | `25c4039a2e5ae20d7ab78559d8c016ed78f97dac328df6ce3ebb4925d9a2ba53` |
| On | `91e83e4f17025f113cec5e531d9d42570aaaa1b9d3b5474ae02b9c6900e21bbb` |

Base HEAD `2f9a83a0c333092167c7253571f56ee82a7ec0d0`, core patch
`827e195c3a7db4422bbd657a5634990adb9d8d1db69658f6331df41a97cb2636`,
CLI `584ae27bb25fed15c038c5c55a9356720c1bff90b8db29ce8b403e8a08010f36`.
No new core repair. Source/core/golden hashes match the previously passing
VAMA full gate, reused without another full-gate execution.

Each case freezes original source, executable, patch, native CSVs, full input /
property / export / chart snapshots, screenshots, analysis, mode receipts,
comparison and verification receipt. The shared verifier checks exact commands,
metadata, input isolation, hashes, native copies, full OHLCV, all comparisons
and cross-case strategy equality. Shared helper and artifact hashes are retained.
HMA60/Close/divider1/Power1, both MA types 1, CF/second MA/cross dots disabled
and both exits enabled were restored on BTCUSD daily and recorded.

Run each case's `run_modes.py`, then from the repository root:

```powershell
python .local/verify_eth_daily_crossdots_20260929.py
```

Aggregate `.local/eth-daily-crossdots-matrix-20260929.json`;
sensitivity `.local/eth-daily-crossdots-sensitivity-20260929.json`;
terminal log `.local/eth-daily-crossdots-verification-20260929.log`.
Qualification applies to this original source and these settings, metadata and
histories. Live ticks, forming bars, different-timeframe requests, arbitrary
scripts and the unresolved Hull v4 discrepancy remain outside the proven scope.
No commit or push; the expansion goal remains active.

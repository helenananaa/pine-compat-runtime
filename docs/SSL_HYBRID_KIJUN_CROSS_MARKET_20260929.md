# SSL Hybrid Kijun v2: GBPUSD four-hour and EURUSD weekly

Two fresh, independent TradingView exports were captured through authenticated
Chrome on 2026-09-29. Both execute the unchanged public Pine v5
[SSL Hybrid Strategy by kevinmck100](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/).
Source SHA-256: `1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.

## Frozen setup and history

This extends the [EURUSD daily Kijun qualification](SSL_HYBRID_FX_EURUSD_DAILY_KIJUN30_HL2_20260929.md)
to FXCM `FX:GBPUSD` four-hour (`240`) and `FX:EURUSD` weekly (`1W`).
Both have identical original script inputs: Kijun v2 baseline length 30,
divider 3, source HL2, JMA SSL2 length 5 / Phase 3 / Power 2 and HMA exit
length 15. Overrides: `12=Kijun v2`, `13=30`, `18=hl2`, `20=3`, `21=2`,
`19=3`. Complete native input snapshots agree after removal of transient
focus/spin-button controls only. Other input values and checkbox states
remain in the comparison.

Kijun v2 combines 30- and 10-bar highest/lowest midpoints. Its one-argument
extrema use chart low/high, rather than selected `src`; frozen HL2 is not
evidence of an HL2 effect on the Kijun baseline. Original SSL2, exit,
channel and strategy paths retain their original behavior. Qualification
covers the composed script's exported fields, without separately exporting
or qualifying its internal midpoints as arbitrary primitives.

Each native property snapshot records USD 5,000 capital, 10% equity sizing,
pyramiding 10, commission 0.04%, zero slippage, default four historical
ticks, on bar close/realtime tick, requested limit price, one-tick delay
and infinite long/short leverage. Runtime metadata: USD, price grid
`1/100000`, integer quantity precision and point value 1.

| History | GBPUSD four-hour | EURUSD weekly |
| --- | --- | --- |
| Native rows | 21,347 | 2,908 |
| Confirmed bars | 21,346 | 2,907 |
| First bar, UTC | 2013-01-02 02:00 | 1971-01-03 22:00 |
| Last confirmed bar, UTC | 2026-09-29 05:00 | 2026-09-20 21:00 |
| Excluded forming bar, UTC | 2026-09-29 09:00 | 2026-09-27 21:00 |
| Native trade cutoff, UTC+8 | 2026-09-29 17:00 | 2026-09-28 |

Native rows are retained in full; only the last forming bar is removed from
runtime input. OHLCV values are copied exactly with seconds-to-milliseconds
conversion, without resampling. These are distinct market histories and
periods; this batch makes no single-parameter causal comparison between them.
GBPUSD qualification is limited to the history actually available/exported
from 2013; it does not establish an earlier four-hour history.

Downloads in `I:\sys\下载`:

| Case | Chart CSV | Trade CSV | Download times, UTC+8 |
| --- | --- | --- | --- |
| GBPUSD four-hour | `FX_GBPUSD, 240 (2).csv` | `SSL_Hybrid_Strategy_FX_GBPUSD_2026-09-29 (4).csv` | 19:09:49 / 19:09:37 |
| EURUSD weekly | `FX_EURUSD, 1W (18).csv` | `SSL_Hybrid_Strategy_FX_EURUSD_2026-09-29 (10).csv` | 19:13:16 / 19:12:57 |

Per-case evidence freezes original source, native chart/trade CSVs, full
input/property/export snapshots, chart state/screenshots, confirmed bars
and immutable CLI/core receipts. The unrelated Percent short market probe's
`Plot` column is excluded from SSL Hybrid comparisons.

## Confirmed native results

| Observation | GBPUSD four-hour | EURUSD weekly |
| --- | ---: | ---: |
| Nonblank exported observations | 149,262 | 20,189 |
| Series mismatches, including NA positions | 0 | 0 |
| Closed trades | 5,166 | 155 |
| Explicit exit fills | 1,538 | 39 |
| Surviving open entries | 0 | 5 |
| Final confirmed position | Flat, average price null | -432 at 1.1482 |
| Forming-period exit/entry records excluded | 0 / 0 | 0 / 0 |
| Maximum displayed net PnL difference | 0.004999948 | 0.004936656 |
| Maximum displayed commission difference | 0.004996736 | 0.003734880 |

All eight selected columns match: Candle Size > 1xATR, MA Baseline, SSL1,
Baseline Upper Channel, Basiline Lower Channel, MA UP, MA DOWN and the
disabled 2nd Multi-TimeFrame Moving Average. Numeric absolute tolerance is
`1e-8`; every missing position is checked. Candle Size false/NA exports as
zero; the disabled second MA is entirely missing.

Baseline and SSL1 retain **29 initial missing bars**, first defined at
zero-based bar **29**, in both histories. Channels start at bar 30 and
MA UP/DOWN at bar 21. Startup values and missing positions are not trimmed.

Closed trades match entry IDs, entry/exit directions, native displayed
dates/times, prices, exact quantities, entry values, durations and displayed
net PnL/commission. Monetary display tolerance remains 0.005; quantity
difference is zero. Explicit exit multisets match IDs, dates/times, prices
and quantities. Four-hour exports retain minute timestamps in UTC+8;
weekly exports retain session dates without finer intraday precision claims.

GBPUSD has no surviving entries and its final size is zero with null average
price. EURUSD's five native short entries are `ShortEntry1` through
`ShortEntry5`, Sep 21, 2026 at 1.1482, quantities **86, 130, 130, 43, 43**.
All five entry fields and the aggregate position of **-432** match locally.
Every executed native transaction precedes the appropriate forming-period
cutoff. Current-period indicator values, unrealized PnL and live ticks are
outside scope.

### Comparator repair

The first weekly comparison matched all chart columns, closed trades,
explicit exits and final position, but rejected its five open entry records
because it used exact binary-float equality for prices. Native CSV price
`1.1482` corresponds to local `1.1481999999999999`, a difference of
`2.220446049250313e-16`; IDs, dates, directions and quantities already agreed.
Open entry price checks now use the existing `1e-8` numeric tolerance used
for closed prices and series. IDs/dates/directions/quantities still match
exactly, and each entry must match one local order. The maximum open price
difference is recorded explicitly. No runtime patch or broader tolerance
change was needed. Raw native data remains unchanged.

## Execution and reproduction

All six executions exit zero without stderr or diagnostics. Batch,
incremental and historical realtime complete output bytes are identical
within each case:

| Case | Complete output SHA-256 |
| --- | --- |
| GBPUSD four-hour | `a2de7795f819e27fa1a746ee31f721576963d98fd31da3fc7e9bb419c8e1e6e3` |
| EURUSD weekly | `2c5b1bf05f52b678164f0716d15496b8e58c743ba2352505551d3e0d823a0656` |

Core base `2f9a83a0c333092167c7253571f56ee82a7ec0d0`, core patch
`827e195c3a7db4422bbd657a5634990adb9d8d1db69658f6331df41a97cb2636`,
immutable CLI `584ae27bb25fed15c038c5c55a9356720c1bff90b8db29ce8b403e8a08010f36`.
No new core repair. Current source/golden hashes match the passing VAMA
full gate, reused without a new full-gate run.

Evidence directories:

- `.local/ssl-hybrid-fx-gbpusd-fourhour-kijun30-divider3-hl2-20260929/`
- `.local/ssl-hybrid-fx-eurusd-weekly-kijun30-divider3-hl2-20260929/`

Prepare with `.local/prepare_kijun_cross_market_20260929.py`, case `gbp4h`
or `eurweek`, followed by the original chart/trade download filenames.
Run each directory's `run_modes.py` and `compare.py`, then
`.local/verify_kijun_cross_market_20260929.py`. The verifier checks current
HEAD/core/golden/CLI/gate identity, original source/download hashes, exact
OHLCV conversion, history and cutoff, startup, fresh source analysis, native
inputs/properties/export metadata, mode arguments and complete hashes,
fresh comparisons, open/flat boundary positions and restored browser state.
Per-case evidence includes `analysis.json`, mode receipts,
`native-comparison.log`, `comparison.json` and `current-verification.json`.
Shared log: `.local/kijun-cross-market-verification-20260929.log`.

| Case | Confirmed OHLCV SHA-256 |
| --- | --- |
| GBPUSD four-hour | `bc6931f5410350092c64a2b323945f118d52e5e24024614fe896977194c32662` |
| EURUSD weekly | `71caccfd0c372f4cd8a7b1bd54a6312661a2a244dfc4c861503e04f037fb438e` |

| Case | Native chart SHA-256 | Native trades SHA-256 |
| --- | --- | --- |
| GBPUSD four-hour | `9f5bbe4b92248cfcc353cdf687db9c17d8377f08b78f0ba32fb1735a54c63133` | `604593ba5a6308da490c365026e87c8c0ac5ac1399d283e1726eacd811c1ac3e` |
| EURUSD weekly | `094aa0ddc1c3284f19741984b7225bf21533784710d8f2564c49c3e009b40664` | `44e28b8b4de67d87dcc144defb6c4d7af4da5be1ebb4c4434e653a4ac7d8e165` |

Chrome was restored to Coinbase BTCUSD daily, HMA 60 close, Kijun divider 1,
JMA Phase 3 / Power 1, with full restored input/state/screenshot evidence.
Qualification remains specific to these source/settings/history/exports.
Historical realtime equality does not establish native live-tick parity
or arbitrary-script compatibility.

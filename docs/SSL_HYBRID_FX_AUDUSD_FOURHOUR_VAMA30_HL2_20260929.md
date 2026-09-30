# SSL Hybrid Strategy on AUDUSD four-hour VAMA 30 HL2 history

Captured 2026-09-29 through authenticated Chrome. The unchanged public
Pine v5 [SSL Hybrid Strategy by kevinmck100](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/)
has source SHA-256
`1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.
This qualifies FXCM `FX:AUDUSD`, four hours, VAMA baseline length 30,
HL2, and the original volatility lookback of **10**. The independently
captured [lookback 20 case](SSL_HYBRID_FX_AUDUSD_FOURHOUR_VAMA30_VOL20_HL2_20260929.md)
cross-checks the defect and parameter sensitivity on identical data.

## Frozen setup

Remaining inputs retain their original values, including the 2021-08-01
to 2030-10-01 trading range. Properties: USD 5,000 initial capital, equity
sizing 10%, pyramiding 10, commission 0.04%, slippage zero, default four
historical ticks, bar-close/realtime-tick execution, requested limit prices,
and one-tick order delay. Host-neutral CLI metadata: `FX:AUDUSD`, currency
USD, timeframe `240`, price grid `1/100000`, integer quantity precision,
and point value 1. Inputs use call sites `12=VAMA`, `13=30`, `18=hl2`.
The original source's volatility input is call site 22, default 10.

The native chart contains 21,345 rows starting 2013-01-02 02:00 UTC.
Local input has **21,344 confirmed bars** through 2026-09-28 21:00 UTC;
the forming 2026-09-29 01:00 UTC row is excluded. Every native execution
precedes that row. Confirmed OHLCV is byte-identical to the preceding
AUDUSD LSMA/TEMA captures. Native chart and trade exports were downloaded
into `I:\sys\下载` at 12:17 and 12:16 UTC+8 respectively.

## Defect and repair

VAMA combines EMA with highest/lowest deviations inside a volatility
window. Native baseline and SSL1 become defined at zero-based bar 29,
when the EMA first becomes valid. The old runtime required every source
sample in the extrema window to be finite. It delayed the two series by
nine additional bars and both baseline channels by eight valid positions;
one Candle Size signal also differed. The lookback 20 capture independently
showed corresponding delays of 19 and 18 positions.

`window_extreme_value` now skips missing source samples inside the existing
bar window, including a missing current sample. It returns `na` when the
window has no finite sample and does not extend the window into older
history. The existing complete-bar-window startup boundary is retained.
This is deterministic runtime behavior; external data supply stays outside
the core. The v3/v4/v5/v6 regression checks leading missing samples,
interior/current `na`, expiry of older samples, and an all-missing window.
It failed before the fix and passes afterward. Existing legacy and
finite-source startup tests also pass.

The original edge-case golden fixture encoded the same erroneous missing
values. Only its two affected highest/lowest result columns were corrected
after checking their three-bar source windows. The fixture itself and
remaining expected columns were preserved. The initial gate failure and
the final passing gate log are retained. Both VAMA cases preserve their
entire strategy result before versus after repair; the defect affected
pre-trading warmup outputs in these histories.

## Native results after repair

| Exported column | Nonblank positions | Mismatches |
| --- | ---: | ---: |
| Candle Size > 1xATR | 21,344 | 0 |
| MA Baseline | 21,315 | 0 |
| SSL1 | 21,315 | 0 |
| Baseline Upper Channel | 21,314 | 0 |
| Basiline Lower Channel | 21,314 | 0 |
| MA UP | 21,323 | 0 |
| MA DOWN | 21,323 | 0 |
| 2nd Multi-TimeFrame Moving Average | 0 | 0 |

All **149,248 nonblank positions** match at absolute tolerance `1e-8`,
including missing positions. Native Candle Size false/NA flattened to zero
is handled explicitly. All **3,890 closed trades** match entry IDs,
directions, UTC+8 minute entry/exit times, prices, integer quantities,
entry values, durations, displayed commission, and net PnL. All **five
open entries** match IDs/times/prices/quantities; forming-tick unrealized
PnL is outside comparison. All **2,112 explicit exit fills** match signal
IDs, times, five-decimal prices, and quantities. Maximum commission and
net-PnL display deltas are below $0.005. Diagnostics are empty.

Batch, incremental, and realtime-history outputs are byte-identical:
`cfbac6d28aa5a6b404466f0eee3afc07739a197536eaea16d6f8de10798de8ca`.
On these same bars, the rebuilt CLI's LSMA 30 control exactly reproduces
the [preceding native-qualified LSMA output](SSL_HYBRID_FX_AUDUSD_FOURHOUR_LSMA30_HL2_20260929.md),
including 3,185 closed trades. All four baseline/SSL1/channel columns change
at every mutually defined position between these algorithm settings.

## Current-source verification

Source base: `2f9a83a0c333092167c7253571f56ee82a7ec0d0`, plus the existing
broker work and this extrema repair. Current crates patch SHA-256:
`827e195c3a7db4422bbd657a5634990adb9d8d1db69658f6331df41a97cb2636`.
Frozen rebuilt CLI SHA-256:
`584ae27bb25fed15c038c5c55a9356720c1bff90b8db29ce8b403e8a08010f36`.
Core source, corrected golden, native inputs, outputs, and receipts are
hash-verified by `.local/verify_vama_cases_20260929.py`.

The fresh `scripts/verify.ps1` gate completed with exit 0: formatting,
workspace clippy with warnings denied, all workspace Rust tests including
**1,984 runtime** and **242 CLI** tests, structure check, **130 tool tests**,
host parity (**940** registered CLI snapshots, **591** required runtime
and **five** legacy-analysis Python/WASM assertions), actual WASM Node
smoke, and **774 installed-wheel Python tests**. Gate log SHA-256:
`0d85d41cdf71fa4a60f166d8737dd6207af6b7f755a15fa96f4de8a2f9f2aec9`.

## Artifacts and reproduction

Evidence directory: `.local/ssl-hybrid-fx-audusd-fourhour-vama30-hl2-20260929/`.
It retains the original source, chart/trade exports, snapshots, confirmed
bars, immutable rebuilt CLI, mode outputs, before-fix artifacts, regression
logs, full-gate logs/receipt, and `current-verification.json`.

| Artifact | SHA-256 |
| --- | --- |
| Native chart | `ba1514f3120aa04fea7ff894e62bcce095744dbf25975efe17d54f90b6e55fb1` |
| Native trades | `216f0466f68157d68944d8a9e80ed8400f31266df8b6bac146c7d5efb07b00b0` |
| Confirmed bars | `40b64a2dd0897ec2c47c206249cda3b9f34ca16bdfd533061285b72d4e34ad58` |

From the repository root:

```powershell
python .local/ssl-hybrid-fx-audusd-fourhour-vama30-hl2-20260929/prepare_bars.py
python .local/ssl-hybrid-fx-audusd-fourhour-vama30-hl2-20260929/run_modes.py
python .local/ssl-hybrid-fx-audusd-fourhour-vama30-hl2-20260929/compare.py
python .local/ssl-hybrid-fx-audusd-fourhour-vama30-hl2-20260929/run_control.py
python .local/verify_vama_cases_20260929.py
```

The shared verifier requires both VAMA cases and the recorded fresh gate.
Chrome is restored to Coinbase BTCUSD daily, HMA 60, close, volatility
lookback 10, verified by screenshot. This qualifies the named confirmed
history and selected exported fields; forming ticks and arbitrary scripts
or parameter combinations need further independent evidence.

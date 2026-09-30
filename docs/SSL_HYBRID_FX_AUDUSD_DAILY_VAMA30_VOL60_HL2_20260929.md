# SSL Hybrid Strategy on AUDUSD daily VAMA 30 HL2, volatility window 60

Captured 2026-09-29 through authenticated Chrome. The unchanged public
Pine v5 [SSL Hybrid Strategy by kevinmck100](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/)
has source SHA-256
`1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.
This qualifies FXCM `FX:AUDUSD`, **daily**, VAMA baseline 30, HL2, volatility
lookback 60. The [four-hour counterpart](SSL_HYBRID_FX_AUDUSD_FOURHOUR_VAMA30_VOL60_HL2_20260929.md)
independently checks the same startup boundary.

## Frozen setup and native session dates

Other inputs retain their original values, including the 2021-08-01 to
2030-10-01 trading range. Properties: USD 5,000 capital, 10% equity sizing,
pyramiding 10, commission 0.04%, no slippage, default four historical ticks,
execution on bar close/realtime tick, requested limit prices, one-tick
delay. Host-neutral metadata: `FX:AUDUSD`, USD, `1D`, grid `1/100000`,
integer quantity precision, point value 1. Overrides: `12=VAMA`, `13=30`,
`18=hl2`, `22=60`.

Native chart: 14,317 rows from 1971-01-03 22:00 UTC, corresponding to
the Jan 4 native session date. Local input has **14,316 confirmed bars**,
ending 2026-09-27 21:00 UTC (Sep 28 session date). The forming row at
2026-09-28 21:00 UTC (Sep 29 session date) is excluded. Native trade CSV
uses dates without times; entry/exit timestamps are compared as UTC+8
session dates, as displayed in the native report. Every exported execution
date precedes the forming session. No intraday timing claim follows from
this date-only export.

`I:\sys\下载\SSL_Hybrid_Strategy_FX_AUDUSD_2026-09-29 (5).csv` and
`FX_AUDUSD, 1D.csv` were downloaded at 12:51:17 and 12:51:38 UTC+8.
Source, settings, screenshot, native exports, and input bars are frozen.

## Native results and control

Both native and local baseline/SSL1/channels first become defined at
zero-based bar **59**. There are no runtime diagnostics.

| Exported column | Nonblank positions | Mismatches |
| --- | ---: | ---: |
| Candle Size > 1xATR | 14,316 | 0 |
| MA Baseline | 14,257 | 0 |
| SSL1 | 14,257 | 0 |
| Baseline Upper Channel | 14,257 | 0 |
| Basiline Lower Channel | 14,257 | 0 |
| MA UP | 14,295 | 0 |
| MA DOWN | 14,295 | 0 |
| 2nd Multi-TimeFrame Moving Average | 0 | 0 |

All **99,934 nonblank observations** match at absolute tolerance `1e-8`;
missing positions are also checked. Native Candle Size false/NA flattened
to zero is handled explicitly. All **336 closed trades**, **four open
entry records**, and **200 explicit exit fills** match IDs, directions,
session dates, prices, integer quantities, entry values, durations, and
displayed commission/net PnL. Monetary display tolerance is 0.005;
maximum PnL difference is 0.004970896, commission 0.002634736.
Open unrealized PnL on the forming bar is excluded.

A local-only window 20 control uses identical bars/source/metadata and
produces 493 closed trades. Its first baseline is at bar 29. Among
14,257 mutually defined observations, baseline/SSL1 change at
13,471/13,528 positions and both channels at 13,494. This control has
**no independent native qualification** and is not counted as another
passing native case.

## Execution evidence and reproduction

Batch, incremental, and historical realtime modes exit zero with empty
stderr and identical complete output SHA-256:
`14478cb1088a87f96f7fda13e547940b3b5de5c845ae1567765dec3e9fb8a1f3`.
Local window 20 control SHA-256:
`70f71d66e1cb45af09597ddb89cccb2c13b72f718d12f6420c35f294dcd16831`.

Core base `2f9a83a0c333092167c7253571f56ee82a7ec0d0`, core patch
`827e195c3a7db4422bbd657a5634990adb9d8d1db69658f6331df41a97cb2636`,
immutable CLI
`584ae27bb25fed15c038c5c55a9356720c1bff90b8db29ce8b403e8a08010f36`.
No additional core fix was needed. Current source/hash checks reuse the
identical-source VAMA window 10 full gate: 1,984 runtime tests, 242 CLI
tests, WASM, host parity, 130 tool tests, 774 wheel tests. The gate was
not rerun for this expansion.

Evidence directory:
`.local/ssl-hybrid-fx-audusd-daily-vama30-vol60-hl2-20260929/`.
Chart/trade SHA-256 values:
`bc92f40eef4058bf6b59866fd0c63f73aab741ffe710d6db5eb3a31411fc99ce`,
`08e76288e3cd850d66f81b7dfd7109506a481290adb4ddb905e71907409397e2`.
Confirmed bars SHA-256:
`67d9157fb50015d6f4fa75b804d387097cfcacfd8d4c94976679776b865f6f08`.
The directory retains input/export settings, screenshots, unchanged source,
immutable CLI/patch, all mode receipts, comparator, native comparison,
local control and its receipt, and current verification.

Reproduce with this case's `run_modes.py`, `compare.py`, `run_control.py`,
then `.local/verify_vama_vol60_cases_20260929.py`. The verifier also checks
the four-hour counterpart, current source and frozen artifacts, startup,
controls, native results and the reused full gate. Browser was restored
to Coinbase BTCUSD daily, HMA 60 close, volatility lookback 10.

Scope is the named original script, frozen parameters and daily history,
and exported fields. Historical realtime equality does not establish
native live-tick parity; arbitrary scripts/settings remain separate work.

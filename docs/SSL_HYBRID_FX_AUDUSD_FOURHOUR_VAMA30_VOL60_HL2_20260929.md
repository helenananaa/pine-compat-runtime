# SSL Hybrid Strategy on AUDUSD four-hour VAMA 30 HL2, volatility window 60

Captured 2026-09-29 through authenticated Chrome after the browser connection
was restored. The unchanged public Pine v5
[SSL Hybrid Strategy by kevinmck100](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/)
has source SHA-256
`1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.
This qualifies FXCM `FX:AUDUSD`, four hours, VAMA baseline 30, HL2, and
volatility lookback **60**, crossing the baseline length of 30.

## Frozen setup and history

Other inputs retain their original values, including the 2021-08-01 to
2030-10-01 trading range. Properties: USD 5,000 initial capital, equity size
10%, pyramiding 10, commission 0.04%, zero slippage, default four ticks per
historical bar, execution on bar close/realtime tick, requested limit prices,
and one-tick order delay. CLI metadata: `FX:AUDUSD`, currency USD, timeframe
`240`, price grid `1/100000`, integer quantity precision, point value 1.
Overrides are `12=VAMA`, `13=30`, `18=hl2`, `22=60`.

Native chart: 21,345 rows beginning 2013-01-02 02:00 UTC. Local input contains
**21,344 confirmed bars**, ending 2026-09-28 21:00 UTC; the forming row at
2026-09-29 01:00 UTC is excluded. Native executions precede that row.
Confirmed OHLCV is byte-identical to the preceding VAMA windows 10/20.
Trade and chart downloads in `I:\sys\下载` were saved at 12:48:10 and
12:48:29 UTC+8 respectively, as the AUDUSD strategy/chart `(4).csv` files.

## Native startup boundary and results

Both native and local baseline/SSL1 are missing for the first **59** bars;
the first defined value is at zero-based bar 59. Baseline channels also
begin there. The independently captured window 20 case begins at bar 29.
This supports retaining the complete global bar-window startup boundary
while skipping missing source samples within an available window. It does
not support removing that boundary. No additional core change was needed.

| Exported column | Nonblank positions | Mismatches |
| --- | ---: | ---: |
| Candle Size > 1xATR | 21,344 | 0 |
| MA Baseline | 21,285 | 0 |
| SSL1 | 21,285 | 0 |
| Baseline Upper Channel | 21,285 | 0 |
| Basiline Lower Channel | 21,285 | 0 |
| MA UP | 21,323 | 0 |
| MA DOWN | 21,323 | 0 |
| 2nd Multi-TimeFrame Moving Average | 0 | 0 |

All **149,130 nonblank observations** match at absolute tolerance `1e-8`;
missing positions are also compared. Native Candle Size false/NA flattened
to zero is handled explicitly. All **2,067 closed trades**, **five open
entry records**, and **1,400 explicit exit fills** match. Comparison covers
IDs, directions, UTC+8 minute timestamps, prices, integer quantities,
entry values, bar durations, displayed net PnL and commission. Monetary
display tolerance is 0.005; maximum net-PnL difference is 0.004998552 and
commission difference 0.004998120. Forming unrealized PnL is excluded.
There are no runtime diagnostics.

The native-qualified window 20 output is used directly as a same-bar control
on the identical current CLI/source. It has 2,964 closed trades. Among
21,285 mutually defined observations, baseline/SSL1 change at 20,263/20,228
positions, and both channels at 20,320. This verifies parameter sensitivity
without treating overlapping bars as additional independent history.

## Execution evidence and reproduction

Batch, incremental, and historical realtime modes all exit zero, with empty
stderr and identical complete output SHA-256:
`145ff946d6edb405dcc4cecb684b98ca481aa80023f523855ec056ffc032ea17`.
This historical realtime mode is not a live forming-tick parity claim.

Core base: `2f9a83a0c333092167c7253571f56ee82a7ec0d0`, plus frozen core patch
`827e195c3a7db4422bbd657a5634990adb9d8d1db69658f6331df41a97cb2636`.
Immutable CLI SHA-256:
`584ae27bb25fed15c038c5c55a9356720c1bff90b8db29ce8b403e8a08010f36`.
The verified identical source reuses the passing VAMA window 10 full gate,
including 1,984 runtime tests, 242 CLI tests, WASM, host parity, 130 tool
tests, and 774 wheel tests. That gate was not rerun in this expansion.

Evidence directory:
`.local/ssl-hybrid-fx-audusd-fourhour-vama30-vol60-hl2-20260929/`.
It contains unchanged source, native files, bars/settings/screenshot hashes,
CLI and patch, all mode commands/exit codes/output hashes, the comparator,
comparison results, capture provenance, current verification, and browser
restore screenshot. Chart/trade SHA-256 values are respectively
`2cffde17deef57b6209d0786b31264878b047cf3af0f7e1d4a96ae6a5069014a` and
`49cc898a1cd61b1a940e2041d3f7116b9836ff61d66b79e48ed616c739a438d7`.
Confirmed bars SHA-256:
`40b64a2dd0897ec2c47c206249cda3b9f34ca16bdfd533061285b72d4e34ad58`.

Run the case's `run_modes.py`, then `compare.py`, and
`.local/verify_vama_vol60_cases_20260929.py` after both documented cases and
the daily control are available. The verifier checks current HEAD/core diff,
source files, native artifacts, frozen CLI, modes, startup, comparisons,
control sensitivity, and the full-gate identity. Browser was restored to
Coinbase BTCUSD daily, HMA 60 close, volatility lookback 10.

The [daily counterpart](SSL_HYBRID_FX_AUDUSD_DAILY_VAMA30_VOL60_HL2_20260929.md)
qualifies the same parameter boundary on another cadence. This report is
limited to this original script, settings, history, and compared fields;
arbitrary parameters, other scripts, and live ticks remain separate work.

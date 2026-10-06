# SSL Hybrid Strategy on USDJPY four-hour EMA 30 HL2 history

Captured 2026-09-29 through the user's authenticated Chrome TradingView
session. The unchanged public Pine v5 [SSL Hybrid Strategy by
kevinmck100](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/)
has SHA-256 `1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.
This case changes baseline type to EMA, length to 30, and source to HL2.
Other original inputs remain at their defaults. The native chart identifies
FXCM `FX:USDJPY`, four hours. Properties show JPY 5,000, 10% equity sizing,
pyramiding 10, 0.04% commission, zero slippage, default four-tick historical
detail, and one-tick execution delay. No currency conversion is exercised.

## Native comparison

The export contains 21,346 bars from 2013-01-02 02:00 UTC. Excluding the forming
2026-09-29 01:00 UTC bar leaves **21,345 confirmed bars**, ending at
2026-09-28 21:00 UTC. There are no forming-bar trades or open entries in the
native report. Host-neutral CLI metadata supplies currency JPY, timeframe
240, price grid 1/1000, integer quantity precision, and point value one.

All **149,255 nonblank positions** across eight exported indicator columns
match, including missing positions, at absolute tolerance `1e-8`. Native
false/NA Candle Size values flattened to zero are accounted for explicitly.
All **470 closed trades** match entry IDs, directions, entry and exit times,
three-decimal prices, quantities, entry values, durations, commission, and
net PnL. All **319 explicit exit fills** match signal, time, price, and quantity.
Maximum net-PnL and commission display deltas are JPY 0.0049892 and 0.0049976,
within the native two-decimal display precision. There are no diagnostics.

These are named-field, confirmed-history comparisons. They do not establish
parity for every report field, chart object, arbitrary script, or live forming
tick. `run-realtime-history` exercises historical input through the realtime API.

## Sensitivity and verification

Compared with the previous HMA 30 export, four confirmed-bar volume fields
were revised; timestamps and OHLC are unchanged. `bars-receipt.json` records
the exact differences. Running HMA 30 on these exact new bars still produces
**892 closed trades** and is byte-identical to its preceding qualified output.
EMA 30 produces 470 trades. Baseline, SSL1, and both baseline channels differ
at all 21,312 mutually defined positions, confirming the type override is active.

Batch, incremental, and realtime-history outputs are byte-identical, SHA-256
`ff400a0bf4201379cd1651fe2aa15e29427cdfbd3ff6f342a1ae7ea963aadc66`.
No new core change was required. Current core file hashes and the immutable
CLI match the preceding HMA 20 full-gate receipt. That source passed formatting,
workspace Clippy and Rust tests, structural and host parity checks, 130 tool
tests, WASM/Node smoke, and 774 fresh-wheel Python tests; runtime and CLI test
counts were 1,983 and 242. The unchanged-source gate is reused and linked by
hash in `current-verification.json`; it is not a new gate run.

The base is `2f9a83a0c333092167c7253571f56ee82a7ec0d0` plus the existing
local broker patch, SHA-256
`fe7c8f1ed5bcaed259a08cd284168cdf752d139bd0271e5a62022501a2cd2fc4`.
The broader expansion goal remains active.

## Evidence and reproduction

The ignored directory `.local/ssl-hybrid-fx-usdjpy-fourhour-ema30-hl2-20260929/`
retains unchanged source, native exports, settings snapshots, chart screenshot,
bars, immutable CLI, mode outputs, comparator, HMA control, and receipts.
Native downloads in `I:\sys\下载` were captured at 11:18 UTC+8:
`FX_USDJPY, 240 (3).csv` and
`SSL_Hybrid_Strategy_FX_USDJPY_2026-09-29 (2).csv`.
The layout was restored to Coinbase BTCUSD daily, HMA 60, close.

Native chart SHA-256:
`d54c6313da1d7d12a1ab88251a0e9479431f05052cccf6c14de1cbc61f0f59fd`.
Native trades SHA-256:
`848cb1d5eedcff5034e00ebb638ddb97b5107318e886ee4781d3ea022602a591`.
Frozen bars SHA-256:
`8341a4896e42b151b51457703d72ebd2d529561f630d9fb5fb1b509fe049d224`.

From the repository root:

```powershell
python .local/ssl-hybrid-fx-usdjpy-fourhour-ema30-hl2-20260929/prepare_bars.py
python .local/ssl-hybrid-fx-usdjpy-fourhour-ema30-hl2-20260929/run_modes.py
python .local/ssl-hybrid-fx-usdjpy-fourhour-ema30-hl2-20260929/compare.py
python .local/ssl-hybrid-fx-usdjpy-fourhour-ema30-hl2-20260929/run_control.py
python .local/ssl-hybrid-fx-usdjpy-fourhour-ema30-hl2-20260929/record_verification.py
```

The mode receipts record all CLI arguments, including input call sites
`12=EMA`, `13=30`, and `18=hl2`, and terminal exit codes.

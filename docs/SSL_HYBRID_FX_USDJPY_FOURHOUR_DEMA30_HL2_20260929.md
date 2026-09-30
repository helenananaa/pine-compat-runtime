# SSL Hybrid Strategy on USDJPY four-hour DEMA 30 HL2 history

Captured 2026-09-29 through the user's authenticated Chrome TradingView
session. The unchanged public Pine v5 [SSL Hybrid Strategy by
kevinmck100](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/)
has source SHA-256
`1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.
Inputs select DEMA baseline, length 30, HL2, and other original defaults.
The chart identifies FXCM `FX:USDJPY`, four hours. Properties show JPY 5,000,
10% equity sizing, pyramiding 10, 0.04% commission, zero slippage, default
four-tick historical detail, and one-tick execution delay. Currency conversion
is not exercised.

## Frozen history and native comparison

The native chart export has 21,346 bars beginning 2013-01-02 02:00 UTC.
Excluding the forming 2026-09-29 01:00 UTC bar leaves **21,345 confirmed bars**
through 2026-09-28 21:00 UTC. Native trades include no forming-bar execution
and no open entry. Confirmed OHLCV and timestamps are identical to the preceding
EMA 30 capture. The CLI receives host-neutral currency JPY, timeframe 240,
price grid 1/1000, integer quantity precision, and point value one.

All **149,141 nonblank positions** across eight exported indicator columns
match at absolute tolerance `1e-8`, with missing positions also checked.
The baseline and SSL1 have 58 initial missing values, versus 29 for EMA 30;
the nested EMA warmup agrees with the native output. The comparator explicitly
accounts for false/NA Candle Size values flattened to zero by the CSV.

All **658 closed trades** match entry IDs, direction, entry and exit times,
three-decimal prices, quantities, entry values, duration, commission, and net
PnL. All **480 explicit exit fills** match signal, time, price, and quantity.
Maximum net-PnL and commission display deltas are both JPY 0.0049856, within
the native two-decimal display precision. There are no runtime diagnostics.
Only these named fields and confirmed-history settings are qualified;
arbitrary scripts, other chart/report fields, and live forming ticks are not.

## Sensitivity and execution modes

On the same bars EMA 30 produces **470 closed trades** and remains byte-identical
to its preceding qualified output. DEMA 30 produces 658. Baseline, SSL1, and
both baseline channels differ at all 21,287 mutually defined positions.
The algorithm override therefore changes executable results.

Batch, incremental, and realtime-history outputs are byte-identical, SHA-256
`dd7dc0f21aafed8150d2c578f333c8b5acfcdbbeb25acac93dbd51a8fc7b2b84`.
The realtime-history mode runs historical bars through the realtime API;
it does not establish native forming-tick parity.

No new core change was needed. Current core file hashes and the immutable CLI
match the full-gate receipt from the preceding HMA 20 case. That exact source
passed formatting, workspace Clippy/Rust tests, structural and host parity
checks, 130 tool tests, WASM/Node smoke, and 774 fresh-wheel Python tests;
runtime and CLI test counts were 1,983 and 242. This unchanged-source gate is
reused, with its receipt and log hashes linked in `current-verification.json`.
It is not presented as a newly run full gate.

The base is `2f9a83a0c333092167c7253571f56ee82a7ec0d0` plus the existing
local broker patch, SHA-256
`fe7c8f1ed5bcaed259a08cd284168cdf752d139bd0271e5a62022501a2cd2fc4`.
The broader compatibility expansion goal remains active.

## Evidence and reproduction

The ignored directory `.local/ssl-hybrid-fx-usdjpy-fourhour-dema30-hl2-20260929/`
retains unchanged source, native CSVs, settings snapshots, screenshot, exact
bars, immutable CLI, comparator, mode outputs, EMA control, patch, and receipts.
Native downloads at 11:25 UTC+8 in `I:\sys\下载` are
`FX_USDJPY, 240 (4).csv` and
`SSL_Hybrid_Strategy_FX_USDJPY_2026-09-29 (3).csv`.
The browser layout was restored to Coinbase BTCUSD daily, HMA 60, close.

Native chart SHA-256:
`278046858116d4d4d4175ec6179e23471c1e283ba42718e79f482002c76ce1b9`.
Native trades SHA-256:
`3426b23b272fd711fdea0dd0e840ef7e994f891b917ae3dd02a69fd89b2d896e`.
Frozen bars SHA-256:
`8341a4896e42b151b51457703d72ebd2d529561f630d9fb5fb1b509fe049d224`.

Reproduce from the repository root:

```powershell
python .local/ssl-hybrid-fx-usdjpy-fourhour-dema30-hl2-20260929/prepare_bars.py
python .local/ssl-hybrid-fx-usdjpy-fourhour-dema30-hl2-20260929/run_modes.py
python .local/ssl-hybrid-fx-usdjpy-fourhour-dema30-hl2-20260929/compare.py
python .local/ssl-hybrid-fx-usdjpy-fourhour-dema30-hl2-20260929/run_control.py
python .local/ssl-hybrid-fx-usdjpy-fourhour-dema30-hl2-20260929/record_verification.py
```

Mode receipts record input overrides `12=DEMA`, `13=30`, `18=hl2`, all
metadata arguments, and terminal exit codes.

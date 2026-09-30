# SSL Hybrid Strategy on USDJPY daily TEMA 30 HL2 history

Captured 2026-09-29 through the user's authenticated Chrome TradingView
session. The unchanged public Pine v5 [SSL Hybrid Strategy by
kevinmck100](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/)
has source SHA-256
`1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.
Inputs select TEMA baseline, length 30, and HL2; other original inputs remain
at defaults. The native chart identifies FXCM `FX:USDJPY`, one day.
Properties show JPY 5,000, 10% equity sizing, pyramiding 10, 0.04% commission,
zero slippage, default four-tick historical detail, and one-tick execution
delay. Currency conversion is not exercised.

## Matching the native history start

The first CSV began in November 1989, while the strategy report identified a
January 1971 history start. Its first exported baseline was already defined,
showing that earlier warmup was missing. That truncated export was replaced
after explicitly loading the 1971 history in Chrome. The final native chart
contains **14,317 rows**, starting at 1971-01-03 22:00 UTC (January 4 session
date). The first baseline is missing, as expected before warmup.

Excluding the forming 2026-09-28 21:00 UTC daily bar leaves **14,316 confirmed
bars** through 2026-09-27 21:00 UTC. Native trades include no forming-bar
execution and no open entry. The host-neutral CLI supplies currency JPY,
timeframe `1D`, price grid `1/1000`, integer quantity precision, and point
value one. Native daily reports display session dates without intraday times;
the comparator checks dates in UTC+8 and does not claim hidden intraday timing.

## Native comparison and controls

All **99,822 nonblank positions** across eight exported indicator columns
match at absolute tolerance `1e-8`, including missing positions. Baseline and
SSL1 have 87 initial missing values. Native false/NA Candle Size values
flattened to zero are accounted for explicitly.

All **134 closed trades** match entry IDs, direction, entry and exit session
dates, three-decimal prices, quantities, entry values, durations, commission,
and net PnL. All **77 explicit exit fills** match signal, date, price, and
quantity. Maximum net-PnL and commission display deltas are JPY 0.004796 and
0.004918, within the native two-decimal display precision. Diagnostics are empty.

The local DEMA 30 sensitivity control produces **138 closed trades** on these
same bars. Baseline, SSL1, and both baseline channels differ at all 14,229
mutually defined positions. This control is local evidence that the type
override changes results; DEMA daily has not been independently qualified
against a native export in this case.

Batch, incremental, and realtime-history outputs are byte-identical, SHA-256
`865455c0159a9bbec9e52149854e87852246e05aaca86d42f3803df749c6e3ac`.
Realtime-history exercises historical bars through the realtime API, and
does not establish live forming-tick parity. Qualification covers this
named confirmed-history case and selected fields; arbitrary scripts and
every chart/report field remain unproven.

## Current source and evidence

No new core change was needed. Current core file hashes and the immutable CLI
match the preceding four-hour HMA 20 full-gate receipt. That exact source
passed formatting, workspace Clippy/Rust tests, structural and host parity
checks, 130 tool tests, WASM/Node smoke, and 774 fresh-wheel Python tests;
runtime and CLI test counts were 1,983 and 242. This unchanged-source gate is
reused and linked by receipt/log hashes in `current-verification.json`,
rather than claimed as a newly run full gate.

The base is `2f9a83a0c333092167c7253571f56ee82a7ec0d0` plus the existing
local broker patch, SHA-256
`fe7c8f1ed5bcaed259a08cd284168cdf752d139bd0271e5a62022501a2cd2fc4`.
The broader expansion goal remains active.

The ignored directory `.local/ssl-hybrid-fx-usdjpy-daily-tema30-hl2-20260929/`
retains unchanged source, final native CSVs, settings snapshots, screenshot,
bars, immutable CLI, comparator, mode outputs, local DEMA control, patch,
and receipts. Final downloads in `I:\sys\下载` are `FX_USDJPY, 1D (1).csv`
(11:40 UTC+8) and `SSL_Hybrid_Strategy_FX_USDJPY_2026-09-29 (5).csv`
(11:38 UTC+8). The browser was restored to Coinbase BTCUSD daily, HMA 60, close.

Native chart SHA-256:
`644dec3e09f7e6ce3bf112d91cc72f5e353a16e6f3f37a0a47eefcc2828648a8`.
Native trades SHA-256:
`991da96bc1f35457dd11c294d25862b50250f73b59283829720056c514a9ce4f`.
Frozen bars SHA-256:
`82d50c04dc1d412abe0a68dc2bf32b540ab0ccc6f2aca3c792b9f85ee37b5e24`.

From the repository root:

```powershell
python .local/ssl-hybrid-fx-usdjpy-daily-tema30-hl2-20260929/prepare_bars.py
python .local/ssl-hybrid-fx-usdjpy-daily-tema30-hl2-20260929/run_modes.py
python .local/ssl-hybrid-fx-usdjpy-daily-tema30-hl2-20260929/compare.py
python .local/ssl-hybrid-fx-usdjpy-daily-tema30-hl2-20260929/run_control.py
python .local/ssl-hybrid-fx-usdjpy-daily-tema30-hl2-20260929/record_verification.py
```

Mode receipts record metadata arguments, overrides `12=TEMA`, `13=30`,
`18=hl2`, and terminal exit codes.

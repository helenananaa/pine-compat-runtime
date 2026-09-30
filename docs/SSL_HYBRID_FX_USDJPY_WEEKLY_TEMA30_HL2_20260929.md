# SSL Hybrid Strategy on USDJPY weekly TEMA 30 HL2 history

Captured 2026-09-29 through the user's authenticated Chrome TradingView
session. The unchanged public Pine v5 [SSL Hybrid Strategy by
kevinmck100](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/)
has source SHA-256
`1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.
Inputs select TEMA baseline, length 30, and HL2; other original defaults
remain. The chart identifies FXCM `FX:USDJPY`, one week. Properties show
JPY 5,000, 10% equity sizing, pyramiding 10, 0.04% commission, zero slippage,
default four-tick historical detail, and one-tick execution delay.
Currency conversion is not exercised.

## Frozen native history

The native chart contains **2,907 bars** starting 1971-01-03 22:00 UTC
(January 4 session date). The first baseline is missing before warmup.
Excluding the forming 2026-09-27 21:00 UTC bar leaves **2,906 confirmed bars**
through 2026-09-20 21:00 UTC. The native report has no forming-bar executions
and no open entry. The host-neutral CLI supplies currency JPY, timeframe
`1W`, price grid `1/1000`, integer quantity precision, and point value one.
Native weekly reports expose session dates without intraday times; the
comparator checks UTC+8 dates, without claiming hidden intraday timing.

All **19,952 nonblank positions** across eight exported indicator columns
match at absolute tolerance `1e-8`, with missing positions also compared.
The baseline and SSL1 have 87 initial missing values. Native false/NA Candle
Size values flattened to zero are accounted for explicitly.

All **28 closed trades** match entry IDs, direction, entry/exit session dates,
three-decimal prices, quantities, entry values, duration, commission, and net
PnL. All **24 explicit exit fills** match signal, date, price, and quantity.
Maximum net-PnL and commission display deltas are JPY 0.0049068 and
0.0048604, within native two-decimal display precision. Diagnostics are empty.

## Controls and verification

A local DEMA 30 sensitivity control produces **24 closed trades** on these
same bars. Baseline, SSL1, and both baseline channels differ at all 2,819
mutually defined positions. This is local sensitivity evidence; DEMA weekly
is not independently qualified against a native export in this case.

Batch, incremental, and realtime-history outputs are byte-identical, SHA-256
`695a1932fdde286359aac4b356125a346c67e81d84966348f22dab328db09905`.
Realtime-history runs historical bars through the realtime API and does not
establish native forming-tick parity. The named fields and confirmed-history
settings qualify; arbitrary scripts and every chart/report field remain unproven.

No new core change was required. Current core file hashes and the immutable
CLI match the preceding four-hour HMA 20 full-gate receipt. That source passed
formatting, workspace Clippy/Rust tests, structural and host parity checks,
130 tool tests, WASM/Node smoke, and 774 fresh-wheel Python tests; runtime and
CLI test counts were 1,983 and 242. The unchanged-source gate is reused and
linked by receipt/log hashes, rather than claimed as a new full-gate run.

The source base is `2f9a83a0c333092167c7253571f56ee82a7ec0d0` plus the existing
local broker patch, SHA-256
`fe7c8f1ed5bcaed259a08cd284168cdf752d139bd0271e5a62022501a2cd2fc4`.
The broader compatibility expansion goal remains active.

## Evidence and reproduction

Chrome transport initially timed out while the browser, installed extension,
and native-host configuration checks passed. The user authorized opening an
empty window in the existing profile; communication then recovered and the
native capture completed. No alternate browser or data provider was used.
The final layout was restored to Coinbase BTCUSD daily, HMA 60, close.

The ignored directory `.local/ssl-hybrid-fx-usdjpy-weekly-tema30-hl2-20260929/`
retains unchanged source, native CSVs, inputs/properties snapshots, screenshot,
bars, immutable CLI, comparator, mode outputs, local control, patch, and receipts.
Downloads in `I:\sys\下载` are `FX_USDJPY, 1W.csv` (11:51 UTC+8) and
`SSL_Hybrid_Strategy_FX_USDJPY_2026-09-29 (6).csv` (11:50 UTC+8).

Native chart SHA-256:
`7499dd699aba652a01f132702d1d1c5db78c224eb908a45c3691d02067b856f2`.
Native trades SHA-256:
`47e7df2b658eb3ca882a0e1a897b529607ec701eeeba7fc10d4e80fc1fa65c30`.
Frozen bars SHA-256:
`1d2d122bccfcb4321191fa3aa0db8a83fb5cfe34483cb2c23a7043dbf9cb8e55`.

Reproduce from the repository root:

```powershell
python .local/ssl-hybrid-fx-usdjpy-weekly-tema30-hl2-20260929/prepare_bars.py
python .local/ssl-hybrid-fx-usdjpy-weekly-tema30-hl2-20260929/run_modes.py
python .local/ssl-hybrid-fx-usdjpy-weekly-tema30-hl2-20260929/compare.py
python .local/ssl-hybrid-fx-usdjpy-weekly-tema30-hl2-20260929/run_control.py
python .local/ssl-hybrid-fx-usdjpy-weekly-tema30-hl2-20260929/record_verification.py
```

The receipts record metadata arguments, overrides `12=TEMA`, `13=30`,
`18=hl2`, terminal exit codes, native comparison, and the linked full gate.

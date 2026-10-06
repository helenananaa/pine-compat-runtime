# SSL Hybrid Strategy on USDJPY four-hour HMA 30 HL2 history

Captured 2026-09-29 from the user's authenticated Chrome TradingView session.
The unchanged public Pine v5 [SSL Hybrid Strategy by
kevinmck100](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/)
has source SHA-256
`1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.
The chart identifies FXCM `FX:USDJPY`, four hours. Inputs are HMA baseline
length 30 and `(H+L)/2` (`hl2`), retaining other original inputs. Properties
show JPY 5,000, 10% equity sizing, pyramiding 10, 0.04% commission, zero
slippage, default four-tick historical detail, and one-tick execution delay.
No foreign-currency conversion is exercised.

## Frozen data and comparison

The native export contains 21,346 bars from 2013-01-02 02:00 UTC. The forming
2026-09-29 01:00 UTC bar is excluded. The runtime receives **21,345 confirmed
bars**, through 2026-09-28 21:00 UTC. The native report contains no forming-bar
trades and no open entries. The host supplies currency `JPY`, timeframe `240`,
price grid `1/1000`, integer quantity precision, and point value one.

Compared with the HMA 20 capture, two confirmed-bar volume fields were revised:
101,782 to 101,509 and 127,100 to 126,664. OHLC and timestamps are identical.
`bars-receipt.json` retains the exact differences and hashes.

All **149,241 nonblank positions** in eight exported indicator columns match
at absolute tolerance `1e-8`, including missing positions. Native false/NA
Candle Size values represented as zero are handled by the comparator.
All **892 closed trades** match entry IDs, directions, UTC+8 report entry/exit
times, three-decimal prices, integer quantities, entry values, durations,
commission, and net PnL. All **581 explicit exit-order fills** match signal,
time, price, and quantity. Maximum net-PnL and commission display deltas are
JPY 0.004954400 and JPY 0.005000000, within the two-decimal report precision.
There are no diagnostics. The named fields qualify; other report fields,
all chart objects, arbitrary scripts, and live forming ticks remain unproven.

## Parameter control and verification

On these exact frozen bars, HMA 20 yields **1,088 closed trades**, while HMA 30
yields **892**. The HMA 20 control is byte-identical to the prior qualified
HMA 20 output despite the two volume revisions. The script's active settings
do not cause those revisions to change its exported runtime output.

Baseline, SSL1, and both baseline channels differ at all 21,312 mutually
defined positions. Batch, incremental, and realtime-history outputs are
byte-identical, SHA-256
`d892f98168421507494f7b408fad8fe243de67946e07541ed7781b02ff33156d`.
`current-verification.json` records terminal exit codes, comparison results,
the sensitivity control, and the unchanged-source gate reference.
This qualifies the named confirmed-history case.

No core change was required. Current core file hashes and the immutable CLI
match the preceding HMA 20 verification receipt. That exact source passed
the full `scripts/verify.ps1` gate: formatting, workspace Clippy and Rust
tests, structural checks, 130 tool tests, host parity checks, WASM/Node smoke,
and 774 fresh-wheel Python tests; runtime and CLI counts are 1,983 and 242.
These gate results are reused for unchanged source and explicitly linked
by hash, rather than presented as a new gate run.

The source base is `2f9a83a0c333092167c7253571f56ee82a7ec0d0` plus the
existing local broker patch, SHA-256
`fe7c8f1ed5bcaed259a08cd284168cdf752d139bd0271e5a62022501a2cd2fc4`.
The broader compatibility expansion goal remains active.

## Evidence and reproduction

Evidence is retained in the ignored directory
`.local/ssl-hybrid-fx-usdjpy-fourhour-hma30-hl2-20260929/`: unchanged source,
native CSVs, inputs/properties snapshots, screenshot, exact bars, comparator,
immutable CLI, mode outputs, same-bars control, source patch, and receipts.
Downloads were saved in `I:\sys\下载` at 11:06–11:07 UTC+8. The Chrome layout
was restored to Coinbase BTCUSD daily, HMA 60, close.

Native chart SHA-256:
`d44f54d3826220d369f09b97e1f31b552608e0552ae08e2c00fe1b3442a357f0`.
Native trades SHA-256:
`95611394e76eb316ec245576401d5be53a2090c795537dba103fbf315dc455ff`.
Frozen bars SHA-256:
`c8877fd8ae6c1a3482461683556d1c736c705e787075e5df1292b4d647e6c486`.

Reproduce from the repository root:

```powershell
python .local/ssl-hybrid-fx-usdjpy-fourhour-hma30-hl2-20260929/prepare_bars.py
python .local/ssl-hybrid-fx-usdjpy-fourhour-hma30-hl2-20260929/run_modes.py
python .local/ssl-hybrid-fx-usdjpy-fourhour-hma30-hl2-20260929/compare.py
python .local/ssl-hybrid-fx-usdjpy-fourhour-hma30-hl2-20260929/run_control.py
python .local/ssl-hybrid-fx-usdjpy-fourhour-hma30-hl2-20260929/record_verification.py
```

`run_modes.py` records the full CLI arguments for batch, incremental, and
realtime-history. The latter is historical execution through the realtime
API and does not establish live-tick parity.

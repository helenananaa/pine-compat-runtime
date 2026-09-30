# SSL Hybrid Strategy on AUDUSD four-hour TEMA 30 HL2 history

Captured 2026-09-29 from the user's authenticated Chrome TradingView session.
The unchanged public Pine v5 [SSL Hybrid Strategy by
kevinmck100](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/)
has source SHA-256
`1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.
This expands the named symbol coverage to FXCM `FX:AUDUSD`, four hours,
TEMA baseline length 30, and `(H+L)/2` (`hl2`). Remaining inputs retain
their original values, including the 2021-08-01 to 2030-10-01 trading range.

## Frozen inputs and native history

Properties: USD 5,000 initial capital, 10% equity sizing, pyramiding 10,
0.04% commission, zero slippage, default four ticks per historical bar,
execution on bar close and realtime ticks, requested limit prices, and
one-tick order delay. The host-neutral CLI supplies symbol `FX:AUDUSD`,
currency `USD`, timeframe `240`, price grid `1/100000`, integer quantity
precision, and point value 1. Data acquisition remains outside the core.

The native export contains 21,345 rows, starting 2013-01-02 02:00 UTC.
After loading the historical start through Chrome's date-range UI, the
baseline and SSL1 have the expected 87 blank startup positions. Local
input contains **21,344 confirmed bars** through 2026-09-28 21:00 UTC.
The forming 2026-09-29 01:00 UTC row is excluded. All native executions
precede that row; none requires forming-bar execution to reconcile.

Native chart and trade exports were downloaded into `I:\sys\下载` at
12:01 and 12:00 UTC+8 respectively. The chart also contains an existing
separate probe's `Plot` column. That probe is not executed locally or
included in this SSL Hybrid comparison. Settings and chart snapshots
identify the selected strategy, symbol, period, and inputs.

## Results

All **149,018 nonblank positions** in eight named exported columns match
at absolute tolerance `1e-8`, including the missing-position comparison:

| Column | Nonblank positions | Mismatches |
| --- | ---: | ---: |
| Candle Size > 1xATR | 21,344 | 0 |
| MA Baseline | 21,257 | 0 |
| SSL1 | 21,257 | 0 |
| Baseline Upper Channel | 21,257 | 0 |
| Basiline Lower Channel | 21,257 | 0 |
| MA UP | 21,323 | 0 |
| MA DOWN | 21,323 | 0 |
| 2nd Multi-TimeFrame Moving Average | 0 | 0 |

The comparator handles the native chart CSV's false/NA Candle Size values
flattened to zero. All **3,452 closed trades** match entry IDs, directions,
UTC+8 minute entry/exit times, prices, integer quantities, entry values,
durations, displayed commission, and net PnL. All **five open entries**
match their IDs, times, prices, and quantities; forming-tick unrealized
PnL is outside this comparison. All **2,031 explicit exit fills** match
signal IDs, times, five-decimal prices, and quantities.

Maximum net-PnL display delta is $0.004999472 and commission display delta
is $0.004996880, within half-cent display precision. Quantity deltas are
zero and entry-value deltas are at binary-roundoff scale. Diagnostics are
empty. No new core change was required for this case.

Batch, incremental, and realtime-history execution are byte-identical:
`e7316569a2a9513c9ed0e96d65eba49c9d3dd15a818955c1a6a454de405ff203`.
On these same bars, local DEMA 30 yields **2,911** closed trades versus
TEMA 30's **3,452**. All four baseline/SSL1/channel columns change at
21,257 mutually defined positions. This is a local sensitivity control;
AUDUSD DEMA 30 has not yet been qualified against native output.

## Verification and reproduction

The source base is `2f9a83a0c333092167c7253571f56ee82a7ec0d0` plus the
existing local broker patch, SHA-256
`fe7c8f1ed5bcaed259a08cd284168cdf752d139bd0271e5a62022501a2cd2fc4`.
Current HEAD, the full crates diff, changed core files, and frozen CLI
hashes were checked. The exact unchanged source reuses the preceding
2026-09-29 full-gate receipt and hash-verified log: workspace fmt/clippy,
Rust tests, structure/host parity, WASM Node smoke, 130 tool tests, and
774 installed-wheel Python tests. That gate was not rerun for this case.

Evidence is retained in
`.local/ssl-hybrid-fx-audusd-fourhour-tema30-hl2-20260929/`: original source,
native chart/trade CSVs, exact bars, settings and screenshot records,
immutable CLI, comparators, outputs, and receipts. `record_verification.py`
checks terminal mode receipts, artifact hashes, comparison results,
control output, unchanged-source gate, and browser restoration.

| Artifact | SHA-256 |
| --- | --- |
| Native chart | `28cd1c0ca4498109281ebd66d9a29b5a66c9d8ec218eb6a9bd2502ad55bfffbd` |
| Native trades | `ab5133cc9bbf896650803a1a3d5737378828139ecd1d4c5ec6c6d210d6041305` |
| Confirmed bars | `40b64a2dd0897ec2c47c206249cda3b9f34ca16bdfd533061285b72d4e34ad58` |

From the repository root:

```powershell
python .local/ssl-hybrid-fx-audusd-fourhour-tema30-hl2-20260929/prepare_bars.py
python .local/ssl-hybrid-fx-audusd-fourhour-tema30-hl2-20260929/run_modes.py
python .local/ssl-hybrid-fx-audusd-fourhour-tema30-hl2-20260929/compare.py
python .local/ssl-hybrid-fx-audusd-fourhour-tema30-hl2-20260929/run_control.py
python .local/ssl-hybrid-fx-audusd-fourhour-tema30-hl2-20260929/record_verification.py
```

The browser was restored to Coinbase BTCUSD daily, HMA 60, close, verified
by screenshot. This qualifies this named confirmed-history case and the
selected exported fields. Live forming ticks, account currency conversion,
and arbitrary scripts or parameter combinations remain unproven.

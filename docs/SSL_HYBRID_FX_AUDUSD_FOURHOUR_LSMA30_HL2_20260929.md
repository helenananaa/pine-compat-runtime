# SSL Hybrid Strategy on AUDUSD four-hour LSMA 30 HL2 history

Captured 2026-09-29 through authenticated Chrome. This uses the unchanged
public Pine v5 [SSL Hybrid Strategy by
kevinmck100](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/),
source SHA-256
`1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.
FXCM `FX:AUDUSD`, four hours, LSMA baseline length 30 and `(H+L)/2`
(`hl2`) expand coverage to the original script's `ta.linreg(src, len, 0)`
branch. All remaining inputs retain their original values.

## Settings and history

Native properties are USD 5,000 initial capital, 10% equity sizing,
pyramiding 10, 0.04% commission, zero slippage, default four historical
ticks per bar, on-bar-close/realtime-tick execution, requested limit prices,
and one-tick order delay. The original trading range remains 2021-08-01
to 2030-10-01. Host-neutral CLI metadata: `FX:AUDUSD`, currency `USD`,
timeframe `240`, grid `1/100000`, integer quantity precision, point value 1.

The native chart has 21,345 rows beginning 2013-01-02 02:00 UTC. Local
input contains **21,344 confirmed bars** ending 2026-09-28 21:00 UTC;
the forming 2026-09-29 01:00 UTC row is excluded. Every native execution
precedes that row. Baseline and SSL1 each have 29 blank startup positions;
the two baseline channels each have 30. Both values and missing positions
are compared against the independently exported native columns.

Confirmed OHLCV is byte-identical to the preceding AUDUSD TEMA 30 capture.
Each native comparison still uses its own frozen chart/trade exports.
Downloads in `I:\sys\下载`: trade export at 12:09 UTC+8, chart export at
12:10 UTC+8. The existing separate probe's duplicate `Plot` column is
outside the SSL Hybrid comparison and is not executed locally.

## Native comparison

All **149,248 nonblank positions** across eight named columns match at
absolute tolerance `1e-8`. Missing positions also match. The native CSV's
Candle Size false/NA flattening to zero is handled explicitly.

| Column | Nonblank positions | Mismatches |
| --- | ---: | ---: |
| Candle Size > 1xATR | 21,344 | 0 |
| MA Baseline | 21,315 | 0 |
| SSL1 | 21,315 | 0 |
| Baseline Upper Channel | 21,314 | 0 |
| Basiline Lower Channel | 21,314 | 0 |
| MA UP | 21,323 | 0 |
| MA DOWN | 21,323 | 0 |
| 2nd Multi-TimeFrame Moving Average | 0 | 0 |

All **3,185 closed trades** match entry IDs, directions, UTC+8 minute
entry/exit times, displayed prices, integer quantities, entry values,
durations, commission, and net PnL. All **five open entries** match IDs,
times, prices, and quantities. All **2,075 explicit exit fills** match
signal IDs, times, five-decimal prices, and quantities. Open-entry
forming-tick unrealized PnL is outside this comparison.

Maximum net-PnL display delta is $0.004997252 and commission display delta
is $0.004999316, within half-cent display precision. Quantity delta is
zero; entry-value delta is at binary-roundoff scale. Diagnostics are empty.
No new core fix was required.

Batch, incremental, and realtime-history outputs are byte-identical:
`bd0dd50277ae66f332f8ccf662a77c4cc34889c0a0a19482e4fe8cd470272de6`.
The same-bar TEMA 30 control retains **3,452** closed trades and exactly
reproduces the [preceding native-qualified output](SSL_HYBRID_FX_AUDUSD_FOURHOUR_TEMA30_HL2_20260929.md),
SHA-256 `e7316569a2a9513c9ed0e96d65eba49c9d3dd15a818955c1a6a454de405ff203`.
All four baseline/SSL1/channel columns differ at all 21,257 mutually
defined positions. This isolates the algorithm change on identical data.

## Evidence and reproduction

The source base is `2f9a83a0c333092167c7253571f56ee82a7ec0d0` plus the
existing local broker patch, SHA-256
`fe7c8f1ed5bcaed259a08cd284168cdf752d139bd0271e5a62022501a2cd2fc4`.
Current HEAD, full crates diff, changed core files, immutable CLI, native
artifacts, execution receipts, output hashes, and comparison are checked
by `record_verification.py`. Unchanged core reuses the preceding full-gate
receipt and hash-verified log; that gate was not rerun for this case.
The gate covers workspace fmt/clippy/Rust tests, structure/host parity,
WASM Node smoke, 130 tool tests, and 774 installed-wheel Python tests.

Artifacts reside in
`.local/ssl-hybrid-fx-audusd-fourhour-lsma30-hl2-20260929/`: original source,
native exports, settings, chart screenshot, exact confirmed bars, frozen
CLI, all three mode outputs, comparator, control, and verification receipt.

| Artifact | SHA-256 |
| --- | --- |
| Native chart | `d17afcfd834b5dea31a04f604296b3717590e67a07c3e79780a2644dc45418d0` |
| Native trades | `24e8d40b7ba0da1c6ad0e3d0c4c57215d3bdf1930d57fe1e87cff36c36ec775b` |
| Confirmed bars | `40b64a2dd0897ec2c47c206249cda3b9f34ca16bdfd533061285b72d4e34ad58` |

From the repository root:

```powershell
python .local/ssl-hybrid-fx-audusd-fourhour-lsma30-hl2-20260929/prepare_bars.py
python .local/ssl-hybrid-fx-audusd-fourhour-lsma30-hl2-20260929/run_modes.py
python .local/ssl-hybrid-fx-audusd-fourhour-lsma30-hl2-20260929/compare.py
python .local/ssl-hybrid-fx-audusd-fourhour-lsma30-hl2-20260929/run_control.py
python .local/ssl-hybrid-fx-audusd-fourhour-lsma30-hl2-20260929/record_verification.py
```

Chrome was restored to Coinbase BTCUSD daily, HMA 60, close, verified by
screenshot. This qualifies the named confirmed-history case and selected
exported fields. Other scripts/settings, live forming ticks, and currency
conversion require further independent evidence. External data remains
outside the deterministic interpreter core.

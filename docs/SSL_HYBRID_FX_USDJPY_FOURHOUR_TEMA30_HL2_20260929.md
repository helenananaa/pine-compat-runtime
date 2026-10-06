# SSL Hybrid Strategy on USDJPY four-hour TEMA 30 HL2 history

Captured 2026-09-29 in the user's authenticated Chrome TradingView session.
The unchanged public Pine v5 [SSL Hybrid Strategy by
kevinmck100](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/)
has SHA-256 `1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.
Inputs select TEMA baseline, length 30, and HL2; other original defaults
remain. The chart identifies FXCM `FX:USDJPY`, four hours. Properties show
JPY 5,000, 10% equity sizing, pyramiding 10, 0.04% commission, zero slippage,
default four-tick historical detail, and one-tick execution delay.
Currency conversion is not exercised.

## Native comparison

The native chart has 21,346 bars from 2013-01-02 02:00 UTC. The forming
2026-09-29 01:00 UTC bar is excluded, leaving **21,345 confirmed bars** through
2026-09-28 21:00 UTC. The native report has no forming-bar executions or open
entries. CLI host metadata supplies currency JPY, timeframe 240, price grid
1/1000, integer quantity precision, and point value one.

All **149,025 nonblank positions** across eight indicator columns match at
absolute tolerance `1e-8`; missing positions are compared too. Baseline and
SSL1 have 87 initial missing values, versus 58 for DEMA and 29 for EMA with
the same length. This qualifies the nested three-layer EMA warmup. Native
false/NA Candle Size values flattened to zero are accounted for explicitly.

All **768 closed trades** match entry IDs, direction, entry/exit times,
three-decimal prices, quantities, entry values, durations, commission, and
net PnL. All **507 explicit exit fills** match signal, time, price, and quantity.
Maximum net-PnL and commission display deltas are JPY 0.0049916 and
0.0049792, within the two-decimal native display precision. Diagnostics are
empty. Qualification covers the named confirmed-history case and fields;
arbitrary scripts, every chart/report field, and live forming ticks remain
unproven.

## Sensitivity, modes, and current source

Compared with the preceding DEMA capture, two confirmed-bar volume fields
were revised: 77,260 to 76,643 and 103,920 to 102,961. OHLC and timestamps are
unchanged; `bars-receipt.json` retains exact differences. DEMA 30 on these
exact new bars still produces **658 closed trades** and its prior output
byte-for-byte. TEMA 30 produces 768. Baseline, SSL1, and both baseline channels
differ at all 21,258 mutually defined positions.

Batch, incremental, and realtime-history outputs are byte-identical, SHA-256
`d1814b86e522abca7afd42afb2a15d1048a3820b99ef1281ba7917f7a50f5fee`.
Realtime-history runs historical bars through the realtime API and does not
establish native forming-tick parity.

No new core change was needed. Current core file hashes and the immutable CLI
match the preceding HMA 20 full-gate receipt. That exact source passed
formatting, workspace Clippy/Rust tests, structural and host parity checks,
130 tool tests, WASM/Node smoke, and 774 fresh-wheel Python tests; runtime
and CLI test counts were 1,983 and 242. This unchanged-source gate is reused
and linked by receipt/log hashes in `current-verification.json`, rather than
claimed as a new full-gate run.

The base is `2f9a83a0c333092167c7253571f56ee82a7ec0d0` plus the existing
local broker patch, SHA-256
`fe7c8f1ed5bcaed259a08cd284168cdf752d139bd0271e5a62022501a2cd2fc4`.
The broader expansion goal remains active.

## Evidence and reproduction

The ignored directory `.local/ssl-hybrid-fx-usdjpy-fourhour-tema30-hl2-20260929/`
retains unchanged source, native CSVs, settings snapshots, screenshot, bars,
immutable CLI, comparator, mode outputs, DEMA control, patch, and receipts.
Downloads in `I:\sys\下载` at 11:32 UTC+8 are `FX_USDJPY, 240 (5).csv` and
`SSL_Hybrid_Strategy_FX_USDJPY_2026-09-29 (4).csv`.
The browser was restored to Coinbase BTCUSD daily, HMA 60, close.

Native chart SHA-256:
`5d9e46493ff117bbb08e10618e32f02d83df3eba762173bb2ce0dca966febaad`.
Native trades SHA-256:
`d2f138e5e0b4da60d1540cf062ea6e7320100dec0d2b64ff05c38ce16c283c7b`.
Frozen bars SHA-256:
`0690b71a702930e6b3d3344be7a57a55b3f775dd2ac87a51d70fd7065f1740fc`.

Reproduce from the repository root:

```powershell
python .local/ssl-hybrid-fx-usdjpy-fourhour-tema30-hl2-20260929/prepare_bars.py
python .local/ssl-hybrid-fx-usdjpy-fourhour-tema30-hl2-20260929/run_modes.py
python .local/ssl-hybrid-fx-usdjpy-fourhour-tema30-hl2-20260929/compare.py
python .local/ssl-hybrid-fx-usdjpy-fourhour-tema30-hl2-20260929/run_control.py
python .local/ssl-hybrid-fx-usdjpy-fourhour-tema30-hl2-20260929/record_verification.py
```

Mode receipts record all metadata arguments, overrides `12=TEMA`, `13=30`,
`18=hl2`, and terminal exit codes.

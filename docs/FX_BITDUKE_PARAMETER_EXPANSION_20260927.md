# Bitduke FX:EURUSD 4H parameter expansion

Date: 2026-09-27. Baseline commit: `6a7a2ac24bf9222cc3e77ca08363956bb3c45b48`.
This extends the [default-input four-hour comparison](FX_BITDUKE_FOURHOUR_20260927.md)
with two TradingView-native input settings on the same public Pine v4 script.
The original source is unchanged (SHA-256
`b6aef6dbf531dac2c0a591a198060be4df554a400d98a4952549a723b77b1eb2`).

| Setting | Native closed trades | Entry/exit rows | Post-warmup plot cells | Result |
| --- | ---: | ---: | ---: | --- |
| `BB Length=20`; other inputs default | 53 | 106 | 27,212 | No differences |
| `BB Length=20`, momentum filter on, `Min for momentum=100`; others default | 48 | 96 | 27,212 | No differences |

The strategy's default date filter remains enabled, from 2019-01-01 through
2019-12-29. TradingView used the FXCM `FX:EURUSD` 240-minute chart. The runtime
received the chart's UTC OHLCV, `FX:EURUSD`, timeframe `240`, price grid
`1/100000`, integer quantity precision `0`, and explicit input call-site
overrides (`1=20`; then `1=20`, `5=true`, `6=100`). These are host-supplied
facts, not data or policy fetched by the core.

The frozen TradingView chart CSV contains 13,637 rows, beginning
2017-12-15 14:00 UTC. The final chart row was excluded as a possible forming
bar, leaving 13,636 confirmed bars. Its SHA-256 is
`ba86ec4c1dde4b1e105ad482ff9e28152ddbcbf3e0939c9e288b66ecdabe1de6`.
Both parameter settings produced byte-identical chart exports: the filter
changes entry admission and trades but does not change the two plots. The
derived local bar CSV has SHA-256
`1a72cc53e39de07d0f435cced29130a23428d27f3d725a7c785046571b0e5eb4`.
Compared numerically with the prior default-setting export, timestamps are
unchanged but 13 late-2026 OHLC rows and 20 late-2026 volume rows differ.
The local runs use the new export; all 2019 trade-window OHLC values agree
with the earlier export.

The separately downloaded native trade CSV hashes are
`33c954e1611cd1d08c15b9764395ebc3e913b35d5c58e0eb9b3bc60013f8e613`
and `53e1fcb51b05a29928cf9ed6e26169b09fb775b80d1139bc37142541058b5986`.
The two reproducible comparison scripts check every entry/exit signal, UTC+8
displayed timestamp, price, and quantity; every trade's displayed profit,
entry notional, commission, cumulative profit, and duration; and both plot
series after the first 30 supplied bars. The 30-bar exclusion isolates the
exported-window warmup: TradingView retains earlier chart state, whereas the
local run begins at the first supplied bar. Native money values are rounded
to cents and checked within half a cent; finite plot values use absolute and
relative tolerance `1e-9`. Both receipts report zero mismatches and no
runtime diagnostics.

For each setting, batch, incremental, and realtime-history full JSON outputs
are byte-identical. Their respective SHA-256 values are
`26a633857271fa7a56c144632b5a3b90c18628d1bbb071c5215a41c259444a90`
and `31051deca25729855aed6ba7468b6ae24bfdc92ca299d2334711c2fb98ea4b0c`.
No runtime code change was needed for these settings. All native files, local
results, two comparison scripts and receipts, and screenshots are retained
under ignored `.local/fx-fourhour-bb20-20260927/`.

This is historical close-bar evidence. The chart CSV does not establish
intrabar event ordering, favorable/adverse excursion parity, or realtime
forming-bar behavior.

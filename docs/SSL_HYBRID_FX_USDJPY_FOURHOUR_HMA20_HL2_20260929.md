# SSL Hybrid Strategy on USDJPY four-hour HMA 20 HL2 history

Captured 2026-09-29 from the user's authenticated Chrome TradingView session.
The public Pine v5 [SSL Hybrid Strategy by
kevinmck100](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/)
is unchanged, SHA-256
`1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.
The chart identifies FXCM `FX:USDJPY`, four hours. Inputs are HMA baseline
length 20 and `(H+L)/2` (`hl2`), retaining the other original inputs.
Properties show **JPY 5,000**, 10% equity sizing, pyramiding 10, 0.04%
commission, zero slippage, default four-tick historical detail, and one-tick
execution delay. The default strategy account follows the chart currency;
this case does not exercise foreign-currency conversion.

## Frozen data and native comparison

The first chart download contained only 300 recent bars. After loading the
2013 history origin through the chart's custom-range controls, a second
native export contains **21,346 bars**. The forming 2026-09-29 01:00 UTC bar
is excluded. Local input contains exactly **21,345 confirmed bars** from
2013-01-02 02:00 UTC through 2026-09-28 21:00 UTC. The native report contains
no trades on the excluded forming bar and no open entries.
The CLI host context is symbol `FX:USDJPY`, currency `JPY`, timeframe `240`,
price grid `1/1000`, quantity precision zero, and point value one.

All **149,285 nonblank positions** in eight exported indicator columns
match at absolute tolerance `1e-8`, including missing positions. The native
CSV's false/NA Candle Size conditions represented as zero are handled by
the comparator. All **1,088 closed trades** match entry IDs, directions,
UTC+8 report entry/exit times, three-decimal prices, integer quantities,
entry values, durations, commission, and net PnL. All **592 explicit
exit-order fills** match signal, time, price, and quantity. Maximum net-PnL
and commission display deltas are JPY 0.004998000 and JPY 0.004997600,
within the two-decimal report display precision. There are no runtime diagnostics.
The comparison checks these named fields; it does not certify every field
in the strategy report, all chart objects, or arbitrary scripts.

## Defect exposed by small opening quantities

Before repair, four closed trades exited too late at different prices.
For example, native trade 782 closes LongEntry3 at 148.795 via ShortEntry1
on 2023-09-26 21:00 UTC+8. The runtime held it until 2023-09-27 17:00 and
exited at 149.209. The original script divides equity sizing into five
entries. At JPY 5,000, a positive requested opening quantity can truncate
to zero integer contracts; the opposite position still contributes a
closing transaction. The runtime discarded the request before its fill
became eligible, preventing that close.

The runtime now retains positive explicit requests that round to zero
opening contracts. At their eligible fill, an opposite position is closed
without creating fractional exposure. Flat and same-side fills remain
empty; explicit caller quantity zero retains its existing no-op behavior.
The close uses normal netting accounting, order metadata, and fill alerts.
Attached exits for other pending entries survive the close. OCA membership
survives exit cleanup until the fill's peer effects are applied; a pure
close reduces peers by the contracts actually closed.

New regression coverage checks both directions, price-triggered entries,
empty same-side requests, explicit zero, close IDs, commission, alerts,
attached exits on the next entry, and OCA quantity reduction. The initial
reversal test failed against the previous implementation. An intermediate
repair lost pending-entry exits and failed the full native comparison;
it was replaced before qualification. The complete native trade comparison
passes after the corrected exit-retention behavior.

## Verification status

Batch, incremental, and realtime-history outputs are byte-identical,
SHA-256 `53625bb4f45d360f1bd61e38f768544f4be5f62b6da4e4158f2e3a1261174207`.
The final strict native comparator passes. Prior frozen GBPUSD HMA 20/30
and EURUSD HMA 20/30 outputs remain byte-identical after this repair.

The full `scripts/verify.ps1` gate completed with exit code zero:

- Formatting and workspace Clippy with warnings denied passed.
- Workspace Rust tests passed, including 1,983 runtime and 242 CLI tests.
- Structure checks, 130 tool tests, and the host parity guard passed.
- WASM/Node build and executable smoke tests passed.
- A freshly built Python wheel passed all 774 Python tests.

`current-verification.json` records terminal exit codes, source and CLI
hashes, the full gate log hash, mode commands, and regression hashes.
The source base is `2f9a83a0c333092167c7253571f56ee82a7ec0d0` plus the
local broker patch; its diff SHA-256 is
`fe7c8f1ed5bcaed259a08cd284168cdf752d139bd0271e5a62022501a2cd2fc4`.
This case is locally qualified for the named confirmed-history scope.

This scope covers confirmed historical bars. Realtime-history is historical
execution through the realtime API, not evidence of live forming-bar tick
parity. The broader compatibility expansion goal remains active.

## Evidence and reproduction

Evidence is retained in the ignored directory
`.local/ssl-hybrid-fx-usdjpy-fourhour-hma20-hl2-20260929/`: unchanged source,
native CSVs, the initial short export, inputs/properties snapshots, screenshots,
exact bars, comparator, before/after local output, source patch, and receipts.
Native exports were downloaded into `I:\sys\下载` at 10:37–10:39 UTC+8.
The browser layout was restored to Coinbase BTCUSD daily, HMA 60, close.

Native chart SHA-256:
`47033edb8a4f0d420022a2a92b666e26e66763ca540729e1f13e03e4e4a5e3fb`.
Native trade CSV SHA-256:
`7d1fe0f57b9f179aab5c00706ccbca08b5b69844f233aa78345d4f3bfc8696e7`.
Frozen bars SHA-256:
`0690b71a702930e6b3d3344be7a57a55b3f775dd2ac87a51d70fd7065f1740fc`.
`bars-receipt.json` hashes the other evidence artifacts.

Reproduce from the repository root:

```powershell
cargo build -p pine-cli
python .local/ssl-hybrid-fx-usdjpy-fourhour-hma20-hl2-20260929/prepare_bars.py
target/debug/pine-compat.exe run .local/ssl-hybrid-fx-usdjpy-fourhour-hma20-hl2-20260929/ssl-hybrid-original.pine --bars .local/ssl-hybrid-fx-usdjpy-fourhour-hma20-hl2-20260929/bars.csv --chart-symbol FX:USDJPY --chart-currency JPY --chart-timeframe 240 --chart-price-grid 1/1000 --chart-quantity-precision 0 --chart-point-value 1 --input-override 13=20 --input-override 18=hl2 > .local/ssl-hybrid-fx-usdjpy-fourhour-hma20-hl2-20260929/local-batch.json
python .local/ssl-hybrid-fx-usdjpy-fourhour-hma20-hl2-20260929/compare.py
```

Use `run-incremental` or `run-realtime-history` with the same inputs for the
other modes. `run_modes.py` records commands, exit codes, and output hashes
using an immutable local copy of the rebuilt CLI while the full gate runs.

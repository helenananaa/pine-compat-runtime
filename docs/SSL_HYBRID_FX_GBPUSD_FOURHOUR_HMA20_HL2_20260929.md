# SSL Hybrid Strategy on GBPUSD four-hour HMA 20 HL2 history

Captured 2026-09-29 from the user's authenticated Chrome TradingView session.
This comparison uses the unchanged public Pine v5 [SSL Hybrid Strategy by
kevinmck100](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/),
source SHA-256 `1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.
The chart identifies FXCM `FX:GBPUSD`, `4h`. Inputs are HMA baseline length
20 and `(H+L)/2` (`hl2`), with the remaining original inputs retained.
Properties show USD 5,000 initial capital, 10% equity order size, pyramiding
10, 0.04% commission, zero slippage, default four-tick historical detail,
and one-tick order delay.

The official chart CSV contains 21,345 rows from 2013-01-02 02:00 UTC.
The final 2026-09-29 01:00 UTC bar was forming at capture and is excluded.
The runtime consumes exactly the remaining **21,344 confirmed bars**, through
2026-09-28 21:00 UTC. The native report has no new entry or closed trade on
the excluded bar. Open entries were established on September 24 UTC; their
changing unrealized PnL is outside this comparison. The host supplies symbol
`FX:GBPUSD`, timeframe `240`, price grid `1/100000`, and integer quantity
precision through the host-neutral CLI contract.

All **149,278 nonblank positions** in the eight exported indicator columns
match at absolute tolerance `1e-8`, including missing-value positions.
TradingView's chart CSV flattens false/NA Candle Size conditions to zero;
the comparator accounts for that representation. Native and local results
each have **4,731 closed trades**, with matching entry IDs, UTC+8 report
entry/exit times, displayed prices, directions, integer quantities, entry
values, durations, commission, and net PnL. All **five open entries** match
by ID, time, price, and quantity. All **2,268 explicit exit-order fills**
match by signal, displayed time, five-decimal price, and quantity. Maximum
net-PnL and commission display deltas are respectively $0.004999984 and
$0.004998544, within half-cent display precision. There are no runtime
diagnostics.

Batch, incremental, and realtime-history execution produce byte-identical
JSON, SHA-256 `83ef6710ed445a9365827fe33babc6ccd72ecd1c9187a81977611798337c59a1`.
The current CLI was rebuilt with `cargo build -p pine-cli` before the final
three-mode verification, at source revision
`2f9a83a0c333092167c7253571f56ee82a7ec0d0`.
`current-verification.json` records the build, revision, comparator hashes,
and output hashes. No core semantic change was needed for this case.
This qualifies the named confirmed-history script, setting, symbol, and
timeframe; forming-bar ticks and universal Pine compatibility remain unproven.

Evidence is retained in the ignored directory
`.local/ssl-hybrid-fx-gbpusd-fourhour-hma20-hl2-20260927/` (prepared September
27, captured September 29): unchanged source, native exports, settings
snapshots, screenshot, exact bar input, hashes, comparator, and local outputs.
The native chart SHA-256 is
`f68327dbe267b14d6d6a864e6db24b9f96ad90746addafba4e5dc18a24f9c749`;
the trade report is
`67d31afcd8f6b54287e6cf9053d896d872f0f57a751fc746a7736d5e3dc0e5a4`.
`bars-receipt.json` records the remaining artifact hashes and the excluded bar.
The Chrome layout was restored to `COINBASE:BTCUSD`, daily, HMA 60, close.

Reproduce from the repository root:

```powershell
python .local/ssl-hybrid-fx-gbpusd-fourhour-hma20-hl2-20260927/prepare_bars.py
target/debug/pine-compat.exe run .local/ssl-hybrid-fx-gbpusd-fourhour-hma20-hl2-20260927/ssl-hybrid-original.pine --bars .local/ssl-hybrid-fx-gbpusd-fourhour-hma20-hl2-20260927/bars.csv --chart-symbol FX:GBPUSD --chart-timeframe 240 --chart-price-grid 1/100000 --chart-quantity-precision 0 --input-override 13=20 --input-override 18=hl2 > .local/ssl-hybrid-fx-gbpusd-fourhour-hma20-hl2-20260927/local-batch.json
python .local/ssl-hybrid-fx-gbpusd-fourhour-hma20-hl2-20260927/compare.py
```

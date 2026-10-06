# SSL Hybrid Strategy HMA 30 with HL2 on EURUSD four-hour history (2026-09-27)

This comparison uses the unchanged 557-line public Pine v5 [SSL Hybrid Strategy by kevinmck100](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/) in the user's Chrome TradingView session on FXCM `FX:EURUSD`, `4h`. `SSL1 / Baseline Type=HMA`, `SSL1 / Baseline Length=30`, and `Source=(H+L)/2` (`hl2`); other Inputs and Properties retain their original values. The source SHA-256 is `1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`. The ignored `.local/ssl-hybrid-fx-eurusd-fourhour-hma30-hl2-20260927/` directory contains source, native Chrome exports, settings snapshots, screenshot, frozen bars, hashes, comparator, and local results.

The chart was first moved to 2013–2015 to load early data, then expanded through 2026. The final native chart CSV contains **21,338 confirmed bars**, from 2013-01-02 02:00 UTC through 2026-09-25 17:00 UTC. The market was closed at capture time. The first export contained only 300 recent bars and was excluded. The final chart export SHA-256 is `f389c1e9204dd0b948c23bbf47c505375a5612bf9ee43cb91d755c184c4d2eeb`; the native trade report is `4e50940f7fb73e70533ccccc4a08c614c7c492e7747901206f5bce255d6a4215`. `bars-receipt.json` records hashes for the remaining artifacts.

The HMA 30 export and the earlier HMA 20 export have the same 21,338 timestamps but **54 changed numeric OHLCV fields**: 36 volume, 11 low, six high, and one close. These revisions occur between browser captures. This comparison uses the exact HMA 30 chart export as its input; conclusions about the parameter effect use both lengths on this same frozen input.

The captured Properties show USD 5,000 initial capital, 10% equity default order size, pyramiding 10, 0.04% commission, zero slippage, default four-tick historical bar detail, and one-tick order delay. The host-neutral chart contract supplies `FX:EURUSD`, timeframe `240`, price grid `1/100000`, and integer quantity precision. CLI input callsites 13 and 18 are overridden with `30` and `hl2`.

All **149,192 nonblank exported indicator positions** match locally within `1e-8`. TradingView's chart CSV flattens the false/`na` Candle Size condition to zero, as accounted for in the comparator. The native report and local runtime each contain **3,603 closed trades** and five open entries. Closed trades match by entry ID, displayed entry/exit time and price, direction, integer quantity, entry value, duration, and cent-displayed commission and net PnL; the maximum net-PnL delta is `$0.004999`, within cent-display precision. The five open entries also match. All **2,150 explicit exit-order fills** match by signal, displayed time, five-decimal price, and quantity. The runtime reports no diagnostics.

The comparison qualifies this exact confirmed historical script, symbol, timeframe, and setting. It does not establish forming-bar tick parity or universal compatibility across Pine scripts and markets.

Batch, incremental, and realtime-history modes produced byte-identical JSON, SHA-256 `f2f6ab4bd90e2191615bd939d5541f321539f83b558fcadabff5b41b9552add3`. On the same HMA 30 frozen bars, changing only the baseline length to 20 produces 4,645 closed trades instead of 3,603. MA Baseline, SSL1, and the two baseline channels change at all 21,316 defined positions. The exact-input HMA 20 result is a local parameter control; the [separately exported HMA 20 native comparison](SSL_HYBRID_FX_EURUSD_FOURHOUR_HMA20_HL2_20260927.md) used its own frozen chart export.

Reproduce from the repository root:

```powershell
python .local/ssl-hybrid-fx-eurusd-fourhour-hma30-hl2-20260927/prepare_bars.py
target/debug/pine-compat.exe run .local/ssl-hybrid-fx-eurusd-fourhour-hma30-hl2-20260927/ssl-hybrid-original.pine --bars .local/ssl-hybrid-fx-eurusd-fourhour-hma30-hl2-20260927/bars.csv --chart-symbol FX:EURUSD --chart-timeframe 240 --chart-price-grid 1/100000 --chart-quantity-precision 0 --input-override 13=30 --input-override 18=hl2 > .local/ssl-hybrid-fx-eurusd-fourhour-hma30-hl2-20260927/local-batch.json
python .local/ssl-hybrid-fx-eurusd-fourhour-hma30-hl2-20260927/compare.py
```

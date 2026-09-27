# SSL Hybrid Strategy HMA 20 with HL2 on ETHUSD weekly history (2026-09-27)

The unchanged public Pine v5 [SSL Hybrid Strategy by kevinmck100](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/) was run in the user's Chrome TradingView session on `COINBASE:ETHUSD`, `1W`, with `SSL1 / Baseline Type=HMA`, `SSL1 / Baseline Length=20`, and `Source=(H+L)/2` (`hl2`). Other script inputs and Properties remained at their original values. The frozen 557-line source has SHA-256 `1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`. The ignored `.local/ssl-hybrid-length20-hl2-ethusd-weekly-20260927/` directory holds the source, native CSV exports, Inputs and Properties snapshots, screenshot, confirmed bars, hashes, comparator, and local results.

The chart was expanded to 2016-05-23 through 2026-09-21 before export. Its chart CSV (SHA-256 `7d43555e4699f4f80bdd67b3bd9c9d96928e294c9ceeb7eaa4e6f39447d66d9d`) has 540 rows; excluding the forming 2026-09-21 week leaves **539 confirmed weekly bars through 2026-09-14**. The List of trades CSV hashes to `f59f3c4722baf7fd8fc6acc508cd60b3cffac6f6f25a32c7fbb4e7db90881290`. Compared with the earlier HMA 30 export, time and OHLC are identical on every confirmed bar. The final confirmed week's volume changed from `917824.4968639` to `917799.38115157`; this run uses the new export's bars. `bars-receipt.json` records the revision and hashes.

The captured Properties include USD 5,000 initial capital, 10% equity order size, pyramiding 10, 0.04% commission, zero slippage, default four-tick historical bar detail, and one-tick order delay. The host-neutral chart contract supplies `COINBASE:ETHUSD`, `1W`, a `1/100` price grid, and four-decimal quantity precision matching the native trade export. CLI input callsites 13 and 18 are overridden with `20` and `hl2`.

All **3,643 nonblank positions** in the eight exported indicator series match locally at `1e-8` absolute tolerance. The optional second moving average is blank. TradingView's chart CSV flattens false/`na` Candle Size conditions to zero; the comparator applies that export interpretation. Native and local results each have **148 closed trades and two open entries**. For every closed trade, entry signal, UTC entry and exit date, displayed entry and exit price, and displayed quantity agree. The two open entries agree in signal, date, displayed entry price, and quantity. Maximum closed net-PnL difference is `$0.004995`, within cent display precision; there are no runtime diagnostics.

HMA 30 evaluated locally on the **same newly frozen bars** produces 98 closed trades, versus 148 for HMA 20, so the parameter changes the historical trade path on this second symbol. All 517 defined MA Baseline, SSL1, upper-channel, and lower-channel positions change; the other exported plots retain their values. Batch, incremental, and realtime-history JSON are byte-identical, SHA-256 `bc53dd67890ba078321dcdc3f230b846323410df3230040341925d45caadd597`. No runtime repair was needed for this slice. The evidence qualifies confirmed historical weekly bars for this script and setting; forming-bar and live-tick parity remain unverified.

Reproduce from the repository root:

```powershell
python .local/ssl-hybrid-length20-hl2-ethusd-weekly-20260927/prepare_bars.py
target/debug/pine-compat.exe run .local/ssl-hybrid-length20-hl2-ethusd-weekly-20260927/ssl-hybrid-original.pine --bars .local/ssl-hybrid-length20-hl2-ethusd-weekly-20260927/bars-confirmed.csv --chart-symbol COINBASE:ETHUSD --chart-timeframe 1W --chart-price-grid 1/100 --chart-quantity-precision 4 --input-override 13=20 --input-override 18=hl2 > .local/ssl-hybrid-length20-hl2-ethusd-weekly-20260927/local-batch.json
python .local/ssl-hybrid-length20-hl2-ethusd-weekly-20260927/compare.py
python .local/ssl-hybrid-length20-hl2-ethusd-weekly-20260927/verify_modes.py
```

For `verify_modes.py`, run the same command with `run-incremental` and `run-realtime-history`, writing `local-run-incremental.json` and `local-run-realtime-history.json`. It also compares `local-length30-control.json`, made with `run` on the same bars and `--input-override 13=30`.

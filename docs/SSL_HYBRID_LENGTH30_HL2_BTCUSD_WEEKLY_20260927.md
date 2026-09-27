# SSL Hybrid Strategy HMA 30 with HL2 on BTCUSD weekly history (2026-09-27)

The unchanged public Pine v5 [SSL Hybrid Strategy by kevinmck100](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/) was run in the user's Chrome TradingView session on `COINBASE:BTCUSD`, `1W`. `SSL1 / Baseline Type=HMA`, `SSL1 / Baseline Length=30`, and `Source=(H+L)/2` (`hl2`); all other script inputs and Properties settings stayed at their original values. The frozen 557-line source has SHA-256 `1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`. The ignored `.local/ssl-hybrid-length30-hl2-btcusd-weekly-20260927/` directory holds the source, native exports, Inputs/Properties snapshots, chart screenshot, confirmed-bar CSV, hashes, comparator, and local results.

TradingView's chart range was expanded to 2014-12-01 through 2026-09-21 before export. Its chart CSV (SHA-256 `43602850064cab5284e498d42cfac016daa8a630b252d5efde4dc068caf4af87`) contains 615 weekly rows. The 2026-09-21 week was still forming and is excluded; **614 confirmed bars** end on 2026-09-14. The List of trades CSV hashes to `6ca4cb54187d9c418944bc7f886f00f1f9781dd18c166831ffc016c71806dc5f`. The captured Properties include USD 5,000 initial capital, 10% equity order size, pyramiding 10, 0.04% commission, zero slippage, default four-tick historical bar detail, and one-tick order delay. The host-neutral chart contract supplies `COINBASE:BTCUSD`, `1W`, a `1/100` price grid, and **six-decimal quantity precision** matching the native trade export.

All **4,124 nonblank positions** in the eight exported indicator series match the local runtime at `1e-8` absolute tolerance. The optional second moving average is blank. TradingView's chart CSV represents false/`na` Candle Size conditions as zero; the comparator applies that export interpretation. Native and local results each contain **115 closed trades and five open entries**. Closed-trade entry IDs, UTC entry/exit dates, displayed entry/exit prices, and displayed quantities match, as do the five open entries' IDs, dates, displayed entry prices, and quantities. The maximum closed net-PnL difference is `$0.004893`, within cent display precision. There are no runtime diagnostics.

On these same frozen bars, changing `Source` from `close` to `hl2` changes all **581** defined upper-channel and all **581** defined lower-channel positions. The local strategy orders and trades are identical for the two sources, so this comparison exercises channel/source semantics without a distinct historical trade path. Batch, incremental, and realtime-history JSON are byte-identical, SHA-256 `f10ea7205898d04ef4c35e334d6448a43e8e1fab3af0a73320788d07fe5dd129`. No runtime repair was needed for this slice. This qualifies confirmed historical bars for this script, symbol, timeframe, and settings; forming-bar and live-tick parity remain unverified.

Reproduce from the repository root:

```powershell
python .local/ssl-hybrid-length30-hl2-btcusd-weekly-20260927/prepare_bars.py
target/debug/pine-compat.exe run .local/ssl-hybrid-length30-hl2-btcusd-weekly-20260927/ssl-hybrid-original.pine --bars .local/ssl-hybrid-length30-hl2-btcusd-weekly-20260927/bars-confirmed.csv --chart-symbol COINBASE:BTCUSD --chart-timeframe 1W --chart-price-grid 1/100 --chart-quantity-precision 6 --input-override 13=30 --input-override 18=hl2 > .local/ssl-hybrid-length30-hl2-btcusd-weekly-20260927/local-batch.json
python .local/ssl-hybrid-length30-hl2-btcusd-weekly-20260927/compare.py
python .local/ssl-hybrid-length30-hl2-btcusd-weekly-20260927/verify_modes.py
```

For `verify_modes.py`, run the same command with `run-incremental` and `run-realtime-history`, writing `local-run-incremental.json` and `local-run-realtime-history.json`; make `local-close-control.json` with `run` but omit `--input-override 18=hl2`.

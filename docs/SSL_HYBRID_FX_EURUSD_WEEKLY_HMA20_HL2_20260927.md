# SSL Hybrid Strategy HMA 20 with HL2 on EURUSD weekly history (2026-09-27)

The unchanged public Pine v5 [SSL Hybrid Strategy by kevinmck100](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/) was run in the user's Chrome TradingView session on FXCM `FX:EURUSD`, `1W`, with `SSL1 / Baseline Type=HMA`, `SSL1 / Baseline Length=20`, and `Source=(H+L)/2` (`hl2`). Other script inputs and Properties remained at their original values. The 557-line source has SHA-256 `1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`. The ignored `.local/ssl-hybrid-fx-eurusd-weekly-hma20-hl2-20260927/` directory retains source, native exports, Inputs and Properties snapshots, chart screenshot, confirmed bars, hashes, comparator, and local results.

The first chart export contained only 2,301 rows, starting in 1982. The chart's custom range was moved to 1971–1995 to load earlier bars, then expanded to 2026. The second export (SHA-256 `ac6bc15b09e69613f57359021fed9d6bd5dd00cbde26f15ec118bf13e65ff8c3`) contains **2,907 rows from the first 1971-01-03 FXCM weekly bar**. Excluding the forming 2026-09-20 UTC week leaves **2,906 confirmed bars through 2026-09-13 UTC**. All confirmed time and OHLCV values agree with the separately frozen prior FX strategy export. The native trade CSV hashes to `533d735b5a662eea7f66facd479b226d95ab3e13cbe3785fdf05a3bd86a98f06`. `bars-receipt.json` records row counts and hashes.

The captured Properties include USD 5,000 initial capital, 10% equity order size, pyramiding 10, 0.04% commission, zero slippage, default four-tick historical bar detail, and one-tick order delay. The host-neutral chart contract supplies `FX:EURUSD`, `1W`, a `1/100000` price grid, and integer quantity precision matching the native trade export. CLI input callsites 13 and 18 are overridden with `20` and `hl2`. TradingView displays the FX weekly trade dates in the chart's UTC+8 zone, one calendar day after the Sunday UTC bar timestamp; the comparator applies that displayed date convention.

All **20,212 nonblank positions** in the eight exported indicator series match locally at `1e-8` absolute tolerance. The optional second moving average is blank. TradingView's chart CSV flattens false/`na` Candle Size conditions to zero; the comparator applies that export interpretation. **170 trades closed by the last confirmed week** match in entry signal, long/short direction, displayed entry/exit date, five-decimal displayed price, integer quantity, entry value, duration in bars, and cent-displayed commission and net PnL. The maximum net-PnL difference is `$0.004995`, within cent precision. There are no runtime diagnostics.

The native report also includes five trades closed on the **forming** 2026-09-20 UTC week and five new open entries from that week. Those ten trade records are excluded from confirmed-history parity. All five earlier entries for the forming-week exits match a local entry order in signal, displayed date, price, and quantity. Their exits and the five new entries require a separate forming-bar/tick comparison before any parity claim. On the same frozen confirmed bars, HMA 30 produces 140 closed trades versus 170 at HMA 20; all 2,884 defined MA Baseline, SSL1, upper-channel, and lower-channel positions change. This parameter therefore changes the historical trade path on a forex symbol as well as on the two Coinbase crypto symbols.

Batch, incremental, and realtime-history JSON are byte-identical, SHA-256 `96af56583913db7402ec38bbd88ebd5fdabb4ee9b4906c1bd77672a4cf96a01e`. No runtime repair was needed for this slice. The comparison qualifies confirmed historical EURUSD weekly chart values and closed trades for this exact script and setting; it does not qualify the forming week or live-tick behavior.

Reproduce from the repository root:

```powershell
python .local/ssl-hybrid-fx-eurusd-weekly-hma20-hl2-20260927/prepare_bars.py
target/debug/pine-compat.exe run .local/ssl-hybrid-fx-eurusd-weekly-hma20-hl2-20260927/ssl-hybrid-original.pine --bars .local/ssl-hybrid-fx-eurusd-weekly-hma20-hl2-20260927/bars-confirmed.csv --chart-symbol FX:EURUSD --chart-timeframe 1W --chart-price-grid 1/100000 --chart-quantity-precision 0 --input-override 13=20 --input-override 18=hl2 > .local/ssl-hybrid-fx-eurusd-weekly-hma20-hl2-20260927/local-batch.json
python .local/ssl-hybrid-fx-eurusd-weekly-hma20-hl2-20260927/compare.py
python .local/ssl-hybrid-fx-eurusd-weekly-hma20-hl2-20260927/verify_modes.py
```

For `verify_modes.py`, run the same command with `run-incremental` and `run-realtime-history`, writing `local-run-incremental.json` and `local-run-realtime-history.json`. It also compares `local-length30-control.json`, made with `run` on the same bars and `--input-override 13=30`.

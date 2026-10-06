# HalfTrend [everget]: native v6 qualification

2026-09-26. The unchanged 104-line, explicit-v6 public indicator is
https://www.tradingview.com/script/U1SJ8ubc-HalfTrend-everget/ . TradingView
showed 13.1K boosts when collected. The source declares GPL-3.0. Its exact
source is retained only in the ignored local evidence folder; no public
script source is vendored.

Local analysis reports zero diagnostics and zero unsupported features. On
COINBASE:BTCUSD daily bars, 299 confirmed bars match the native export's
timestamps and OHLC exactly. All three numerical plots and four sparse shape
series agree: 925 nonempty cells, zero numerical mismatches, and zero NA
presence mismatches. This includes seven up arrows, seven down arrows, seven
Buy labels, and seven Sell labels. The one additional native forming bar is
excluded. Alert deliveries, fill colors, and visual rendering were not
independently compared.

The exact source, native CSV, local result, comparison program, and JSON
receipt are in `.local/continued-popular-20260926/` under `halftrend-*`.
The original CSV remains in `I:\sys\下载`. This v6 qualification required no
core change beyond the separately tested legacy strategy fix in this batch.

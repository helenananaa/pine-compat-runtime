# UT Bot Alerts: native default-input qualification

2026-09-26. The unchanged explicit-v4, 42-line public indicator is
https://www.tradingview.com/script/n8ss8BID-UT-Bot-Alerts/ . TradingView showed
58.8K boosts when collected. Local analysis reports zero diagnostics and zero
unsupported features.

On COINBASE:BTCUSD daily bars, 299 closed bars have exact timestamp/OHLC
alignment with the native chart export. The two shape series each agree on all
299 bars: 17 Buy signals and 17 Sell signals, with no position mismatch.
These are default inputs, including `Signals from Heikin Ashi Candles = false`.
The native CSV includes one additional forming bar that is not in the local
closed-bar file. Native bar colors and actual alert deliveries are outside
this receipt; the matched shape conditions are the script's alert conditions.

The original source, native CSV, local result, comparison program, and JSON
receipt are in `.local/continued-popular-20260926/` under `ut-bot-*`. The
native export also remains in `I:\sys\下载`. No core change was required for
the tested default path.

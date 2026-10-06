# UT Bot Strategy: native full-trade qualification

2026-09-26. The unchanged, 43-line public Pine v4 strategy is
https://www.tradingview.com/script/VKMgZdYr-UT-Bot-Strategy/ . TradingView
showed 8.8K boosts when collected. Its original calls
`strategy.entry("long", true, when=buy)` and
`strategy.entry("short", false, when=sell)` failed local analysis because the
modern signature required a string-compatible direction.

The analyzer and runtime now recognize boolean `strategy.entry` direction in
Pine v1-v4: `true` is long and `false` is short. Pine v5-v6 remain strict.
Focused analyzer and runtime regressions cover both directions, reversal,
and modern-version rejection. The original script now analyzes and executes
without diagnostics; no rewrite of its source was needed.

The full COINBASE:BTCUSD daily history begins on 2014-12-01. The native
TradingView Strategy Tester CSV has 458 numbered trades: 457 closed and one
open. All 457 closed trades agree with the local runtime on direction,
entry/exit UTC dates, entry/exit prices to the CSV's cent precision, absolute
quantity, and displayed net PnL to the cent. The remaining open entry agrees
on direction, UTC date, price, and quantity. Its mark-to-market PnL was not
compared because TradingView included the still-forming 2026-09-26 bar while
the local source data ended on the last confirmed bar. No other report metric
or intrabar behavior is claimed by this receipt.

The exact public source, downloaded native trade CSV, local result,
comparison program, and JSON receipt are in the ignored
`.local/continued-popular-20260926/` folder under `ut-bot-strategy-*`.
The original trade export also remains in `I:\sys\下载`. External market data
and TradingView's UI remain outside the runtime core.

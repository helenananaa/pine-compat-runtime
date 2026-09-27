# UT Bot Strategy: AAPL weekly comparison

Date: 2026-09-27. This is local historical evidence with four fully compared input settings. It does not qualify live-tick execution or distribution.

The unchanged public Pine v4 [UT Bot Strategy by QuantNomad](https://www.tradingview.com/script/VKMgZdYr-UT-Bot-Strategy/) is retained at `.local/ut-bot-aapl-weekly-20260927/ut-bot-strategy-original.pine` (SHA-256 `0e6ea37251e0a35b45de238427c6a9692a64b62abc502198aad6dad7a514d0c1`). The symbol picker and trade report identify `NASDAQ:AAPL`; TradingView's chart export identifies its actual series as `BATS:AAPL, 1W`, and the chart says “NASDAQ by Cboe One.” The local host uses `BATS:AAPL`, timeframe `1W`, USD, price grid `1/100`, and integer quantity precision.

The export contains **2,390 weekly bars**, from 1980-12-08 14:30 UTC to 2026-09-21 13:30 UTC. No prefix or final bar was excluded; the market was closed at capture. All four independently exported chart CSVs have identical time/OHLC fields. The HA export has a revised final-week volume of 186,838,541 versus 193,764,266 in the earlier exports; all preceding volume rows agree. This script does not read volume. TradingView's report dates are displayed in UTC+8. Strategy properties were 1,000,000 USD initial capital, fixed quantity 1, pyramiding 1, zero commission/slippage, and on-bar-close execution.

| Original script inputs | Official chart and trade CSVs | Native/local closed trades | Open entry | Buy/Sell signal positions | Result |
| --- | --- | ---: | ---: | ---: | --- |
| Sensitivity 1, ATR 10, ordinary candles | both retained | 315 / 315 | 1 checked | 4,780 | Full row comparison: 0 differences |
| Sensitivity 2 | both retained | 139 / 139 | 1 checked | 4,780 | Full row comparison: 0 differences |
| ATR period 14 | both retained | 309 / 309 | 1 checked | 4,780 | Full row comparison: 0 differences |
| Heikin Ashi signals | both retained | 211 / 211 | 1 checked | 4,780 | Full row comparison: 0 differences; final-week volume revision recorded |

For all four settings, `compare.py` checks every bar's time/OHLC and Buy/Sell bits, plus every closed trade's direction, entry/exit date, reported price within `1e-9`, quantity, and net PnL. Volume matches on every bar except the explicitly recorded final HA row. This covers **974 native closed trades and 19,120 signal positions**, with no signal or trade differences. Each last open entry matches in direction, date, reported price, and quantity; its changing mark-to-market PnL was not compared. The largest absolute PnL difference is floating-point noise below `6e-14` USD. This independently confirms the chart-tick market-fill behavior also exercised on AAPL daily history.

The ATR 14 and Heikin Ashi reports were subsequently exported through Chrome and compared row by row. The HA chart initially exported only 300 visible bars; zooming the weekly chart out to include 1980 produced the full 2,390-row export retained here. A local host-neutral HA provider was constructed from the frozen weekly bars using the same formula documented in the [daily receipt](UT_BOT_AAPL_DAILY_EXPANSION_20260927.md); its request key is `{"chart":"heikinashi","symbol":"BATS:AAPL"}:1W`, and its CSV SHA-256 is `b24da55da147a5896bbf26e7fa98baa013c9f897d178b8fa235178361a823919`.

The native HA input dialog and numbered trade report screenshots are retained
as `tradingview-ha-settings.png` and `tradingview-ha-report.png` beside the CSVs.

| Retained official export | SHA-256 |
| --- | --- |
| `tv-chart-default.csv` | `80d31dde38b1dba5947f935ef3109e80626332a8d340a6575751904c32246050` |
| `tv-trades-default.csv` | `21a8f06bedfb5b4d3f2c8a2fa240d0dd4224f355fa90bb37e51c17d0e10adaa0` |
| `tv-chart-sensitivity2.csv` | `2bd91223d25280eac99214c0cbc888295643e49a3bd7ea44bdcf971b1b33f9bc` |
| `tv-trades-sensitivity2.csv` | `3accc584e5b542fd2e82aded9cd814431a7b251ec415bfa66d5c07ba99eca829` |
| `tv-chart-atr14.csv` | `840126d2a3dc2eabbd0445412d1fdec3a9f0d4e24d78d32a62f32970a25c4248` |
| `tv-trades-atr14.csv` | `924def6e62b065b2a2efab8437a63a58a69c338d29a5b8173fc722ff21f17ac1` |
| `tv-chart-heikin-ashi.csv` | `64b32252a78d0b458213be856abc9080bf0327eb6803dc98f0b9159b5f427ae3` |
| `tv-trades-heikin-ashi.csv` | `957e8609705775b4feba07b662eee4d9c7d899851b121218996ada866048efa0` |

The retained `comparison.json` covers all four settings. All four local runs have zero diagnostics and byte-identical full JSON across `run`, `run-incremental`, and `run-realtime-history`. This slice required no further runtime code change.

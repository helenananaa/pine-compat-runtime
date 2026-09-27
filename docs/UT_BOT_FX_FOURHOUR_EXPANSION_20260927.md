# UT Bot Strategy: EURUSD four-hour parameter expansion

Date: 2026-09-27. This is local historical comparison evidence, not live-tick or distribution qualification.

The unchanged public Pine v4 [UT Bot Strategy by QuantNomad](https://www.tradingview.com/script/VKMgZdYr-UT-Bot-Strategy/) is 43 lines. Its source, including the original boolean `strategy.entry` directions and Heikin Ashi branch, is retained at `.local/ut-bot-fx-fourhour-20260927/ut-bot-strategy-original.pine` with SHA-256 `0e6ea37251e0a35b45de238427c6a9692a64b62abc502198aad6dad7a514d0c1`. The publication's visible source still shows the matching v4 declaration, inputs, and entry calls. No script rewrite was used.

TradingView ran the original script on `FX:EURUSD`, `240` minutes. Four independently exported chart CSVs contain the same 21,338 time/OHLCV rows from 2013-01-02 02:00 UTC through 2026-09-25 17:00 UTC, checked field by field. The FX market was closed at capture; no forming row or warmup prefix was excluded. The native report displays trade times in UTC+8. Its properties were set to 1,000,000 USD initial capital, fixed order size 1, pyramiding 1, zero commission and slippage, and on-bar-close execution. The local host used the exported bars, `FX:EURUSD`, timeframe `240`, USD, price grid `1/100000`, and integer quantity precision.

| Inputs relative to published defaults | CLI override | Native/local closed trades | Open entries checked | Buy/Sell signal positions | Differences |
| --- | --- | ---: | ---: | ---: | ---: |
| Sensitivity 1, ATR 10, ordinary candles | none | 2,459 / 2,459 | 1 | 42,676 | 0 |
| Sensitivity 2 | `1=2` | 1,074 / 1,074 | 1 | 42,676 | 0 |
| ATR period 14 | `2=14` | 2,471 / 2,471 | 1 | 42,676 | 0 |
| Heikin Ashi signals | `3=true` plus host data | 1,815 / 1,815 | 1 | 42,676 | 0 |

Across the four cases, **7,819 closed trades and 170,704 Buy/Sell signal positions** match. Each closed trade was checked for long/short direction, entry/exit UTC+8 timestamp, five-decimal price within `1e-9`, fixed quantity, and PnL within its native CSV display precision. Each last open entry matches in direction, timestamp, price, and quantity; its changing mark-to-market PnL was not compared. Missing or extra Buy/Sell shapes are checked on every bar. Other chart indicators, colors, visual placement, and alert delivery are outside this receipt.

The Heikin Ashi input exercises the script's `security(heikinashi(syminfo.tickerid), timeframe.period, close, lookahead=false)` branch. The host-neutral request provider was explicitly supplied 21,338 synthetic OHLCV bars built from the same chart data: HA close is `(open+high+low+close)/4`; initial HA open is `(open+close)/2`; later HA open is the mean of the preceding HA open and close; high/low are the corresponding envelopes. The reproducible `build_ha_provider.py` and provider CSV (SHA-256 `3561f5ed7d7041d5fbab8dbcc389c375c3215117e50b0d6ec81e7375d943d036`) are retained with the evidence. The core did not acquire market data or implement a concrete provider.

| Native export | SHA-256 |
| --- | --- |
| `tv-chart-default.csv` | `0e30ded07feb3a2bf9f079bd8531c4fb366e0e301f28613992f0dff88be579b0` |
| `tv-trades-default.csv` | `606e6e6cbb1eaed3c7f3fb600f07882b1e8b3fa80747db14cd909a9b6ee9651` |
| `tv-chart-sensitivity2.csv` | `251da797d1abf2c6e8a0928c68febe489f4db88fa772e169357d8285bc338f1a` |
| `tv-trades-sensitivity2.csv` | `0f0e65796555ff4f1dc3c1f3ea53cf50b65587055539bdb839476ea822be4b32` |
| `tv-chart-atr14.csv` | `3ecde2705dbd9d14c54ebf6f0a23fb996742edb1cc8a072115677d1dc4186755` |
| `tv-trades-atr14.csv` | `e46b7123717754bf6b7215ea2c9e6d73b33919443adaec13d366a94737d1e628` |
| `tv-chart-heikin-ashi.csv` | `a291446f9f8c9c55915ba3e82f7a336023281c77761ba7639e0ae6a9fba7d816` |
| `tv-trades-heikin-ashi.csv` | `729bc9fae436da03c613541aaee40fc820483ee5477896dc5c21b00063535de9` |

The retained `compare.py` verifies every bar and trade and writes `comparison.json`. All four local cases have zero diagnostics. For each input, `run`, `run-incremental`, and `run-realtime-history` produced byte-identical full JSON. The per-case SHA-256 hashes are `5b75605f17a791f4ef7662ba46d24667dd6a521ad65ec1ac38226525452bdf96` (default), `2400b08692d0331cafedb3e90fc70328cf3fed5ef7b172f1d7ee34fb9a3a2d33` (sensitivity 2), `70db35ae0171534f1aa8aa2089ea816f2a0cc3959173f16359eafd687957643d` (ATR 14), and `65b7b904e985b4271e5be3698779696d5cbd1ca32f08fc5cc0a1d398204216b6` (Heikin Ashi). This slice required no runtime code change.

Follow-up on 2026-09-27: the AAPL stock fill-grid change also corrected one half-tick EURUSD reversal. The chart open `1.088115` now fills at the native report price `1.08812` in the default and ATR-14 cases; the previous five-decimal comparison had admitted that half-tick difference. The comparator now requires reported fill prices within `1e-9` and demonstrably rejects the old result. On-grid opens retain their original floating-point value. The four native comparisons were rerun with zero signal, trade, and open-entry errors, and all three historical modes remain byte-identical per case. The runtime hashes above and retained `comparison.json` reflect this follow-up.

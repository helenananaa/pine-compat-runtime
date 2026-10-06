# UT Bot Strategy: AAPL daily parameter and fill-grid comparison

Date: 2026-09-27. This is local historical comparison evidence, not live-tick or distribution qualification.

The unchanged public Pine v4 [UT Bot Strategy by QuantNomad](https://www.tradingview.com/script/VKMgZdYr-UT-Bot-Strategy/) is retained at `.local/ut-bot-aapl-daily-20260927/ut-bot-strategy-original.pine` (SHA-256 `0e6ea37251e0a35b45de238427c6a9692a64b62abc502198aad6dad7a514d0c1`). TradingView's symbol picker identified Apple Inc. as `NASDAQ:AAPL`; its chart-data export identified the actual series as `BATS:AAPL, 1D`, with the chart displaying “NASDAQ by Cboe One.” The native trade CSV names `NASDAQ_AAPL`. This receipt preserves those identities rather than treating them as interchangeable feeds.

The chart export contains **11,533 daily bars**, from 1980-12-12 14:30 UTC to 2026-09-25 13:30 UTC. All four independently exported chart CSVs have identical time/OHLCV fields at every row. The market was closed at capture; no forming bar or warmup prefix was excluded. Native report dates are displayed in UTC+8. Properties: 1,000,000 USD initial capital, fixed quantity 1, pyramiding 1, zero commission/slippage, on-bar-close execution. The local host supplied those same bars, `BATS:AAPL`, `1D`, USD, price grid `1/100`, and integer quantity precision.

| Original script inputs | CLI override | Native/local closed trades | Open entries checked | Buy/Sell signal positions | Differences |
| --- | --- | ---: | ---: | ---: | ---: |
| Sensitivity 1, ATR 10, ordinary candles | none | 1,585 / 1,585 | 1 | 23,066 | 0 |
| Sensitivity 2 | `1=2` | 683 / 683 | 1 | 23,066 | 0 |
| ATR period 14 | `2=14` | 1,591 / 1,591 | 1 | 23,066 | 0 |
| Heikin Ashi signals | `3=true` plus host data | 1,197 / 1,197 | 1 | 23,066 | 0 |

Across the four cases, **5,056 closed trades and 92,264 Buy/Sell signal positions** agree. `compare.py` checks every bar's time/OHLCV, each signal bit, and every closed trade's direction, entry/exit date, price, quantity, and net PnL. Reported fill prices agree within `1e-9`; PnL uses the native CSV's two-decimal display precision and all 5,056 values agree to floating-point noise. Each final open entry agrees in direction, date, price, and quantity. Its changing mark-to-market PnL was not compared. Other indicators present on the chart are outside this receipt.

The initial local run matched every signal, trade direction, date, and displayed fill price, but 368 of 1,585 default-case net PnL values disagreed by more than half a cent. Historical adjusted AAPL OHLC contains sub-cent prices: one reversal used raw opens `0.142299` and `0.145089`; the native report filled at `0.14` and `0.15`, realizing `0.01`, while the old broker used the raw opens and realized `0.002790`. The broker now snaps off-grid market fill prices to the host-supplied chart tick before slippage and accounting, while preserving already aligned prices exactly. This also matches the native displayed tie cases `35.315 → 35.31` and `35.735 → 35.74`; a focused regression test covers them. This is deterministic Pine broker behavior; the runtime still receives data and tick metadata from its host.

For the Heikin Ashi branch, the host-neutral request provider supplies 11,533 synthetic bars from the same exported chart: HA close is `(open+high+low+close)/4`, initial HA open is `(open+close)/2`, later HA open is the preceding HA open/close mean, and high/low are the corresponding envelopes. The request key is `{"chart":"heikinashi","symbol":"BATS:AAPL"}:1D`. `build_ha_provider.py` and the provider CSV (SHA-256 `7edc11228530c0e712b668a03a87a64d1f0eef1f562cdbeed193cf5542aca0a9`) are retained with the evidence. The core did not acquire market data.

| Native export | SHA-256 |
| --- | --- |
| `tv-chart-default.csv` | `801a90e1622ec75b1b2f09a31b8e640e944f034495b154fd845c4a653a4eaa75` |
| `tv-trades-default.csv` | `37ab90eb968b09be3abca4b3c6945a159122ce1859ef655be35f92eea62f2ef7` |
| `tv-chart-sensitivity2.csv` | `0eaeda982572d51c035c4c23709cd29f40e334983be0f2e4c529276a39f01fae` |
| `tv-trades-sensitivity2.csv` | `4b59a73c12599e26171e1c87d5beb6f72997f44539eab8d980c842ed675c5781` |
| `tv-chart-atr14.csv` | `b5cdce61154bd01ec32d1ef3c050ff5f23137e4a0afe19eaa778a0cfb9ab9bba` |
| `tv-trades-atr14.csv` | `715f398d19f7ec66111e754b593db57abec9d91cc6176bf0f2bca68275ae7039` |
| `tv-chart-heikin-ashi.csv` | `ac1671d93252ad8889ba6bc9a2d00dd45001ef6f107bff3ce0f49bfa9969580c` |
| `tv-trades-heikin-ashi.csv` | `192654a19c70027195d14fbc30920ff76d6f198f7fada41ee66875f523a04fad` |

The retained `comparison.json` records the four full comparisons. `run`, `run-incremental`, and `run-realtime-history` produced byte-identical full JSON per case, with no diagnostics. The full `pine-runtime` library suite passed 1,973 tests; formatting and whitespace checks passed. This evidence does not cover live-tick recalculation, other stocks, or another data provider.

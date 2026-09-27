# Hull Suite Strategy: EURUSD four-hour expansion

Date: 2026-09-27. This is a local historical comparison, not a live-tick or distribution qualification.

The unchanged public Pine v4 [Hull Suite Strategy by DashTrader](https://www.tradingview.com/script/Q9OQye4C-Hull-Suite-Strategy/) source (SHA-256 `235a18e8a69aa6479d446bebc8c023eb0ce1b4a6e293e706cec079293354037d`) was run on `FX:EURUSD`, `240` minutes. This extends the [ETHUSD daily comparison](HULL_ETH_DAILY_EXPANSION_20260927.md) to forex, an intraday timeframe, the THMA branch, and a numeric length change. The original source and official exports are retained in ignored `.local/hull-fx-fourhour-20260927/`; no script edits were used to make the cases pass.

The chart export has **21,338 bars**, from 2013-01-02 02:00 UTC through 2026-09-25 17:00 UTC. All bars were passed to the runtime, with no warmup or comparison exclusion. The script's own default 2016-01-01 to 2030-12-30 backtest window remained in force. TradingView's initial capital was set to 1,000,000 USD; the strategy retained 100% equity order size, pyramiding 1, zero commission/slippage, and on-bar-close execution. The local host supplied the exported OHLCV, `FX:EURUSD`, `240`, USD, price grid `1/100000`, and integer quantity precision. The native trade report displays times in UTC+8; the chart CSV uses UTC timestamps. The comparison converts runtime timestamps to UTC+8 before checking report rows.
All three independently exported chart CSVs contain the same 21,338 time/OHLCV rows, checked field by field. The final native input dialog confirmed `all`, `Thma`, and length 89; the properties dialog confirmed 1,000,000 USD initial capital.

| Inputs | Native closed trades | Runtime closed trades | Trade time, price, direction differences | Quantity differences | Numeric plot cells | Plot differences |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `long`, `Hma`, length 55 | 378 | 378 | 0 | 0 | 42,554 | 0 |
| `all`, `Hma`, length 55 | 756 | 756 | 0 | 0 | 42,554 | 0 |
| `all`, `Thma`, length 55 | 848 | 848 | 0 | 0 | 42,570 | 0 |
| `all`, `Thma`, length 89 | 584 | 584 | 0 | 0 | 42,502 | 0 |

The three `all` cases each have one native open position, excluded from the closed-trade check. `MHULL` and `SHULL` were compared from the first chart bar, with missing values checked as missing and numeric absolute tolerance `1e-10`. Entry/exit timestamps, five-decimal displayed prices, direction, and integer quantities agree for every closed trade. Native PnL is display-rounded; its largest absolute difference from the local result is 0.00669 USD across all cases. This does not assert bit-exact native broker internals or chart color/fill appearance.

The comparison script is `.local/hull-fx-fourhour-20260927/compare.py`, and its machine-readable receipt is `comparison.json`. Official export SHA-256 hashes:

| File | SHA-256 |
| --- | --- |
| `tv-chart-hma-long.csv` | `91fb7698f8e24e177bcec0843c7cc991761a467e5029433a1383f3cca9cc411e` |
| `tv-chart-thma-all.csv` | `bc90133be0498cf03833980e5d9f4cefff5b41bc8ba05d7be48602b6ce6302f9` |
| `tv-chart-thma89-all.csv` | `5fd5c609ecd029b84df4016320963813d6ad2c9f714d278ed43121a58f063b1f` |
| `tv-trades-hma-long.csv` | `b874dd26d4af10f33d137c6833ce7c5c96d3542ee663c0ba3eb0ba0abb87467a` |
| `tv-trades-hma-all.csv` | `410927b866e0b351435c3cb2ce594ac9dd5ce2a6d32f827b7d4afb3c26e5b308` |
| `tv-trades-thma-all.csv` | `dbf46d0dea3797d649849b7f2e11176126fb7cc255659048dd436efdddcd32bf` |
| `tv-trades-thma89-all.csv` | `4b0b4016e523a8fbf016bc8601c1d2078d27efca84edb0ad5433d08224c3a384` |

For both THMA length 55 and length 89, `run`, `run-incremental`, and `run-realtime-history` produced byte-identical full JSON. Their SHA-256 hashes are `c654fb1033a3c8245c04669b2927ff088407df7c2b292b760db5ccd18a191f4a` and `7930000291cf956684c7194b221392ba98c3c7dde4a8fcfd896b6ebf226231bb`, respectively. This slice required no further runtime code change.

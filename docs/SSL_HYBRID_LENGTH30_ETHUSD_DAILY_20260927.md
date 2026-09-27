# SSL Hybrid Strategy HMA 30 on ETHUSD daily history (2026-09-27)

The unchanged public Pine v5 [SSL Hybrid Strategy by kevinmck100](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/) was run in Chrome on `COINBASE:ETHUSD`, `1D`. Only `SSL1 / Baseline Length` changed from default 60 to 30; `SSL1 / Baseline Type` remained `HMA`, `Source` remained `close`, and all other input values were retained. The 557-line source is frozen at `.local/ssl-hybrid-length30-ethusd-daily-20260927/ssl-hybrid-original.pine` with SHA-256 `1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`, identical to the [BTCUSD HMA 30 comparison](SSL_HYBRID_LENGTH30_BTCUSD_DAILY_20260927.md).

The ignored evidence directory retains the original TradingView chart CSV (`native-chart.csv`, SHA-256 `d936e2de40fb5f780b4f394b965e73b6ce06e26f0f492f7dd6548167a8096aed`), the List of trades CSV (`native-trades.csv`, SHA-256 `42b3e20c7a5813ce64455596ad69eca7f99cddef8c5b1d49140d89ad7f53943a`), a screenshot, and the visible TradingView Inputs and Properties panels as text. The Properties panel shows USD 5,000 initial capital, 10% of equity default order size, pyramiding 10, 0.04% commission, zero slippage, default four-tick historical bar detail, and one-tick order delay. The chart used the Coinbase ETHUSD feed; the local host contract explicitly supplied symbol, daily timeframe, USD chart currency by default, and a `1/100` price grid.

The chart CSV has 3,780 rows from 2016-05-23 through 2026-09-27 UTC. Its last September 27 bar was forming and excluded, leaving **3,779 confirmed bars** through September 26. `prepare_bars.py` converts the frozen chart's OHLCV into host-provided bars and compares them with the separately frozen ETHUSD daily chart used for the Bitduke strategy: all confirmed time/OHLCV rows agree, with no revision on this overlap.

`compare.py` checks the eight CSV-exposed indicator series at `1e-8` absolute tolerance. All **26,279 nonblank exported positions match**, including 3,746 values in each baseline, SSL1, upper-channel, and lower-channel series. The optional second moving average is blank throughout. TradingView's chart CSV flattens the script's false/`na` Candle Size condition to zero; the comparator applies that export meaning. The unchanged source analyzes and runs with zero diagnostics.

The native report contains **786 closed trades and four open entries**. The local result has the same counts. Every closed entry ID, UTC entry/exit date, and displayed entry/exit price agrees, as do the four open entry IDs, dates, and prices. Native trade CSV quantities display only four decimal places in this case: the maximum difference against local internal quantities is `0.000100080`, and the maximum net-PnL difference is `$0.061555`. These display comparisons do not prove exact internal quantity or cent-by-cent PnL parity. A local default-length 60 control on the same frozen bars has 525 closed trades, so HMA 30 exercises a distinct trade path on ETHUSD as well as on BTCUSD.

Analysis schema 6 identifies the baseline-length input as callsite 13. The CLI run used `--input-override 13=30`. Batch, incremental, and realtime-history JSON are byte-identical, each SHA-256 `05bfd97fa7f0e2e126310622942bbb6191183ffb6e9d60ac0a18bc590eaba572`; `verify_modes.py` checks this alongside the native comparison receipt. No runtime semantic repair was needed in this slice. Scope is confirmed historical daily bars for one source, parameter, and symbol; no forming-bar or live-tick parity is claimed.

Reproduce from the repository root:

```powershell
python .local/ssl-hybrid-length30-ethusd-daily-20260927/prepare_bars.py
target/debug/pine-compat.exe run .local/ssl-hybrid-length30-ethusd-daily-20260927/ssl-hybrid-original.pine --bars .local/ssl-hybrid-length30-ethusd-daily-20260927/bars-confirmed.csv --chart-symbol COINBASE:ETHUSD --chart-timeframe 1D --chart-price-grid 1/100 --input-override 13=30 > .local/ssl-hybrid-length30-ethusd-daily-20260927/local-batch.json
python .local/ssl-hybrid-length30-ethusd-daily-20260927/compare.py
python .local/ssl-hybrid-length30-ethusd-daily-20260927/verify_modes.py
```

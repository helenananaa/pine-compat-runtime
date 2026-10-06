# Pine v6 short market order quantity check

The complete nine-line probe is `.local/community-coverage-20260923/percent-short-market-probe-v6.pine` (SHA-256 `e9b96657a78ec71d9656db5adde65c4b38afd3ef93aa1fdebe194076c06da589`). It declares `default_qty_type=strategy.percent_of_equity`, `default_qty_value=50`, `calc_on_order_fills=true`, and `margin_short=0`. It places `strategy.order("S", strategy.short)` when flat and `strategy.close_all()` when short during June 1–3, 2020. TradingView compiled this unmodified Pine v6 probe in the independent `Pine quantity oracle 20260925` COINBASE:BTCUSD 1D layout (`https://www.tradingview.com/chart/K1m3gqVM/`). The native *List of trades* visible on September 25 reported:

| Trade | Native short entry | Native exit | Native entry value | Native PnL |
| --- | --- | --- | ---: | ---: |
| 1 | 2020-06-02, 10,208.96 | 2020-06-02, 10,208.96 | USD 499.99 | USD 0 |
| 2 | 2020-06-02, 10,237.60 | 2020-06-02, 9,270.00 | USD 499.99 | USD 47.26 |
| 3 | 2020-06-03, 9,521.53 | 2020-06-03, 9,521.53 | USD 523.57 | USD 0 |
| 4 | 2020-06-03, 9,385.22 | 2020-06-03, 9,695.00 | USD 523.62 | USD -17.28 |
| 5 | 2020-06-04, 9,668.06 | Open | USD 514.98 | Open |

The local run on `.local/community-coverage-20260923/btcusd-bars.csv` (SHA-256 `23e2f25d5a93af010443c41c60fd32e3feac3b37d983d9c857233c6992916e08`) produced `.local/community-coverage-20260923/percent-short-market-probe-v6-local.json` (SHA-256 `01c2e687f723e000c699746c6989a1c38d3b86f0fc246e93776b7f7c22942539`), with zero diagnostics, four closed trades and one open order. All five entry dates/prices and all four exit dates/prices match the native visible report. The local entry values are USD 500.0000, 500.0000, 523.5774, 523.6286, and 514.9863, respectively. These agree with the native displayed values to within USD 0.01; the displayed native values appear truncated to cents. The native UI displays quantity only to two decimals here, so this check does not establish exact unit quantity. Clicking its CSV download did not produce a new file in the configured downloads directory, `I:\sys\下载\`; this audit relies on the visible report, not a CSV export.

The focused runtime regression `percent_of_equity_short_strategy_order_uses_same_bar_fill_price_across_versions` also exercises the same host-neutral `strategy.order` behavior with Pine v1 through v6. It checks that a previous-bar market order retains its creation-close quantity across a gap, while a same-bar high fill sizes from the high. Only the v6 case above has independent native evidence; the v1–v5 cases are versioned regression coverage, not claims of native parity.

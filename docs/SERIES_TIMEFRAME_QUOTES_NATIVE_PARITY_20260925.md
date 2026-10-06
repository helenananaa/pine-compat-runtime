# v6 series timeframe and quote variables: native parity (2026-09-25)

TradingView compiled the retained v6 oracle on `COINBASE:BTCUSD` 1D and exported `I:\sys\下载\COINBASE_BTCUSD, 1D (40).csv`. An ignored copy is `.local/community-coverage-20260923/series-timeframe-quotes-native-v6-20260925.csv` (SHA-256 `cb913ca28efbca7c5f8286dcb0a54fa5dd72b95867575c75e38bdc79f05f70ee`).

The oracle alternates timeframe arguments `"1"` and `"5"` by `bar_index`, plots `timeframe.in_seconds(tf)`, and plots whether `ask` and `bid` are `na`. The comparison ran the identical source locally on the retained 4,282 confirmed Coinbase daily bars. It joined 299 of the native export's 300 rows by UTC timestamp; the unmatched last row was the forming daily bar. All 299 times and OHLC rows aligned, and all 1,196 output cells matched exactly: seconds alternated 60/300 with the recorded parity, and both quote checks returned 1 throughout. The local comparison script is `.local/community-coverage-20260923/compare-series-timeframe-quotes-20260925.py`.

This proves the v6 time-based chart subset of these features on this dataset. It does not cover 1T quote values or the remaining `TradingView/ta/14` request dependency gaps described in `TRADINGVIEW_TA_V14_VOLUME_AUDIT_20260925.md`.

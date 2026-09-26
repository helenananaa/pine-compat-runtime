# Public RSI Mean Reversion Bot Strategy v4 native parity (2026-09-25)

The complete public [RSI Mean Reversion Bot Strategy](https://www.tradingview.com/script/PhfmO5kT-RSI-Mean-Reversion-Bot-Strategy/) by Solutions1978 has a `//@version=4` directive and a `RSI Bot Strategy v3` short title. Its visible source was saved with nonbreaking spaces normalized to ordinary spaces in ignored `.local/community-coverage-20260923/solutions1978-rsi-mean-reversion-v4-20260925.pine` (SHA-256 `a41dc91ee84967c22b1db0e1f070e135ba051816c30eeee0b3a52707cbf9832f`). No published source or market data is committed.

The unmodified source initially failed semantic admission because `strategy(..., scale=scale.left)` was absent from the strategy declaration signature. The host-neutral declaration signature now accepts and validates the documented scale constants. The visual scale setting is accepted as metadata but is not emitted as a render contract here. The original script now analyzes and executes with zero diagnostics.

The original source uses an RSI-backed stochastic, RSI, EMA and VWAP conditions, date-window inputs, long and short orders, percentage-of-equity sizing, percent commission, and fixed stop and take-profit orders. Its stochastic plot exposed a runtime warmup error: `ta.stoch` required all `length` high/low samples to be non-`na`, while native output uses the available non-`na` extrema inside a full bar window. The runtime now preserves the full bar-window requirement and skips `na` extrema samples. A focused sparse-source regression test covers the earlier first value.

TradingView ran the publication with default inputs on `COINBASE:BTCUSD` 1D, including its default 1,000 USD initial capital. The chart CSV `I:\sys\下载\COINBASE_BTCUSD, 1D (20).csv` is retained as ignored `solutions1978-rsi-native-chart-v4-20260925.csv` (SHA-256 `2a80e985623ef2a9c5c4cf1e27e5b459c3b680d056b888c6b78ffac6b5824319`). The native trade CSV `I:\sys\下载\RSI_Bot_Strategy_v3_COINBASE_BTCUSD_2026-09-25.csv` is retained as ignored `solutions1978-rsi-native-trades-v4-20260925.csv` (SHA-256 `3b8d6eaff65fd5e5f6d87f9f64fa21a36a0b6bf9e9f5d3aa5bd05c8cbcca614a`). The chart export contains 4,283 rows, of which the last is forming; the other 4,282 match the retained local bar input by timestamp and OHLC.

The ignored `compare_rsi_mean_reversion_v4_20260925.py` checks all nine plot columns and the full trade list. **All 38,538 plot positions match**, including 24,387 paired blanks. Maximum absolute numeric difference is `1.46e-11`. **All 11 closed trades match** by direction, entry and exit UTC date, displayed price, and six-decimal quantity when local chart quantity precision is set to 6. Local PnL differs from native cent-rounded display by at most `0.004475` USD. This does not establish equality of hidden native internal values beyond export precision. The script's default trading window extends into early January 2022 for its last exit.

The research strategy and date-range change were undone, and the TradingView chart layout was saved. Reproduce from the repository root:

```powershell
target/debug/pine-compat.exe analyze .local/community-coverage-20260923/solutions1978-rsi-mean-reversion-v4-20260925.pine --format json
target/debug/pine-compat.exe run .local/community-coverage-20260923/solutions1978-rsi-mean-reversion-v4-20260925.pine --bars .local/community-coverage-20260923/lazybear-volume-flow-chart-bars-v1-20260925.csv --chart-symbol COINBASE:BTCUSD --chart-timeframe 1D --chart-quantity-precision 6 > .local/community-coverage-20260923/solutions1978-rsi-mean-reversion-local-q6-v4-20260925.json
python .local/community-coverage-20260923/compare_rsi_mean_reversion_v4_20260925.py
```

This qualifies the default historical path on the named daily chart. Other inputs, symbols, timeframes, and realtime forming updates need their own native evidence.

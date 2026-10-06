# AlphaTrend Strategy v5 native parity (2026-09-25)

The complete public [AlphaTrend Strategy](https://www.tradingview.com/script/3wdQu7P3-AlphaTrend-Strategy/) by KivancOzbilgic had 7,076 boosts in the September 25 Chrome page. The current visible source is Pine v5, retained with nonbreaking spaces normalized to ordinary spaces in ignored `.local/community-coverage-20260923/kivanc-alphatrend-strategy-v5-20260925.pine` (SHA-256 `862d00e7b1106aa2104e2210973db98b43ac7148f267c23c6c4e2c1925d4a970`). It analyzes and runs unmodified with zero diagnostics.

TradingView ran the publication with default inputs on `COINBASE:BTCUSD` 1D. Its report displayed 1 M USD initial capital and one BTC per entry. The chart CSV `I:\sys\下载\COINBASE_BTCUSD, 1D (24).csv` is retained as ignored `kivanc-alphatrend-strategy-native-chart-v5-20260925.csv` (SHA-256 `55ab6f74480cd49a1865277a567587ce7a2cf8bb18f9f8bfbaefb4f0e6812b6b`). The trade report `I:\sys\下载\ATSt_COINBASE_BTCUSD_2026-09-25.csv` is retained as ignored `kivanc-alphatrend-strategy-native-trades-v5-20260925.csv` (SHA-256 `adec4b5b59780d69ebd3b0047b64a9acdc8abc168375c79bab24dda340f1d7a3`). The chart export has 4,283 rows; the final forming bar was excluded. The other 4,282 match the retained runtime input by timestamp and OHLC.

The ignored `compare_kivanc_alphatrend_strategy_v5_20260925.py` checks both trend-line plots, both shape columns, and the full trade list. **All 17,128 plot/shape positions match**, including 8,594 paired blanks. Both shape columns are blank by default because the published strategy defaults `Show Signals?` to false. The maximum absolute numeric plot difference is `1.46e-11`. **All 98 closed trades match** by direction, UTC entry/exit date, entry/exit price, one-BTC quantity, and net PnL; maximum PnL difference is `1.16e-11` USD. The 99th native row pair describes an open long entered July 16, 2026 at 64,712.04 USD, matching the local last order. Its open profit is time-dependent and is excluded from the closed-trade comparison. Batch, incremental, and historical realtime execution produce byte-identical full JSON output (SHA-256 `f69413865ed8c2024f1ffceba55904c0702f8bfb1f1063168bf28c3927166b78`).

This comparison qualifies default historical fills on the named daily chart. Other inputs, symbols, timeframes, intrabar fills, chart magnifier behavior, and forming updates need separate native evidence. No core semantic change was needed for this source. The temporary strategy and date-range change were undone, the chart layout saved, and research tabs closed.

Reproduce from the repository root:

```powershell
target/debug/pine-compat.exe analyze .local/community-coverage-20260923/kivanc-alphatrend-strategy-v5-20260925.pine --format json
target/debug/pine-compat.exe run .local/community-coverage-20260923/kivanc-alphatrend-strategy-v5-20260925.pine --bars .local/community-coverage-20260923/lazybear-volume-flow-chart-bars-v1-20260925.csv --chart-symbol COINBASE:BTCUSD --chart-timeframe 1D --chart-quantity-precision 6 > .local/community-coverage-20260923/kivanc-alphatrend-strategy-local-v5-20260925.json
python .local/community-coverage-20260923/compare_kivanc_alphatrend_strategy_v5_20260925.py
```

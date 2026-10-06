# AlphaTrend v5 native parity (2026-09-25)

The complete public [AlphaTrend](https://www.tradingview.com/script/o50NYLAZ-AlphaTrend/) by KivancOzbilgic had 29,125 boosts in the September 25 Chrome page. Its current visible source is Pine v5, retained with nonbreaking spaces normalized to ordinary spaces in ignored `.local/community-coverage-20260923/kivanc-alphatrend-v5-20260925.pine` (SHA-256 `ca1531cacd2b601d7267241a9e88581edef9715dadcaa4744f43e4aec463f382`). The original source analyzes and runs with zero diagnostics.

TradingView ran the publication on `COINBASE:BTCUSD` 1D with its default settings, then with only `Change calculation (no volume data)?` enabled. The default MFI path's native chart CSV `I:\sys\下载\COINBASE_BTCUSD, 1D (22).csv` is retained as ignored `kivanc-alphatrend-native-v5-20260925.csv` (SHA-256 `03ef6ba3d7e001b7f0721d03a260d780edbbcd955ad5f7a2b9a6a4b0b064a90`). The no-volume RSI path's CSV `I:\sys\下载\COINBASE_BTCUSD, 1D (23).csv` is retained as ignored `kivanc-alphatrend-no-volume-native-v5-20260925.csv` (SHA-256 `3e52f5391a70d4548aa1db62e8da690b13598fd29da7fd97c328643fd98e81b6`). Each contains 4,283 chart rows, with the final forming bar excluded. The other 4,282 rows match the retained local bar input by timestamp and OHLC.

The ignored `compare_kivanc_alphatrend_v5_20260925.py` checks both unnamed trend lines and both BUY/SELL shape columns, preserving their exported column order. **All 17,128 positions match in each input mode**, including 8,497 paired blanks in the default MFI path and 8,493 in the no-volume RSI path. Maximum absolute numeric difference in either path is `1.46e-11`. For each mode, batch, incremental, and historical realtime execution produce byte-identical full JSON output. The local no-volume run uses the source's input call-site override `6=true`.

The native CSV does not verify the line/fill colors, alert delivery, or forming-bar behavior. Other settings, symbols, and timeframes need separate native evidence. This qualification required no core semantic change: the current runtime already covers both published calculation paths on the named daily chart.

The changed input, date range, and research indicator were undone, the TradingView layout saved, and research tabs closed. Reproduce from the repository root:

```powershell
target/debug/pine-compat.exe analyze .local/community-coverage-20260923/kivanc-alphatrend-v5-20260925.pine --format json
target/debug/pine-compat.exe run .local/community-coverage-20260923/kivanc-alphatrend-v5-20260925.pine --bars .local/community-coverage-20260923/lazybear-volume-flow-chart-bars-v1-20260925.csv --chart-symbol COINBASE:BTCUSD --chart-timeframe 1D > .local/community-coverage-20260923/kivanc-alphatrend-local-v5-20260925.json
target/debug/pine-compat.exe run .local/community-coverage-20260923/kivanc-alphatrend-v5-20260925.pine --bars .local/community-coverage-20260923/lazybear-volume-flow-chart-bars-v1-20260925.csv --chart-symbol COINBASE:BTCUSD --chart-timeframe 1D --input-override 6=true > .local/community-coverage-20260923/kivanc-alphatrend-no-volume-local-v5-20260925.json
python .local/community-coverage-20260923/compare_kivanc_alphatrend_v5_20260925.py
```

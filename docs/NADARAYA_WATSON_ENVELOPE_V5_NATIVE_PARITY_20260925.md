# Nadaraya-Watson Envelope v5 native parity (2026-09-25)

The complete public [Nadaraya-Watson: Envelope (Non-Repainting)](https://www.tradingview.com/script/WeLssFxl-Nadaraya-Watson-Envelope-Non-Repainting/) by jdehorty had 2,923 boosts in the September 25 Chrome page. Its visible source uses Pine v5 and imports `jdehorty/KernelFunctions/2`. The source is retained with nonbreaking spaces normalized to ordinary spaces in ignored `.local/community-coverage-20260923/jdehorty-nw-envelope-v5-20260925.pine` (SHA-256 `ba32b7e9311bf81a97131c605b6b5e18a17b78d0b3f8e72fc2e46179c0b41e99`). The exact public `/2` library source already retained as `jdehorty-kernel-functions-v2.pine` (SHA-256 `4b6bb1961986244e15a643769f05c3eb7cdbf736049fec8c3b0517e102d1c506`) was supplied through the host-neutral library-source input. The original indicator analyzes and runs with zero diagnostics.

TradingView ran the publication with default inputs on `COINBASE:BTCUSD` 1D. `I:\sys\下载\COINBASE_BTCUSD, 1D (26).csv` is retained as ignored `jdehorty-nw-envelope-native-v5-20260925.csv` (SHA-256 `1600bec17e4a9722bb107071f6c926913061de8fd01abeeff25830128240ebfe`). The chart export has 4,283 rows, with the final forming bar excluded. The other 4,282 rows match the local input in timestamp and OHLC.

The ignored `compare_jdehorty_nw_envelope_v5_20260925.py` checks the kernel estimate and six upper/lower boundaries. **All 29,974 plot positions match**, including 536 paired blanks; maximum absolute numeric difference is `5.68e-14`. Batch, incremental, and historical realtime execution produce byte-identical full JSON output (SHA-256 `2a9357f4433f9c9a7ae97cc2eca0445d96cf01f2853f7340afcc263a5904a6df`). The script also creates four fills. Native CSV does not expose their visual colors or geometry, so this receipt covers the seven numerical plots, not fill appearance. Other inputs, symbols, timeframes, and forming updates need separate native evidence.

No core semantic change was needed for the tested default path. The temporary indicator and date-range change were undone, the TradingView layout saved, and the research tabs closed. Reproduce from the repository root:

```powershell
target/debug/pine-compat.exe analyze .local/community-coverage-20260923/jdehorty-nw-envelope-v5-20260925.pine --library-source jdehorty/KernelFunctions/2=.local/community-coverage-20260923/jdehorty-kernel-functions-v2.pine --format json
target/debug/pine-compat.exe run .local/community-coverage-20260923/jdehorty-nw-envelope-v5-20260925.pine --library-source jdehorty/KernelFunctions/2=.local/community-coverage-20260923/jdehorty-kernel-functions-v2.pine --bars .local/community-coverage-20260923/lazybear-volume-flow-chart-bars-v1-20260925.csv --chart-symbol COINBASE:BTCUSD --chart-timeframe 1D > .local/community-coverage-20260923/jdehorty-nw-envelope-local-v5-20260925.json
python .local/community-coverage-20260923/compare_jdehorty_nw_envelope_v5_20260925.py
```

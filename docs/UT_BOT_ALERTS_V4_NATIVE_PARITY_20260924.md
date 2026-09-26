# UT Bot Alerts v4 native comparison (2026-09-24)

The public [UT Bot Alerts by QuantNomad](https://www.tradingview.com/script/n8ss8BID-UT-Bot-Alerts/) is a widely used, open-source Pine v4 indicator. Its 42-line published source was transcribed from TradingView's visible source panel into ignored `.local/community-coverage-20260923/quantnomad-ut-bot-alerts-v4.pine`, preserving tokens and continuation structure while normalizing blank lines and indentation (SHA-256 `c4131b5ae84834ca82ffa127b2da3c67f0a6778e465494338040dd8384996de2`). The complete script analyzes as executable with zero diagnostics. It exercises v4 `study`, untyped `input`, `atr`, eager `iff`, recursive `:=` series, `nz`, `ema`, `crossover`, `plotshape`, `barcolor`, and `alertcondition`.

The published default `Signals from Heikin Ashi Candles=false` was used on `BINANCE:BTCUSDT` 1h. TradingView exported `I:\sys\下载\BINANCE_BTCUSDT, 60 (3).csv` (SHA-256 `d7ccb72e3a606dd4c58e3e87546199617d1ce334e50361fbee7e8e5b392516d0b`). Its first two study columns are the script's `Buy` and `Sell` shape signals. The chart CSV supplies 2,350 hourly OHLC rows. The local input `.local/community-coverage-20260923/ut-bot-v4-native-hourly-with-200-prefix.csv` (SHA-256 `8b683f781e117ac4fcd48eddd1845720a978b249f9960fa6f4cb7d7c68bd6986`) contains those exact rows plus 200 earlier TradingView-derived hourly bars for warmup. The script's default path does not read volume; the native export has no volume column.

`compare_ut_bot_v4.py` verifies all 2,350 native OHLC rows and compares the two shape columns by bar time. All **128 Buy and 127 Sell markers match on the exact native bars**, with zero missing or extra markers. The comparison receipt is `.local/community-coverage-20260923/ut-bot-v4-native-comparison.json`. The complete local run produced zero diagnostics. Batch, incremental, and historical realtime output JSON hashes are identical: `62ecd07973f9edb3acec3526a14e774706e2bf7016f05c7a01073f6ee95a3b29`.

The default comparison uses `Signals from Heikin Ashi Candles=false`. The enabled path has a separate native comparison below. TradingView's chart CSV does not show alert delivery or bar colors. Those behaviors and forming-tick behavior are outside these native parity claims.

## Heikin Ashi option enabled

The same complete v4 script was run with `Signals from Heikin Ashi Candles=true` on the same chart. TradingView exported `I:\sys\下载\BINANCE_BTCUSDT, 60 (4).csv` (SHA-256 `d30673d4169ec085abbea0a6959bebed934b09e44bed7c67ac9d972f95d9fee70`). This export contains 2,351 hourly OHLC rows, including the next hour reached during export. The local chart input `.local/community-coverage-20260923/ut-bot-v4-ha-chart-bars.csv` (SHA-256 `dd8174d37405185b479b6de08b41d0409cfc63814cb62936d62c4b8c3f47d322`) contains those exact bars and the same 200 earlier TradingView-derived warmup bars. The request provider input `.local/community-coverage-20260923/ut-bot-v4-ha-provider-bars.csv` (SHA-256 `2825cc8b362ed02047702de58a14e7442fd88c3530d64f5cee8b037c9c789990`) supplies Heikin Ashi OHLC for the same timestamps: `HA close=(open+high+low+close)/4`, `HA open=(previous HA open+previous HA close)/2` after the first bar, and HA high/low are the corresponding envelopes. The script requests only HA close.

The comparison receipt `.local/community-coverage-20260923/ut-bot-v4-ha-native-comparison.json` verifies **2,351/2,351 chart OHLC rows** and **92/92 Buy plus 91/91 Sell markers**, with zero missing or extra markers and zero local diagnostics. Batch, incremental, and historical realtime output JSON hashes are identical: `d57edb0d6d6e28ebe923d19048f15a99c0de3c56a7c78a4e06c5d3d5378cc6ab`.

The host-neutral `request.security` contract supplies the Heikin Ashi ticker's bars. The core does not acquire or persist market data. The temporary indicator was removed, and the original TradingView layout was saved again.

Reproduction from the repository root:

```powershell
$b = '.local/community-coverage-20260923'
target/debug/pine-compat.exe run "$b/quantnomad-ut-bot-alerts-v4.pine" --bars "$b/ut-bot-v4-native-hourly-with-200-prefix.csv" --chart-symbol BINANCE:BTCUSDT --chart-timeframe 60 > "$b/quantnomad-ut-bot-alerts-v4-run.json"
python "$b/compare_ut_bot_v4.py"
```

Reproduce the enabled Heikin Ashi comparison using the ignored inputs and the original native export:

```powershell
$b = '.local/community-coverage-20260923'
$request = '{"chart":"heikinashi","symbol":"BINANCE:BTCUSDT"}:60=' + "$b/ut-bot-v4-ha-provider-bars.csv"
target/debug/pine-compat.exe run "$b/quantnomad-ut-bot-alerts-v4.pine" --bars "$b/ut-bot-v4-ha-chart-bars.csv" --chart-symbol BINANCE:BTCUSDT --chart-timeframe 60 --request-bars $request --input-override 3=true > "$b/quantnomad-ut-bot-alerts-v4-ha-run.json"
python "$b/compare_ut_bot_v4.py" 'I:\sys\下载\BINANCE_BTCUSDT, 60 (4).csv' "$b/ut-bot-v4-ha-chart-bars.csv" "$b/quantnomad-ut-bot-alerts-v4-ha-run.json" "$b/ut-bot-v4-ha-native-comparison.json"
```

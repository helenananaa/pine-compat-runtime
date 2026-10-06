# VWAP Reversal Strategy V1 native comparison, 2026-09-25

The complete, unmodified 131-line [public Pine v6 strategy](https://www.tradingview.com/script/EA1AXZui-VWAP-Reversal-Strategy-V1/) was loaded into TradingView on `BINANCE:BTCUSDT`, 1 minute, with default inputs. The frozen source is ignored locally as `.local/community-coverage-20260923/vwap-reversal-v1-v6.pine` (SHA-256 `9f095054f0410ef0491d25f76ec1d1a061df74380b6a0f0a64f5df2b243f17c1`). The independent `Pine quantity oracle 20260925` layout was used temporarily; its saved chart and editor were restored after the comparison.

The chart export `vwap-reversal-native-btcusdt-1m-chart-20260925.csv` has SHA-256 `5f2767507315c25b271d4932ee7714cc378f0a81c5bb65665e296e1081c70fcc`. It contains 26,124 confirmed minute bars from September 7, 2026 00:00 UTC through September 25, 2026 03:23 UTC. The final forming 03:24 UTC bar was excluded. The ignored `compare_vwap_reversal_native_20260925.py` builds host-neutral minute input and hourly OHLCV by grouping the native minute bars. The CLI runs the complete source with that hourly provider, `--chart-symbol BINANCE:BTCUSDT`, `--chart-timeframe 1`, `--chart-price-grid 1/100`, and `--chart-currency USD`. The currency override allows the USD-account strategy to execute on the USDT chart; it is not evidence of native FX accounting.

The comparison was rerun after `cargo build -p pine-cli --locked --quiet` from the current working tree. From the repository root, reproduce it with:

```powershell
python .local/community-coverage-20260923/compare_vwap_reversal_native_20260925.py build
target/debug/pine-compat.exe run .local/community-coverage-20260923/vwap-reversal-v1-v6.pine --bars .local/community-coverage-20260923/vwap-reversal-native-20260925-bars.csv --request-bars BINANCE:BTCUSDT:60=.local/community-coverage-20260923/vwap-reversal-native-20260925-synthetic-60m.csv --chart-symbol BINANCE:BTCUSDT --chart-currency USD --chart-timeframe 1 --chart-price-grid 1/100 > .local/community-coverage-20260923/vwap-reversal-native-20260925-local.json
python .local/community-coverage-20260923/compare_vwap_reversal_native_20260925.py compare
```

| Compared native output | Positions | Mismatches | Largest absolute difference |
| --- | ---: | ---: | ---: |
| `VWAP (Daily)`, all confirmed bars | 26,124 | 0 | 0 |
| `VWAP H1 (Bias)`, stable history from September 7 01:00 UTC to September 25 03:19 UTC | 26,060 | 0 | `4.37e-11` |
| `VWAP H1 (Bias)`, entire export | 26,124 | 63 | `30.18` |

The 63 hourly mismatches are confined to the first 59 exported minutes, for which the native requested series has earlier higher-timeframe history absent from the exported minute input, and the last four confirmed minutes inside the currently forming 03:00 UTC hourly bar. TradingView's live higher-timeframe value changes on those final minutes; this historical CLI run holds the last completed hourly value. The receipt `vwap-reversal-native-20260925-comparison.json` records hashes, counts, and first mismatches. This comparison qualifies the stable historical plot path and identifies these two boundary cases; it does not claim full hourly parity at the export edges.

TradingView's report displayed 37 closed trades; the local run also produced 37. The six latest visible native trade rows, numbers 32–37, match the local direction, entry and exit times, displayed entry and exit prices, and size 1 exactly. The native trade CSV download did not create a file in `I:\sys\下载\`, so older trade identities and exact unrounded profit/accounting remain unverified. The source enables `calc_on_every_tick` and `calc_on_order_fills`; live tick behavior was not tested by the historical chart export. The native hourly provider was not exported separately, so the local provider is explicitly derived from the exported native minute OHLCV.

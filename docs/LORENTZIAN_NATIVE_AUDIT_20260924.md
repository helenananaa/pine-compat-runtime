# Lorentzian Classification v6 native audit (2026-09-24)

The complete public [Machine Learning: Lorentzian Classification](https://www.tradingview.com/script/WhBzgfDu-Machine-Learning-Lorentzian-Classification/) was run with its published default settings on `BINANCE:BTCUSDT` 1h. TradingView exported 2,350 chart rows to `I:\sys\下载\BINANCE_BTCUSDT, 60 (2).csv` (SHA-256 `d867ad3297256b8c27d2f20fc5e47242e776957ee61ebda04a610bb2fd83108f`). Its first five study columns are `Kernel Regression Estimate`, `Buy`, `Sell`, `StopBuy`, and `StopSell`. The temporary study was removed and the original layout saved afterward; TradingView reported all changes saved.

The script imports `jdehorty/MLExtensions/2` and `jdehorty/KernelFunctions/2`. The public [MLExtensions page](https://www.tradingview.com/script/ia5ozyMF-MLExtensions/) currently shows `/3` and its v2 release notes, but does not expose a verified copy of `/2` through the viewed source panel. This audit supplies frozen `/3` source as a **diagnostic proxy**, plus exact public KernelFunctions/2 source. The results below therefore do not prove exact-library compatibility or full native parity.

## Comparison

`compare_lorentzian_native_proxy.py` compares native CSV values with the full-source local run by Unix bar time. The first local run used 2,985 previously frozen hourly bars from April to August 2026. On its 1,163 overlapping native hours, 1,162 OHLC bars and 1,162 kernel values match; the final previously captured bar was forming and differs. Buy and Sell have eight and six mismatches, respectively. This input has a different final `last_bar_index` and less future coverage than the native chart; the script uses `last_bar_index` to choose its training start, so this run is not a controlled signal comparison.

The controlled run used the 2,350 hourly OHLC bars in the new native export, with 200 earlier hourly bars from the existing local TradingView-derived export prepended for indicator warmup. The local input file is `.local/community-coverage-20260923/lorentzian-native-hourly-with-200-prefix.csv` (SHA-256 `a75f6a4d44f2ecce3d0e2dd840334ee375a0f6f5f9615e1c1fb2bef2ec47d01f`). The native CSV has no volume column; the local input sets volume to zero, and this complete script and its supplied libraries do not read `volume`.

| Exported output | Result against local `/3` proxy |
| --- | --- |
| OHLC | 2,350 / 2,350 match |
| Kernel Regression Estimate | 2,349 / 2,349 closed bars match; final forming bar differs by about `0.000912` USDT |
| Buy | 10 mismatched cells across 2,350 bars |
| Sell | 1 mismatched cell across 2,350 bars |
| StopBuy, StopSell | Both blank throughout in native and local default settings |

The local proxy run produced zero runtime diagnostics. Its JSON SHA-256 is `6330d63839300c2a32acb8592978fceb1d1fefc20b9c2352c2c6d86203ef4192`. The machine-readable comparison receipt is `.local/community-coverage-20260923/lorentzian-native-window-prefix200-comparison.json`. The export contains only the visible 2,350-hour window; TradingView may have computed earlier hidden history. The 11 remaining signal differences are unresolved. They require the exact MLExtensions/2 source or stronger prehistory and signal-state evidence before attribution to the interpreter.

The earlier exact-window run without a prefix had 26 blank local kernel values at the start, while native values were populated. Adding 200 prior bars removed all closed-bar kernel differences. Its separate receipt is `lorentzian-native-window-no-prefix-comparison.json`.

## Prehistory sensitivity (2026-09-25)

The earlier local BTCUSDT 1h fixture also supplies longer continuous prehistory. For controlled input variants, the 2,350 freshly exported native bars take precedence over that fixture in their overlap (one old forming bar has a different close). The exact same public indicator and `/3` diagnostic proxy were rerun with 1,000 and 1,500 prior hours, preserving the native OHLC window. Both inputs have contiguous hourly timestamps. The source/input/runtime receipts are ignored comparison artifacts, not new runtime dependencies.

| Prior hours | Input SHA-256 | Runtime JSON SHA-256 | Buy mismatches | Sell mismatches | Closed-bar kernel mismatches |
| ---: | --- | --- | ---: | ---: | ---: |
| 200 | `a75f6a4d44f2ecce3d0e2dd840334ee375a0f6f5f9615e1c1fb2bef2ec47d01f` | `6330d63839300c2a32acb8592978fceb1d1fefc20b9c2352c2c6d86203ef4192` | 10 | 1 | 0 |
| 1,000 | `05244662b10e27c8ad1c9690a37ba3d737e2b69c5be97c284f054a56c1e7c612` | `47e679207d9cb3c58d33e9a2a82480849d6b3a9dffea2bff3973aa7a3b69e4d2` | 3 | 1 | 0 |
| 1,500 | `d5679791eaf97e1a967b4d358ac4b1799d12a5bb6209234a18cb50c1e9a4e21a` | `f7199be6f1a14a8232a874ecbdbecd1f8399bfd3a73afd7691b3baaf45dbdea91` | 12 | 1 | 0 |

Every variant matches 2,350/2,350 native OHLC rows; each has only the final forming-bar kernel estimate difference. The Buy mismatch count is **not monotonic** with prehistory length, so the 1,000-hour result cannot be taken as a convergence proof or used to tune the core. Four signal cells remain different even in that best observed variant. The first three Buy differences in the 1,000-hour run are 2026-07-03 13:00, July 4 15:00, and July 7 15:00 UTC; the Sell difference is July 8 02:00 UTC.

The repo-local copy of the author's `MLExtensions.pine` and the frozen public `/3` proxy have 140 identical executable lines before their backtest/statistics section after whitespace and comment removal. This supports the proxy's feature and filter calculations, but neither it nor the TradingView publication page exposes the exact immutable `/2` source. The page's version notes describe the v2 backtest signature and v3 additions without showing a selectable v2 source. The native chart's hidden prehistory and exact library version therefore remain unresolved; no interpreter semantic change is inferred from these four signals.

On September 26, the author-affiliated [lorentzian-classification Git repository](https://github.com/artificial-intelligence-edge/lorentzian-classification) was unshallowed and its complete fetched history inspected. It has 23 reachable commits, but `ports/pinescript/libraries/MLExtensions.pine` first appears in commit `7d02a36` on June 25, 2026, with no earlier revision of that path. This checkout cannot establish the immutable TradingView `jdehorty/MLExtensions/2` source. It supplies no basis for attributing the remaining signal differences to the interpreter.

To reproduce the controlled proxy run from the repository root:

```powershell
$b = '.local/community-coverage-20260923'
target/debug/pine-compat.exe run "$b/jdehorty-lorentzian-v6.pine" --bars "$b/lorentzian-native-hourly-with-200-prefix.csv" --chart-symbol BINANCE:BTCUSDT --chart-timeframe 60 --library-source "jdehorty/MLExtensions/2=$b/jdehorty-ml-extensions-v3-current.pine" --library-source "jdehorty/KernelFunctions/2=$b/jdehorty-kernel-functions-v2.pine" > "$b/lorentzian-proxy-native-window-prefix200.json"
python "$b/compare_lorentzian_native_proxy.py" "$b/lorentzian-proxy-native-window-prefix200.json" "$b/lorentzian-native-hourly-with-200-prefix.csv"
```

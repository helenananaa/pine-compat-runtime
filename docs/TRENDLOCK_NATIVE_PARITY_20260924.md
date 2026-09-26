# TrendLock v6 native comparison (2026-09-24)

## Scope and sources

The public [TrendLock strategy](https://www.tradingview.com/script/FDeprmix-TrendLock-Multi-Timeframe-Supertrend-Donchian-Breakout/) was run on `BINANCE:BTCUSDT` 1h in TradingView with its published default settings. Its frozen 119-line source is `.local/community-coverage-20260923/trendlock-mtf-v6.pine` (SHA-256 `f0f889d7e6f8282527dfc4ff0e256dbc64300e6d3c291c880b55fbdb603195ec`). The run uses exact frozen sources for `TradingView/ta/14` (SHA-256 `8f8c503618bf213e5b0378f0559a4fe1b6e1a94af4a6f5810b7675dfda88d07c`) and its `TradingView/RelativeValue/3` dependency (SHA-256 `11850708783161673a00173f222a8aa823bf2aec2788c0af98c41bdd8afe4a0f`). Source analysis reported `executable=true` with zero diagnostics.

TradingView exports in `I:\sys\下载\`:

| Export | SHA-256 |
| --- | --- |
| `BINANCE_BTCUSDT, 60 (1).csv` (chart and study series) | `49bd5fdc49d75a87c4d52f8c8113e355e5e4af468d7a46a57b7b46c4ad06206d` |
| `STDB_BINANCE_BTCUSDT_2026-09-24.csv` (strategy trades) | `144028ea4c93b13ac17796de6cbe01ad650e572ac5340079e27c43c826051280` |

The chart export covers 2,295 recent hours. To supply the older strategy warmup and position path, the local fixture contains 22,755 contiguous BTCUSDT hourly bars from January 2024 to August 2026. Its 2024–April 2026 portion comes from [Binance public monthly spot archives](https://github.com/binance/binance-public-data), with each archive checked against its published SHA-256 `.CHECKSUM`. The remaining hours come from an earlier local TradingView-derived hourly export. Binance timestamps after 2024 were converted from microseconds to milliseconds. A local, host-neutral fixture builder rolled hourly bars into 5,689 UTC four-hour buckets for `request.security`. These ignored evidence files are:

| File below `.local/community-coverage-20260923/` | SHA-256 |
| --- | --- |
| `trendlock-btcusdt-long-history-bars.csv` | `f310f573f12f304ca022031afe13a7ee30fe5a695cb0acf303530f33d879bb10` |
| `trendlock-btcusdt-long-history-4h-bars.csv` | `73dbb9a18a0768cf3f0697ab74d8cc6f89bda218d4c674b8ca1e4ec4e865fb99` |

`build_trendlock_long_history.py` and `build_trendlock_4h.py` reproduce these inputs. `trendlock-btcusdt-monthly-archive-hashes.txt` records the verified monthly archive digests. The input builder belongs to the comparison harness; it is not part of the Pine core.

Reproduction from the repository root (replace `run` with `run-incremental` or `run-realtime-history` for the other paths):

```powershell
$b = '.local/community-coverage-20260923'
target/debug/pine-compat.exe run "$b/trendlock-mtf-v6.pine" --bars "$b/trendlock-btcusdt-long-history-bars.csv" --chart-symbol BINANCE:BTCUSDT --chart-timeframe 60 --chart-quantity-precision 5 --library-source "TradingView/ta/14=$b/tradingview-ta-v14.pine" --library-source 'TradingView/RelativeValue/3=.local/delivery-20260909/RelativeValue-v3.pine' --request-bars "BINANCE:BTCUSDT:240=$b/trendlock-btcusdt-long-history-4h-bars.csv" > "$b/trendlock-long-history-quantity-grid.json"
python "$b/compare_trendlock_trades.py" "$b/trendlock-long-history-quantity-grid.json"
```

## Results

`compare_trendlock_plots.py` matched five native study series (`Supertrend`, both Donchian bounds, and both entry markers) on all 1,109 overlapping local hourly bars, without a series mismatch. The first 1,108 OHLC bars matched; the final local bar was still forming when its earlier export was captured and has a different close. Receipt: `trendlock-native-local-plot-comparison.json`.

`compare_trendlock_trades.py` interprets the native report's displayed timestamps in the chart's UTC+8 timezone, pairs trades by entry timestamp, then compares entry/exit timestamps and prices. With the extended history and runtime fixes, all **420 native closed trades in the local data window** have a local counterpart with identical entry and exit timestamps and prices (floating-point entry error at most `1.46e-11` USDT; exit comparison tolerance `0.011` USDT). There are zero local-only or native-only entries in that window, and zero exit-time mismatches. Receipt: `trendlock-long-history-trade-comparison.json`.

The earlier batch, incremental, and historical realtime JSON outputs, before the quantity-grid correction, had the same SHA-256 `3c467db4554275336d00c2e2ec8248195410ccd901cfbed56c0ea3a2611cfaf9`.

The native export contains **452 trades** through September 2026. The final 32 occur after the local hourly input ends. Before the quantity-grid correction, the local/native quantity ratio for the 420 matched trades ranged from `1.00045` to `1.00095` (median `1.00065`). The native 452 entry quantities are reproduced exactly by flooring `(equity_before_entry * 0.25) / (entry_price * 1.0005)` to five decimal places, where `equity_before_entry` comes from the previous closed trade's cumulative PnL. This is an inference from the native export, not a published formula. With host-supplied `--chart-quantity-precision 5`, the interpreter now matches all 420 in-window entry quantities exactly (local/native ratio min = median = max = 1), as well as their entry and exit times and prices. All 420 local trade net PnLs differ from the native report's two-decimal values by less than `0.005` USDT (maximum `0.004973`); calculated entry-plus-exit commissions also differ from the displayed native commission by less than `0.005` USDT (maximum `0.004999876`). The final local cumulative PnL is `-715.185837` USDT versus native displayed `-715.19` USDT. One intermediate cumulative PnL comparison exceeds `0.005` USDT by about `0.000007`, so these are display-precision comparisons rather than exact internal-ledger equality. Study-series parity covers the 1,109-bar chart overlap. The TradingView layout was restored and saved after export.

## Runtime changes implicated by the comparison

- `strategy.position_size[1]` now reads the position captured at the previous script pass even when its first evaluation is inside a short-circuited condition. Generic expression history had incorrectly returned `na` there, preventing initial bracket placement.
- With `process_orders_on_close=true`, marketable stop/limit/bracket exits updated at bar close may fill at that close. The close candidate path admits only fill events, so trailing activation is never recorded as a fill.
- Focused regression tests cover guarded position history and close-price exit behavior with close processing enabled and disabled. The cross-component Rust test suite and batch/incremental comparison verify the changes.
- For percent-of-equity default orders, entry sizing reserves percentage commission and floors to the host's explicitly supplied decimal quantity grid. The same rule applies to orders re-sized for `process_orders_on_close`. Explicit `qty` and an unspecified quantity grid retain their prior behavior. A focused v6 regression uses the first native BTCUSDT entry (`46979.37` USDT, `0.05318` BTC) for both `strategy.entry` and `strategy.order`.

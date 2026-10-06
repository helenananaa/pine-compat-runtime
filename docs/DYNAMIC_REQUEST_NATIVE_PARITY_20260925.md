# Dynamic request in a local scope: native comparison (2026-09-25)

The Pine v6 oracle in ignored `.local/community-coverage-20260923/dynamic-request-local-scope-native-oracle-20260925.pine` (SHA-256 `db045daa90c8dc18504d2e95ede822d6979d413ca7cd0afa9555590ea9bdb26f`) calls `request.security()` inside an `if` block. A `series string` alternates between the `COINBASE:BTCUSD` chart and `BINANCE:BTCUSDT`; every third bar skips the request. It plots the request result, selector parity, and whether the request was reached. The exact source analyzed locally with zero diagnostics and compiled and ran in TradingView's Pine editor on the Coinbase daily chart.

TradingView's chart-data download `I:\sys\下载\COINBASE_BTCUSD, 1D (39).csv` was copied to ignored `.local/community-coverage-20260923/dynamic-request-local-scope-native-20260925.csv` (SHA-256 `760881c7675f5bc04be9272a5366bf48dbe67711008996787f532a9222528ddf`). It contains 300 daily rows from 2025-11-30 through 2026-09-25 UTC. The local run used the existing 4,280-bar Coinbase daily archive `btcusd-bars.csv` (SHA-256 `23e2f25d5a93af010443c41c60fd32e3feac3b37d983d9c857233c6992916e08`) and the independent 3,325-bar Binance daily archive `dynamic-swing-btcusdt-2017-2026-bars.csv` (SHA-256 `02d486f061acd77d62207090bea2282e45985d8aac5265668b2392ec0b2e50ee`) as host-provided request data.

The comparator checks the four chart OHLC cells and three oracle plot cells at every matching timestamp. **All 297 overlapping rows and 891 plot cells match**, with zero blank-position or numeric mismatches at absolute tolerance `1e-6`. Among those rows, 98 reached the chart context, 100 reached Binance, and 99 skipped the request. Three native rows after the local Coinbase archive ends have no local counterpart and are excluded. This establishes the conditional dynamic-selector historical path on one chart and two data contexts; it does not establish realtime, nested-request, library-export, or general popular-script parity.

Reproduce from the repository root:

```powershell
cargo run -p pine-cli --locked --quiet -- analyze .local/community-coverage-20260923/dynamic-request-local-scope-native-oracle-20260925.pine
cargo run -p pine-cli --locked --quiet -- run .local/community-coverage-20260923/dynamic-request-local-scope-native-oracle-20260925.pine --bars .local/community-coverage-20260923/btcusd-bars.csv --chart-symbol COINBASE:BTCUSD --chart-timeframe 1D --request-bars BINANCE:BTCUSDT:1D=.local/community-coverage-20260923/dynamic-swing-btcusdt-2017-2026-bars.csv > .local/community-coverage-20260923/dynamic-request-local-scope-local-20260925.json
python .local/community-coverage-20260923/compare_dynamic_request_local_scope_20260925.py
```

The temporary indicator was removed with TradingView's undo action. The saved layout's Coinbase daily chart, SSL Hybrid Strategy, and Percent short market probe remained present; the research tab was closed without saving the test layout or Pine draft. The export followed TradingView's [chart-data download workflow](https://www.tradingview.com/support/solutions/43000537255-how-to-export-chart-data/).

# TradingView `ta` v14 volume request audit (2026-09-25)

The saved, unmodified public `TradingView/ta/14` library now passes semantic analysis with zero diagnostics when a v6 script imports it and calls `requestUpAndDownVolume()` with a timeframe that alternates between `"1"` and `"5"` by bar. The original library also imports `TradingView/RelativeValue/3`. The first analysis had 31 diagnostics. The remaining nested `request.security("", timeframe.main_period, time)` call is now accepted only in a bounded dynamic request context. The empty symbol inherits the current requested symbol; `timeframe.main_period` retains the main chart period. The v6 default and v5 explicit `dynamic_requests=true` cases pass, while v5 without that option and v6 with `dynamic_requests=false` reject the request.

The unmodified library executes on synthetic 5-minute chart bars with a separate 1-minute intrabar provider. `requestUpAndDownVolume("1")` yields positive volume `[90, 160]`, negative volume `[-60, -240]`, and delta `[30, -80]`. A separate nested request test confirms that an outer request for a second symbol at 1 minute resolves the inner empty-symbol request against that second symbol at the main 5-minute period.

The genuine lower-timeframe check uses two exports from the same `COINBASE:BTCUSD` TradingView chart. The 5-minute export contains the native `TradingView/ta/14` three-plot result (`I:\sys\下载\COINBASE_BTCUSD, 5.csv`, SHA-256 `026F1905B2D898BE73544C14A1A9EFC2D232866B5690A2B35464D689049E64F4`); the 1-minute export supplies independent intrabar OHLCV (`I:\sys\下载\COINBASE_BTCUSD, 1.csv`, SHA-256 `8E894DB322E348365A228E7C3CBD2AC9A01EE0F143ADDA0D496AFC74DA054F27`). Copies and the preparation script are retained under `.local/community-coverage-20260923/`. Excluding forming bars and keeping only chart bars with all five corresponding confirmed minute bars leaves **59** complete chart bars, from Unix time `1790325300` through `1790342700`. Every five-minute volume equals the sum of its five minute volumes. Running the original library on those bars matches all **177** native plot values (59 bars × 3 plots) within absolute tolerance `1e-8` and relative tolerance `1e-10`. Both positive and negative branches are exercised.

For a native output check, TradingView compiled the public library on `COINBASE:BTCUSD` 1D and exported `Positive volume`, `Negative volume`, and `Volume delta`. The CSV in `I:\sys\下载\COINBASE_BTCUSD, 1D (41).csv` was copied to `.local/community-coverage-20260923/tradingview-ta-v14-up-down-native-1d-20260925.csv` (SHA-256 `52A50B5764448F9DA5E332049A59485E880275597C0C8176E1D63DC0C8179455`). The last, forming daily bar was excluded. With the 299 confirmed OHLCV rows from that same export, the local unmodified-library run matches all **897** native plot values (299 bars × 3 plots) within absolute tolerance `1e-9` and relative tolerance `1e-10`. An older saved OHLCV snapshot had five volume values that differed from the fresh export; those five bars account for all differences when that older snapshot is used.

The CLI no-provider path now constructs its request environment from the selected chart context. Previously, `--chart-timeframe 1D` changed `timeframe.period` but left `timeframe.main_period` at the default 1 minute, so the library incorrectly rejected `"1D"` as a lower-timeframe request. A CLI regression test covers the chart-period initialization.

This evidence covers native parity on 1D and on a real 5-minute chart backed by exported 1-minute data. It does not establish tick-chart parity. The public function's tick branch depends on 1T bid/ask data, which the current host-neutral request provider does not supply.

Reproduce analysis:

```powershell
cargo run -p pine-cli --locked -- analyze .local/community-coverage-20260923/tradingview-ta-v14-up-down-dynamic-probe-20260925.pine --library-source TradingView/ta/14=.local/community-coverage-20260923/tradingview-ta-v14.pine --library-source TradingView/RelativeValue/3=.local/delivery-20260909/RelativeValue-v3.pine --format json
```

Reproduce native 1D parity with the ignored local fixtures:

```powershell
cargo run -p pine-cli --locked -- run .local/community-coverage-20260923/ta-v14-up-down-native-1d-probe-20260925.pine --bars .local/community-coverage-20260923/ta-v14-up-down-native-1d-confirmed-bars-20260925.csv --chart-symbol COINBASE:BTCUSD --chart-timeframe 1D --library-source TradingView/ta/14=.local/community-coverage-20260923/tradingview-ta-v14.pine --library-source TradingView/RelativeValue/3=.local/delivery-20260909/RelativeValue-v3.pine
```

Reproduce the native lower-timeframe run after `python .local/community-coverage-20260923/prepare-ta-v14-native-ltf-20260925.py`:

```powershell
cargo run -p pine-cli --locked -- run .local/community-coverage-20260923/ta-v14-up-down-native-5m-probe-20260925.pine --bars .local/community-coverage-20260923/ta-v14-up-down-native-5m-confirmed-bars-20260925.csv --chart-symbol COINBASE:BTCUSD --chart-timeframe 5 --library-source TradingView/ta/14=.local/community-coverage-20260923/tradingview-ta-v14.pine --library-source TradingView/RelativeValue/3=.local/delivery-20260909/RelativeValue-v3.pine --request-bars COINBASE:BTCUSD:1=.local/community-coverage-20260923/ta-v14-up-down-native-1m-provider-20260925.csv
```

The `.local` source, bar, and output fixtures are retained research inputs and are ignored by Git. The regression tests are in `crates/pine-sema/src/tests/compatibility.rs`, `crates/pine-runtime/src/tests/request.rs`, and `crates/pine-cli/src/commands/run/tests.rs`. [TradingView's request context documentation](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/) describes nested request inheritance; [chart information documentation](https://www.tradingview.com/pine-script-docs/concepts/chart-information/) describes `timeframe.main_period`.

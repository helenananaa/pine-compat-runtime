# TASC Low-Risk ETF v6: complete-source execution boundary (2026-09-26)

The unchanged public [TASC 2026.10 A Low-Risk ETF Trading Strategy](https://www.tradingview.com/script/2S4BqQzQ-TASC-2026-10-A-Low-Risk-ETF-Trading-Strategy/) declares an **indicator**, despite its publication title. The original Pine v6 source in ignored `.local/continued-popular-20260926/tasc-low-risk-etf-v6.pine` has SHA-256 `dfd4758888a12e8fc0438b228c3dbec62e7d9bb37d1d9b15518d76c69bc9bbc7`. With the earlier optional-alert, line-style and display-mask changes, current `pine-compat analyze` returns `executable: true` and no diagnostics.

The default source requests `AMEX:SPY` at the chart timeframe via `request.security`. The host-neutral `requirements` contract identifies this as call site 11, with benchmark input at call site 10. No matching SPY export exists in `I:\sys\下载` for this batch. Running the default source on the `COINBASE:BTCUSD` chart without that provider fails explicitly with `missing request data for symbol AMEX:SPY timeframe 1D`; this is a missing host capability, not a request to make the runtime fetch market data.

For an execution-only control, input call site 10 was set to `COINBASE:BTCUSD` on 4,283 confirmed daily bars (`chart-bars.csv`, SHA-256 `0a5f4b9c7b04338bab534088e07c3b13c29bc32a1e992c831122566bcdfe3f13`). The full unchanged source then ran with **zero runtime diagnostics**. It produced the expected five plot, two plotshape and one fill records. `RSMK` has 4,245 nonblank values and `RS MA` has 4,196; entry and exit shapes are empty because a series compared against itself makes this particular relative-strength signal degenerate. Batch, incremental and historical-realtime runs produce byte-identical full JSON (SHA-256 `96cccfe1cd3ac7cdcb040e398c56c60eb47264cd76573a419e81a28dbc82ca04`). The three JSON outputs are retained in the ignored research directory.

This control verifies complete-script execution and consistency across historical execution modes for a supplied same-symbol request. It does **not** qualify the published default benchmark, trading signals, visible chart output against TradingView, or realtime forming ticks. Native qualification requires an exact SPY provider history plus a TradingView chart export under the same symbol, timeframe, inputs and confirmed-bar cutoff. Chrome extension communication still timed out while listing tabs, so no new native export was acquired in this pass.

Reproduce from the repository root:

```powershell
$p = '.local/continued-popular-20260926/'
target/debug/pine-compat.exe analyze "${p}tasc-low-risk-etf-v6.pine" --format json
target/debug/pine-compat.exe requirements "${p}tasc-low-risk-etf-v6.pine"
target/debug/pine-compat.exe run "${p}tasc-low-risk-etf-v6.pine" --bars "${p}chart-bars.csv" --chart-symbol COINBASE:BTCUSD --chart-timeframe 1D --input-override 10=COINBASE:BTCUSD
```

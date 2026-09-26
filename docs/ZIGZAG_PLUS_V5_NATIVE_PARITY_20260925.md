# ZigZag++ v5 compatibility check (2026-09-25)

The public [ZigZag++ script](https://www.tradingview.com/script/lj8djt1n-ZigZag/)
by DevLucem (Pine v5) imports the public
[DevLucem/ZigLib/1](https://www.tradingview.com/script/NC3cgEcT-ZigLib/).
Their exact published sources are retained as ignored local evidence in
`.local/community-coverage-20260923/devlucem-zigzag-plus-v5-20260925.pine`
and `devlucem-ziglib-v1-20260925.pine`. The imported library makes calls to
`chart.point.copy()` and uses `-ta.highestbars(depth)` and
`-ta.lowestbars(depth)` as history indexes.

The runtime now admits `chart.point.copy()` through method lookup and returns
non-positive offsets from `ta.highestbars()` and `ta.lowestbars()`. TradingView's
[v5 documentation](https://www.tradingview.com/pine-script-docs/v5/concepts/text-and-shapes/)
likewise negates `ta.highestbars()` before a history reference. Runtime tests
cover independent point copies and history indexing in v5 and v6. Existing
offset tests and golden snapshots were updated to the corrected sign.

The original script and its exact library analyze without diagnostics. They
complete a 4,282-bar `COINBASE:BTCUSD` 1D historical run using the retained
Coinbase chart data. A separate TradingView chart export for the original
script contains 300 bars and a `direction` column. Running the original source
on those same 300 OHLCV bars gives 296/300 identical direction values; all
differences are at bars 0–3 of the exported slice. TradingView calculated the
export with earlier chart history available, whereas the local slice begins at
bar 0. From bar 4 onward the direction series matches exactly. Evidence:
`devlucem-zigzag-native-export-v5-20260925.csv`,
`devlucem-zigzag-native-chart-bars-v5-20260925.csv`, and
`devlucem-zigzag-plus-local-v5-20260925.json` under the ignored local evidence
directory. The comparison covers the default inputs and historical direction
series, not pixel-level drawing parity, alternate inputs, or forming ticks.

The native export was downloaded via TradingView's chart-data export with
`COINBASE:BTCUSD` at 1D. The original 300-row export is 21,526 bytes. The
five-package locked test command and formatting check are recorded in the
companion local test log.

## Full-history follow-up

The original source and library were rerun on all 4,282 retained confirmed
Coinbase daily bars. The ignored full run is
`.local/community-coverage-20260923/devlucem-zigzag-plus-full-local-v5-20260925.json`.
`compare_devlucem_zigzag_full_history.py` aligns the native export by timestamp
against that full run and checks OHLC plus the `plotarrow` direction values.
All **299 confirmed overlapping bars** match, including the four bars that
previously differed when the export slice was run without earlier history.
The export's 300th row, September 25, 2026, was forming and is outside the
retained confirmed-bar input. The full run has zero diagnostics. This improves
historical direction parity; drawing geometry and forming-bar behavior remain
outside this comparison.

```powershell
cargo run -p pine-cli --locked -- run .local/community-coverage-20260923/devlucem-zigzag-plus-v5-20260925.pine --library-source DevLucem/ZigLib/1=.local/community-coverage-20260923/devlucem-ziglib-v1-20260925.pine --bars .local/community-coverage-20260923/lazybear-volume-flow-chart-bars-v1-20260925.csv --chart-symbol COINBASE:BTCUSD --chart-timeframe 1D > .local/community-coverage-20260923/devlucem-zigzag-plus-full-local-v5-20260925.json
python .local/community-coverage-20260923/compare_devlucem_zigzag_full_history.py
```

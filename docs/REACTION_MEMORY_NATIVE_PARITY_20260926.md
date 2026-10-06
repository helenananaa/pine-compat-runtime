# Reaction and memory indicators: native evidence, 2026-09-26

Two more complete public Pine v6 indicators execute unchanged, with 55,679
read-only state values agreeing with TradingView on 4,283 confirmed
`COINBASE:BTCUSD` daily bars. A loop-return repair enables Reaction Level Matrix.
This is local source qualification, not general Pine parity or a release receipt.

## Selection and sources

Chrome's Popular / Open-source listing supplied these five new candidates.
Boost counts were observed during collection and are not a permanent ranking.
Sources are retained in ignored `.local/continued-popular-20260926/`, with
author comments intact and only DOM-rendered nonbreaking spaces normalized.
No third-party source is added to the tracked regression fixtures.

| Candidate | Observed boosts | Current outcome |
| --- | ---: | --- |
| [Reaction Level Matrix, WillyAlgoTrader](https://www.tradingview.com/script/kcyXDIFe-Reaction-Level-Matrix-WillyAlgoTrader/) | 1,273 | Admitted after loop field-return repair; native state comparison passes |
| [Price Reaction Memory Heatmap, tradewsamet](https://www.tradingview.com/script/i750ODBF-Price-Reaction-Memory-Heatmap-tradewsamet/) | 254 | Original admitted; native state comparison passes |
| [TASC 2026.10 Low-Risk ETF Trading Strategy](https://www.tradingview.com/script/2S4BqQzQ-TASC-2026-10-A-Low-Risk-ETF-Trading-Strategy/) | 85 | Five admission diagnostics; despite its title, source declares `indicator` |
| [Fractional EMA Cross Backtest Strategy](https://www.tradingview.com/script/adstHm9N/) | 52 | 76 admission diagnostics involving table position/font qualifiers |
| [Ichimoku 5 Rules Backtest](https://www.tradingview.com/script/oG5a3MJU/) | 24 | Same 76 table position/font admission diagnostics |

Source SHA-256 identities, in table order:

```text
46167acca22f666753808c68b9fc335b7f92400f7905de8671a766e352b7592c
27cb505a08efd788456440c5a1b1f811afbf02c7c8d424a2078d2d3eda72d848
dfd4758888a12e8fc0438b228c3dbec62e7d9bb37d1d9b15518d76c69bc9bbc7
2fa96e0a312c5d48771430d1e7367483093f65fba6a5892d7229018a444ab477
846951d911fc034adacdd8cbec4fbc4bc15e6895b1ba8df822c489ea446c62fe
```

## Repaired semantics

The 1,192-line Reaction Level Matrix ends a function with an `if` and a `for`
whose final statement assigns a UDT field. Previously this incorrectly produced
`E_LOOP_RETURN`. Native probes confirm that final field assignments in `for`,
`for...in`, and `while` return the resulting field value, including compound
assignments. The analyzer and lowering now reuse the existing function field
return semantics, evaluating the mutation once and reading the resulting field.
No host capability or network integration was added to the core.

The original tracked probe checks all three loop forms and array element identity.
Additional regression cases cover Pine v5/v6, `break`, `continue`, persistent
mutation and an empty `while` returning `na`. Existing rejected non-value loop
tails remain covered by the full semantic suite.

## Independent comparisons

Native CSVs were downloaded through Chrome into `I:\sys\下载` and frozen locally.
The full window is December 1, 2014 through September 25, 2026; the forming
September 26 bar is excluded. The fresh bars file has SHA-256
`0a5f4b9c7b04338bab534088e07c3b13c29bc32a1e992c831122566bcdfe3f13`.
Three recent historical volume values differed from the preceding evidence batch,
so the complete native OHLCV export was refreshed and local runs repeated.
Comparisons require exact time/OHLC/volume input agreement and matching missing
values; numeric outputs use absolute and relative tolerances of `1e-9`.

| Case | Confirmed bars | Compared cells | Differences |
| --- | ---: | ---: | ---: |
| Loop field-return probe, five plots | 299 | 1,495 | 0 |
| Reaction Level Matrix, seven state plots | 4,283 | 29,981 | 0 |
| Price Reaction Memory Heatmap, six state plots | 4,283 | 25,698 | 0 |

Reaction compares direction, entry, stop, win/loss counts and long/short signals.
Heatmap compares retained zone count, memory-strength sum, reinforcement sum,
touch/invalidation signals and the next candidate ID. These indicators implement
their own state machines; these counts are not broker-emulator trade parity.
Appending the read-only plots preserves every original local JSON result field
after the added plots are removed. Geometry, colors, table layout, alert delivery,
other settings/symbols/timeframes and native realtime ticks remain unverified.

Downloads `(65)` and `(66)` contain the recent loop and Reaction windows; `(67)`
contains full Reaction history, `(68)` full Heatmap history, all with filename
prefix `COINBASE_BTCUSD, 1D`. Screenshots and comparisons are retained in the
ignored evidence directory. The temporary study and editor changes were restored
without saving or publishing a script/layout.

## Validation and next work

```powershell
cargo build -p pine-cli --locked
cargo test -p pine-sema -p pine-runtime --locked --quiet
cargo test -p pine-cli --locked --quiet
python .local/continued-popular-20260926/compare_overlap.py reaction-full-native.csv reaction-probe-fresh.json reaction-full-comparison.json
python .local/continued-popular-20260926/compare_overlap.py heatmap-native.csv heatmap-probe.json heatmap-comparison.json
python .local/continued-popular-20260926/freeze_evidence.py
cargo fmt --all --check
git diff --check
```

Semantic/runtime tests: 5,810 passed. CLI tests: 240 passed. The loop probe also
produces identical full JSON in batch, incremental and realtime-history modes;
this does not establish native forming-tick parity. Wheels/WASM were not rebuilt.

Next candidates are the two strategy table qualifier failures and TASC's
`plot(linestyle=...)`, display-mask subtraction and optional `alertcondition`
arguments. TASC also needs independently supplied benchmark-symbol data for
numeric validation. All three remain explicitly unqualified at this checkpoint.

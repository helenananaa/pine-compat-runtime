# Expanded indicator and strategy native comparison, 2026-09-26

Four more complete public Pine v6 scripts run unchanged on the local interpreter.
Five parameter scenarios pass the comparisons below on 4,283 confirmed
`COINBASE:BTCUSD` daily bars (2014-12-01 through 2026-09-25). The strategy scenarios
cover 334 closed trades and one open trade. This receipt does **not** establish
general equivalence with TradingView, all Pine versions, or a packaged release.

## Sources and acquisition

The current public Popular listings supplied the candidates; boost counts are
observations at selection, not a stable ranking. Chrome was used to read the
published source, execute native scripts and download TradingView chart/trade CSVs
to `I:\sys\下载\`. Complete original sources and market data remain ignored under
`.local/expanded-20260926/`; only original minimal regression controls are tracked.
The editor-rendered nonbreaking spaces were normalized to ordinary spaces.

| Script | Observed boosts | Source SHA-256 |
| --- | ---: | --- |
| [MACD Pullback Sniper, blitz_locked](https://www.tradingview.com/script/V87nRt9v-MACD-Pullback-Sniper-Trend-ADX-Filtered-Strategy/) | 285 | `bdfa426b7a52f3a5c60ba604250def2c8d5ecf89e27f2e38d37d3415bfbd8f4c` |
| [Harmonic Pulse, blitz_locked](https://www.tradingview.com/script/edChhkxo-Harmonic-Pulse-XABCD-Fibonacci-Pattern-Strategy/) | 12 | `8d08490004473db7d921700b6914c5c345a0fb8266652385358c92c1919cf37d` |
| [Volatility Adaptive Filtered Trend, SchizoQuant](https://www.tradingview.com/script/cQcESVhS-Volatility-Adaptive-Filtered-Trend-SchizoQuant/) | 501 | `2a07c9c16e0c882c6be84bcdfbae76217f4829d38a0bda5a5b6d0a4744fcf855` |
| [State-Dependent EMA, BackQuant](https://www.tradingview.com/script/jdVw4YmG-State-Dependent-EMA-BackQuant/) | 76 | `a4e71b3ab6b9e706eaf934588ea6997128231eab4abab283202234275c309ccc` |

The confirmed input `chart-bars.csv` has SHA-256
`872f06656ef147307ae754023b18a5ecb3aab474b851b76a4670f87766379eb1`.
Each comparator checks timestamps, OHLC and volume against the independent native
export before comparing script outputs. The final forming September 26 bar is
excluded. Host metadata is explicit: `1D`, price grid `1/100`, quantity precision
6, and unit point value. A native read-only metadata probe confirmed mincontract
`0.000001`, mintick `0.01` and pointvalue `1`. Missing instrument metadata must not
silently be interpreted as this particular instrument's configuration.

## Results

| Scenario | Compared chart cells | Closed / open trades | Result |
| --- | ---: | ---: | --- |
| MACD, default inputs | 34,264 | 28 / 0 | Pass |
| MACD, trend/zero-line/ADX filters disabled | 34,264 | 236 / 1 | Pass |
| Harmonic, default inputs | 25,690 | 70 / 0 | Pass |
| Volatility Adaptive, default inputs | 21,415 | — | Pass |
| State-Dependent EMA, default inputs | 38,547 | — | Pass |

These are 154,180 chart cells across five scenarios. MACD covers its six plots and
two boolean signals; Volatility Adaptive covers three plots and two signals. EMA
covers nine exported plots; six `display.none` plots are explicitly excluded.
Harmonic covers two pivot signals plus four appended read-only outputs: pattern
count, retained pivot count, position size and equity. Its declared `offset=-4`
is applied when aligning pivot signals; eight cells beyond the confirmed input
boundary are excluded, rather than treated as runtime failures.

Plot comparison requires matching missing values and uses absolute/relative
tolerances of `1e-9`. Native CSV boolean shapes are compared as 0/1. A separate
MACD sizing audit adds requested quantity, equity and position: all 47,113 cells
pass, including the original 34,264 cells. Requested quantities are exactly equal;
maximum equity difference is `5.73e-9` USD. Read-only instrumentation is checked
locally to preserve every original result field, allowing only appended plots.

Every closed trade is checked for entry/exit UTC date, direction, entry signal,
price, quantity, net PnL and commission. Tolerances are `1e-8` for price, `1e-9`
for quantity, and `0.00501` USD for the cent-displayed PnL/commission. All quantities
are exactly equal in the trade CSV comparison; maximum price error is `1.46e-11`.
Maximum PnL difference is `0.005002071` USD (Harmonic), so this is agreement within
the stated tolerance, not bitwise monetary equality. Commission is independently
reconstructed from entry/exit price, quantity and the scripts' 0.05% rate.

TradingView's Harmonic entry signal column displays the script's pattern comment,
not its Long/Short order ID. The comparison checks the same pattern name in the
local creation-bar label, as well as trade direction. It does not claim an
independent native order-ID check for that script. The one open MACD trade checks
its ID/direction, entry date, average price and quantity; its changing unrealized
PnL on the excluded forming bar is not compared.

## Repaired quantity semantics

Explicit `strategy.entry` and `strategy.order` quantities previously bypassed the
host-supplied minimum-contract grid. An initial binary multiply/floor repair
matched the default MACD trades but failed ten quantities in the expanded reversal
scenario by one minimum contract. Read-only native outputs showed identical raw
sizing expressions before the first discrepancy, isolating order admission.

Independent native controls confirm decimal truncation for the observed profile:

| Requested value | Native admitted quantity |
| --- | ---: |
| `0.1234569` | `0.123456` |
| `161253 * syminfo.mincontract` = `0.16125299999999998` | `0.161252` |
| literal `0.161253` | `0.161253` |
| `488565 * syminfo.mincontract` = `0.48856499999999997` | `0.488564` |
| literal `0.488565` | `0.488565` |
| `101250 * syminfo.mincontract` = `0.10124999999999999` | `0.101249` |
| literal `0.129515` | `0.129515` |
| one quarter of `syminfo.mincontract` | no order |

The implementation truncates the shortest decimal representation to the configured
precision. It does not first multiply by a binary scale, which can move either
side of a boundary. Invalid raw quantities are still rejected before truncation;
unconfigured profiles preserve existing behavior. Default-sized quantities are
not quantized again. No instrument discovery, market-data access or host dependency
was added to the core.

The first native control covers six cases for both entry/order (12 cycles), and
the second covers eleven cases for both (22 cycles). All 21,415 exported control
cells match exactly. Tracked tests cover long/short, market/limit/stop/stop-limit,
decimal boundaries, below-minimum suppression, invalid negative input, configured
precision endpoints and the unconfigured profile. Native evidence here covers
the decimal precision-6 market-order cases; other order types/precisions are local
regression coverage and must not be presented as separately native-qualified.

## Reproduction and validation

The ignored evidence directory retains original sources, read-only probes, frozen
CSVs, comparator scripts, full CLI JSON, logs and a SHA-256 manifest. Native chart
downloads `COINBASE_BTCUSD, 1D (57).csv` through `(64).csv` respectively contain
MACD/default metadata, Volatility Adaptive, EMA, first quantity control, Harmonic
audit, MACD/relaxed, MACD/sizing audit and second quantity control. The two MACD
trade exports have the `MACD_Pullback_Sniper___Trend_+_ADX_Filtered_Strategy` prefix
(default and `(1)`); Harmonic uses `Harmonic_(Simple)`, all dated 2026-09-26.

```powershell
cargo build -p pine-cli --locked
python .local/expanded-20260926/verify_final.py
python .local/expanded-20260926/run_candidate.py decimal-grid-probe.pine chart-bars.csv decimal-grid-fixed.json
python .local/expanded-20260926/compare_plot_csv.py decimal-grid-native.csv decimal-grid-fixed.json decimal-grid-comparison.json
cargo test -p pine-runtime -p pine-cli --locked --quiet
cargo fmt --all --check
git diff --check
```

All four original sources and the relaxed MACD scenario produce identical complete
JSON in batch, incremental and realtime-history execution: ten mode comparisons.
This is local mode agreement on historical data, not independent native live-tick
qualification. Runtime/CLI tests pass: 2,494 total, including 1,952 runtime library
and 240 CLI tests plus integration tests. No wheel/WASM artifact was rebuilt or
qualified. Temporary chart probes were removed and the original editor content
and visible date range restored; no cloud script or layout was saved/published.

Still outside this receipt: different symbols/timeframes, other input combinations,
native intrabar/repainting/realtime updates, drawing geometry/colors, hidden plots,
alert delivery, non-unit point values, and all-language/all-version equivalence.

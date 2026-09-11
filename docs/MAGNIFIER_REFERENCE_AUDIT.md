# Independent Magnifier path reference

Revision magnifier-60m-10m-r1, captured 2026-09-09. Code baseline remains
5920add7e68a51ea307c8dcbc5fc8c42cace22e9; no runtime changes were required.

## Matched inputs and frozen scenario

The [official interval table](https://www.tradingview.com/support/solutions/43000669285-what-is-bar-magnifier-backtesting-mode/)
maps a 60-minute chart to 10-minute intrabars. An original native-only acquisition
indicator uses request.security_lower_tf to export the six OHLCV/time records
for each chart hour. This is data acquisition, not an interpreter compatibility
claim for that function; the separately frozen executable strategy has no such
request. The core receives host-neutral MagnifierInputV1.

The predeclared window starts 2026-09-03 00:00 UTC and ends at chart opening time
1788922800000. All 148 chart bars have six complete, ordered intrabars: 888 total.
Parent open/close/high/low and aggregated volume match the independent lower
records. Native chart-time identities map to zero-based captured input rows.
The strategy uses absolute times and places no orders before the signal window.

Four contrasting paths were found from independent data before comparing runtime
trades. The first was frozen: signal at 1788487200000, entry stop 80944.6,
protective stop 80818.3, quantity 1. The next chart hour opens closer to its low;
the standard OHLC path therefore reaches the protective-stop area before entry,
whereas the captured lower data reaches entry and then the stop later in that
same hour. The paired sources differ only by title and use_bar_magnifier.
Both use initial capital 1,000,000, 100% margin and cash-per-order fee 1.25.

## Independent outcomes

| Case | Native entry time UTC | Native exit time UTC | Trades | Compared plot values |
| --- | --- | --- | ---: | ---: |
| long-on | 2026-09-04 03:00 | 2026-09-04 03:00 | 1 | 1,480 |
| long-off | 2026-09-04 03:00 | 2026-09-04 04:00 | 1 | 1,480 |

Both native trades fill at 80944.6 / 80818.3, charge 2.5 commission and realize
approximately -128.8. Native trade CSV identities, prices and quantities are
cross-checked against precise chart profit/commission/time fields. Trade indices
are normalized from chart timestamps; time, position and count plots compare
exactly. Other values retain absolute/relative 1e-9 tolerances. There are no
skipped warmup bars or mismatches. The on case confirms chart-hour public fill
timestamps even though the lower exit path occurs later within that hour.

CLI, the retained installed Windows wheel and actual generated WASM have equal
complete JSON for both cases. CLI incremental and realtime-history also match.
This is historical Magnifier coverage; live/forming consumption, short paths,
gap fallbacks, calc_on_order_fills sequencing and margin liquidation are not
qualified by these two cases. B1 private ordering remains unverified.

## Reproduction and artifacts

Under `.local/delivery-20260909/magnifier-reference/`: plan.json,
capture-inputs.pine, native-inputs-raw.csv, prepare_inputs.py,
chart-bars.csv, magnifier-bars.json, input-manifest.json, candidate-paths.json,
frozen-case.json, paired long-on/off source and native CSV/DOM captures,
compare_references.py, comparison-summary.json, and verify_hosts.py with
host-mode-comparison.json. The host verifier uses the existing retained
technical-5920add7e-hosts wheel/WASM and the mtf-reference Node helper.
Original inputs and prior captures remain intact. These Windows qualification
artifacts are not final Linux/release packaging or long-session acceptance.

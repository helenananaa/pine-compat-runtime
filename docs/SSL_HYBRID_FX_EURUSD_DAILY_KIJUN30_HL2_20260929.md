# SSL Hybrid Strategy: EURUSD daily Kijun v2 30, divider 1 and 3

Two independent native TradingView exports were captured through authenticated
Chrome on 2026-09-29. Both execute the unchanged public Pine v5
[SSL Hybrid Strategy by kevinmck100](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/).
Source SHA-256: `1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.

## Frozen setup and branch coverage

FXCM `FX:EURUSD`, daily (`1D`), USD, price grid `1/100000`, integer quantity
precision, point value 1. Common overrides: `12=Kijun v2`, `13=30`, `18=hl2`,
`20=3`, `21=2`. Only input `19=1` versus `19=3` changes. Original JMA SSL2
length 5 uses Phase 3 / Power 2, HMA exit length 15, other inputs unchanged.

The original Kijun v2 branch averages the 30-bar highest/lowest midpoint
with the `len / kidiv` highest/lowest midpoint. Divider 1 uses two 30-bar
windows; divider 3 uses 30- and 10-bar windows. These one-argument extrema
calls use chart low/high rather than the selected `src`; the frozen HL2
setting is not evidence of an HL2 effect on this baseline. Other script
paths retain their original source behavior. Complete exported outputs,
startup positions and original strategy lifecycles are qualified; internal
midpoints are not separately exported or qualified as arbitrary primitives.

Properties: USD 5,000 capital, 10% equity sizing, pyramiding 10, commission
0.04%, zero slippage, default four historical ticks, on bar close/realtime
tick, requested limit price, one-tick execution delay and infinite leverage.

Each native chart contains 14,327 rows. The runtime receives **14,326
confirmed bars**, 1971-01-03 22:00 UTC through 2026-09-27 21:00 UTC
(Sep 28 session). The Sep 29 forming session, Unix `1790629200`, is excluded.
Confirmed OHLCV CSVs are byte-identical, SHA-256
`6d76dc133c08bb0e3204b144524de551ce5198548172778bf3d518cbd059dc0d`.

Downloads in `I:\sys\下载`:

| Divider | Chart CSV | Trade CSV | Download times, UTC+8 |
| --- | --- | --- | --- |
| 1 | `FX_EURUSD, 1D (8).csv` | `SSL_Hybrid_Strategy_FX_EURUSD_2026-09-29 (8).csv` | 19:00:20 / 19:00:08 |
| 3 | `FX_EURUSD, 1D (9).csv` | `SSL_Hybrid_Strategy_FX_EURUSD_2026-09-29 (9).csv` | 19:02:05 / 19:01:46 |

Original source, native CSVs, complete input/property/export snapshots,
chart state/screenshots, confirmed bars and immutable CLI/core receipts
are frozen per setting. The unrelated Percent short market probe's `Plot`
column is excluded from SSL Hybrid comparisons.

## Confirmed native comparison

| Observation | Divider 1 | Divider 3 |
| --- | ---: | ---: |
| Confirmed chart bars | 14,326 | 14,326 |
| Nonblank exported observations | 100,122 | 100,122 |
| Series mismatches, including NA positions | 0 | 0 |
| Closed trades | 494 | 854 |
| Explicit exit fills | 211 | 226 |
| Surviving open entries | 1 | 1 |
| Final confirmed position | -43 at 1.15953 | -42 at 1.15953 |
| Maximum displayed net PnL difference | 0.004996108 | 0.004994344 |
| Maximum displayed commission difference | 0.002231400 | 0.003248900 |

All eight selected columns match: Candle Size > 1xATR, MA Baseline, SSL1,
Baseline Upper Channel, Basiline Lower Channel, MA UP, MA DOWN and the
disabled 2nd Multi-TimeFrame Moving Average. Absolute numeric tolerance is
`1e-8`, with every missing position checked. Candle Size false/NA exports as
zero; the disabled second MA is entirely missing.

Baseline and SSL1 are missing for their first **29 bars**, and first defined
at zero-based bar **29**, in both settings. Channels begin at bar 30,
MA UP/DOWN at bar 21. All startup values remain in the comparison.

Closed trades match IDs, directions, UTC+8 session dates, prices, exact
quantities, entry values, durations and displayed net PnL/commission.
Monetary display tolerance is 0.005. Explicit exit multisets match IDs,
dates, prices and quantities. Both native open records are `ShortEntry5`,
entered Sep 14, 2026 at 1.15953, quantities 43/42. Entry fields and final
aggregate positions match locally. Daily exports provide session dates,
without a finer intraday timestamp claim. All executed transactions precede
the forming session; current-day indicators, unrealized PnL and live ticks
remain outside scope.

## Independent divider sensitivity

Changing divider 1 to 3 on identical confirmed OHLCV changes baseline and
SSL1 at **13,976 of 14,297** mutually defined positions by more than `1e-8`.
Each channel changes at **13,975 of 14,296** mutually defined positions.
Closed trades change from 494 to 854; surviving short quantity changes
from 43 to 42. Both settings have independent native qualification but
share one market history. Complete native input snapshots are compared,
normalizing the divider value and transient focus/spin-button UI only;
all remaining input values and checkbox states must agree.

## Execution and reproduction

All six executions exit zero without stderr or diagnostics. Batch,
incremental and historical realtime complete outputs are byte-identical
within each setting:

| Divider | Complete output SHA-256 |
| --- | --- |
| 1 | `f06c8f6ce9a66bde339ddd0527e26caa551553544eccbde907546a0dc4f82ed9` |
| 3 | `4703b12884003640b1a0cfc043f656ce6458c2514c2ed8febd63ade0271d73cc` |

Core base `2f9a83a0c333092167c7253571f56ee82a7ec0d0`, core patch
`827e195c3a7db4422bbd657a5634990adb9d8d1db69658f6331df41a97cb2636`,
immutable CLI `584ae27bb25fed15c038c5c55a9356720c1bff90b8db29ce8b403e8a08010f36`.
No new core repair. Current source/golden hashes match the passing VAMA
full gate, reused without another full-gate run.

Evidence directories:

- `.local/ssl-hybrid-fx-eurusd-daily-kijun30-divider1-hl2-20260929/`
- `.local/ssl-hybrid-fx-eurusd-daily-kijun30-divider3-hl2-20260929/`

Prepare with `.local/prepare_eurusd_daily_kijun_20260929.py 1` or `3`.
Run each directory's `run_modes.py` and `compare.py`, then
`.local/verify_eurusd_daily_kijun_20260929.py`. The verifier checks current
HEAD/core/golden/CLI/gate identity, original source/download hashes, exact
OHLCV conversion, startup, fresh source analysis, native settings, mode
arguments/full hashes, fresh comparisons, native cutoff positions,
forming-session exclusion, input isolation, sensitivity and restored browser
state. Per-case evidence includes `analysis.json`, mode receipts,
`native-comparison.log`, `comparison.json` and `current-verification.json`.
Shared log: `.local/eurusd-daily-kijun-verification-20260929.log`.

| Divider | Native chart SHA-256 | Native trades SHA-256 |
| --- | --- | --- |
| 1 | `1dffc12723356605b320eeed660146f45a81eb5e40eceba67859c33ddced7373` | `e263e37f9cb51ee1515c1facef0efc63b85715b53e8544073fc5c48015b6a9ea` |
| 3 | `0d26cfa57ad39ceb12c31b918dc7a6c4e5812297d842199b82b5792835b0d7a3` | `d7fdc66b1bdf9f4d15ee5ef9c2acc5fb448336d4a9f08f50b8857ea1dd0c026c` |

Chrome was restored to Coinbase BTCUSD daily, HMA 60 close, Kijun divider 1,
JMA Phase 3 / Power 1, with complete restored input, state and screenshot
evidence. Qualification is specific to this source, frozen inputs, confirmed
history and exported fields. Historical realtime equality does not establish
native live-tick parity or arbitrary-script compatibility.

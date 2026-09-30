# SSL Hybrid Strategy: EURUSD daily MF 30 HL2, feedback off and on

Two independent native TradingView exports were captured through authenticated
Chrome on 2026-09-29. Both use the unchanged public Pine v5
[SSL Hybrid Strategy by kevinmck100](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/).
Source SHA-256: `1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.

## Frozen setup

FXCM `FX:EURUSD`, daily (`1D`), USD, price grid `1/100000`, integer quantity
precision, point value 1. Overrides: `12=MF`, `13=30`, `18=hl2`, `20=3`,
`21=2`, `23=0.8`, `24=false` or `24=true`, `25=0.5`. Only the Modular Filter
feedback checkbox changes between captures. JMA SSL2 length 5 uses Phase 3 /
Power 2; the HMA exit length remains 15. Other original inputs are frozen.

MF exercises the original recursive `ts`, upper/lower `b`/`c`, and `os`
selection state, its numeric equality branches, and the separate high,
low and selected-source calls. Feedback adds the previous filter output
to the selected input with weight 0.5. Qualification covers the complete
script's exported results; these internal states are not independently
exported or qualified as arbitrary primitive behavior.

Properties: USD 5,000 capital, 10% equity sizing, pyramiding 10, commission
0.04%, zero slippage, default four historical ticks, on bar close/realtime
tick, requested limit price, one-tick execution delay and infinite leverage.

Each native chart has 14,327 rows. The runtime receives the same **14,326
confirmed bars**, 1971-01-03 22:00 UTC through 2026-09-27 21:00 UTC
(Sep 28 session). The forming Sep 29 session, Unix `1790629200`, is excluded.
Confirmed OHLCV CSVs are byte-identical, SHA-256
`6d76dc133c08bb0e3204b144524de551ce5198548172778bf3d518cbd059dc0d`.

Downloads in `I:\sys\下载`:

| Feedback | Chart CSV | Trade CSV |
| --- | --- | --- |
| Off | `FX_EURUSD, 1D (4).csv` | `SSL_Hybrid_Strategy_FX_EURUSD_2026-09-29 (4).csv` |
| On | `FX_EURUSD, 1D (5).csv` | `SSL_Hybrid_Strategy_FX_EURUSD_2026-09-29 (5).csv` |

Original source, native CSVs, full input/property snapshots, chart/export
state, screenshots, confirmed bars and immutable CLI/core receipts are
frozen per case. The unrelated Percent short market probe `Plot` column
is excluded from SSL Hybrid comparisons.

## Native comparison

| Observation | Feedback off | Feedback on |
| --- | ---: | ---: |
| Confirmed chart bars | 14,326 | 14,326 |
| Nonblank exported observations | 100,180 | 100,180 |
| Series mismatches, including NA positions | 0 | 0 |
| Closed trades | 654 | 454 |
| Explicit exit fills | 263 | 271 |
| Surviving open entries | 1 | 1 |
| Final confirmed position | -43 at 1.15778 | -43 at 1.15778 |
| Maximum displayed net PnL difference | 0.004998624 | 0.004967280 |
| Maximum displayed commission difference | 0.003317480 | 0.002186400 |

All eight selected columns match: Candle Size > 1xATR, MA Baseline, SSL1,
Baseline Upper Channel, Basiline Lower Channel, MA UP, MA DOWN and the
disabled 2nd Multi-TimeFrame Moving Average. Absolute numeric tolerance is
`1e-8`; all missing positions are checked. Candle Size false/NA exports as
zero; the disabled second MA is entirely missing.

Baseline and SSL1 start at zero-based bar **0**, with native initial value
**0.5368** in both settings. Channels first appear at bar 30; MA UP/DOWN at
bar 21. Startup values and NA positions are compared without trimming.

Closed trades match IDs, directions, UTC+8 session dates, prices, exact
quantities, entry values, durations and displayed net PnL/commission.
Display tolerance remains 0.005. Explicit exit multisets match IDs, dates,
prices and quantities. Each surviving entry is `ShortEntry5`, Aug 31, 2026,
quantity 43 at 1.15778; its native entry and aggregate position match locally.
Daily exports provide session dates, without a finer intraday timestamp claim.
No executed transaction belongs to the excluded forming day. Current-day
indicator values, unrealized PnL and live ticks remain outside scope.

## Independent feedback sensitivity

On identical confirmed OHLCV, enabling feedback changes baseline and SSL1
at **14,325 of 14,326** mutually defined positions by more than `1e-8`.
Both channels change at all **14,296** mutually defined positions.
The shared first baseline value stays 0.5368. Closed trades change from
654 to 454, with explicit exits 263 to 271. Both results have independent
native qualification but share one market history.

## Execution and reproducibility

Batch, incremental and historical realtime executions have identical complete
output bytes within each setting, with zero exit status, stderr or diagnostics:

| Feedback | Complete output SHA-256 |
| --- | --- |
| Off | `b9b8f9363d2bfe8c1d293c78866f3626299eb144da582f284aa30058bc853b3b` |
| On | `f3c21b1994f21993ecb4b87710dbd8796a619d9b85e151c7f747ab5010c285c3` |

Core base `2f9a83a0c333092167c7253571f56ee82a7ec0d0`, core patch
`827e195c3a7db4422bbd657a5634990adb9d8d1db69658f6331df41a97cb2636`,
immutable CLI `584ae27bb25fed15c038c5c55a9356720c1bff90b8db29ce8b403e8a08010f36`.
No new core repair. Current HEAD/core/golden hashes match the passing VAMA
full gate; that unchanged-source gate is reused, without a new full-gate run.

Evidence directories:

- `.local/ssl-hybrid-fx-eurusd-daily-mf30-feedback-off-hl2-20260929/`
- `.local/ssl-hybrid-fx-eurusd-daily-mf30-feedback-on-hl2-20260929/`

Prepare with `.local/prepare_eurusd_daily_mf_20260929.py off` or `on`.
Run each case's `run_modes.py` and `compare.py`, then
`.local/verify_eurusd_daily_mf_20260929.py`. The verifier checks current
source/golden/CLI/gate identity, original download hashes, exact OHLCV
conversion, native startup, fresh source analysis, native settings,
mode arguments/full output hashes, fresh comparisons, boundary positions,
forming-day exclusion, parameter sensitivity and restored browser state.
Each case retains `analysis.json`, mode receipts, `native-comparison.log`,
`comparison.json` and `current-verification.json`. Shared log:
`.local/eurusd-daily-mf-verification-20260929.log`.

| Feedback | Native chart SHA-256 | Native trades SHA-256 |
| --- | --- | --- |
| Off | `b37d5fcfe6f82911d954202cb70e81f9199eac7bebd2a6e49ff913c0e9b02bb1` | `16dbe48e9e9360b1ce48f846c5320bd44d7998f5aea823b4c3b46ef7efbae598` |
| On | `2c8d0e392f59be1eed4eb076d381535df2e7cc020aee62288e0c8325f66bbbc2` | `6aae3e72eafba0ba28435fc1212bdf34ffe092f49ce86bbacfd53ff3ab281475` |

Chrome was restored to Coinbase BTCUSD daily, HMA 60 close, JMA Phase 3 /
Power 1, feedback off, beta 0.8 and weight 0.5, with screenshot/state evidence.
Qualification is specific to this source, frozen inputs, history and exported
fields. Historical realtime equality does not establish native live-tick
parity or arbitrary-script compatibility.

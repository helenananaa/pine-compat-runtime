# SSL Hybrid Strategy: EURUSD daily MF beta and feedback weight

Two fresh, independent TradingView exports were captured through authenticated
Chrome on 2026-09-29. They execute the unchanged public Pine v5
[SSL Hybrid Strategy by kevinmck100](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/).
Source SHA-256: `1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.

## Frozen inputs and official data

FXCM `FX:EURUSD`, daily (`1D`), USD, price grid `1/100000`, integer quantity
precision, point value 1. Baseline MF length 30, source HL2, feedback on,
JMA SSL2 length 5 / Phase 3 / Power 2, HMA exit length 15. All original
other inputs remain frozen. Relative to the independently qualified
[feedback-on control](SSL_HYBRID_FX_EURUSD_DAILY_MF30_HL2_20260929.md),
each new capture changes one parameter:

| Setting | Beta, input 23 | Feedback, input 24 | Weight, input 25 |
| --- | ---: | --- | ---: |
| Prior native control | 0.8 | true | 0.5 |
| New beta setting | 0.2 | true | 0.5 |
| New weight setting | 0.8 | true | 0.8 |

Common overrides: `12=MF`, `13=30`, `18=hl2`, `20=3`, `21=2`, `24=true`.
MF's beta blends recursive upper/lower states; feedback weight blends the
current source and previous `ts`. The complete script also exercises separate
high, low and selected-source function states and its five entry/target/stop
lifecycles. These internal values are not separately exported or qualified
as arbitrary primitive behavior.

Native properties are independently frozen: USD 5,000 capital, 10% equity
sizing, pyramiding 10, commission 0.04%, zero slippage, default four historical
ticks, on bar close/realtime tick, requested limit price, one-tick execution
delay and infinite long/short leverage.

Both charts contain 14,327 rows. The runtime receives **14,326 confirmed
bars**, 1971-01-03 22:00 UTC through 2026-09-27 21:00 UTC (Sep 28 session).
The Sep 29 forming session, Unix `1790629200`, is excluded. Confirmed
OHLCV is byte-identical between both settings and the prior control,
SHA-256 `6d76dc133c08bb0e3204b144524de551ce5198548172778bf3d518cbd059dc0d`.

Native downloads in `I:\sys\下载`:

| Setting | Chart CSV | Trade CSV | Download times, UTC+8 |
| --- | --- | --- | --- |
| Beta 0.2 | `FX_EURUSD, 1D (6).csv` | `SSL_Hybrid_Strategy_FX_EURUSD_2026-09-29 (6).csv` | 18:49:25 / 18:49:13 |
| Weight 0.8 | `FX_EURUSD, 1D (7).csv` | `SSL_Hybrid_Strategy_FX_EURUSD_2026-09-29 (7).csv` | 18:50:58 / 18:50:40 |

Full original source, chart/trade CSVs, input/property/export snapshots,
completed chart state, screenshots, confirmed bars and immutable CLI/core
receipts are frozen per setting. The unrelated Percent short market probe's
`Plot` column is excluded from SSL Hybrid comparisons.

## Confirmed native comparison

| Observation | Beta 0.2 / weight 0.5 | Beta 0.8 / weight 0.8 |
| --- | ---: | ---: |
| Confirmed bars | 14,326 | 14,326 |
| Nonblank exported observations | 100,180 | 100,180 |
| Series mismatches, including NA positions | 0 | 0 |
| Raw native closed trades | 188 | 564 |
| Confirmed closed trades | 187 | 564 |
| Confirmed explicit exit fills | 156 | 276 |
| Raw native open entries | 2 | 1 |
| Confirmed surviving entries | 3 | 1 |
| Final confirmed position | -217 at 1.14645 | -43 at 1.15778 |
| Forming-day exits excluded | 1 | 0 |
| Maximum displayed net PnL difference | 0.004967280 | 0.004991632 |
| Maximum displayed commission difference | 0.002293436 | 0.002797464 |

All eight selected columns match: Candle Size > 1xATR, MA Baseline, SSL1,
Baseline Upper Channel, Basiline Lower Channel, MA UP, MA DOWN and the
disabled 2nd Multi-TimeFrame Moving Average. Numeric absolute tolerance
remains `1e-8`, with every missing position checked. Candle Size false/NA
exports as zero; the disabled second MA is entirely missing.

Baseline/SSL1 start at zero-based bar **0** with value **0.5368**.
Channels start at bar 30 and MA UP/DOWN at bar 21. Startup is not trimmed.
Closed trades match IDs, directions, UTC+8 session dates, prices, quantities,
entry values, durations and displayed net PnL/commission. Display tolerance
remains 0.005, with zero quantity difference. Explicit exit multisets match
IDs, dates, prices and quantities. Daily exports provide session dates,
without a finer intraday timestamp claim.

For beta 0.2, native trade 188 closes `ShortEntry3` on the forming Sep 29
session with `ShortExit3`, quantity 131 at 1.13357. This exit is excluded.
Its untouched native entry row establishes the Sep 17 confirmed entry,
quantity 131 at 1.14645. Together with genuinely open `ShortEntry4` and
`ShortEntry5`, quantities 43 each at the same price/date, it establishes the
confirmed final short of **217**. All three entry fields and the aggregate
position match locally. Native records remain complete in the frozen CSV.

For weight 0.8, no executed transaction belongs to the forming session.
Its surviving `ShortEntry5` is Aug 31, quantity 43 at 1.15778. Current-day
indicator values, unrealized PnL and live ticks remain outside scope.

## Independent parameter sensitivity

Each new setting is compared with the prior independently qualified
beta 0.8 / weight 0.5 feedback-on control. Its current source, original
native downloads, mode receipts and native comparison are reverified.
The complete native input snapshots, normalized only for the named
parameter changes, focus markers and transient Increase/Decrease spin
buttons, agree on all remaining inputs. Input values and checkbox states
remain in the comparison.

Each parameter change affects baseline and SSL1 at **14,325 of 14,326**
mutually defined positions by more than `1e-8`. Each channel changes at all
**14,296** mutually defined positions. The shared first value is unchanged.
Confirmed closed trades change from control 454 to 187 for beta 0.2,
and to 564 for weight 0.8. These qualify parameter effects on one fixed
market history, without claiming independent histories or optimal settings.

## Execution and reproduction

All six new executions exit zero without stderr or diagnostics. Complete
batch, incremental and historical realtime outputs are byte-identical
within each setting:

| Setting | Complete output SHA-256 |
| --- | --- |
| Beta 0.2 | `93c0293c87caed8abb9cd2d23c45a672b5b1ea7f9a9298abd17e701627964682` |
| Weight 0.8 | `f41b6986713b489f49a44eaceab87cd5182964b95d5cc425ce651c85f3a536dc` |

Core base `2f9a83a0c333092167c7253571f56ee82a7ec0d0`, core patch
`827e195c3a7db4422bbd657a5634990adb9d8d1db69658f6331df41a97cb2636`,
immutable CLI `584ae27bb25fed15c038c5c55a9356720c1bff90b8db29ce8b403e8a08010f36`.
No new core repair. Current source/golden hashes match the passing VAMA
full gate, reused without a new full-gate run.

Evidence directories:

- `.local/ssl-hybrid-fx-eurusd-daily-mf30-beta02-feedback-on-hl2-20260929/`
- `.local/ssl-hybrid-fx-eurusd-daily-mf30-weight08-feedback-on-hl2-20260929/`

Prepare with `.local/prepare_eurusd_daily_mf_parameters_20260929.py beta02`
or `weight08`. Run each directory's `run_modes.py` and `compare.py`, then
`.local/verify_eurusd_daily_mf_parameters_20260929.py`. The verifier checks
current HEAD/core/golden/CLI/gate identity, original downloads, exact OHLCV
conversion, startup, fresh original-source analysis, native settings,
mode arguments and full hashes, fresh comparisons, forming-day exclusion,
native cutoff positions, the independently qualified control and restored
browser state. Per-case evidence includes `analysis.json`, mode receipts,
`native-comparison.log`, `comparison.json` and `current-verification.json`.
Shared logs:

- `.local/eurusd-daily-mf-parameters-verification-20260929.log`
- `.local/eurusd-daily-mf-control-reverification-20260929.log`

| Setting | Native chart SHA-256 | Native trades SHA-256 |
| --- | --- | --- |
| Beta 0.2 | `68014d03f36f1d21b5a75ced83ede962f396835fcf05811966c7c290afdcfd56` | `8e00d5c58abb952c9e51a3980c33ff49cd74e75f3a9ea5cf705bd045c7495ece` |
| Weight 0.8 | `5758007a9f522c4cc7dc7670c0042cfbfa2f6ca647cfbe40a91283f62016c8ae` | `e927aab543bc89758fc0375c7137fcbd9a3394980a929b09ea844e5f176b0437` |

Chrome was restored to Coinbase BTCUSD daily, HMA 60 close, JMA Phase 3 /
Power 1, feedback off, beta 0.8 and weight 0.5, with full input snapshot,
state and screenshot evidence. Qualification remains specific to the
source, inputs, confirmed history and exported fields. Historical realtime
equality does not establish native live-tick or arbitrary-script parity.

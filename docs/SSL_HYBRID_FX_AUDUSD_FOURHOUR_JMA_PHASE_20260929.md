# SSL Hybrid Strategy: AUDUSD four-hour JMA Phase -101 and 101

Two new independent TradingView captures were obtained through authenticated
Chrome on 2026-09-29. Both use the unchanged public Pine v5
[SSL Hybrid Strategy by kevinmck100](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/).
Source SHA-256: `1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.

## Setup and parameter coverage

FXCM `FX:AUDUSD`, four-hour (`240`), USD, price grid `1/100000`, integer
quantity precision, point value 1. Overrides are `12=JMA`, `13=30`,
`18=hl2`, `21=2`, and `20=-101` or `20=101`. All other original inputs
remain unchanged, including JMA SSL2 length 5 and HMA exit length 15.

The original JMA function sets its phase ratio to 0.5 below -100, to 2.5
above 100, and otherwise computes `phase / 100 + 1.5`. These captures
qualify both outer branches in the complete strategy. The previous
[Phase 3, Power 1/2 cases](SSL_HYBRID_FX_AUDUSD_FOURHOUR_JMA30_HL2_20260929.md)
cover an interior Phase setting. Phase also affects SSL2's separate JMA
function calls. This report does not qualify every Phase value or exact
-100/100 endpoints.

Properties are independently frozen per capture: USD 5,000 initial capital,
10% equity sizing, pyramiding 10, commission 0.04%, zero slippage,
default four historical ticks, on bar close/realtime tick, requested limit
price, one-tick order delay, infinite long/short leverage.

Each native chart has 21,347 rows. The runtime receives **21,346 confirmed
bars**, from 2013-01-02 02:00 UTC to 2026-09-29 05:00 UTC. The forming bar
at 2026-09-29 09:00 UTC (17:00 UTC+8), Unix `1790672400`, is excluded.
The native capture crossed a four-hour boundary after the preceding JMA
captures; this report includes that additional confirmed bar explicitly.
Both confirmed OHLCV inputs are identical, SHA-256
`6bcb42c991e3c6281d06b37dde15a8b3bb954dfb4a62ba37492fbcae2961fa71`.

Native downloads in `I:\sys\下载`:

| Phase | Chart CSV | Trade CSV | Download times, UTC+8 |
| --- | --- | --- | --- |
| -101 | `FX_AUDUSD, 240 (7).csv` | `SSL_Hybrid_Strategy_FX_AUDUSD_2026-09-29 (10).csv` | 17:00:54 / 17:00:36 |
| 101 | `FX_AUDUSD, 240 (8).csv` | `SSL_Hybrid_Strategy_FX_AUDUSD_2026-09-29 (11).csv` | 17:03:45 / 17:03:25 |

Original native CSVs, full source, input/property snapshots, chart state,
export settings, screenshots, confirmed OHLCV and CLI/source receipts are
frozen separately for each case. The unrelated saved Percent short market
probe's `Plot` column is excluded from SSL Hybrid comparison.

## Confirmed native results

| Observation | Phase -101 | Phase 101 |
| --- | ---: | ---: |
| Confirmed chart bars | 21,346 | 21,346 |
| Nonblank exported observations | 149,320 | 149,320 |
| Series mismatches, including missing positions | 0 | 0 |
| Confirmed closed trades | 2,841 | 3,455 |
| Explicit exit fills | 1,831 | 2,085 |
| Surviving entry records at cutoff | 0 | 0 |
| Position at confirmed cutoff | Flat | Flat |
| Maximum displayed net PnL difference | 0.004997072 | 0.004998472 |
| Maximum displayed commission difference | 0.004999360 | 0.004992660 |

All eight selected native columns are checked: Candle Size > 1xATR,
MA Baseline, SSL1, Baseline Upper Channel, Basiline Lower Channel, MA UP,
MA DOWN and the disabled 2nd Multi-TimeFrame Moving Average. Absolute
numeric tolerance is `1e-8`, with missing positions checked separately.
Candle Size false/NA is exported as zero; the disabled second MA is entirely
missing and its enabled behavior remains outside this receipt.

Baseline/SSL1 are defined from zero-based bar **0**, channels from **30**,
MA UP/DOWN from **21**. Native and local startup match without discarding
the original recursive JMA zero-initialization transient.

Closed trade entry IDs, entry/exit directions, UTC+8 minute timestamps,
prices, quantities, entry values, durations and displayed PnL/commission
match. Monetary display tolerance remains 0.005; quantities have zero
difference. Explicit exit order multisets match IDs, times, prices and
quantities. The confirmed boundary contains no surviving entry records;
both final local positions have size zero and no average price.

### Forming-bar entries

Phase -101's native trade file contains 2,841 closed trades and no open
entries. Every transaction precedes the forming bar.

Phase 101's native file contains 3,455 closed trades plus five current short
entries, `ShortEntry1` through `ShortEntry5`, filled at **17:00 UTC+8**, price
**0.69909**. These five entries occur on the excluded forming bar. They are
preserved in the original CSV and explicitly excluded; they are not described
as confirmed open positions. The local confirmed result is flat. This
receipt does not compare the current short position or its unrealized PnL.

## Independent Phase sensitivity

On identical confirmed OHLCV, changing Phase -101 to 101 changes all
**21,346** mutually defined baseline and SSL1 observations by more than
`1e-8`. Each channel changes at **21,315 of 21,316** mutually defined
positions. Confirmed closed trades change from 2,841 to 3,455. Both results
have independent native captures. The histories are shared; they are two
parameter cases on one dataset.

## Execution, source identity and reproduction

All six executions exit zero without stderr or diagnostics. Complete batch,
incremental and historical realtime outputs are byte-identical per setting:

| Phase | Complete output SHA-256 |
| --- | --- |
| -101 | `c6224ad3f11ebdb6c260bec7e720a99cb865fe1f6c68a1ebda672686ecc5f2ac` |
| 101 | `8159422669a9d1f88adfc911a79250e09395988ebd53c566f178f8c5e0ebb03f` |

Current core base `2f9a83a0c333092167c7253571f56ee82a7ec0d0`, patch
`827e195c3a7db4422bbd657a5634990adb9d8d1db69658f6331df41a97cb2636`,
immutable CLI `584ae27bb25fed15c038c5c55a9356720c1bff90b8db29ce8b403e8a08010f36`.
No runtime core change was needed. HEAD, core and golden file hashes match
the passing VAMA gate (1,984 runtime tests, 242 CLI tests, workspace
clippy/fmt, WASM Node, host parity, 130 tool tests and 774 wheel tests).
The identical-source gate is reused, without another full-gate run.

Evidence directories:

- `.local/ssl-hybrid-fx-audusd-fourhour-jma30-phase-minus101-power2-hl2-20260929/`
- `.local/ssl-hybrid-fx-audusd-fourhour-jma30-phase101-power2-hl2-20260929/`

Preparation: `.local/prepare_jma_phase_20260929.py PHASE CHART_FILENAME TRADE_FILENAME`.
Run each case's `run_modes.py` and `compare.py`, then
`.local/verify_jma_phase_20260929.py`. The verifier checks current source,
CLI and gate identity, original native download hashes, complete OHLCV
translation, startup, input call sites from fresh original-source analysis,
mode arguments/output hashes, fresh native comparisons, final flat
positions, forming-bar exclusions, Phase sensitivity and restored browser
state. Each directory retains mode receipts, `analysis.json`,
`comparison.json`, `native-comparison.log` and `current-verification.json`.
The shared verification log is `.local/jma-phase-verification-20260929.log`.

| Phase | Native chart SHA-256 | Native trades SHA-256 |
| --- | --- | --- |
| -101 | `cd110cef55764c734e57264d77cb1548113b104afee8b59deccdb16bb6f8938b` | `ab621a894b065fb50020deffc810bfb9415e4bbb9277f7f0f5cbd0d76e25aba8` |
| 101 | `b2ee1af4b01627669047a31bfb3d11e28ff662b0f394bf35efa5b0ddbf9f3669` | `58731309ac5e316f7135d0251daeb14741286a1b365b690bd889ddbb38c4e2b6` |

Chrome was restored to Coinbase BTCUSD daily, HMA 60 close, Phase 3,
Power 1 and volatility lookback 10, with screenshot/state evidence.
Qualification is specific to this unchanged source, settings, confirmed
history and exported fields. Historical realtime equality does not prove
native live-tick parity or arbitrary-script compatibility.

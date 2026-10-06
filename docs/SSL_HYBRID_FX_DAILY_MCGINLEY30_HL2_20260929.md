# SSL Hybrid Strategy: GBPUSD and EURUSD daily McGinley 30 HL2

Two independent native TradingView captures were obtained through authenticated
Chrome on 2026-09-29. Both execute the unchanged public Pine v5
[SSL Hybrid Strategy by kevinmck100](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/).
Source SHA-256: `1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.

## Frozen setup and semantics covered

FXCM `FX:GBPUSD` and `FX:EURUSD`, daily (`1D`), USD, price grid `1/100000`,
integer quantity precision, point value 1. Overrides: `12=McGinley`, `13=30`,
`18=hl2`, `20=3`, `21=2`. The last two inputs affect the original JMA SSL2
calls, whose length remains 5; they are not McGinley parameters. HMA exit
length 15 and all other original strategy inputs remain unchanged.

The original McGinley user function initializes from `ta.ema(src, len)` when
its prior state is missing, then recursively applies a fourth-power denominator.
The complete script exercises separate high, low and selected-source call
states, followed by its original five entry/target/stop order lifecycles.
No standalone replacement algorithm is substituted.

Native properties are independently frozen per symbol: USD 5,000 capital,
10% equity sizing, pyramiding 10, commission 0.04%, zero slippage,
default four historical ticks, on bar close/realtime tick, requested limit
price, one-tick order delay and infinite long/short leverage.

Both histories start at 1971-01-03 22:00 UTC and end with the confirmed bar
at 2026-09-27 21:00 UTC (Sep 28 session). The forming day at
2026-09-28 21:00 UTC (Sep 29 session), Unix `1790629200`, is excluded.
GBPUSD has 14,322 native rows and **14,321 confirmed bars**; EURUSD has
14,327 native rows and **14,326 confirmed bars**. Their separate native
calendars and OHLCV are preserved without resampling or padding.

Downloads in `I:\sys\下载`:

| Symbol | Chart CSV | Trade CSV | Download times, UTC+8 |
| --- | --- | --- | --- |
| GBPUSD | `FX_GBPUSD, 1D (1).csv` | `SSL_Hybrid_Strategy_FX_GBPUSD_2026-09-29 (3).csv` | 17:28:27 / 17:28:11 |
| EURUSD | `FX_EURUSD, 1D (1).csv` | `SSL_Hybrid_Strategy_FX_EURUSD_2026-09-29 (1).csv` | 17:24:47 / 17:24:38 |

Full original source, native CSVs, inputs, properties, chart state, export
options, screenshots, confirmed bars and CLI/core receipts are frozen per
symbol. The unrelated saved Percent short market probe's `Plot` column
is excluded from SSL Hybrid comparisons.

## Confirmed native results

| Observation | GBPUSD daily | EURUSD daily |
| --- | ---: | ---: |
| Confirmed chart bars | 14,321 | 14,326 |
| First baseline / SSL1, zero-based bar | 29 | 29 |
| First baseline channels, zero-based bar | 30 | 30 |
| Nonblank exported observations | 100,087 | 100,122 |
| Series mismatches, including missing positions | 0 | 0 |
| Confirmed closed trades | 193 | 187 |
| Explicit exit fills | 162 | 151 |
| Surviving entry records at confirmed cutoff | 2 | 3 |
| Final confirmed position | -74 at 1.33783 | -216 at 1.14645 |
| Maximum displayed net PnL difference | 0.004990344 | 0.004945608 |
| Maximum displayed commission difference | 0.002063040 | 0.002142476 |

All eight selected columns match: Candle Size > 1xATR, MA Baseline, SSL1,
Baseline Upper Channel, Basiline Lower Channel, MA UP, MA DOWN and the
disabled 2nd Multi-TimeFrame Moving Average. Absolute numeric tolerance is
`1e-8`; missing positions are checked explicitly. Candle Size false/NA
exports as zero, while the disabled second MA is entirely missing.
The baseline's leading 29 missing positions are retained in the comparison;
MA UP/DOWN start at bar 21. Enabling the second MA is not qualified here.

Closed trades match original entry IDs, entry/exit directions, UTC+8
session dates, prices, quantities, entry values, durations and displayed
net PnL/commission. Monetary display tolerance remains 0.005. There is no
quantity difference. Explicit exit multisets match IDs, dates, prices and
quantities. Surviving entries match ID, direction, date, price and quantity,
and their aggregate size/average price equals the local final position.
Daily trade exports provide session dates; no finer timestamp claim is made.

### EURUSD forming-day boundary

EURUSD's raw native export has **188 closed trades and two current open
entries**. One closed trade is `ShortExit3`, filled on Sep 29 at 1.13357
for quantity 130, on the excluded forming day. Its original entry row is
`ShortEntry3`, Sep 17 at 1.14645. Together with the two genuinely open
native entries (`ShortEntry4/5`, quantity 43 each), it identifies **three
entries surviving at the confirmed cutoff**, total -216 at 1.14645.
These match the local confirmed entries and aggregate position.

The original forming-day exit is retained unchanged and excluded from the
187-trade confirmed comparison. This does not describe `ShortEntry3` as
currently open in TradingView. GBPUSD's two native entries are genuinely
open, with no executed transaction on the forming day. Current-day indicator
values, EURUSD's exit and both symbols' unrealized PnL remain unqualified.

## Native data revisions

Compared with the preceding [JMA daily captures](SSL_HYBRID_FX_DAILY_JMA30_POWER2_HL2_20260929.md),
confirmed timestamps are unchanged, but GBPUSD has one revised `low` value
and EURUSD one revised `open` value. All other confirmed OHLCV fields match.
These revisions are recorded in `current-verification.json`; each new case
uses its own current native input. This is qualification of two new baseline
cases, without attributing every difference from JMA solely to the parameter.

## Execution and reproduction

All six executions exit zero without stderr or diagnostics. Batch,
incremental and historical realtime complete outputs are byte-identical
within each symbol:

| Symbol | Complete output SHA-256 |
| --- | --- |
| GBPUSD | `7265e0b6a6260c7d25af68c1f403e4f4bcc51a4ced21752f4cc3706d0921a6d7` |
| EURUSD | `7611555fb09596707591b349a6128a929abd5211560105a1ab51dad2dd948308` |

Core base `2f9a83a0c333092167c7253571f56ee82a7ec0d0`, core patch
`827e195c3a7db4422bbd657a5634990adb9d8d1db69658f6331df41a97cb2636`,
immutable CLI `584ae27bb25fed15c038c5c55a9356720c1bff90b8db29ce8b403e8a08010f36`.
No new core repair was needed. Current HEAD/core/golden hashes match the
passing VAMA gate (1,984 runtime tests, 242 CLI tests, workspace clippy/fmt,
WASM Node, host parity, 130 tool tests and 774 wheel tests). The unchanged-source
gate is reused, without another full-gate run.

Evidence directories:

- `.local/ssl-hybrid-fx-gbpusd-daily-mcginley30-hl2-20260929/`
- `.local/ssl-hybrid-fx-eurusd-daily-mcginley30-hl2-20260929/`

Prepare with `.local/prepare_fx_daily_mcginley_20260929.py GBPUSD` or `EURUSD`.
Run each case's `run_modes.py` and `compare.py`, then
`.local/verify_fx_daily_mcginley_20260929.py`. The verifier checks current
source/golden/CLI/gate identity, original download hashes, exact OHLCV
conversion, native startup, source input call sites from fresh analysis,
mode arguments/full output hashes, fresh native comparisons, boundary
positions, forming-day exclusion, revisions against previous captures and
restored browser state. Each case retains `analysis.json`, mode receipts,
`native-comparison.log`, `comparison.json` and `current-verification.json`.
The shared log is `.local/fx-daily-mcginley-verification-20260929.log`.

| Symbol | Native chart SHA-256 | Native trades SHA-256 | Confirmed bars SHA-256 |
| --- | --- | --- | --- |
| GBPUSD | `2336ebfd4af8576e866d3f12a6fa834bb4119769259ff03818c5362bac5d7f12` | `c80ae8676f2c63e1dd0253eb4953dc0c59420a524efbac0979e2dbcdd2ec3404` | `8c41664432e33820b1543cc9a43af39b63f35226d3932a39b5b0939bc5ba8b63` |
| EURUSD | `89f5161f90049c2f6a04c86e26593f4d8c0e5f7bfa9d8ed38474d468987ccda0` | `f80cac4f5d74a8c9ecd1e241545c7e8f3ce815ac92c15a3e8aa0529b9304b9ef` | `6d76dc133c08bb0e3204b144524de551ce5198548172778bf3d518cbd059dc0d` |

Chrome was restored to Coinbase BTCUSD daily, HMA 60 close, Phase 3,
Power 1 and volatility window 10, with screenshot/state evidence.
Qualification remains specific to this unchanged source, frozen settings,
confirmed history and exported fields. Historical realtime equality does
not establish native live-tick parity or arbitrary-script compatibility.

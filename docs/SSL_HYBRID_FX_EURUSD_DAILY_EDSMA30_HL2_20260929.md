# SSL Hybrid Strategy: EURUSD daily EDSMA 30 HL2, two and three poles

Two independent native TradingView captures were obtained through authenticated
Chrome on 2026-09-29. Both execute the unchanged public Pine v5
[SSL Hybrid Strategy by kevinmck100](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/).
Source SHA-256: `1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.

## Frozen setup and branch coverage

FXCM `FX:EURUSD`, daily (`1D`), USD, price grid `1/100000`, integer quantity
precision, point value 1. Overrides are `12=EDSMA`, `13=30`, `18=hl2`,
`20=3`, `21=2`, `26=20` and `27=2` or `27=3`. Only the Super Smoother
pole count changes. Phase 3 / Power 2 apply to the original JMA SSL2
calls, whose length remains 5. HMA exit length 15 and all other original
inputs remain unchanged.

EDSMA selects the original two- or three-pole recursive filter, computes
its rolling standard deviation, and uses the scaled result to drive the
adaptive recursive baseline. The complete script exercises separate high,
low and selected-source call states and its original five entry/target/stop
order lifecycles. These captures qualify the exported results of that
composition; internal filter or standard-deviation values are not separately
exported or independently qualified as general primitive behavior.

Native properties are independently frozen per setting: USD 5,000 capital,
10% equity sizing, pyramiding 10, commission 0.04%, zero slippage,
default four historical ticks, on bar close/realtime tick, requested limit
price, one-tick order delay and infinite long/short leverage.

Each native chart contains 14,327 rows. The runtime receives the same
**14,326 confirmed bars**, from 1971-01-03 22:00 UTC through
2026-09-27 21:00 UTC (Sep 28 session). The forming day at
2026-09-28 21:00 UTC (Sep 29 session), Unix `1790629200`, is excluded.
Confirmed OHLCV CSVs are byte-identical, SHA-256
`6d76dc133c08bb0e3204b144524de551ce5198548172778bf3d518cbd059dc0d`.

Downloads in `I:\sys\下载`:

| Poles | Chart CSV | Trade CSV | Download times, UTC+8 |
| --- | --- | --- | --- |
| 2 | `FX_EURUSD, 1D (2).csv` | `SSL_Hybrid_Strategy_FX_EURUSD_2026-09-29 (2).csv` | 17:36:48 / 17:36:31 |
| 3 | `FX_EURUSD, 1D (3).csv` | `SSL_Hybrid_Strategy_FX_EURUSD_2026-09-29 (3).csv` | 17:38:50 / 17:38:30 |

Full original source, native CSVs, settings, chart state, export options,
screenshots, confirmed bars and immutable CLI/core receipts are frozen per
case. The unrelated saved Percent short market probe's `Plot` column is
excluded from SSL Hybrid comparisons.

## Confirmed native results

| Observation | Two poles | Three poles |
| --- | ---: | ---: |
| Confirmed chart bars | 14,326 | 14,326 |
| Nonblank exported observations | 100,180 | 100,180 |
| Series mismatches, including missing positions | 0 | 0 |
| Closed trades | 524 | 514 |
| Explicit exit fills | 276 | 276 |
| Surviving native open entries | 1 | 1 |
| Final confirmed position | -43 at 1.15778 | -42 at 1.15778 |
| Maximum displayed net PnL difference | 0.004997520 | 0.004967280 |
| Maximum displayed commission difference | 0.002881752 | 0.002393728 |

All eight selected native columns match: Candle Size > 1xATR, MA Baseline,
SSL1, Baseline Upper Channel, Basiline Lower Channel, MA UP, MA DOWN and
the disabled 2nd Multi-TimeFrame Moving Average. Absolute numeric tolerance
is `1e-8`, with missing positions checked. Candle Size false/NA exports as
zero; the disabled second MA is entirely missing. Enabling it is not covered.

Baseline/SSL1 are defined from zero-based bar **0**, with **30 initial zero
values** and their first nonzero value at bar **30**, in both cases.
Channels first appear at bar 30 and MA UP/DOWN at bar 21. All startup
values and missing positions remain in the comparison, without trimming.

Closed trades match entry IDs, entry/exit directions, UTC+8 session dates,
prices, quantities, entry values, durations and displayed net PnL/commission.
Monetary display tolerance remains 0.005, with zero quantity difference.
Explicit exit multisets match IDs, dates, prices and quantities.
Both genuinely open native records are `ShortEntry5`, entered Aug 31 at
1.15778, with quantities 43 and 42 respectively. Their entry fields and
aggregate position match the local final result. Daily trade exports provide
session dates; this receipt makes no finer intraday timestamp claim.

All executed transaction dates precede the forming day. Current-day
indicator values, unrealized PnL and live ticks remain outside scope.

## Independent pole sensitivity

Changing two poles to three on identical confirmed OHLCV changes baseline
and SSL1 at **14,296 of 14,326** mutually defined positions by more than
`1e-8`. Each channel changes at all **14,296** mutually defined positions.
The first 30 baseline/SSL1 values remain zero in both settings. Closed
trades change from 524 to 514 and the remaining quantity from 43 to 42.
Both parameter results have independent native qualification. They share
one dataset and are not independent market histories.

## Execution, source and reproduction

All six executions exit zero without stderr or diagnostics. Batch,
incremental and historical realtime complete outputs are byte-identical
within each setting:

| Poles | Complete output SHA-256 |
| --- | --- |
| 2 | `7d3ea73a4ddcb8a4725bd152db4a578ceffddae6e93ab676ffc648d56da1632d` |
| 3 | `036415dac5602cb92d7f99361683dce0fd168f6c4b66248fc9cc7cd118eba30c` |

Core base `2f9a83a0c333092167c7253571f56ee82a7ec0d0`, core patch
`827e195c3a7db4422bbd657a5634990adb9d8d1db69658f6331df41a97cb2636`,
immutable CLI `584ae27bb25fed15c038c5c55a9356720c1bff90b8db29ce8b403e8a08010f36`.
No new core repair was needed. Current HEAD/core/golden hashes match the
passing VAMA gate (1,984 runtime tests, 242 CLI tests, workspace clippy/fmt,
WASM Node, host parity, 130 tool tests and 774 wheel tests). The unchanged-source
gate is reused, without another full-gate run.

Evidence directories:

- `.local/ssl-hybrid-fx-eurusd-daily-edsma30-poles2-hl2-20260929/`
- `.local/ssl-hybrid-fx-eurusd-daily-edsma30-poles3-hl2-20260929/`

Prepare with `.local/prepare_eurusd_daily_edsma_20260929.py 2` or `3`.
Run each case's `run_modes.py` and `compare.py`, then
`.local/verify_eurusd_daily_edsma_20260929.py`. The verifier checks current
source/golden/CLI/gate identity, original download hashes, exact OHLCV
conversion, startup including zero values, input call sites from fresh
source analysis, native settings, mode arguments/full output hashes,
fresh comparisons, boundary positions, forming-day exclusion, parameter
sensitivity and restored browser state. Each case retains `analysis.json`,
mode receipts, `native-comparison.log`, `comparison.json` and
`current-verification.json`; the shared log is
`.local/eurusd-daily-edsma-verification-20260929.log`.

| Poles | Native chart SHA-256 | Native trades SHA-256 |
| --- | --- | --- |
| 2 | `26aff6ebd1723237cddead8c850a0c29998ea0d0d47d024e680219734770134a` | `b498d628a7ffa5dfe8e8ad73ce1fe6eac6169188131f2240b6efcec5af268015` |
| 3 | `b2743ae00d8f429a9280d8dc49752ecf471a421ca66a41f97b4c8ee31fb80407` | `b9adc9efb1fc07fb598f399bfac12a30fa88afa1d2f0b65bddd248a722afcd6c` |

Chrome was restored to Coinbase BTCUSD daily, HMA 60 close, JMA Phase 3 /
Power 1, volatility window 10, and Super Smoother length 20 / two poles,
with screenshot/state evidence. Qualification remains specific to the
unchanged source, frozen settings, confirmed history and exported fields.
Historical realtime equality does not establish native live-tick parity
or arbitrary-script compatibility.

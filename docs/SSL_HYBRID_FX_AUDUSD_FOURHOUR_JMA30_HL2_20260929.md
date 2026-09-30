# SSL Hybrid Strategy: AUDUSD four-hour JMA 30 HL2, Power 1 and 2

Two native TradingView captures were obtained through authenticated Chrome
on 2026-09-29. Both execute the unchanged public Pine v5
[SSL Hybrid Strategy by kevinmck100](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/).
Source SHA-256: `1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.

## Frozen setup

FXCM `FX:AUDUSD`, four-hour (`240`), USD, price grid `1/100000`, integer
quantity precision, point value 1. Overrides: `12=JMA`, `13=30`, `18=hl2`,
`20=3`, and `21=1` or `21=2`. Phase remains 3; Power is the only changed
parameter. Other settings remain original, including JMA SSL2 length 5,
HMA exit length 15, and the five target/stop settings.

Each capture freezes native inputs and properties: USD 5,000 capital,
10% equity sizing, pyramiding 10, 0.04% commission, zero slippage,
default four historical ticks, on bar close/realtime tick, requested limit
price, one-tick order delay, and infinite long/short leverage.

Each chart has 21,346 rows; **21,345 confirmed bars** enter the runtime,
from 2013-01-02 02:00 UTC to 2026-09-29 01:00 UTC. The forming bar at
2026-09-29 05:00 UTC (13:00 UTC+8), Unix `1790658000`, is excluded.
The confirmed input CSVs have identical OHLCV and SHA-256
`d1e68f96b93989dfd915da8fb98f0e47ab0f672bd3bc9e439f6b372b573e53bc`.

Native downloads in `I:\sys\下载`:

| Power | Chart CSV | Trade CSV | Download times, UTC+8 |
| --- | --- | --- | --- |
| 1 | `FX_AUDUSD, 240 (5).csv` | `SSL_Hybrid_Strategy_FX_AUDUSD_2026-09-29 (8).csv` | 13:32:27 / 13:31:53 |
| 2 | `FX_AUDUSD, 240 (6).csv` | `SSL_Hybrid_Strategy_FX_AUDUSD_2026-09-29 (9).csv` | 16:48:35 / 16:48:25 |

The original source, native CSVs, chart state, settings, export options,
screenshots, input bars, CLI and core patch are preserved per case.
The unrelated saved Percent short market probe's `Plot` column is excluded.

## Confirmed native comparisons

| Observation | Power 1 | Power 2 |
| --- | ---: | ---: |
| Confirmed chart bars | 21,345 | 21,345 |
| Nonblank exported observations | 149,313 | 149,313 |
| Series mismatches, including missing positions | 0 | 0 |
| Confirmed closed trades | 2,229 | 3,122 |
| Explicit exit fills | 1,629 | 1,990 |
| Surviving entry records at cutoff | 1 | 5 |
| Final position | -66 at 0.71058 | +655 at 0.70271 |
| Maximum displayed net PnL difference | 0.004999120 | 0.004992180 |
| Maximum displayed commission difference | 0.004990368 | 0.004999320 |

Eight exported columns are compared: Candle Size > 1xATR, MA Baseline,
SSL1, Baseline Upper Channel, Basiline Lower Channel, MA UP, MA DOWN,
and the disabled 2nd Multi-TimeFrame Moving Average. Absolute numeric
tolerance is `1e-8`; missing positions are checked. Candle Size false/NA
exports as zero. The disabled second MA remains entirely missing.

Baseline/SSL1 are defined from zero-based bar **0**, channels from bar **30**,
and MA UP/DOWN from bar **21** in both native and local output. The original
recursive JMA uses `nz` zero initialization; its first-bar transient is
included in the comparison, without discarding startup bars.

Closed trades match original entry IDs, entry/exit directions, UTC+8
minute timestamps, prices, quantities, entry values, durations, and displayed
net PnL/commission. Monetary display tolerance is 0.005. Surviving entries
match IDs, directions, timestamps, prices and quantities; their aggregate
size/average price matches the local final position. No quantity difference
is allowed. The native export does not qualify sub-minute timestamps.

### Power 2 forming-bar boundary

The unmodified native trade CSV contains **3,127 closed trades and ten current
short entries**. Five long exits at 13:00 UTC+8, price 0.6996, occur on the
excluded forming bar. Ten short entries are stamped 16:47 UTC+8 in that
same bar, with prices 0.70158 and 0.69901. These fifteen transitions are
retained in the native CSV and excluded from the confirmed comparison.
Their live sequence and unrealized PnL are not qualified.

The original long entry rows paired with those five forming-bar exits
identify the positions surviving at the confirmed cutoff. Their quantities
131, 197, 197, 65 and 65 total **655 at 0.70271**, matching the five local
surviving long entries. They are not open in the current native export;
the comparator records this boundary derivation explicitly. Power 1's one
open short entry is genuinely open in its captured export.

The preparation guard initially rejected Power 2's forming-bar transactions.
The evidence helper was extended to compare the confirmed boundary explicitly.
This required no interpreter change. Capture times differ, but all confirmed
input bars are identical; the unconfirmed remainder stays outside scope.

## Independent parameter sensitivity

Changing Power 1 to 2 changes all **21,345** mutually defined baseline
observations, **21,344** SSL1 observations, and all **21,315** observations
of each channel by more than `1e-8`. Confirmed closed trades change from
2,229 to 3,122. Each parameter result has independent native qualification
on the same data. These are two parameter cases sharing one dataset.
Changing Power also affects the original JMA SSL2 call; qualification covers
the complete original script and its separate recursive function call states.

## Execution and reproduction

Batch, incremental, and historical realtime each exit zero without stderr
or diagnostics. Complete outputs are byte-identical across the three modes:

| Power | Complete output SHA-256 |
| --- | --- |
| 1 | `5f2e174623ea8f377dab2fdb0c82718965633bb33351bd48e509be42f04b6935` |
| 2 | `86a9148e4655049f2a25a8d53975e89d724ba25f1d966dc68fe5c12f71fb1c9c` |

Core base `2f9a83a0c333092167c7253571f56ee82a7ec0d0`, core patch
`827e195c3a7db4422bbd657a5634990adb9d8d1db69658f6331df41a97cb2636`,
immutable CLI `584ae27bb25fed15c038c5c55a9356720c1bff90b8db29ce8b403e8a08010f36`.
No additional core repair was needed. Current HEAD/core/golden hashes match
the earlier passing VAMA gate (1,984 runtime tests, 242 CLI tests, workspace
clippy/fmt, WASM Node, host parity, 130 tool tests and 774 wheel tests).
That unchanged-source gate is reused, without another full-gate run.

Evidence directories:

- `.local/ssl-hybrid-fx-audusd-fourhour-jma30-hl2-20260929/`
- `.local/ssl-hybrid-fx-audusd-fourhour-jma30-power2-hl2-20260929/`

Run each directory's `run_modes.py` and `compare.py`, then run
`.local/verify_jma_cases_20260929.py`. Preparation is reproducible with
`.local/prepare_jma_cases_20260929.py POWER CHART_FILENAME TRADE_FILENAME`.
The verifier checks source/CLI/gate identity, original download hashes,
exact OHLCV conversion, input call sites from fresh analysis, mode arguments
and full output hashes, fresh comparisons, startup, boundary positions,
parameter sensitivity and restored browser state. Each case retains
`analysis.json`, `comparison.json`, mode receipts, `native-comparison.log`
and `current-verification.json`; the shared run log is
`.local/jma-verification-20260929.log`.

| Power | Native chart SHA-256 | Native trades SHA-256 |
| --- | --- | --- |
| 1 | `98def760fd40b5b8bc6dbb4174e4f85c51646f4007c350039a877299b605a2ae` | `3141f7fce2ab6f1dec24a708e9fd2aeeef03b0cb58b567d92d9349b468b692dc` |
| 2 | `79b4c460a01ffee86e967e427328c97c63a3a2a55f4c250fd49841a3b1b0dca5` | `8e207a2dadafd0835dc3835d60ffc53e81dca5d10bc45dc886505cde8f2091c7` |

Chrome was restored to Coinbase BTCUSD daily, HMA 60 close, Power 1 and
volatility window 10, with screenshot/state receipts. Qualification remains
specific to this source, frozen settings/history and exported fields.
Historical realtime equality does not establish native live-tick parity
or arbitrary-script compatibility.

# SSL Hybrid Strategy: AUDUSD weekly VAMA 30 HL2, windows 60 and 20

Two independent native TradingView captures were obtained through authenticated
Chrome on 2026-09-29 after the extension connection recovered. These use the
unchanged public Pine v5 [SSL Hybrid Strategy by kevinmck100](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/).
Source SHA-256: `1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.

## Frozen setup

FXCM `FX:AUDUSD`, weekly (`1W`), USD, price grid `1/100000`, integer quantity
precision, point value 1. Input overrides are `12=VAMA`, `13=30`, `18=hl2`,
and `22=60` or `22=20`. Other inputs remain original, including the
2021-08-01 to 2030-10-01 trading range, JMA SSL2 length 5, HMA exit length 15,
and the original five target/stop settings.

Native properties were independently frozen for each capture: USD 5,000
capital, 10% equity sizing, pyramiding 10, commission 0.04%, zero slippage,
default four historical ticks, on bar close/realtime tick, requested limit
price, one-tick order delay, infinite long/short leverage.

Each chart export contains 2,908 weekly rows from 1971-01-03 22:00 UTC.
The local input contains the same **2,907 confirmed bars**, ending
2026-09-20 21:00 UTC (Sep 21 session). The forming week at
2026-09-27 21:00 UTC (Sep 28 session), Unix `1790542800`, is excluded.
Confirmed OHLCV is byte-identical across the two input CSVs:
`d8a0716513864530a858e6a4ec8ed0047cb5a4bff43c695f54eeacde67b73345`.

Native downloads in `I:\sys\下载`:

| Window | Chart CSV | Trade CSV | Download times, UTC+8 |
| --- | --- | --- | --- |
| 60 | `FX_AUDUSD, 1W.csv` | `SSL_Hybrid_Strategy_FX_AUDUSD_2026-09-29 (6).csv` | 13:18:05 / 13:17:45 |
| 20 | `FX_AUDUSD, 1W (1).csv` | `SSL_Hybrid_Strategy_FX_AUDUSD_2026-09-29 (7).csv` | 13:19:57 / 13:19:45 |

Source, inputs, properties, chart state, export options, screenshots, original
native CSVs, and confirmed bars are frozen separately per setting.
The unrelated saved Percent short market probe's `Plot` column is excluded.

## Native results at the confirmed cutoff

| Observation | Window 60 | Window 20 |
| --- | ---: | ---: |
| First baseline / SSL1, zero-based bar | 59 | 29 |
| First baseline channels, zero-based bar | 59 | 30 |
| Nonblank exported observations | 20,071 | 20,189 |
| Series mismatches, including missing positions | 0 | 0 |
| Confirmed closed trades | 55 | 120 |
| Explicit exit fills | 33 | 52 |
| Surviving entry records at cutoff | 5 | 5 |
| Final position | -713 at 0.70227 | +685 at 0.71566 |
| Maximum displayed net PnL difference | 0.004935200 | 0.004962504 |
| Maximum displayed commission difference | 0.003925524 | 0.003516188 |

All eight selected exported columns are compared: Candle Size > 1xATR,
MA Baseline, SSL1, Baseline Upper Channel, Basiline Lower Channel, MA UP,
MA DOWN, and the disabled 2nd Multi-TimeFrame Moving Average. Numeric
tolerance is absolute `1e-8`, with missing positions checked explicitly.
The Candle Size export flattens false/NA to zero. The disabled second MA
is entirely missing; this does not qualify enabling that input.

Closed trades match original entry IDs, entry/exit directions, session dates,
prices, quantities, entry values, durations and displayed net PnL/commission.
Display monetary tolerance remains 0.005. Entry records match IDs, directions,
dates, prices and quantities, and their aggregate size/average price matches
the final local position. Native trade CSV dates are UTC+8 session dates;
this comparison makes no finer intraday timestamp claim.

### Window 20 forming-week boundary

The original window 20 trade CSV contains **125 closed trades and five current
short entries**. Five of those closed trades are the long position reversed
at this forming week's open (Sep 28). The five short entries were also
filled on Sep 28. All ten transitions belong beyond the confirmed cutoff
and are retained unchanged in the native CSV; they are excluded from the
120-trade confirmed comparison.

The five long entry rows attached to the forming-week reversal exits identify
the entries that survived at the confirmed cutoff. Their native entry fields
match the five local surviving long entries (+685 total at 0.71566).
They are **not** described as open in the current native export. The
comparator records this derivation explicitly and verifies aggregate position.
Window 60 has five genuinely open native short entries at capture time;
all executed trade dates precede the forming week.

The initial preparation guard correctly rejected window 20's forming-week
transactions. Preparation was then extended to represent the confirmed
boundary explicitly before qualification; a UTF-8 verifier read was also
corrected. These were evidence-helper changes, with no runtime change.
The forming week's indicator calculations, reversal fills, new shorts and
unrealized PnL are not qualified by this receipt.

## Independent parameter sensitivity

Both windows have native qualification on the identical confirmed input.
Among 2,848 mutually defined observations, changing 60 to 20 changes
baseline/SSL1 at 2,657 positions and each channel at 2,663 positions.
The startup boundary changes from bar 59 to bar 29 (channels 30), and
confirmed closed trades change from 55 to 120. This covers both a volatility
window larger than baseline length and one smaller than baseline length.
The histories overlap; they are two parameter cases, not independent datasets.

## Execution, current source, and reproduction

Batch, incremental, and historical realtime each exit zero without stderr
or diagnostics. Complete outputs are byte-identical across all three modes
for each setting:

| Window | Complete output SHA-256 |
| --- | --- |
| 60 | `9b39b75b894ac1eff61634faff8affbe46192244e8d0c7958f73aafa594a1d8c` |
| 20 | `f6d81907d4be301d36b90e23e8c4a27a6c1dea6a26d3ddaa6ff7ea9b28d7bfd7` |

Core base `2f9a83a0c333092167c7253571f56ee82a7ec0d0`, core patch
`827e195c3a7db4422bbd657a5634990adb9d8d1db69658f6331df41a97cb2636`,
immutable CLI `584ae27bb25fed15c038c5c55a9356720c1bff90b8db29ce8b403e8a08010f36`.
No additional core fix was needed. Current HEAD/core/golden hashes match
the prior VAMA full gate (1,984 runtime tests, 242 CLI tests, workspace
clippy/fmt, WASM Node, host parity, 130 tool tests and 774 wheel tests).
The unchanged-source gate is reused; it was not rerun for these data cases.

Evidence directories:

- `.local/ssl-hybrid-fx-audusd-weekly-vama30-vol60-hl2-20260929/`
- `.local/ssl-hybrid-fx-audusd-weekly-vama30-vol20-hl2-20260929/`

Each retains `prepare_bars.py`, `run_modes.py`, `compare.py`, complete outputs,
mode receipts, `native-comparison.log`, `comparison.json`, and
`current-verification.json`. Reproduce each case with its three scripts,
then `.local/verify_weekly_vama_20260929.py`. The shared verifier checks
original download hashes, source/CLI/gate identity, exact OHLCV translation,
mode arguments/output hashes, fresh native comparisons, cutoff treatment,
startup, positions, parameter sensitivity, and restored browser state.
The preparation generator is `.local/prepare_weekly_vama_20260929.py`.

Chart/trade SHA-256:

| Window | Chart | Trades |
| --- | --- | --- |
| 60 | `04a59ab115eb3b6abeec6f9349cb238b3be98a8e61e86cbd90488d00f1188598` | `733613d532eeb0b51ede5aa6aa48efbed5d2a9386efd6b1261316f600791ba77` |
| 20 | `2060a1824f8c47c38fdd5cc0002c8ee32be5f1984c5f1a07d9b01ccf452f2b43` | `135ca6370d5f56266e51b07db4c17c27ef0b51364a21482de6acc3ac20942a96` |

Chrome was restored to Coinbase BTCUSD daily, HMA 60 close, volatility
window 10, with screenshot/state evidence. Scope is this unchanged source,
frozen settings/history and exported fields. Historical realtime equality
does not establish native live-tick parity or arbitrary-script compatibility.

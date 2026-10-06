# SSL Hybrid EURUSD daily: Tilson T3 factor 0.3 and 0.9

Two fresh independent TradingView chart and trade exports were captured through
authenticated Chrome on 2026-09-29. Both execute the unchanged public Pine v5
[SSL Hybrid Strategy](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/).
Original source SHA-256:
`1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.
This extends the [enabled CF filter captures](SSL_HYBRID_FX_EURUSD_DAILY_CF_FILTERS_20260929.md)
with two further input settings.

## Frozen setup

Both use FXCM `FX:EURUSD`, daily `1D`, Kijun v2 baseline length 30 / divider 3,
HL2, original JMA SSL2 length 5 / Phase 3 / Power 2 and HMA exit length 15.
CF Ultimate MA MTF is enabled, with current chart resolution, primary type
8 (Tilson T3), length 20, direction comparison interval 2 and second MA disabled.
Primary factor input is 3 or 9, multiplied by 0.10 in the original source.
The second, disabled MA's separate factor remains 7.

CLI original call-site overrides: `12=Kijun v2`, `13=30`, `18=hl2`, `20=3`,
`21=2`, `19=3`, `587=true`, `589=20`, `590=3` or `590=9`, `591=8`,
`594=2`, `652=true`. Fresh original-source analysis checks these identities.
Complete native input snapshots differ only in primary factor after removing
transient focus and spin controls; all other inputs and checkbox states remain checked.

Native properties: USD 5,000 capital, 10% equity sizing, pyramiding 10,
commission 0.04%, zero slippage, default four historical ticks, bar close/realtime
execution, requested limit price, one-tick delay, infinite leverage. Runtime
metadata: USD, grid `1/100000`, integer quantity precision and point value 1.

CF uses chart close independently of the baseline's selected source. Its triple
nested generalized DEMA computation gates actual entries through MA UP/DOWN.
The internal numeric MA is hidden and is not independently qualified by this
export. Coverage applies to the composed direction/trade results with
same-symbol/same-period request.security; other timeframe contexts are unqualified.

## History and downloads

Each export contains 14,327 bars, with 14,326 confirmed bars retained exactly.
First bar: `31788000` (1971-01-03 22:00 UTC); last confirmed:
`1790542800` (2026-09-27 21:00 UTC). Forming bar `1790629200`
(2026-09-28 21:00 UTC) is preserved in raw CSV and excluded from execution.
The native trade cutoff is session date 2026-09-29 in UTC+8. No resampling;
OHLCV is copied as exported, converting seconds to milliseconds.

Confirmed bars are byte-identical between these new cases, SHA-256
`6d76dc133c08bb0e3204b144524de551ce5198548172778bf3d518cbd059dc0d`.
The previous enabled CF factor-0.7 capture differs in last confirmed volume
(175968 versus newly exported 179747); it is not used as an identical-data
control for this comparison.

Native downloads in `I:\sys\下载`:

| Factor | Chart CSV | Trade CSV | Download times, UTC+8 |
| --- | --- | --- | --- |
| 0.3 | `FX_EURUSD, 1D (12).csv` | `SSL_Hybrid_Strategy_FX_EURUSD_2026-09-29 (13).csv` | 19:38:23 / 19:37:54 |
| 0.9 | `FX_EURUSD, 1D (13).csv` | `SSL_Hybrid_Strategy_FX_EURUSD_2026-09-29 (14).csv` | 19:39:58 / 19:39:25 |

## Confirmed results

| Observation | Factor 0.3 | Factor 0.9 |
| --- | ---: | ---: |
| Nonblank exported observations | 99,932 | 99,932 |
| Series mismatches, including NA positions | 0 | 0 |
| Confirmed closed trades | 170 | 192 |
| Confirmed explicit exit fills | 162 | 184 |
| Surviving entries at confirmed cutoff | 5 | 3 |
| Final confirmed position | -442 at 1.13826 | -216 at 1.14757 |
| Forming-period exits / entries excluded | 0 / 0 | 1 / 0 |

All eight exported columns match: Candle Size > 1xATR, MA Baseline, SSL1,
Baseline Upper Channel, Basiline Lower Channel, MA UP, MA DOWN and disabled
second MA. Numeric absolute tolerance remains `1e-8`; every missing position is
checked. Baseline/SSL1 start at zero-based bar 29, channels at 30, direction
columns at 116 in both cases. Startup values are retained. Candle Size false/NA
exports as zero; second MA is entirely missing. The unrelated Percent short
market probe's Plot is excluded.

Under identical confirmed bars, changing primary factor changes **2,310**
positions in each direction column; baseline, SSL1 and channels are unchanged.
Different trade counts and entry dates demonstrate an actual parameter effect.

Closed trades match IDs, directions, displayed session dates, prices, exact
quantities, entry values, durations and displayed net PnL/commission. Monetary
display tolerance stays 0.005; observed maximum PnL differences are
0.004946704/0.004991360 and commission 0.002274228/0.002288248.
Quantity difference is zero. Explicit exit multisets match IDs, dates, prices
and quantities. Daily session dates do not qualify finer intraday timing.

### Forming-bar separation and positions

Factor 0.3 retains Sep 24 short entries ShortEntry1 through ShortEntry5 at
1.13826, quantities **88, 133, 133, 44, 44**, aggregating to **-442**.
All native transactions precede the forming cutoff.

Factor 0.9 raw CSV reports **193 closed trades and two open entries**, because
ShortExit3 occurred on Sep 29 at 1.1348, quantity **130**, against Sep 18
ShortEntry3 at 1.14757. This forming-period exit is preserved in raw evidence
and excluded from confirmed-history comparisons. At the confirmed cutoff,
ShortEntry3, ShortEntry4 and ShortEntry5 still survive, quantities **130, 43, 43**,
aggregating to **-216** at 1.14757. The comparison derives that third boundary
entry from the original native closed pair. It does not discard its prior entry
or compare current open count against a preceding closed-bar state.

All boundary IDs/dates/directions/quantities and unique filled orders match.
Open-entry price differences are zero; the local factor-0.3 weighted average
differs from the displayed price by floating representation only. Current-period
unrealized PnL, direction values and live ticks are outside qualification.

## Execution and reproduction

Batch, incremental and historical realtime complete outputs match byte for byte
within each setting, with zero exit codes, stderr and diagnostics:

| Factor | Complete output SHA-256 |
| --- | --- |
| 0.3 | `4b715b856e5b7e9990da9127ab14ad7b73bb396c8a8417cf5174e671c1ca70ab` |
| 0.9 | `f795ce3aec40f86e6dda0c883f09429c568a2dbfacf2b79882373f64bff637d7` |

Core base `2f9a83a0c333092167c7253571f56ee82a7ec0d0`, patch
`827e195c3a7db4422bbd657a5634990adb9d8d1db69658f6331df41a97cb2636`,
immutable CLI `584ae27bb25fed15c038c5c55a9356720c1bff90b8db29ce8b403e8a08010f36`.
No new core repair. Current core/golden hashes agree with the passing VAMA
full-gate source; that gate is reused without a new full-gate execution.

Local evidence:

- `.local/ssl-hybrid-fx-eurusd-daily-kijun30-cf-t3-factor3-hl2-20260929/`
- `.local/ssl-hybrid-fx-eurusd-daily-kijun30-cf-t3-factor9-hl2-20260929/`

Each freezes native CSVs, source, bars, complete input/property/export snapshots,
chart screenshots, CLI/patch receipts, three mode outputs, source analysis,
comparison and current-verification receipts. BTCUSD daily, HMA60/Close,
divider1/Power1, CF disabled/type1/factor7 was restored and recorded.
Run each directory's `run_modes.py`, then:

```powershell
python .local/verify_eurusd_daily_t3_factors_20260929.py
```

Verification log: `.local/eurusd-daily-t3-factors-verification-20260929.log`.
Helper hashes are frozen in current verification receipts. Coverage remains
source/settings/metadata/history specific. Hidden numeric MA values, different
timeframe requests, live ticks, unrelated scripts and the earlier Hull v4
quantity discrepancy remain unqualified. No commit or push; expansion goal active.

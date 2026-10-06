# SSL Hybrid EURUSD daily: enabled CF TEMA and Tilson T3 filters

Two independent TradingView native chart and trade exports were captured through
authenticated Chrome on 2026-09-29. Both execute the unchanged public Pine v5
[SSL Hybrid Strategy](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/).
Source SHA-256: `1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.

## Setup and evidence boundary

Both use FXCM `FX:EURUSD`, daily `1D`, Kijun v2 baseline length 30 / divider 3,
HL2, original JMA SSL2 length 5 / Phase 3 / Power 2 and HMA exit length 15.
CF Ultimate MA MTF is enabled and uses the current chart resolution, length 20,
direction smoothing 2 and Tilson factor input 7 (0.7). Primary MA type is
7 (TEMA) or 8 (Tilson T3); the optional second MA remains disabled.

Original input overrides are `12=Kijun v2`, `13=30`, `18=hl2`, `20=3`,
`21=2`, `19=3`, `587=true`, `589=20`, `590=7`, `591=7` or `591=8`,
`594=2`, `652=true`. Fresh source analysis checks these call-site identities.
Native full input snapshots differ only in primary type after removing transient
focus and spin controls. Native properties freeze USD 5,000 initial capital,
10% equity sizing, pyramiding 10, commission 0.04%, zero slippage, default four
historical ticks, bar close/realtime execution, requested limit price, one-tick
delay and infinite leverage. Runtime metadata uses USD, grid `1/100000`,
integer quantity precision and point value 1.

Source inspection establishes that SSL2 continuation calculations do not gate
this script's orders. The enabled CF filter does gate entry conditions through
MA UP/DOWN. CF uses close independently of the selected HL2 source. Its internal
MA value is hidden and is not independently numerically qualified by this export.
This qualifies the composed direction and trading behavior using same-symbol,
same-period request.security; different-timeframe requests remain outside scope.

Both exports contain 14,327 daily bars, with 14,326 confirmed bars retained.
First bar: `31788000` (1971-01-03 22:00 UTC); last confirmed:
`1790542800` (2026-09-27 21:00 UTC). The forming bar `1790629200`
(2026-09-28 21:00 UTC) is retained in raw CSV and excluded from execution.
Trade cutoff is session date 2026-09-29 in UTC+8. OHLCV is copied exactly,
with seconds converted to milliseconds and no resampling.

Confirmed bars are byte-identical between the two new captures, SHA-256
`3aacf13fbdcc5da14709977fafad05f34f031eb201059a967c91118e05cc9dde`.
The earlier CF-disabled Kijun daily capture differs in the last confirmed
volume (179747 versus newly exported 175968). It is not used as an identical-data
control here. No arbitrary market or live-tick parity is inferred.

Native downloads in `I:\sys\下载`:

| Case | Chart CSV | Trade CSV | Download times, UTC+8 |
| --- | --- | --- | --- |
| TEMA | `FX_EURUSD, 1D (10).csv` | `SSL_Hybrid_Strategy_FX_EURUSD_2026-09-29 (11).csv` | 19:27:56 / 19:27:31 |
| T3 | `FX_EURUSD, 1D (11).csv` | `SSL_Hybrid_Strategy_FX_EURUSD_2026-09-29 (12).csv` | 19:29:32 / 19:29:02 |

## Confirmed results

| Observation | TEMA | Tilson T3 |
| --- | ---: | ---: |
| Nonblank observations | 100,046 | 99,932 |
| Series mismatches including NA positions | 0 | 0 |
| Closed trades | 574 | 193 |
| Explicit exit fills | 251 | 188 |
| Surviving entries | 1 | 2 |
| Final position | -43 at 1.15953 | -86 at 1.1482 |
| Direction first defined, zero-based bar | 59 | 116 |
| Forming-period exits / entries excluded | 0 / 0 | 0 / 0 |

All eight columns match: Candle Size > 1xATR, MA Baseline, SSL1, Baseline
Upper Channel, Basiline Lower Channel, MA UP, MA DOWN and disabled second MA.
Numeric absolute tolerance remains `1e-8`; every missing position is checked.
Baseline and SSL1 retain 29 startup missing bars, channels 30, and CF directions
59/116. The disabled second MA is entirely missing; Candle Size false/NA
exports as zero. The unrelated Percent short market probe's Plot is excluded.

Under identical confirmed bars, changing primary type changes 5,703 positions
in each direction column, including startup positions. Baseline, SSL1 and
channels are unchanged. The differing closed trade counts establish that the
parameter affects actual trading behavior in these frozen cases.

Closed records match IDs, directions, displayed session dates, entry/exit prices,
exact quantities, entry values, durations and displayed net PnL/commission.
Display money tolerance is 0.005; observed maximum net PnL differences are
0.004998296/0.004989640 and commission 0.002027616/0.002198504.
Quantity difference is zero. Explicit exit multisets match IDs, dates, prices
and quantities. TEMA retains ShortEntry5 on Sep 14 at 1.15953, quantity 43.
T3 retains ShortEntry4 and ShortEntry5 on Sep 21 at 1.1482, quantity 43 each.
Both directions and aggregate positions match; maximum open price differences
are 0 and 2.22e-16, respectively. All transactions precede the forming cutoff.
Native daily trade dates do not establish finer intraday timing parity.

## Execution, source and reproduction

Batch, incremental and historical realtime complete outputs agree byte for byte
within each setting, with zero exit codes, stderr and diagnostics:

| Case | Complete output SHA-256 |
| --- | --- |
| TEMA | `3df017a2a69e64c97c384d0247d433749dd8b21eebdb31d8e7852e5ddf0a3cba` |
| T3 | `5090716937f4c1374f22d972e0cd4b93536debbb8e3bb68e6dbe0b3901bdf1bb` |

Core base `2f9a83a0c333092167c7253571f56ee82a7ec0d0`, patch
`827e195c3a7db4422bbd657a5634990adb9d8d1db69658f6331df41a97cb2636`,
immutable CLI `584ae27bb25fed15c038c5c55a9356720c1bff90b8db29ce8b403e8a08010f36`.
No new core repair. Current core and golden hashes match the passing VAMA
full-gate source; that gate is reused without another full-gate run.

Local evidence directories:

- `.local/ssl-hybrid-fx-eurusd-daily-kijun30-cf-tema20-hl2-20260929/`
- `.local/ssl-hybrid-fx-eurusd-daily-kijun30-cf-t3-20-hl2-20260929/`

Each freezes native chart/trade CSVs, source, bars, complete input/property/export
snapshots, chart screenshots, CLI/patch receipts, three mode outputs, source
analysis, comparison and current-verification receipts. Original BTCUSD daily,
HMA60/Close/divider1/Power1, CF disabled/type1 was restored and recorded.

Reproduce with each directory's `run_modes.py`, then:

```powershell
python .local/verify_eurusd_daily_cf_20260929.py
```

Verification log: `.local/eurusd-daily-cf-verification-20260929.log`.
Shared preparation and verification helper hashes are frozen in each receipt.
Qualification remains limited to these original-source/settings/metadata/history
combinations. Live ticks, hidden numeric MA values, differing-timeframe contexts,
unrelated scripts and the earlier unresolved Hull v4 quantity case remain unqualified.
No commit or push was performed. The expansion goal remains active.

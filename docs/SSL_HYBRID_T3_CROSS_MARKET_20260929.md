# SSL Hybrid enabled T3: GBPUSD four-hour and EURUSD weekly

Two independent TradingView chart and trade exports were captured through
authenticated Chrome on 2026-09-29. Both execute the unchanged public Pine v5
[SSL Hybrid Strategy](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/).
Source SHA-256: `1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.
This extends [enabled EURUSD daily CF filters](SSL_HYBRID_FX_EURUSD_DAILY_CF_FILTERS_20260929.md)
to two further market histories and periods.

## Frozen setup and history

FXCM `FX:GBPUSD` four-hour (`240`) and `FX:EURUSD` weekly (`1W`) use identical
script inputs: Kijun v2 baseline 30 / divider 3, HL2, original JMA SSL2
length 5 / Phase 3 / Power 2 and HMA exit length 15. CF Ultimate MA MTF is
enabled; its primary type is Tilson T3 (8), length 20, factor input 7 (0.7),
direction comparison interval 2, current chart resolution enabled and second MA
disabled. Native full input snapshots agree after removing transient focus/spin
controls only. The alternate timeframe remains D but is unused.

Original call-site overrides: `12=Kijun v2`, `13=30`, `18=hl2`, `20=3`,
`21=2`, `19=3`, `587=true`, `589=20`, `590=7`, `591=8`, `594=2`, `652=true`.
Fresh original-source analysis verifies the call-site identities and defaults.

CF uses chart close independently of the baseline's selected source. Its
triple nested generalized DEMA calculation gates actual entries through
MA UP/DOWN. The hidden numeric MA is not independently qualified by this export.
This covers composed direction and trading behavior with same-symbol and
same-period request.security; different timeframe requests remain unqualified.

Native properties freeze USD 5,000 capital, 10% equity sizing, pyramiding 10,
commission 0.04%, zero slippage, default four historical ticks, bar close/realtime
execution, requested limit price, one-tick delay and infinite leverage. Runtime
metadata is USD, grid `1/100000`, integer quantity precision and point value 1.

| History | GBPUSD four-hour | EURUSD weekly |
| --- | --- | --- |
| Native rows | 21,347 | 2,908 |
| Confirmed bars | 21,346 | 2,907 |
| First bar, UTC | 2013-01-02 02:00 | 1971-01-03 22:00 |
| Last confirmed bar, UTC | 2026-09-29 05:00 | 2026-09-20 21:00 |
| Forming bar excluded, UTC | 2026-09-29 09:00 | 2026-09-27 21:00 |
| Native trade cutoff, UTC+8 | 2026-09-29 17:00 | 2026-09-28 |

Raw native CSVs are retained in full. Only the final forming bar is removed
from runtime input, copying OHLCV exactly with seconds-to-milliseconds conversion
and no resampling. These are different histories, not a single-parameter causal
comparison. Four-hour qualification is limited to the available/exported 2013
onward history. Compared with the earlier CF-disabled Kijun captures, GBPUSD
confirmed bars are identical; EURUSD's last confirmed weekly volume was revised
from 1425864 to 1420125. Neither old capture is used as an identical-input
single-parameter control in this batch.

Confirmed bars SHA-256:

- GBPUSD: `bc6931f5410350092c64a2b323945f118d52e5e24024614fe896977194c32662`.
- EURUSD: `10f32c9551c4d6061c15d34daa2eb74af16290dc341f98c642ae89b86f56d1ae`.

Downloads in `I:\sys\下载`:

| Case | Chart CSV | Trade CSV | Download times, UTC+8 |
| --- | --- | --- | --- |
| GBPUSD four-hour | `FX_GBPUSD, 240 (3).csv` | `SSL_Hybrid_Strategy_FX_GBPUSD_2026-09-29 (5).csv` | 19:50:38 / 19:50:00 |
| EURUSD weekly | `FX_EURUSD, 1W (19).csv` | `SSL_Hybrid_Strategy_FX_EURUSD_2026-09-29 (15).csv` | 19:53:43 / 19:53:03 |

## Confirmed results

| Observation | GBPUSD four-hour | EURUSD weekly |
| --- | ---: | ---: |
| Nonblank observations | 149,072 | 19,999 |
| Series mismatches including NA positions | 0 | 0 |
| Closed trades | 1,162 | 45 |
| Explicit exit fills | 1,099 | 43 |
| Surviving entries | 0 | 0 |
| Final position | Flat, average null | Flat, average null |
| Forming-period exit / entry records excluded | 0 / 0 | 0 / 0 |

All eight columns match: Candle Size > 1xATR, MA Baseline, SSL1, Baseline
Upper Channel, Basiline Lower Channel, MA UP, MA DOWN and disabled second MA.
Numeric absolute tolerance remains `1e-8`; every missing position is checked.
Baseline/SSL1 retain 29 initial missing bars, channels 30, direction columns
116 in both histories. Startup values are not trimmed. Candle Size false/NA
exports as zero; the second MA is entirely missing. The unrelated Percent short
market probe's Plot is excluded.

Closed records match IDs, entry/exit directions, displayed dates/times, prices,
exact quantities, entry values, durations and displayed net PnL/commission.
Monetary display tolerance stays 0.005; maximum PnL differences are
0.004996400/0.004542464 and commission 0.004289960/0.002739888.
Quantity difference is zero. Explicit exit multisets match IDs, dates/times,
prices and quantities. Four-hour trade timestamps retain UTC+8 minute precision;
weekly session dates do not qualify finer intraday timing. All native transactions
precede their forming-period cutoffs; current unrealized PnL and live ticks are
outside qualification. Both local final positions have size zero and null average.

## Execution and reproduction

Batch, incremental and historical realtime complete outputs match byte for byte
within each case, with zero exit codes, stderr and diagnostics:

| Case | Complete output SHA-256 |
| --- | --- |
| GBPUSD four-hour | `e7bd5474193e3b6f25c9d2579ace478e3445ed97a2ab82c94effc9635c9462f0` |
| EURUSD weekly | `86457b5a839fcceb70958a313b9a1ff47af3a3fbb92cb96713c6788a793e19c4` |

Core base `2f9a83a0c333092167c7253571f56ee82a7ec0d0`, patch
`827e195c3a7db4422bbd657a5634990adb9d8d1db69658f6331df41a97cb2636`,
immutable CLI `584ae27bb25fed15c038c5c55a9356720c1bff90b8db29ce8b403e8a08010f36`.
No new core repair. Current core/golden hashes match the passing VAMA full-gate
source; the existing gate is reused without a new full-gate execution.

Local evidence directories:

- `.local/ssl-hybrid-fx-gbpusd-fourhour-kijun30-cf-t3-20-hl2-20260929/`
- `.local/ssl-hybrid-fx-eurusd-weekly-kijun30-cf-t3-20-hl2-20260929/`

Each freezes native CSVs, full source, bars, input/property/export snapshots,
screenshots, CLI/patch receipts, three mode outputs, source analysis, comparison
and current-verification receipts. BTCUSD daily, HMA60/Close/divider1/Power1,
CF disabled/type1 was restored and recorded. Chrome transient transport failure
was recovered using the previously approved blank-window action.

Run each directory's `run_modes.py`, then:

```powershell
python .local/verify_t3_cross_market_20260929.py
```

Verification log: `.local/t3-cross-market-verification-20260929.log`.
Shared helper hashes are frozen in current verification receipts. These captures
qualify the named original-source/settings/metadata/history combinations only.
Hidden MA numeric values, different timeframe requests, live ticks, unrelated
scripts and the earlier Hull v4 quantity discrepancy remain unqualified.
No commit or push was performed; the expansion goal remains active.

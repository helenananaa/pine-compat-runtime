# SSL Hybrid EURUSD daily: disabled TP/SL and fixed stop

Two independent TradingView chart and trade exports were captured through
authenticated Chrome on 2026-09-29 for the unchanged public Pine v5
[SSL Hybrid Strategy](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/).
Source SHA-256: `1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.
This extends indicator and entry-filter coverage to two exit-order paths.

## Frozen inputs and native history

Both cases use FXCM `FX:EURUSD`, daily `1D`, Kijun v2 baseline length 30 /
divider 3, HL2, original JMA SSL2 length 5 / Phase 3 / Power 2 and HMA exit
length 15. CF Ultimate MA MTF is enabled with primary T3 type 8, length 20,
factor input 7 (0.7), direction interval 2, current chart resolution enabled
and second MA disabled. Full input snapshots differ only at the following
two checkboxes after removal of transient focus/spin controls:

| Setting | No TP/SL | Fixed SL |
| --- | --- | --- |
| Use TP & SL, original call site 669 | false | true |
| Move SL on TP1, original call site 671 | true | false |

The original source gates initial `strategy.exit` submissions on Use TP & SL.
TP1 stop rewrites require both switches. Thus the first case retains an inert
move-stop input while closing trades through opposite entry orders; the second
submits TP/SL orders and disables the TP1 stop rewrite. The pair changes two
inputs and does not isolate a single parameter's causal effect.

Common original call-site overrides: `12=Kijun v2`, `13=30`, `18=hl2`,
`20=3`, `21=2`, `19=3`, `587=true`, `589=20`, `590=7`, `591=8`,
`594=2`, `652=true`. Add `669=false,671=true` or `669=true,671=false`.
Fresh analysis of the original source verifies these input identities.

Native properties retain USD 5,000 capital, 10% equity sizing, pyramiding 10,
commission 0.04%, zero slippage, default historical execution and infinite
leverage. Runtime metadata is USD, grid `1/100000`, integer quantity precision
and point value 1. Input/property/export snapshots are retained.

Each native chart contains 14,327 rows. The 14,326 confirmed rows span
1971-01-03 22:00 through 2026-09-27 21:00 UTC. The forming row at
2026-09-28 21:00 UTC is retained in the raw export and excluded from runtime
input; the native trade cutoff is 2026-09-29 UTC+8. There are no forming-period
trade records in either capture. OHLCV is copied without resampling, converting
timestamps from seconds to milliseconds only.

Both confirmed bar files are byte-identical, SHA-256:
`a1726fb8ec50390b9cfb5bf755a8588b209412d1e6896d6a53d33b956b14a1a2`.
Compared with the earlier daily T3-0.7 capture, one open is represented as
`1.1482` instead of `1.1481999999999999` (bar 14,320, difference about
2.22e-16); all other OHLCV fields agree. That older capture is not used as a
byte-identical single-parameter control in this batch.

Downloads are preserved in `I:\sys\下载` and copied with matching SHA receipts:

| Case | Chart CSV | Trade CSV | Download times, UTC+8 |
| --- | --- | --- | --- |
| No TP/SL | `FX_EURUSD, 1D (14).csv` | `SSL_Hybrid_Strategy_FX_EURUSD_2026-09-29 (16).csv` | 20:01:59 / 20:01:28 |
| Fixed SL | `FX_EURUSD, 1D (15).csv` | `SSL_Hybrid_Strategy_FX_EURUSD_2026-09-29 (17).csv` | 20:03:47 / 20:03:09 |

## Confirmed results

| Observation | No TP/SL | Fixed SL |
| --- | ---: | ---: |
| Nonblank observations | 99,932 | 99,932 |
| Series mismatches, including NA positions | 0 | 0 |
| Closed trades | 190 | 193 |
| Explicit exit-order fills | 0 | 177 |
| Surviving entries | 5 | 2 |
| Final position size | -435 | -86 |
| Average entry price | 1.1482 | 1.1482 |

All eight exported columns match: Candle Size > 1xATR, MA Baseline, SSL1,
Baseline Upper Channel, Basiline Lower Channel, MA UP, MA DOWN and disabled
second MA. Both native cases have identical series. Baseline/SSL1 start after
29 missing bars, channels after 30 and CF direction after 116. Every missing
position is checked, with no startup trimming. Numeric absolute tolerance
remains `1e-8`; false/NA Candle Size exports as zero and the second MA is
entirely missing. The unrelated Percent short market probe's Plot is excluded.

Closed records match entry/exit IDs, directions, displayed session dates,
prices, exact quantities, entry values, durations and displayed PnL/commission.
Native no-TP/SL exit signals are exclusively LongEntry1 and ShortEntry1.
Explicit exit multisets match IDs, dates, prices and quantities, including the
first case's absence of explicit exits. Monetary display tolerance stays 0.005;
maximum PnL differences are 0.004886620 / 0.004989640 and commission differences
0.004666640 / 0.002198504. Quantity differences are zero.

All surviving shorts entered on 2026-09-21 at 1.1482. No-TP/SL has
ShortEntry1..5 quantities 87/131/131/43/43; fixed-SL retains ShortEntry4/5,
43 each. Individual open-entry price differences are zero. Weighted position
averages differ only in floating-point representation (first local average
1.1482000000000003). Current unrealized PnL and live ticks are not qualified.

## Execution, receipts and scope

Batch, incremental and historical realtime complete outputs agree byte for
byte within each case. All six executions exit zero with empty stderr and
no diagnostics:

| Case | Complete output SHA-256 |
| --- | --- |
| No TP/SL | `32a77d48e9df5f9070ee5d77958ae535bdb2f0431a025fe9eed51f714de54db8` |
| Fixed SL | `5e93303bcaae6faa9e0c1b35955f4dfe246fe9ddc4609a9a051273c197bb2fac` |

Core base `2f9a83a0c333092167c7253571f56ee82a7ec0d0`, patch
`827e195c3a7db4422bbd657a5634990adb9d8d1db69658f6331df41a97cb2636`,
immutable CLI `584ae27bb25fed15c038c5c55a9356720c1bff90b8db29ce8b403e8a08010f36`.
No new core repair was needed. Current core/golden hashes match the passing
VAMA full-gate receipt, which is reused without a new full-gate execution.

Local evidence directories:

- `.local/ssl-hybrid-fx-eurusd-daily-kijun30-cf-t3-no-tp-sl-hl2-20260929/`
- `.local/ssl-hybrid-fx-eurusd-daily-kijun30-cf-t3-fixed-sl-hl2-20260929/`

Each freezes original source, native CSVs, bars, screenshots, settings,
CLI/patch, source analysis, all three mode outputs, comparison results and
current verification receipts. BTCUSD daily with HMA60/Close/divider1/Power1,
CF disabled/type1 and both exit switches enabled was restored and recorded.

Run each directory's `run_modes.py`, then:

```powershell
python .local/verify_eurusd_daily_exit_parameters_20260929.py
```

Verifier log: `.local/eurusd-daily-exit-parameters-verification-20260929.log`.
Shared helper hashes are frozen in current verification receipts. Qualification
is limited to these original-source/settings/metadata/history combinations.
Hidden MA numeric values, different timeframe requests, live ticks, unrelated
scripts and the earlier Hull v4 quantity discrepancy remain unqualified.
No commit or push was performed; the expansion goal remains active.

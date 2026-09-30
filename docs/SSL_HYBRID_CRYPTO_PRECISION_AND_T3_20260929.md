# SSL Hybrid: crypto quantity precision and BTCUSD T3 expansion

Verified 2026-09-29 against unchanged original Pine v5
[SSL Hybrid Strategy](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/),
source SHA-256 `1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.
This batch adds one fresh authenticated Chrome capture and strengthens two
existing native captures on the current core with explicit quantity metadata.
The original captures and their historical receipts remain intact.

## Fresh BTCUSD Kijun / T3 capture

Coinbase `COINBASE:BTCUSD`, daily `1D`: Kijun v2 baseline length 30,
divider 3, HL2; original JMA SSL2 length 5 / Phase 3 / Power 2;
HMA exit length 15. CF Ultimate MA MTF is enabled with primary T3 type 8,
length 20, factor input 7 (0.7), direction interval 2, current chart resolution
enabled and second MA disabled. Both TP/SL and Move SL on TP1 are enabled.
This extends the [FX T3 matrix](SSL_HYBRID_T3_CROSS_MARKET_20260929.md)
to a crypto market with fractional quantities.

Original call-site overrides: `12=Kijun v2`, `13=30`, `18=hl2`, `20=3`,
`21=2`, `19=3`, `587=true`, `589=20`, `590=7`, `591=8`, `594=2`,
`652=true`, `669=true`, `671=true`. Original-source analysis and native full
input snapshots are retained. CF reads chart close independently of HL2.
Same-symbol, same-period request.security is exercised; different timeframe
requests and the hidden numeric MA remain unqualified.

Native properties retain USD 5,000 capital, 10% equity sizing, pyramiding 10,
0.04% commission, zero slippage, default four historical ticks, bar-close /
realtime execution, requested limit price, one-tick delay and infinite leverage.
Host-neutral chart metadata is USD, price grid `1/100`, quantity precision 6,
point value 1. The native quantities use six decimal places, including values
that cannot be represented at five; every local closed quantity matches them.

The raw chart has 4,287 rows. Its 4,286 confirmed daily bars span
2014-12-01 through 2026-09-28 UTC. The forming 2026-09-29 bar is retained
in the raw export and excluded from runtime input. No forming-period trade
records occur in this capture. Bars are copied without resampling, converting
timestamps from seconds to milliseconds only. Confirmed bars SHA-256:
`380f9c2f4fe416214878d98d7e0b273670d3bd7d6983199e4cd0bb0732e43911`.

Native downloads in `I:\sys\下载`:

- `COINBASE_BTCUSD, 1D (88).csv`, 20:22:38 UTC+8,
  SHA `070f25f5d731c48f83e571315e311d06f1d28d26c429debed7efdd9c6ef7a6d2`.
- `SSL_Hybrid_Strategy_COINBASE_BTCUSD_2026-09-29.csv`, 20:20:56 UTC+8,
  SHA `c48170df5eafb4d7e5ce75f7ab4d3c23a0a39eff02437267c62d739b3ed140f2`.

Full CSVs, input/property/export/chart snapshots and screenshots are frozen in
`.local/ssl-hybrid-coinbase-btcusd-daily-kijun30-cf-t3-20-hl2-20260929/`.
After recovery from a Chrome transport timeout and user plugin reinstallation,
the browser remained in Chrome. HMA60/Close/divider1/Power1, CF disabled/type1
and both exit switches enabled were restored on BTCUSD daily and recorded.

## Current results and stronger historical comparisons

| Case | Fresh capture | Confirmed bars | Nonblank observations | Closed trades | Explicit exits | Surviving entries | Final long position |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| BTCUSD HMA30 / Close | No, September 27 | 4,284 | 29,814 | 868 | 487 | 2 | 0.001146 at 80875.04 |
| ETHUSD HMA30 / Close | No, September 27 | 3,779 | 26,279 | 786 | 432 | 4 | 0.17 at 2611.38 |
| BTCUSD Kijun30 / T3 / HL2 | Yes, September 29 | 4,286 | 29,652 | 249 | 215 | 1 | 0.000747 at 73010.86 |

All eight named native indicator columns match in each case at absolute
numeric tolerance `1e-8`, including every missing position. HMA30 baseline,
SSL1 and channels retain 33 initial missing bars, direction 21; new Kijun
baseline/SSL1 retain 29, channels 30 and T3 direction 116. Candle Size false/NA
exports as zero, and disabled second MA is entirely missing. The unrelated
Percent short market probe's Plot is excluded.

The old BTC/ETH cases use original HMA30/Close inputs, JMA Power 1,
CF disabled and both exit switches enabled. Their current commands supply
quantity precision 6/4, USD, price grid `1/100`, point value 1 and override
`13=30`. Capture provenance checks every original hash-manifest file and
the old receipt hashes before copying to separate new directories:

- `.local/ssl-hybrid-btcusd-daily-hma30-precision-requalification-20260929/`
- `.local/ssl-hybrid-ethusd-daily-hma30-precision-requalification-20260929/`

These are current-source reruns of the
[BTC](SSL_HYBRID_LENGTH30_BTCUSD_DAILY_20260927.md) and
[ETH](SSL_HYBRID_LENGTH30_ETHUSD_DAILY_20260927.md) captures, not new settings
or browser exports. Original BTC evidence contains source, chart, trades and
screenshot; ETH additionally contains full input/property panel text. Each
uses its own frozen OHLCV, not revised data from another capture. The fresh
Kijun/T3 case differs in multiple settings and history and is not a causal
single-parameter control for these HMA captures.

Closed comparisons now check entry/exit IDs, directions, native UTC session
dates, prices, quantities, durations, entry values and display PnL/commission.
Open comparisons check unique IDs, dates, direction, price and quantities;
explicit exit multisets match IDs, dates, prices and quantities. Price
tolerance remains `1e-8`, quantity tolerance `1e-10` (observed differences zero),
and monetary display tolerance 0.005. Open-entry price differences are zero.

Maximum display PnL differences are 0.004991886 / 0.004991694 / 0.004981189;
commission differences are 0.004997189 / 0.004993948 / 0.004972525.
The older reports had quantity/PnL differences beyond these bounds. Current
core plus explicit quantity metadata satisfies the tighter checks; this batch
does not isolate which earlier core change or metadata choice caused each
improvement and makes no claim of undisplayed internal TradingView precision.

BTC native Size (value) uses up to ten significant digits: values around
100 retain seven decimal places, values below 100 up to eight, with trailing
zeros omitted. The old fixed `1e-8` entry-value comparator incorrectly rejected
valid rounding near 100. The BTC helper checks the half quantum implied by
ten significant digits at each native magnitude, plus four ULPs for arithmetic
representation. Maximum BTC entry-value differences are about `5e-8` in both
cases. No price, quantity or PnL bound was loosened. ETH's entry values fit
the export directly and retain `1e-8`, maximum difference `2.84e-14`.

## Execution and evidence chain

All nine runs exit zero with empty stderr and no diagnostics. Batch,
incremental and historical realtime complete JSON is byte-identical within
each case:

| Case | Complete output SHA-256 |
| --- | --- |
| BTC HMA30 | `7a2de337e1c1db1a533d0711d081593b9ba429cb198bac7668983449913510a5` |
| ETH HMA30 | `c8750260690d9fd5adc4975c73fd2ad6d677316287e9cbe74e59e1430244eab8` |
| BTC Kijun/T3 | `8c0cee3dc3610678a6732e35cda1b1f639af4338d681feaf79008c8959904792` |

Base HEAD `2f9a83a0c333092167c7253571f56ee82a7ec0d0`, current core patch
`827e195c3a7db4422bbd657a5634990adb9d8d1db69658f6331df41a97cb2636`,
immutable CLI `584ae27bb25fed15c038c5c55a9356720c1bff90b8db29ce8b403e8a08010f36`.
No new core repair. All current core/golden hashes match the previously
passing VAMA full gate, which is reused without another full-gate execution.
External market data stays in local evidence fixtures, outside the runtime.

Each directory has terminal mode receipts, fresh analysis, comparison and
current verification receipts. The shared verifier validates exact commands,
metadata, source/CLI/patch hashes, native CSV copies, OHLCV, startup values,
comparison coverage and capture provenance. Run each directory's `run_modes.py`,
then from the repository root:

```powershell
python .local/verify_crypto_precision_and_t3_20260929.py
```

Aggregate receipt `.local/crypto-precision-t3-matrix-20260929.json`;
log `.local/crypto-precision-t3-verification-20260929.log`.
Shared helper hashes are frozen in the receipts. Qualification applies to
these original-source/settings/metadata/history combinations. Live ticks,
forming bars, different timeframe requests, arbitrary scripts and the unresolved
Hull v4 discrepancy remain outside the proven scope. No commit or push;
the expansion goal remains active.

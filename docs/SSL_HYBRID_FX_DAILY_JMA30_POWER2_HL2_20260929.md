# SSL Hybrid Strategy: GBPUSD and EURUSD daily JMA 30 Power 2 HL2

Two new native TradingView captures were obtained through authenticated
Chrome on 2026-09-29. Both use the unchanged public Pine v5
[SSL Hybrid Strategy by kevinmck100](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/).
Source SHA-256: `1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.

## Frozen setup

FXCM `FX:GBPUSD` and `FX:EURUSD`, daily (`1D`), USD, price grid `1/100000`,
integer quantity precision, point value 1. Identical input overrides are
`12=JMA`, `13=30`, `18=hl2`, `20=3`, `21=2`. Other inputs remain original,
including JMA SSL2 length 5, HMA exit length 15, the original five targets
and stop, and the original 2021-08-01 to 2030-10-01 trading range.

Native inputs and properties are frozen independently per symbol:
USD 5,000 capital, 10% equity sizing, pyramiding 10, 0.04% commission,
zero slippage, default four historical ticks, on bar close/realtime tick,
requested limit price, one-tick order delay, infinite long/short leverage.

The captured histories begin at 1971-01-03 22:00 UTC and end with the
confirmed bar at 2026-09-27 21:00 UTC (Sep 28 session). The forming bar
at 2026-09-28 21:00 UTC (Sep 29 session), Unix `1790629200`, is excluded.
Native exports contain 14,322 GBPUSD rows and 14,327 EURUSD rows;
the runtime receives **14,321 / 14,326 confirmed bars**, respectively.
Each uses its own original OHLCV, including its own calendar and quantities.
The five-row count difference is retained without resampling or padding.

Downloads in `I:\sys\下载`:

| Symbol | Chart CSV | Trade CSV | Download times, UTC+8 |
| --- | --- | --- | --- |
| GBPUSD | `FX_GBPUSD, 1D.csv` | `SSL_Hybrid_Strategy_FX_GBPUSD_2026-09-29 (2).csv` | 17:12:42 / 17:12:25 |
| EURUSD | `FX_EURUSD, 1D.csv` | `SSL_Hybrid_Strategy_FX_EURUSD_2026-09-29.csv` | 17:15:59 / 17:15:43 |

The full original source, original native CSVs, settings, chart state,
export options, screenshots, confirmed bars and immutable CLI/core receipts
are preserved per symbol. The unrelated saved Percent short market probe's
`Plot` column is excluded from SSL Hybrid comparison.

## Confirmed native results

| Observation | GBPUSD daily | EURUSD daily |
| --- | ---: | ---: |
| Confirmed bars | 14,321 | 14,326 |
| Nonblank exported observations | 100,145 | 100,180 |
| Series mismatches, including missing positions | 0 | 0 |
| Closed trades | 484 | 484 |
| Explicit exit fills | 295 | 293 |
| Surviving open entry records | 1 | 1 |
| Final position | -37 at 1.35312 | -43 at 1.15778 |
| Maximum displayed net PnL difference | 0.004990344 | 0.004991632 |
| Maximum displayed commission difference | 0.002373180 | 0.002881752 |

All eight selected native columns match: Candle Size > 1xATR, MA Baseline,
SSL1, Baseline Upper Channel, Basiline Lower Channel, MA UP, MA DOWN,
and the disabled 2nd Multi-TimeFrame Moving Average. Absolute numeric
tolerance is `1e-8`; missing positions are checked explicitly. Candle Size
false/NA exports as zero. The disabled second MA is entirely missing;
this does not qualify enabling it.

Baseline and SSL1 are defined from zero-based bar **0**, channels from **30**,
MA UP/DOWN from **21**, in native and local output. Original recursive JMA
zero initialization and startup transients remain in the comparison.

Closed trades match original entry IDs, entry/exit directions, UTC+8 session
dates, prices, quantities, entry values, durations and displayed net
PnL/commission. Display monetary tolerance remains 0.005. Quantities have
zero difference. Explicit exit multisets match IDs, dates, prices and
quantities. Daily native exports provide session dates; this receipt makes
no finer intraday timestamp claim.

Each current native open record is `ShortEntry5`, entered on 2026-08-31.
Its ID, direction, date, price and quantity match a local filled entry;
the aggregate quantity and average price match the final local position.
All executed transaction dates precede the forming day. The forming day's
indicator values, unrealized PnL and live ticks are excluded.

These are two symbol datasets with separate native qualification. Equal
closed-trade counts do not establish equal executions: their explicit exit
counts, remaining quantities and prices differ, and both are checked
against their own files. This extends the previous
[AUDUSD four-hour JMA cases](SSL_HYBRID_FX_AUDUSD_FOURHOUR_JMA30_HL2_20260929.md)
to two additional currencies and daily execution.

## Execution, current source and reproduction

All six executions exit zero without stderr or diagnostics. Batch,
incremental and historical realtime complete outputs are byte-identical
within each symbol:

| Symbol | Complete output SHA-256 |
| --- | --- |
| GBPUSD | `4f613de83258c9c9854a184805de53931ddfd67fbf957090db4ad007ecbc09f9` |
| EURUSD | `0ea8e4148cf7a9fdde4ec4673296a1b69691c68b4975e79bdfab7d4491fbef79` |

Core base `2f9a83a0c333092167c7253571f56ee82a7ec0d0`, core patch
`827e195c3a7db4422bbd657a5634990adb9d8d1db69658f6331df41a97cb2636`,
immutable CLI `584ae27bb25fed15c038c5c55a9356720c1bff90b8db29ce8b403e8a08010f36`.
No additional core repair was needed. Current HEAD/core/golden hashes match
the earlier passing VAMA gate (1,984 runtime tests, 242 CLI tests, workspace
clippy/fmt, WASM Node, host parity, 130 tool tests and 774 wheel tests).
That identical-source gate is reused, without a new full-gate run.

Evidence directories:

- `.local/ssl-hybrid-fx-gbpusd-daily-jma30-power2-hl2-20260929/`
- `.local/ssl-hybrid-fx-eurusd-daily-jma30-power2-hl2-20260929/`

Preparation: `.local/prepare_fx_daily_jma_20260929.py GBPUSD` or `EURUSD`.
Run each case's `run_modes.py` and `compare.py`, then
`.local/verify_fx_daily_jma_20260929.py`. The verifier checks current
HEAD/core/golden and CLI identity, full-gate receipt/log hashes, source
input call sites from fresh analysis, native download hashes, exact
OHLCV translation, startup, mode arguments/full output hashes, fresh
comparisons, original open entry fields, final positions, forming-day
exclusion and restored browser state. Each case retains `analysis.json`,
mode receipts, `native-comparison.log`, `comparison.json` and
`current-verification.json`. The shared run log is
`.local/fx-daily-jma-verification-20260929.log`.

| Symbol | Native chart SHA-256 | Native trades SHA-256 | Confirmed bars SHA-256 |
| --- | --- | --- | --- |
| GBPUSD | `f5601ace13b08d79e35997406844c51fc84996b2e2c1e2bce0bb45fc2069ce43` | `9a43d38c10a59a8f01798670d85de337679e1b1371a9685c807b2aff9d1b4088` | `2239fce24c3a982ba46645dc839daf1161b9ad0463f12107abfc3f4ca710efc6` |
| EURUSD | `e6921399f622caf08456a27dba3c97714390df7c0b20fc13698436bc96589da8` | `4ca3d69effcc8096c903e1a88a36acf79422701d67e4211facec3e598556dbb3` | `eb0e9f198c621f06fb6dd88b3c2a34f56c5129270a9d79d9d61f5fb653c05dd7` |

Chrome was restored to Coinbase BTCUSD daily, HMA 60 close, Phase 3,
Power 1 and volatility window 10, with screenshot/state evidence.
This receipt qualifies the unchanged source, frozen settings/history and
exported fields. Historical realtime equality does not establish native
live-tick parity or arbitrary-script compatibility.

# SSL Hybrid: ETHUSD T3 daily and weekly expansion

Verified 2026-09-29 with two fresh authenticated Chrome captures of the unchanged
original Pine v5 [SSL Hybrid Strategy](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/).
Original source SHA-256:
`1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.
This extends the [BTCUSD T3 capture](SSL_HYBRID_CRYPTO_PRECISION_AND_T3_20260929.md)
to Coinbase ETHUSD daily and weekly histories, including four-decimal quantities
and a weekly confirmed boundary with surviving short entries.

## Native settings and data

Both captures use Kijun v2 baseline length 30 / divider 3 / HL2,
original JMA SSL2 length 5 / Phase 3 / Power 2, and HMA exit length 15.
CF Ultimate MA MTF is enabled: primary T3 type 8, length 20, factor input
7 (0.7), direction interval 2, current chart resolution enabled, second MA
disabled. TP/SL and Move SL on TP1 are both enabled. The complete native
Inputs dialogs are equal after removing the focus marker. Each period has
its own frozen OHLCV; this is coverage across periods, not an isolated
parameter sensitivity experiment.

Exact original call-site overrides: `12=Kijun v2`, `13=30`, `18=hl2`, `20=3`,
`21=2`, `19=3`, `587=true`, `589=20`, `590=7`, `591=8`, `594=2`,
`652=true`, `669=true`, `671=true`. CF reads chart close independently of HL2.
Same-symbol, same-period request.security is exercised.

Native properties: USD 5,000 initial capital, 10% equity sizing, pyramiding 10,
0.04% commission, zero slippage, default four historical ticks, bar-close /
realtime execution, requested limit price, one-tick delay and infinite leverage.
Runtime chart metadata: `COINBASE:ETHUSD`, `1D` / `1W`, USD, price grid
`1/100`, quantity precision 4, point value 1. All native quantities fit four
decimals and each capture contains quantities requiring the fourth decimal.

| Period | Raw rows | Confirmed bars | First confirmed UTC | Last confirmed UTC | Excluded forming bar UTC |
| --- | ---: | ---: | --- | --- | --- |
| Daily | 3,782 | 3,781 | 2016-05-23 | 2026-09-28 | 2026-09-29 |
| Weekly | 541 | 540 | 2016-05-23 | 2026-09-21 | 2026-09-28 |

Raw chart and trade CSVs are retained without alteration. Runtime bars copy
confirmed OHLCV exactly, converting timestamps from seconds to milliseconds
only. Coinbase date-only trade timestamps are compared as UTC session dates.

Native downloads in `I:\sys\下载`, times UTC+8 on September 29:

- Daily chart `COINBASE_ETHUSD, 1D (7).csv`, 20:33:37;
  SHA `b04d75c04642daffd66d64784f56ac1829a91999c8767c3b47a0b3035b9ab755`.
- Daily trades `SSL_Hybrid_Strategy_COINBASE_ETHUSD_2026-09-29.csv`, 20:32:45;
  SHA `606579dde2d801cfdce13bc590f15b13133aa485cc3211be03a66cdb81f5c36d`.
- Weekly chart `COINBASE_ETHUSD, 1W (8).csv`, 20:36:52;
  SHA `b25174b8523a97049b61ead4e001b80b4df5520fc76ac46a0cbf189cbccc65a2`.
- Weekly trades `SSL_Hybrid_Strategy_COINBASE_ETHUSD_2026-09-29 (1).csv`, 20:35:16;
  SHA `df4f9988c4569e44b416728d0bcb1041999b6c68ad4400706bcc7ac6798d032b`.

Confirmed bars SHA-256: daily
`024920aa0087bb8be84e41588bc749811d8a54f190fa881be5f87bbe7f9657af`, weekly
`29c62660df6214947e5b79a42bcbaf1f3f30632458c8aa1111e0065adee75ca1`.

## Confirmed results

| Period | Nonblank indicator observations | Closed trades | Explicit exit fills | Surviving entries | Boundary position |
| --- | ---: | ---: | ---: | ---: | --- |
| Daily | 26,117 | 245 | 223 | 0 | Flat, average price null |
| Weekly | 3,430 | 43 | 39 | 2 | Short 0.0312 ETH at 3143.15 |

All eight named native series match at absolute tolerance `1e-8`, including
every missing position. Baseline and SSL1 retain 29 initial missing bars,
channels 30, CF direction 116. Disabled second MA remains entirely missing;
Candle Size false/NA is exported as zero. The unrelated Percent short market
probe's Plot is excluded.

Closed comparisons check IDs, direction, entry/exit dates, prices, quantities,
duration, entry value and displayed PnL/commission. Surviving entries check
unique IDs, dates, direction, price and quantity; exit multisets check IDs,
dates, prices and quantities. Quantity tolerance is `1e-10`, with observed
difference zero; price tolerance is `1e-8`. Entry-value tolerance stays `1e-8`,
maximum observed difference `2.842170943040401e-14` in each case.

Daily maximum displayed net PnL / commission differences are
`0.004960179199995807` / `0.004998932799999994`.
Weekly maxima are `0.0049209036000092965` / `0.0049614543999999955`.
All satisfy the 0.005 native monetary display tolerance.

### Weekly forming-period boundary

The raw weekly native report has 45 closed trades and five open entries.
Two exits and five new entries belong to the forming week beginning September
28, so they are retained in raw evidence and excluded from confirmed history.
At the end of the September 21 weekly bar, 43 trades are closed and two older
short entries survive. Those entries are reconstructed from the native trade
records whose exits occur during the forming week: ShortEntry4 and
ShortEntry5, both entered January 5, 2026 at 3143.15, quantity 0.0156 each.
Their combined position is -0.0312 at 3143.15. The daily capture has no
forming-period entry or exit records. This qualification does not validate
the forming week's execution.

## Execution and reproducibility

All six runs terminate with exit zero, empty stderr and zero diagnostics.
Batch, incremental and historical realtime complete JSON is byte-identical
within each period:

| Period | Complete output SHA-256 |
| --- | --- |
| Daily | `343fe79ee51adc40f13c29ffb3b9afd363b3a591b27521ac349be10e6ebc1190` |
| Weekly | `8d7971008fa4f9c3ab5873d2f00e3cfca3c01080275167ea2d88b44d27e12b72` |

Base HEAD `2f9a83a0c333092167c7253571f56ee82a7ec0d0`, core patch
`827e195c3a7db4422bbd657a5634990adb9d8d1db69658f6331df41a97cb2636`,
CLI `584ae27bb25fed15c038c5c55a9356720c1bff90b8db29ce8b403e8a08010f36`.
No new core repair. Current source, core and golden hashes match the passing
VAMA full gate, which is reused without another full-gate execution. External
market data remains in evidence fixtures, outside the runtime core.

Frozen directories:

- `.local/ssl-hybrid-coinbase-ethusd-daily-kijun30-cf-t3-20-hl2-20260929/`
- `.local/ssl-hybrid-coinbase-ethusd-weekly-kijun30-cf-t3-20-hl2-20260929/`

Each retains original source, executable, patch, terminal mode receipts, native
CSVs, full input/property/export/chart snapshots, screenshot, analysis,
comparison and current verification receipt. The browser's original
HMA60/Close/divider1/Power1 and disabled CF/type1 settings were restored on
BTCUSD daily, with both exit switches enabled; restoration evidence is saved.

Run each directory's `run_modes.py`, then from the repository root:

```powershell
python .local/verify_eth_t3_cross_period_20260929.py
```

Aggregate receipt `.local/eth-t3-cross-period-matrix-20260929.json`;
terminal verification log `.local/eth-t3-cross-period-verification-20260929.log`.
The verifier checks exact commands and metadata, source/core/CLI/golden hashes,
native CSV copies, complete OHLCV, startup values, equal native input panels,
all comparisons, forming-record partition and restored browser state. Shared
helper hashes and frozen artifact hashes are stored in each receipt.

Qualification applies to these source/settings/metadata/history combinations.
Live ticks, forming bars, different-timeframe requests, arbitrary scripts and
the unresolved Hull v4 discrepancy remain outside the proven scope. No commit
or push; the expansion goal remains active.

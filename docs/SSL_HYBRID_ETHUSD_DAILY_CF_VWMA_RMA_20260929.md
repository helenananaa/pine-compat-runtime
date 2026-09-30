# SSL Hybrid: ETHUSD daily enabled CF VWMA / RMA

Verified 2026-09-29 using two fresh authenticated Chrome native exports of
the unchanged original Pine v5
[SSL Hybrid Strategy](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/).
Original source SHA-256:
`1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.
This adds enabled CF primary VWMA and RMA branches to the full-script matrix,
following [ETHUSD T3 daily / weekly](SSL_HYBRID_ETHUSD_T3_CROSS_PERIOD_20260929.md).

## Controlled native inputs

Both cases use `COINBASE:ETHUSD`, daily `1D`, Kijun v2 baseline length 30,
divider 3, HL2; original JMA SSL2 length 5 / Phase 3 / Power 2; HMA exit
length 15. CF Ultimate MA MTF is enabled, primary length 20, direction
comparison interval 2, current chart resolution enabled, second MA disabled.
TP/SL and Move SL on TP1 are enabled. CF primary type is 5 (VWMA) or 6 (RMA).
The retained T3 factor input 7 has no effect on the selected VWMA/RMA branch.
CF reads chart close independently of the baseline's HL2 source.

The complete native Inputs panels differ only in primary CF type after removing
focus markers and the focused numeric field's Increase/Decrease buttons.
Confirmed OHLCV is byte-identical between captures. The other six compared
indicator columns are identical; each direction column changes at 544 bars.
These controls support a native-qualified CF type comparison. The unplotted
numeric primary MA itself is not directly qualified by this capture.

Exact original call-site overrides: `12=Kijun v2`, `13=30`, `18=hl2`, `20=3`,
`21=2`, `19=3`, `587=true`, `589=20`, `590=7`, `591=5` / `591=6`,
`594=2`, `652=true`, `669=true`, `671=true`.

Native properties: USD 5,000 capital, 10% equity sizing, pyramiding 10,
0.04% commission, zero slippage, default four historical ticks, bar-close /
realtime execution, requested limit price, one-tick delay and infinite leverage.
Host-neutral chart metadata: USD, price grid `1/100`, quantity precision 4,
point value 1. Each case includes native quantities requiring the fourth decimal.
Same-symbol, same-period request.security is exercised; market data remains
an external fixture, outside the interpreter core.

## Frozen native data

Each chart export contains 3,782 rows. Confirmed input is 3,781 bars from
2016-05-23 through 2026-09-28 UTC; the forming 2026-09-29 row is retained
in the raw export and excluded from input. Neither trade capture contains
forming-period entry or exit records. All confirmed OHLCV fields are copied
exactly, with timestamps converted from seconds to milliseconds only.
Native date-only trades are compared as UTC session dates.

Shared confirmed bars SHA-256:
`bc03bb086c4fecfa8a3732b5bc3d4327da4cc43d8bd5324bff81753a78386f90`.
These fixtures are independent of the earlier T3 capture and its bars hash;
no assumption of unchanged provider history across batches is required.

Native downloads in `I:\sys\下载`, September 29, UTC+8:

| Case | Chart download | Time | Trades download | Time |
| --- | --- | --- | --- | --- |
| VWMA | `COINBASE_ETHUSD, 1D (8).csv` | 20:47:35 | `SSL_Hybrid_Strategy_COINBASE_ETHUSD_2026-09-29 (2).csv` | 20:47:16 |
| RMA | `COINBASE_ETHUSD, 1D (9).csv` | 20:49:38 | `SSL_Hybrid_Strategy_COINBASE_ETHUSD_2026-09-29 (3).csv` | 20:49:23 |

Full CSVs, source, metadata, input/property/export/chart snapshots and screenshots
are frozen separately in:

- `.local/ssl-hybrid-coinbase-ethusd-daily-kijun30-cf-vwma20-hl2-20260929/`
- `.local/ssl-hybrid-coinbase-ethusd-daily-kijun30-cf-rma20-hl2-20260929/`

## Results

| CF type | Nonblank observations | Closed trades | Explicit exit fills | Surviving entries | Confirmed boundary position |
| --- | ---: | ---: | ---: | ---: | --- |
| VWMA20 | 26,307 | 479 | 315 | 1 | Long 0.0209 ETH at 2251.68 |
| RMA20 | 26,307 | 460 | 263 | 0 | Flat, average price null |

All eight named native columns match at absolute numeric tolerance `1e-8`,
including every missing position. Baseline and SSL1 retain 29 initial missing
bars, channels 30, direction 21. The disabled second MA is entirely missing.
Candle Size false/NA exports as zero. The unrelated Percent short market
probe's Plot is excluded.

Closed checks cover IDs, directions, entry/exit dates, prices, quantities,
duration, entry value and displayed PnL/commission. Open checks cover unique
entry ID, date, direction, price and quantity. Explicit exit multisets match
IDs, dates, prices and quantities; weighted boundary positions also match.
VWMA's remaining LongEntry5 entered 2026-08-20 at 2251.68, quantity 0.0209.

Quantity tolerance is `1e-10`, observed differences zero. Price tolerance is
`1e-8`, with open-entry difference zero. Entry-value tolerance remains `1e-8`,
maximum difference `2.842170943040401e-14` for both. Monetary display tolerance
is 0.005; maximum displayed net PnL / commission differences:

- VWMA: `0.004996190000001732` / `0.004999651999999993`.
- RMA: `0.004999323999999916` / `0.004966256000000002`.

## Execution and evidence chain

All six runs terminated with exit zero, empty stderr and no diagnostics.
Batch, incremental and historical realtime complete JSON is byte-identical
within each case:

| Case | Complete output SHA-256 |
| --- | --- |
| VWMA | `052c45a5060d9c1ba02760ea20d4781df38678c34870d6d4febb17b0f4553087` |
| RMA | `5fcb460a8319f877304108128fd218231058a474a9883ec71cb8fc91ebd846cf` |

Base HEAD `2f9a83a0c333092167c7253571f56ee82a7ec0d0`, core patch
`827e195c3a7db4422bbd657a5634990adb9d8d1db69658f6331df41a97cb2636`,
CLI `584ae27bb25fed15c038c5c55a9356720c1bff90b8db29ce8b403e8a08010f36`.
No new core repair. Current core and golden hashes match the previously
passing VAMA full gate, reused without another full-gate execution.

Each directory retains terminal mode receipts, analysis, comparison and current
verification receipts. The shared verifier checks exact commands, source/core/
CLI/golden hashes, native CSV copies, OHLCV equality, quantities, startup values,
input isolation, all comparisons and restoration to HMA60/Close/divider1/Power1,
CF disabled/type1, both exits enabled on BTCUSD daily. Native file hashes and
shared helper hashes are frozen in the receipts. Run each directory's
`run_modes.py`, then from the repository root:

```powershell
python .local/verify_eth_daily_cf_vwma_rma_20260929.py
```

Aggregate `.local/eth-daily-cf-vwma-rma-matrix-20260929.json`;
sensitivity `.local/eth-daily-cf-vwma-rma-sensitivity-20260929.json`;
terminal log `.local/eth-daily-cf-vwma-rma-verification-20260929.log`.
Qualification applies to these original-source/settings/metadata/history
combinations. Live ticks, forming bars, different-timeframe requests, arbitrary
scripts and the unresolved Hull v4 discrepancy remain outside the proven scope.
No commit or push; the expansion goal remains active.

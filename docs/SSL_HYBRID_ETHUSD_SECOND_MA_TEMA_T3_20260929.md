# SSL Hybrid: ETHUSD original second TEMA / T3 branches

Verified 2026-09-29 with two fresh authenticated Chrome native captures of
the unchanged Pine v5
[SSL Hybrid Strategy](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/).
Source SHA-256:
`1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.
This follows [second VWMA / RMA numeric qualification](SSL_HYBRID_ETHUSD_SECOND_MA_VWMA_RMA_20260929.md)
and covers two further original secondary-average branches.

## Original-source behavior

The second MA's TEMA branch uses `3 * (ema1 - ema2) + ema3`, referencing
the primary EMA chain computed with primary length 20. It does not reference
the separately calculated `sema1`, `sema2`, `sema3` at second length 50.
Thus the selected secondary TEMA is numerically a length-20 TEMA in this
unchanged public script, despite its second-length input being 50.

The second T3 branch uses `stilT3 = st3(source, length2, sfactor)` at second
length 50, factor input 7 (0.7). Its original `st3` body nests one `gd` and
two `sgd` calls; those functions have the same algebraic form. The runtime
executes these original definitions. No formula rewrite or source repair was
used to obtain agreement. The source-reference detail above is a property
of this script, not a change to interpreter semantics.

## Native inputs and properties

Coinbase `COINBASE:ETHUSD`, daily `1D`; Kijun v2 baseline length 30 /
divider 3 / HL2; original JMA SSL2 length 5 / Phase 3 / Power 2; HMA exit
length 15. CF is enabled with primary RMA type 6 / length 20 / direction
interval 2, current chart resolution enabled. Optional second MA is enabled,
input length 50, factor 7, type 7 (TEMA) or 8 (T3). TP/SL and Move SL
on TP1 are enabled. Crossing highlights and cross dots remain disabled.

Original call-site overrides: `12=Kijun v2`, `13=30`, `18=hl2`, `20=3`,
`21=2`, `19=3`, `587=true`, `589=20`, `590=7`, `591=6`, `594=2`,
`595=true`, `597=50`, `598=7`, `599=7` / `599=8`, `652=true`,
`669=true`, `671=true`.

Native properties: USD 5,000 initial capital, 10% equity sizing, pyramiding 10,
0.04% commission, zero slippage, default four historical ticks, bar-close /
realtime execution, requested limit price, one-tick delay and infinite leverage.
Host-neutral chart metadata: USD, price grid `1/100`, quantity precision 4,
point value 1. Both CF averages use chart close independently of baseline HL2.
Same-symbol, same-period request.security is exercised. The hidden numeric
primary plot and cross dots are outside this qualification.

The complete native Inputs panels differ only in secondary type after removing
focus markers and numeric Increase/Decrease buttons. The second MA affects
its plotted output; the original strategy's entry filter uses primary direction.
Confirmed OHLCV is byte-identical across captures. The other seven compared
indicator columns, native trade CSV bytes and local complete strategy objects
are identical across the two cases.

## Capture provenance

Each raw chart contains 3,782 rows. Confirmed runtime input is 3,781 bars,
2016-05-23 through 2026-09-28 UTC. The forming September 29 row is retained
in raw evidence and excluded from input. Neither native trade report contains
forming-period entry or exit records. All confirmed OHLCV fields are copied
exactly; timestamps are converted from seconds to milliseconds only. Date-only
native trade timestamps are compared as UTC session dates.

Shared confirmed bars SHA-256:
`19f8799bc4361f09d7841db950a9e98a314929e99194d4a305bd5342fce45ea1`.
Each capture uses its own frozen native history; no equality with an older
batch's provider history is assumed.

Native downloads in `I:\sys\下载`, September 29, UTC+8:

| Second type | Chart CSV | Time | Trade CSV | Time |
| --- | --- | --- | --- | --- |
| TEMA | `COINBASE_ETHUSD, 1D (12).csv` | 21:07:38 | `SSL_Hybrid_Strategy_COINBASE_ETHUSD_2026-09-29 (6).csv` | 21:07:22 |
| T3 | `COINBASE_ETHUSD, 1D (13).csv` | 21:09:08 | `SSL_Hybrid_Strategy_COINBASE_ETHUSD_2026-09-29 (7).csv` | 21:08:45 |

Source, CSVs, native settings and screenshots are frozen in separate directories:

- `.local/ssl-hybrid-coinbase-ethusd-daily-kijun30-cf-rma20-second-tema50-hl2-20260929/`
- `.local/ssl-hybrid-coinbase-ethusd-daily-kijun30-cf-rma20-second-t350-hl2-20260929/`

The directory labels record the second-length input 50; the actual TEMA
calculation uses primary length 20 as described above.

## Comparison results

| Second branch | Nonblank observations | Second MA values | Initial missing bars | Max numeric absolute error | Closed trades | Explicit exit fills | Boundary |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| Original TEMA, effective length 20 | 30,031 | 3,724 | 57 | 8.640199666842818e-12 | 460 | 263 | Flat |
| T3 length 50, factor 0.7 | 29,794 | 3,487 | 294 | 1.2732925824820995e-11 | 460 | 263 | Flat |

All eight named native columns match at absolute tolerance `1e-8`, including
every missing position. Baseline/SSL1 retain 29 initial missing bars, channels
30, primary direction 21. Secondary startup 57/294 is checked without dropping
initial bars. Candle Size false/NA exports as zero; the unrelated Percent short
market probe's Plot is excluded. Between the two native cases, the secondary
column differs at 3,724 positions, including 237 missing-versus-numeric
positions followed by 3,487 different numeric values.

Closed trades are checked for IDs, direction, entry/exit UTC dates, prices,
quantities, duration, entry value and displayed PnL/commission. Explicit exit
multisets match IDs, dates, prices and quantities. Both confirmed boundary
positions are zero, average price null, with no surviving entries.
Quantity tolerance is `1e-10`, observed difference zero. Price tolerance is
`1e-8`; entry-value tolerance stays `1e-8`, maximum difference
`2.842170943040401e-14`. Maximum net PnL / commission display differences
are `0.004999323999999916` / `0.004966256000000002`, within 0.005 in both cases.

## Execution and reproducibility

All six mode runs terminate with exit zero, empty stderr and no diagnostics.
Batch, incremental and historical realtime complete JSON is byte-identical
within each case:

| Second branch | Complete output SHA-256 |
| --- | --- |
| TEMA | `8ed86944e19aba5c488df6a041e4378dc043fb117f5cad70fc42ba816a7fafe7` |
| T3 | `25c4039a2e5ae20d7ab78559d8c016ed78f97dac328df6ce3ebb4925d9a2ba53` |

Base HEAD `2f9a83a0c333092167c7253571f56ee82a7ec0d0`, core patch
`827e195c3a7db4422bbd657a5634990adb9d8d1db69658f6331df41a97cb2636`,
CLI `584ae27bb25fed15c038c5c55a9356720c1bff90b8db29ce8b403e8a08010f36`.
No new core repair. Current source/core/golden hashes match the previously
passing VAMA full gate, reused without another full-gate execution. External
market data stays in local evidence fixtures, outside the core.

Each directory includes original-source analysis, immutable executable, patch,
terminal mode receipts, comparison and current verification receipt. Shared
verifier checks exact commands and metadata, native CSV copies, source/core/
CLI/golden hashes, complete OHLCV, input isolation, startup missing values,
numeric errors, all transactions and cross-case strategy equality. Native
artifact and shared helper hashes are frozen in each receipt. Original
HMA60/Close/divider1/Power1, both MA types 1, CF and second MA disabled,
and both exit switches enabled were restored on BTCUSD daily and recorded.

Run each directory's `run_modes.py`, then from the repository root:

```powershell
python .local/verify_eth_daily_second_tema_t3_20260929.py
```

Aggregate `.local/eth-daily-second-tema-t3-matrix-20260929.json`;
sensitivity `.local/eth-daily-second-tema-t3-sensitivity-20260929.json`;
terminal log `.local/eth-daily-second-tema-t3-verification-20260929.log`.
Qualification is scoped to these original-source/settings/metadata/history
combinations. Live ticks, forming bars, different-timeframe requests, arbitrary
scripts and the unresolved Hull v4 discrepancy remain outside the proven scope.
No commit or push; the expansion goal remains active.

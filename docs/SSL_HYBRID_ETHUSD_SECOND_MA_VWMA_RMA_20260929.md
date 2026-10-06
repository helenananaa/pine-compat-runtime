# SSL Hybrid: ETHUSD second VWMA / RMA numeric output

Verified 2026-09-29 from two fresh authenticated Chrome native captures of
the unchanged original Pine v5
[SSL Hybrid Strategy](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/).
Original source SHA-256:
`1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.
This extends [primary CF VWMA / RMA coverage](SSL_HYBRID_ETHUSD_DAILY_CF_VWMA_RMA_20260929.md)
with the original optional second MA enabled and directly exported numerically.

## Inputs and scope

Both cases use Coinbase `COINBASE:ETHUSD`, daily `1D`, Kijun v2 baseline
length 30 / divider 3 / HL2, original JMA SSL2 length 5 / Phase 3 / Power 2,
HMA exit length 15. CF is enabled with primary RMA type 6 / length 20 /
direction interval 2, current chart resolution enabled. TP/SL and Move SL
on TP1 are enabled. The optional second MA is enabled, length 50, type 5
(VWMA) or type 6 (RMA). Both retained factor inputs are 7 and are inactive
for the selected branches. Price crossing highlights and cross dots are disabled.

Exact original call-site overrides: `12=Kijun v2`, `13=30`, `18=hl2`,
`20=3`, `21=2`, `19=3`, `587=true`, `589=20`, `590=7`, `591=6`,
`594=2`, `595=true`, `597=50`, `598=7`, `599=5` / `599=6`,
`652=true`, `669=true`, `671=true`.

Both primary and second CF averages read chart close independently of HL2.
Same-symbol, same-period request.security is exercised. The second average
changes the plotted numeric output; the original strategy's entry filter uses
the primary average's direction. Different timeframe requests, cross-dot
output and the hidden numeric primary MA are outside this qualification.

Native properties: USD 5,000 capital, 10% equity sizing, pyramiding 10,
0.04% commission, zero slippage, default four historical ticks, bar-close /
realtime execution, requested limit price, one-tick delay and infinite leverage.
Host-neutral metadata: USD, price grid `1/100`, quantity precision 4,
point value 1. No network or host application dependency is added to the core.

## Independent native captures

Each raw chart contains 3,782 rows. Runtime input is 3,781 confirmed daily bars,
2016-05-23 through 2026-09-28 UTC; the forming September 29 bar is retained
in raw evidence and excluded from input. Neither native trade capture has
forming-period entry or exit records. Confirmed OHLCV is copied exactly,
converting timestamp seconds to milliseconds only. Date-only native trades
are compared as UTC session dates.

Confirmed OHLCV is byte-identical across the two captures, SHA-256
`024920aa0087bb8be84e41588bc749811d8a54f190fa881be5f87bbe7f9657af`.
The complete native Inputs panels differ only in second MA type after removing
focus markers and numeric Increase/Decrease buttons. The remaining seven
compared indicator columns are equal. Native trade CSVs are byte-identical;
the local complete strategy objects are also equal across cases.

Downloads in `I:\sys\下载`, September 29, UTC+8:

| Second MA | Chart download | Time | Trades download | Time |
| --- | --- | --- | --- | --- |
| VWMA50 | `COINBASE_ETHUSD, 1D (10).csv` | 20:58:00 | `SSL_Hybrid_Strategy_COINBASE_ETHUSD_2026-09-29 (4).csv` | 20:57:44 |
| RMA50 | `COINBASE_ETHUSD, 1D (11).csv` | 20:59:37 | `SSL_Hybrid_Strategy_COINBASE_ETHUSD_2026-09-29 (5).csv` | 20:59:17 |

Each source, CSV, metadata, full input/property/export/chart snapshot, screenshot
and native download hash is frozen separately:

- `.local/ssl-hybrid-coinbase-ethusd-daily-kijun30-cf-rma20-second-vwma50-hl2-20260929/`
- `.local/ssl-hybrid-coinbase-ethusd-daily-kijun30-cf-rma20-second-rma50-hl2-20260929/`

## Numeric and strategy results

| Second MA | Total nonblank observations | Second MA values | Initial missing bars | Max second MA absolute error | Closed trades | Explicit exit fills | Boundary |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| VWMA50 | 30,039 | 3,732 | 49 | 3.228706191293895e-11 | 460 | 263 | Flat |
| RMA50 | 30,039 | 3,732 | 49 | 1.7763568394002505e-15 | 460 | 263 | Flat |

All eight named indicator columns match at absolute tolerance `1e-8`, including
every missing position. Second MA values differ between the native cases at
all 3,732 valid bars. Baseline/SSL1 retain 29 initial missing bars, channels
30 and primary direction 21. Candle Size false/NA exports as zero. The
unrelated Percent short market probe's Plot is excluded.

Trade checks cover entry/exit IDs, direction, UTC session dates, prices,
quantities, duration, entry value and displayed net PnL/commission. Explicit
exit multisets match IDs, dates, prices and quantities. Both confirmed boundary
positions are zero, average price null, with no surviving entries. Quantity
tolerance is `1e-10`, observed difference zero; price tolerance is `1e-8`.
Entry-value tolerance stays `1e-8`, maximum difference `2.842170943040401e-14`.
Maximum net PnL / commission display differences are
`0.004999323999999916` / `0.004966256000000002`, within 0.005 for both cases.

## Execution and reproducibility

All six mode runs terminate with exit zero, empty stderr and no diagnostics.
Batch, incremental and historical realtime complete JSON is byte-identical
within each case:

| Second MA | Complete output SHA-256 |
| --- | --- |
| VWMA50 | `cd66ecc10fea5c777fdecab9b62956c05b0d92f183a88161f1f166b47d97ed74` |
| RMA50 | `61fcc2031d9ab372824a4bffbbb110c5e41c6bf60d352e71caaab38e0768e7bb` |

Base HEAD `2f9a83a0c333092167c7253571f56ee82a7ec0d0`, core patch
`827e195c3a7db4422bbd657a5634990adb9d8d1db69658f6331df41a97cb2636`,
CLI `584ae27bb25fed15c038c5c55a9356720c1bff90b8db29ce8b403e8a08010f36`.
No new core repair. Current core and golden hashes match the passing VAMA
full gate, reused without another full-gate execution.

Each directory includes original-source analysis, executable, patch, terminal
mode receipts, comparison and current verification receipt. The shared verifier
checks source/core/CLI/golden hashes, native file copies, exact mode commands
and metadata, OHLCV equality, input isolation, startup missing values, numerical
errors, transactions, and cross-case strategy equality. Shared helper and frozen
artifact hashes are saved. Original HMA60/Close/divider1/Power1, both MA types 1,
CF disabled, second MA disabled and both exits enabled were restored on BTCUSD
daily, with restoration evidence retained.

Run each directory's `run_modes.py`, then from the repository root:

```powershell
python .local/verify_eth_daily_second_vwma_rma_20260929.py
```

Aggregate `.local/eth-daily-second-vwma-rma-matrix-20260929.json`;
sensitivity `.local/eth-daily-second-vwma-rma-sensitivity-20260929.json`;
terminal log `.local/eth-daily-second-vwma-rma-verification-20260929.log`.
Qualification applies to these original-source/settings/metadata/history
combinations. Live ticks, forming bars, arbitrary scripts and the unresolved
Hull v4 discrepancy remain outside the proven scope. No commit or push;
the expansion goal remains active.

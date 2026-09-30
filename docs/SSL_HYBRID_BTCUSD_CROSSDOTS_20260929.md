# SSL Hybrid: BTCUSD second T3 and cross dots

Verified 2026-09-29 with a fresh authenticated Chrome capture of unchanged
Pine v5 [SSL Hybrid Strategy](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/).
Source SHA-256: `1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.
This extends the [ETHUSD cross-dot qualification](SSL_HYBRID_ETHUSD_CROSSDOTS_20260929.md)
to BTCUSD daily with native quantity precision 6. It is one new market capture;
different histories do not establish an isolated parameter effect across markets.

## Settings and provenance

Coinbase `COINBASE:BTCUSD`, `1D`: Kijun v2 length 30, divider 3, HL2;
JMA SSL2 length 5 / Phase 3 / Power 2; HMA exit length 15. Primary CF is
enabled RMA type 6 / length 20 / direction interval 2. Optional second MA
is enabled T3 type 8 / length 50 / factor input 7 (0.7), with cross dots on.
Current chart resolution, TP/SL and Move SL on TP1 remain enabled.
Both CF averages use close separately from the baseline HL2 source.

Original call-site overrides: `12=Kijun v2`, `13=30`, `18=hl2`, `20=3`,
`21=2`, `19=3`, `587=true`, `589=20`, `590=7`, `591=6`, `594=2`,
`595=true`, `597=50`, `598=7`, `599=8`, `603=true`, `652=true`,
`669=true`, `671=true`.

Native properties: USD 5,000 capital, 10% equity sizing, pyramiding 10,
0.04% commission, zero slippage, default four historical ticks, bar-close /
realtime execution, requested limit price, one-tick delay, infinite leverage.
Host-neutral metadata: USD, price grid `1/100`, quantity precision 6,
point value 1. Market data remains in fixtures outside the runtime core.

Raw chart has 4,287 rows. Confirmed input contains 4,286 daily bars from
2014-12-01 through 2026-09-28 UTC. The forming September 29 bar is retained
in raw evidence and excluded from input. Native trades have no forming-period
entries or exits. OHLCV is copied exactly, converting seconds to milliseconds
only; date-only trades use UTC session dates. Confirmed bars SHA-256:
`7e3e8de6baa2066f09f89a6112085e7a5068ddd8d83e50810ea438fa6c4ab4e7`.

Native downloads in `I:\sys\下载`, September 29, UTC+8:

| File | Time |
| --- | --- |
| `COINBASE_BTCUSD, 1D (89).csv` | 21:30:24 |
| `SSL_Hybrid_Strategy_COINBASE_BTCUSD_2026-09-29 (1).csv` | 21:30:00 |

## Results

| Measure | Matching result |
| --- | ---: |
| Named nonblank observations | 33,834 |
| Cross dots | 66 |
| Cross-plot blank positions | 4,220 |
| Total nonblank observations | 33,900 |
| Confirmed closed trades | 420 |
| Explicit exit fills | 266 |
| Surviving entries | 0 |

All eight named series and every cross-plot missing position match. Numeric
tolerance is `1e-8`; maximum cross-dot error is `1.6007106751203537e-10`.
Second T3 retains 294 initial missing bars and 3,992 numeric values, maximum
error `2.3283064365386963e-10`. Baseline / SSL1 startup is 29, channels 30,
primary direction 21. Candle Size false/NA exports as zero; startup bars remain.

The native CSV includes two columns named Plot. The comparator asserts the
complete 16-column header and reads zero-based column 13 with `csv.reader`,
comparing it to untitled runtime plot ID 656. Column 15 belongs to the unrelated
Percent short market probe; both columns remain intact in the frozen evidence.

Trade IDs, directions, dates, prices, quantities, duration, entry value and
displayed net PnL / commission match. Quantity difference is zero at `1e-10`
tolerance; price tolerance is `1e-8`. Maximum net PnL / commission display
differences are `0.004957888608004168` / `0.004933601440000011`, within 0.005.
Native Size (value) uses ten significant digits: each value is checked against
its decimal half quantum plus four ULP of the computed value. Maximum entry
value difference is `5.000001124244591e-08`; price and quantity bounds remain
unchanged. Exit multisets agree and the confirmed final position is flat,
average price null.

## Execution and reproducibility

All three mode runs terminate with exit zero, empty stderr and no diagnostics.
Batch, incremental and historical realtime complete JSON is byte-identical:
`8961f89f036dcee1489bc537d728705260d5b0be7869d5d53eeafe61612c2a7e`.

Base HEAD `2f9a83a0c333092167c7253571f56ee82a7ec0d0`; core patch
`827e195c3a7db4422bbd657a5634990adb9d8d1db69658f6331df41a97cb2636`;
CLI `584ae27bb25fed15c038c5c55a9356720c1bff90b8db29ce8b403e8a08010f36`.
No new core repair. Current source/core/golden hashes match the previously
passing VAMA full gate, reused without another full-gate execution.

Frozen directory:
`.local/ssl-hybrid-coinbase-btcusd-daily-kijun30-cf-rma20-second-t350-dots-on-hl2-20260929/`.
Includes original source, executable, patch, native CSVs, full input/property/
export/chart snapshots, screenshot, analysis, mode receipts, comparison and
verification receipt. Shared verifier checks exact commands, metadata, hashes,
native copies, complete OHLCV, all comparisons and recorded browser restoration.
HMA60/Close/divider1/Power1, both MA types 1, CF/second MA/cross dots disabled
and both exits enabled were restored on BTCUSD daily.

Run the capture directory's `run_modes.py`, then from the repository root:

```powershell
python .local/verify_btc_daily_crossdots_20260929.py
```

Aggregate `.local/btc-daily-crossdots-matrix-20260929.json`; terminal mode log
`.local/btc-crossdots-modes-20260929.log`; verifier log
`.local/btc-crossdots-verifier-20260929.log`. Verifier exits zero.
Qualification applies to this source, settings, metadata and confirmed history.
Live ticks, forming bars, different-timeframe requests, arbitrary scripts and
the unresolved Hull v4 discrepancy remain outside the proven scope.
No commit or push; the expansion goal remains active.

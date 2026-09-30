# SSL Hybrid: BTCUSD weekly second T3 and cross dots

Verified 2026-09-29 with a fresh authenticated Chrome capture of unchanged
Pine v5 [SSL Hybrid Strategy](https://www.tradingview.com/script/2it69GUx-SSL-Hybrid-Strategy/).
Source SHA-256: `1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`.
This extends [BTCUSD daily second T3 and cross dots](SSL_HYBRID_BTCUSD_CROSSDOTS_20260929.md)
to weekly execution. Native Inputs panels match after normalizing focus and
numeric stepper controls. Market history is independently exported per period.

## Settings and provenance

Coinbase `COINBASE:BTCUSD`, `1W`: Kijun v2 length 30, divider 3, HL2;
JMA SSL2 length 5 / Phase 3 / Power 2; HMA exit length 15. Primary CF is
enabled RMA type 6 / length 20 / direction interval 2. Optional second MA
is enabled T3 type 8 / length 50 / factor input 7 (0.7), with cross dots on.
Current chart resolution, TP/SL and Move SL on TP1 remain enabled.
Both CF averages use close separately from baseline HL2. Same-period
request.security executes at the weekly chart resolution.

Original call-site overrides: `12=Kijun v2`, `13=30`, `18=hl2`, `20=3`,
`21=2`, `19=3`, `587=true`, `589=20`, `590=7`, `591=6`, `594=2`,
`595=true`, `597=50`, `598=7`, `599=8`, `603=true`, `652=true`,
`669=true`, `671=true`.

Properties: USD 5,000 capital, 10% equity sizing, pyramiding 10,
0.04% commission, zero slippage, default four historical ticks, bar-close /
realtime execution, requested limit price, one-tick delay, infinite leverage.
Host-neutral metadata: USD, price grid `1/100`, quantity precision 6,
point value 1. External market data remains in fixtures outside the core.

Raw chart has 616 rows. Confirmed input contains 615 weekly bars from
2014-12-01 through 2026-09-21 UTC. The forming September 28 week is retained
in raw evidence and excluded from input. The native history jumps from
2014-12-15 to 2015-01-05; its two absent weekly timestamps are preserved.
No artificial bars are inserted. The verifier checks this exact gap and
strictly increasing timestamps. OHLCV is copied exactly with seconds converted
to milliseconds. Confirmed bars SHA-256:
`66bd686a4938db3aefb9c7d933c53a82705cae907c75f7866b6e256010e85d67`.
Trade dates use UTC session dates. No native entry or exit belongs to the
excluded forming week; five earlier entries survive at the confirmed boundary.

Downloads in `I:\sys\下载`, September 29, UTC+8:

| File | Time |
| --- | --- |
| `COINBASE_BTCUSD, 1W (2).csv` | 21:37:34 |
| `SSL_Hybrid_Strategy_COINBASE_BTCUSD_2026-09-29 (2).csv` | 21:37:19 |

## Results

| Measure | Matching result |
| --- | ---: |
| Named nonblank observations | 4,466 |
| Cross dots | 3 |
| Cross-plot blank positions | 612 |
| Total nonblank observations | 4,469 |
| Confirmed closed trades | 60 |
| Explicit exit fills | 34 |
| Surviving entries | 5 |

All eight named series and every cross-plot missing position match. Numeric
tolerance is `1e-8`; maximum cross-dot error is `1.3096723705530167e-10`.
Second T3 retains 294 initial missing bars and 321 numeric values, maximum
error `1.4551915228366852e-10`. Baseline / SSL1 startup is 29, channels 30,
primary direction 21. Candle Size false/NA exports as zero; startup bars remain.

The native CSV has two columns named Plot. The comparator asserts the complete
16-column header and reads zero-based column 13 with csv.reader, comparing
it to untitled runtime plot ID 656. Column 15 belongs to the unrelated
Percent short market probe; both remain intact in the frozen CSV.

Closed-trade IDs, directions, dates, prices, quantities, duration, entry value
and displayed net PnL / commission match. Quantity difference is zero at
`1e-10` tolerance; prices use `1e-8`. Maximum net PnL / commission display
differences are `0.004541048640000156` / `0.0049194862599999944`, within 0.005.
Size (value) uses native ten-significant-digit decimal half quantum plus four
ULP of the computed value; maximum difference `4.999998282073648e-08`.
Five open entries match IDs, directions, dates, prices and quantities.
Exit multisets match. Confirmed final position is long `0.00715 BTC`,
weighted average price `77672.10`.

## Execution and reproducibility

All three mode runs terminate with exit zero, empty stderr and no diagnostics.
Batch, incremental and historical realtime complete JSON is byte-identical:
`1bd9e45d35513f58e3208ea1a773fa7401fd47a329df7a3050a4f8c39cc376e4`.

Base HEAD `2f9a83a0c333092167c7253571f56ee82a7ec0d0`; core patch
`827e195c3a7db4422bbd657a5634990adb9d8d1db69658f6331df41a97cb2636`;
CLI `584ae27bb25fed15c038c5c55a9356720c1bff90b8db29ce8b403e8a08010f36`.
No new core repair. Source/core/golden hashes match the previously passing
VAMA full gate, reused without another full-gate execution.

Frozen directory:
`.local/ssl-hybrid-coinbase-btcusd-weekly-kijun30-cf-rma20-second-t350-dots-on-hl2-20260929/`.
Contains original source, executable, patch, native CSVs, full input/property/
export/chart snapshots, screenshot, analysis, mode receipts, comparison and
verification receipt. Shared verifier checks exact commands, metadata, hashes,
native copies, full OHLCV, native gaps, comparisons and browser restoration.
It also verifies the prior daily receipt's frozen artifact hashes and the
equality of the two native Inputs panels; it does not assume equal histories
or equal trades across periods. The receipt records this daily reference.

Original HMA60/Close/divider1/Power1, both MA types 1, CF/second MA/cross dots
disabled and both exits enabled were restored; browser returned to BTCUSD daily.
Run the capture directory's run_modes.py, then from the repository root:

```powershell
python .local/verify_btc_weekly_crossdots_20260929.py
```

Aggregate `.local/btc-weekly-crossdots-matrix-20260929.json`; terminal mode log
`.local/btc-weekly-crossdots-modes-20260929.log`; verifier log
`.local/btc-weekly-crossdots-verifier-20260929.log`. Verifier exits zero.
Qualification applies to this source, settings, metadata and confirmed history.
Live ticks, forming bars, different-timeframe requests, arbitrary scripts and
the unresolved Hull v4 discrepancy remain outside the proven scope.
No commit or push; the expansion goal remains active.

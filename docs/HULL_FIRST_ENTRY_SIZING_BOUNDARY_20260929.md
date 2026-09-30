# Hull v4 residual: isolated first-entry sizing boundary

Follow-up: [magnitude, percentage, direction and helper controls](HULL_SIZING_MAGNITUDE_CONTROLS_20260929.md)
contradict fixed three-decimal rounding and support ten significant digits
at the percentage-budget stage. The candidate is not yet a core repair.

Verified 2026-09-29. This is diagnostic evidence; the one-unit Hull / HL2
residual remains unqualified. It follows the
[native equity probe](HULL_CURRENT_CORE_EQUITY_PROBE_20260929.md).

## Controlled native experiment

A nine-line Pine v4 strategy places exactly one short market entry when
`time == 1611770400000`, then plots equity, net profit, position and average
price. It has no previous position, trades, indicators or accumulated PnL.
The only changed source literal across four captures is initial capital.
TradingView Properties independently confirm each precise capital, USD,
100% equity sizing, pyramiding 1, zero commission/slippage, infinite leverage,
bar-close calculation and one-tick order delay. No strategy was published.

All entries fill on FX:EURUSD four-hour at 2021-01-28 06:00 UTC+8,
price 1.21078; native contract quantity is integer. Current local executions
use the exported signal/fill bars with price grid 1/100000 and point value 1.

| Initial capital USD | Native short quantity | Current local quantity | Difference |
| ---: | ---: | ---: | ---: |
| 1157890.70824 | 956317 | 956318 | 1 |
| 1157890.70849 | 956317 | 956318 | 1 |
| 1157890.70851 | 956318 | 956318 | 0 |
| 1157890.70999 | 956318 | 956318 | 0 |

All twelve local executions (batch, incremental, realtime-history) terminate
with exit zero, empty stderr and no diagnostics. The complete outputs agree
byte for byte across modes for each source. All current local orders have the
same entry time, price, direction and quantity; native CSVs independently
establish the two quantity mismatches.

## Conclusions and remaining limits

- The original one-unit difference is reproducible on a first entry.
  Reversal and accumulated ledger differences are unnecessary conditions.
- Exact capital / price exceeds 956318 contracts in all four cases.
  Native quantity changes between .70849 and .70851, above the mathematical
  boundary capital 1157890.70804.
- Flooring the budget to cents is inconsistent with the .70999 control,
  which already produces 956318 natively.
- Rounding a USD budget to three decimals fits these observations. This is
  a hypothesis: fixed decimal rounding and rounding by significant digits
  cannot yet be distinguished at this capital magnitude. The next control
  must vary capital magnitude or equity percentage, and test both directions.

No general arithmetic rule is established yet. No core behavior was changed
to fit this one boundary, and the full gate was not rerun for unchanged code.
The previously passing VAMA gate still supplies the executable/source pins.

## Frozen evidence and replay

Directory: `.local/sizing-boundary-native-20260929/`.
Four sources, effective Properties snapshots, chart state/screenshots,
native trade CSVs, historical native chart CSV, reduced two-bar input,
twelve local outputs and command receipts, and `verification.json` are retained.
The verifier checks HEAD/core/golden/executable pins, effective capitals,
native entry fields, deterministic mode agreement and the exact residuals.

Native downloads from `I:\sys\下载`:

- `Sizing_boundary_first-entry_probe_20260929_FX_EURUSD_2026-09-29.csv`
  and suffixes `(1)`, `(2)`, `(3)` capture the four capitals in acquisition
  order .70824, .70999, .70849, .70851. Open mark-to-market fields vary with
  live quotes; only entry execution fields are compared across captures.
- `FX_EURUSD, 240 (21).csv` and `(22).csv` initially contained recent chart
  windows, excluding the entry. They are not used as historical bar evidence.
- `FX_EURUSD, 240 (23).csv` was exported after explicitly selecting the
  January 2021 date range. It contains both signal and fill timestamps;
  its raw file is retained as `boundary-native-chart.csv`.

```powershell
python .local/sizing-boundary-native-20260929/run_verify.py
```

The two-bar reduction is valid only for this stateless single-entry probe;
it does not replace full-history qualification of the Hull strategy.
Base HEAD: `2f9a83a0c333092167c7253571f56ee82a7ec0d0`.
CLI SHA-256: `584ae27bb25fed15c038c5c55a9356720c1bff90b8db29ce8b403e8a08010f36`.
Core patch: `827e195c3a7db4422bbd657a5634990adb9d8d1db69658f6331df41a97cb2636`.
The temporary strategy was removed through the object tree; final state confirms
Coinbase BTCUSD daily with the original SSL Hybrid and Percent short strategies.
The previous Hull diagnostic editor draft was restored without adding it to
the chart. No commit or push. Goal remains active.

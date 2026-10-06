# Hull sizing residual: magnitude, percentage, direction and helper controls

Follow-up: [budget rounding repair and full-history replay](HULL_PERCENT_BUDGET_FIX_20260930.md)
implements the candidate, matches all nine isolated controls, removes the
original trade-265 quantity residual and passes the release gate. Remaining
equity marks and monetary display differences are recorded separately.

Verified 2026-09-29. Follow-up to the
[first-entry capital boundary](HULL_FIRST_ENTRY_SIZING_BOUNDARY_20260929.md).
The original Hull one-unit residual is still unqualified. This turn adds four
independent v4 entry controls and a v5 default quantity helper control.

## Native observations

Metadata stays FX:EURUSD, four hours, USD, integer contracts, price grid
1/100000, point value 1. Properties independently confirm exact source capital,
100% or 25% equity, zero commission/slippage, infinite leverage, pyramiding 1,
bar-close execution and one-tick fill delay. Entries signal at
1611770400000 and fill at 1611784800000, price 1.21078.

| Control | Capital USD | Equity percent | Direction | Native quantity | Current local |
| --- | ---: | ---: | --- | ---: | ---: |
| Lower magnitude above midpoint | 115788.10216 | 100 | Short | 95631 | 95630 |
| Lower magnitude below midpoint | 115788.10214 | 100 | Short | 95630 | 95630 |
| Quarter equity | 463152.40868 | 25 | Short | 95631 | 95630 |
| Lower magnitude long | 115788.10216 | 100 | Long | 95631 | 95630 |
| v5 default_entry_qty(1.21078) | 115788.10216 | 100 | No orders | 95631 | 95630 |

The helper source has no entries or equity changes. Its 299 observed native rows
expose constant default quantity, capital, zero profit, zero position and missing
average price. The final forming bar at timestamp 1790686800 has all five probe
columns blank; the verifier checks this exact missing row explicitly. Local checks use the independently
exported signal/fill bars frozen in the preceding turn. That two-bar reduction
is valid for these stateless controls, not for full Hull qualification.

## Arithmetic interpretation

95631 contracts cost exactly 115788.10218 USD at decimal price 1.21078.
The .10216 capital is slightly below that cost, yet the native entry and helper
both return 95631. A fixed three-decimal budget would give only 95630 in both
lower-magnitude captures, contradicting the .10216 observation.

Rounding the percentage budget to ten significant decimal digits, then dividing
by price and flooring to the contract grid, fits all new controls and the four
prior million-dollar controls:

- 1157890.70824 -> 1157890.708 -> 956317.
- 1157890.70851 -> 1157890.709 -> 956318.
- 115788.10214 -> 115788.1021 -> 95630.
- 115788.10216 -> 115788.1022 -> 95631.
- 463152.40868 * 25% = 115788.10217 -> 115788.1022 -> 95631.

The quarter-equity case also distinguishes budget rounding from merely rounding
initial capital: rounding capital to ten digits gives 463152.4087, and its
unrounded quarter budget 115788.102175 is still below the contract cost.
Both directions agree. The v5 helper agrees with v4 actual entries, so any repair
must keep shared sizing behavior consistent.

This is now a concrete candidate semantic rule, not a completed repair.
Commission, other currencies, extreme magnitudes, exact ties, and full-strategy
effects remain unqualified by these controls. Next: implement and test the
candidate, replay the original full-history Hull settings, inspect all changed
quantity boundaries, and run the required release gate if the repair is retained.

## Reproducibility

Frozen directory: `.local/sizing-magnitude-native-20260929/`.
Sources, effective Properties, chart states/screenshots, four trade CSVs,
helper native chart CSV, two-bar input, fifteen local complete mode outputs,
command receipts and `verification.json` are retained. All fifteen executions
exit zero with empty stderr and diagnostics; per-source mode outputs agree
byte for byte. Four controls retain a one-unit residual against current core.
The verifier also checks a Decimal ten-digit candidate calculation against
native quantities; this calculation is explicitly separate from core execution.

Native downloads from `I:\sys\下载`:

- `Sizing_magnitude_probe_20260929_FX_EURUSD_2026-09-29.csv` and `(1)` through
  `(3)` capture lower-below, lower-magnitude, quarter-equity and lower-long in
  acquisition order. Open valuation/excursion rows remain raw; only execution
  fields are compared.
- `FX_EURUSD, 240 (24).csv` captures the constant v5 helper columns.

The initial trade-download clicks while the editor covered the export region
did not create files. The captures above were acquired after explicitly closing
the editor. An initial helper source still retained the entry due to newline
matching; it was corrected before the helper capture, whose source contains no
strategy.entry and whose native position is zero in every exported row.

```powershell
python .local/sizing-magnitude-native-20260929/run_verify.py
```

HEAD `2f9a83a0c333092167c7253571f56ee82a7ec0d0`.
Core patch `827e195c3a7db4422bbd657a5634990adb9d8d1db69658f6331df41a97cb2636`.
CLI `584ae27bb25fed15c038c5c55a9356720c1bff90b8db29ce8b403e8a08010f36`.
No new core edit or gate run. Temporary probe is removed; original chart and
Hull diagnostic editor draft are restored. No commit or push. Goal remains active.

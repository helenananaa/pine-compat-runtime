# Display algebra: native findings and implementation

2026-09-26. Display arithmetic is now implemented with a nominal `plot_display`
semantic type and dedicated HIR operations. The unchanged public TASC indicator
analyzes with zero diagnostics; benchmark-data/numeric qualification is pending.

The native v6 compiler accepts display addition/subtraction, intermediate
variables and equality comparisons. Two independent CSV exports establish:

| Relation | Native result |
| --- | --- |
| `pane + pane == pane` | true |
| `pane - data_window == pane` | true |
| `all - pane - pane == all - pane` | true |
| `pane + price_scale + status_line + data_window == all` | **false** |
| `pane - pane == none` | true |
| `all - all == none` | true |
| `pane + data_window == data_window + pane` | true |
| `all + pane == all` | true |
| `none + pane == pane` | true |
| `all - none == all` | true |

All names above are `display.*`. A separate probe also reports
`all - pane == status_line + price_scale + data_window` as false.
Each exported relation is constant across 300 chart rows, including the forming
row; these are data-independent algebra probes, not a historical-price parity
claim. Downloads: `COINBASE_BTCUSD, 1D (72).csv` and `(73).csv` under
`I:\sys\下载`.

Further native diagnostics expose a fifth member, `display.pine_screener`.
A second two-column CSV probe establishes that the sum of all five named
locations still differs from `all`, and removing the original four from `all`
does not equal `pine_screener`. Both columns are false throughout export (74).
The v6 compiler accepts the new member; the same v5 source reports an undeclared
identifier at `display.pine_screener`. Its version gate is therefore v6.

The implementation uses set union/difference over five named locations and an
additional internal membership preserving universal `all`. These internal bits
are not a claim about TradingView's integer encoding and are never exposed.
Nominal identity survives variables, conditionals and UDF returns. Dedicated
HIR operations avoid changing ordinary string concatenation. Shared algebra is
used in constant evaluation as well as runtime execution, including expressions
that determine history offsets.

Native negative probes reject a plain string `"display.all"` as a display
argument and reject a series-qualified display conditional where input is
required. Local tests retain these boundaries. Input/output display parameters
use the new semantic type; other string parameters remain strings.

## Host-neutral output

The existing `display` string wire field is retained. Single named constants
remain unchanged. Composite values use canonical expressions without spaces:
named locations joined by `+` in pane/price_scale/status_line/data_window/
pine_screener order, or `display.all` followed by `-` exclusions in that order.
Empty sets emit `display.none`. Hosts must interpret union and difference;
silently treating composites as `display.all` loses behavior. Snapshot JSON
roundtrips preserve these expressions. Runtime schema 9/render metadata 1 stay
unchanged; no external host renderer or distribution build is qualified here.

## Validation

Two relation matrices match native CSVs over 299 confirmed overlapping bars:
2,990 + 598 = 3,588 exact numeric cells, zero differences. Tests additionally
cover UDFs, variables, inputs, plain strings, invalid operators, qualifier
rejection and constant history offsets. All 6,098 builtins/sema/runtime/CLI tests
passed; workspace compilation, formatting and diff checks passed. The original
TASC source has zero diagnostics and 101 supported feature uses. This does not
establish full TASC numerical parity or benchmark availability.

Ignored evidence directory `.local/continued-popular-20260926` contains the
two `display-*-probe-v6.pine` sources, corresponding native CSVs,
`display-relations-observed.json`, and `display-evidence-manifest.json`.
The relation report checks every exported row, not only the chart legend.
Implementation evidence additionally includes `display-five-native.csv`,
`display-five-locations-probe.pine`, `display-invalid-string-native.txt`,
`display-series-native-error.txt`, `display-v5-native-dom.txt`,
`display-native-comparison.json`, `compare_display.py`, and
`test-display-final.log`. Fixtures `display_relations.pine` and
`display_five_locations.pine` are executed by the Rust integration gate.

# Discovering host inputs before execution

`host_requirements` describes potential external inputs of a compiled program.
It does not execute Pine, fetch market data, inspect a provider, or certify that
a particular dataset is sufficient. Compilation remains the first step:
unsupported language features and missing library sources retain the existing
source diagnostics and do not produce an executable requirements report.

## APIs

This API is currently unreleased; use the qualified development build.

Rust accepts the compiled HIR and returns a typed `HostRequirements`:

```rust
let requirements = pine_runtime::host_requirements(&hir);
let json = pine_runtime::host_requirements_json(&hir);
```

CLI compiles the source and optional complete library graph, then prints JSON:

```text
pine-compat requirements script.pine --library-source Author/Library/1=library.pine
```

An unsupported script returns a nonzero status with the existing analysis JSON
diagnostics on stderr. A valid report on stdout uses its own schema version 1;
it does not change analysis schema 5 or runtime result schema 8.

Python uses the compiled program, including libraries supplied to compilation:

```python
program = pine_compat.compile_script(source, library_sources=libraries)
requirements = program.host_requirements()  # Python dict
```

The actual WASM `Program` exposes the same data as a JSON string:

```javascript
const program = pine.compileScriptWithLibraries(source, JSON.stringify(libraries));
const requirements = JSON.parse(program.hostRequirements());
program.free();
```

## Contract and interpretation

The discovery mode is `conservativeExecutableHirInventory`. It visits all
lowered executable expressions, including inlined library bodies, loop bounds,
tuple members and dynamic history offsets. It does not attempt path feasibility:
a conditional read may appear even when a particular run never reaches it.
Unused function declarations that are not lowered into executable code are not
external-input obligations merely because their text appears in a library.

| Field | Meaning for the host |
| --- | --- |
| `chart.bars` | Supply standard OHLCV chart bars through the chosen execution API. |
| `chart.priceGrid` | Supply positive integer minMove/priceScale for the intended instrument; no exchange lookup occurs. |
| `chart.quantityPrecision` | Decimal-power minimum-contract profile, integer precision 0 through 9. This is not arbitrary lot-step rounding. |
| `chart.symbolMetadata` | Metadata names read by the executable HIR. Other context-dependent behavior, such as broker fills and chart time boundaries, can still need chart context without a direct metadata read. |
| `chart.defaults` | Actual synthetic chart symbol/timeframe/grid/quantity defaults, fixed currency/timezone/point value, and explicit absence of symbol lookup. These are defaults, not inferred metadata for the supplied bars. |
| `account` | Null for indicators. Strategies disclose same-currency linear accounting with unit point value and unsupported currency-conversion, contract-multiplier and nonstandard-chart broker profiles. |
| `execution.clock` | `explicitTimestampWhenEvaluated` means a reached `timenow` read needs an explicit host execution timestamp. `notUsed` means the inventory found no read. A skipped branch does not become a runtime requirement. |
| `execution.realtimeUpdates` | The host supplies ordered forming replacements and confirmations; the core implements their state semantics. This is not a scheduler or market feed. |
| `execution.calcOnEveryTick`, `calcOnOrderFills`, `processOrdersOnClose` | The compiled strategy switches. Indicator reports set these strategy-only flags to false. |
| `execution.magnifier` | `historicalIntrabarsOrReportedStandardOhlcFallback` means enabled historical Magnifier accepts supplied intrabars and retains the established diagnostic/fallback behavior for absence or gaps. `notEnabled` does not require intrabars. |
| `execution.sessionWindows` | Window-scoped risk rules can consume host window/trading-day IDs or retain the existing UTC fallback. `notUsedByWindowRiskRules` means no such rule was found. |
| `requests` | Potential provider-backed contexts, identified by compiled call-site IDs. A same-context request evaluates its expression without a provider. External requests need a same-or-higher timeframe that is an integer multiple of the current context timeframe. |
| `inputCallSiteIds` | Input calls available to override. Obtain titles, value types, defaults and constraints from existing analysis `inputs` or Rust `input_calls`. |

Request symbol/timeframe arguments distinguish `literal`, `currentContextSymbol`,
`currentContextTimeframe`, `rootChartSymbol`, and `runtimeExpression`. Literal
values preserve the compiled string; timeframe interpretation and runtime
validation still apply. Context references refer to the active request context,
not unconditionally the root chart. A runtime expression is unresolved, not a
claim that its input default is the final requested key.

Each request records its `function`, `gaps`, and `lookahead`. Legacy `security`
can use its existing gaps-on/lookahead-on profiles; this report does not upgrade
legacy support labels. Modern request admission is unchanged: empty literal
contexts and input/local symbol aliases rejected by the compiler remain
rejected. The runtime-expression classification is exercised by the existing
legacy provider profile and is not a new modern dynamic-request capability.

Call-site IDs identify this compiled program, not permanent source locations or
IDs stable across recompilations. The report is not a source map. Existing
analysis errors still carry source diagnostics; per-request source provenance
and runtime-resolved readiness checks remain separate work.

## Explicit point-value validation

The host may declare `pointValue` explicitly: Rust
`ChartContext::with_point_value(1.0)`, CLI `--chart-point-value 1`, or
Python/WASM `$chart.pointValue = 1`. Python retains required minMove/priceScale.
The accepted unit value preserves all existing prices, quantities, fees and
accounting. Omission retains the unit default. Non-unit values, non-finite
numbers and invalid types fail configuration parsing before Pine execution,
with an explicit pointValue/unsupported-contract-multiplier error.

This validates the supported profile; it does not implement contract multipliers
or infer currencies/inverse-contract accounting from a symbol name. Quantity
precision and point value describe different properties: fractional quantity
precision remains supported with unit point value. No numerical tolerances,
conditional execution rules or optional data fallbacks change.

## Host integration sequence

1. Supply the original source and complete exact library dependencies to analysis
   or compilation. Handle unsupported-feature/source diagnostics first.
2. Read the requirements report and explicitly choose the chart/account profile.
   Do not assume that synthetic defaults describe an arbitrary real instrument.
3. Configure chart metadata, inputs, requested datasets and optional fallback
   policies through the existing host APIs. Resolve runtime request expressions
   through the supported execution/provider contract; the report is not a
   closed list of concrete keys in those cases.
4. Run with the appropriate history/forming/confirmation lifecycle. Handle input
   and capability errors when the relevant expressions actually execute.

Do not translate every inventory entry into an unconditional rejection rule.
For example, `false ? timenow : close` still executes without an execution clock,
and a same-context request still needs no external provider. Likewise, optional
Magnifier and session-window fallbacks remain available under their documented
contracts. This API adds discovery without changing those behaviors.

## Validation status

The original shared strategy fixture and requirements JSON are tested through
CLI, Python and actual WASM, alongside Rust tests for context classification,
conditional clock reads, imports, dynamic history traversal, legacy request
arguments and unsupported modern input forms. These are product-contract tests,
not independent TradingView numerical references. The Windows gate passes 6640 Rust tests, 693 fresh installed-wheel Python tests,
103 tool tests and actual WASM/Node smoke. Retained CLI, wheel and generated
WASM artifacts also agree on six frozen real workload inventories: complete
TechnicalRating dependencies, matched MTF, both margin sources and paired
Magnifier sources. These checks discover inputs without executing historical
bars; they do not add to the independent numerical reference denominator.
Evidence and hashes are in `.local/delivery-20260909/host-contracts/`.

This report does not prove dataset completeness, static branch reachability,
real tick ordering, latency or bounded memory. It also does not implement the
unsupported account profiles it lists. D3 remains open until the remaining
profile/readiness and source-diagnostic acceptance is addressed.


Initial checks found unsupported modern empty/input-symbol request forms in two
new tests; the tests were corrected to respect existing admission, with explicit
negative coverage retained. A separate valid legacy-input test then exposed a
real inventory omission: lowered legacy security names were not recognized.
The implementation now discovers them and preserves their gaps/lookahead labels.
The first full gate also caught a Clippy style issue; the final gate passes
without changing warning policies or any prior runtime golden.


Explicit point-value follow-up: the Windows gate passes 6643 Rust tests, 705
fresh installed-wheel Python tests, and 103 tool tests. Updated actual WASM smoke
covers accepted unit metadata and rejected values/types; the final smoke file
was rerun after its test-only update. CLI, installed wheel and actual WASM retain
complete prior long/short margin output equality when pointValue=1 is supplied.
The initial gate caught an invalid Debug bound in a new parser test; that test
was corrected without changing production validation. No runtime golden changed.
Evidence: `.local/delivery-20260909/point-value-contract/`.

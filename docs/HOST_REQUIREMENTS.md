# Discovering host inputs before execution

`host_requirements` describes potential external inputs of a compiled program.
It does not execute Pine, fetch market data, inspect a provider, or certify that
a particular dataset is sufficient. Compilation remains the first step:
unsupported language features and missing library sources retain the existing
source diagnostics and do not produce an executable requirements report.

## APIs

This API ships in the local `0.3.0-rc.1` candidate (Python wheel `0.3.0rc1`).
It is not part of the published `v0.2.0` GitHub release.

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
| `execution.realtimeUpdates` | The host supplies ordered forming replacements and confirmations; the core implements their state semantics. Rust/Python also accept an optional opening-update marker for attachment to an already open bar; omission retains first-observation inference. This is not a scheduler or market feed. |
| `execution.calcOnEveryTick`, `calcOnOrderFills`, `processOrdersOnClose` | The compiled strategy switches. Indicator reports set these strategy-only flags to false. |
| `execution.magnifier` | `historicalIntrabarsOrReportedStandardOhlcFallback` means enabled historical Magnifier accepts supplied intrabars and retains the established diagnostic/fallback behavior for absence or gaps. `notEnabled` does not require intrabars. |
| `execution.sessionWindows` | Window-scoped risk rules can consume host window/trading-day IDs or retain the existing UTC fallback. `notUsedByWindowRiskRules` means no such rule was found. |
| `requests` | Potential provider-backed contexts, identified by compiled call-site IDs. A same-context request evaluates its expression without a provider. External requests need a same-or-higher timeframe that is an integer multiple of the current context timeframe. |
| `inputCallSiteIds` | Input calls available to override. Obtain titles, value types, defaults and constraints from existing analysis `inputs` or Rust `input_calls`. |
| `callSites` | Source locations for discovered request/input call IDs; null source means unavailable. See provenance coverage below. |

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
IDs stable across recompilations. The report maps discovered request/input calls,
not every expression in a program. Existing analysis errors retain their source
diagnostics; runtime-resolved readiness checks remain separate work.

### Source provenance

`callSites` is an additive field in the version-1 requirements
inventory for discovered request and input calls. Each entry joins by
`callSiteId` and has either `source: null` (unavailable) or a source location with
`sourceId`, `libraryKey`, `start`, and `end`. Offsets are half-open UTF-8 byte
ranges in the original supplied source. Root source ID is 0 with null library
key; libraries use their exact normalized import key and analysis-local physical
source ID. IDs are not stable across changes to the compilation input graph.

The mapping is recorded while lowering source calls, using the import plan's
explicit context-to-physical-source mapping. It does not infer a library from
call numbering, rewritten aliases, or potentially duplicate filenames. Two
aliases and a transitive import can therefore point to the same physical library
range while retaining distinct executable call-site IDs. Canonicalized legacy
`security` retains its original source spelling/range. Missing provenance in
manually constructed HIR is not mislabeled as a root source location.

The five new provenance cases and seven existing host-inventory cases pass.
Full Windows qualification passes 6660 Rust, 712 installed-wheel Python and
117 tool tests plus actual WASM. Semantic HIR comparisons distinguish provenance
from execution semantics as explained below. This is development qualification,
not a final release or configured-provider readiness certificate.

Cross-host contract tests retain the existing semantic-field golden
and add an independent three-call source-text fixture. Expected byte ranges are
computed from each host's actual input bytes; the entire resulting report is
then compared. This avoids pinning CRLF offsets into an LF-only golden without
dropping provenance from assertions. Added Python and actual-WASM controls use
Unicode source with both LF and CRLF; these controls passed the full gate.

Existing legacy `normalized_hir` comparisons already excluded language-version
metadata to compare execution semantics across rewritten source spellings. They
now also exclude the new location vector for that same purpose. The existing
alias-source-span test separately checks that the lowered `ta.sma` call maps to
the original `sma(close, 2)` bytes. No execution-IR field or runtime-output golden
is ignored or regenerated by this adjustment. Module constants and omitted
function defaults reject calls under current admission rules, so they cannot
silently import an external request with a caller-origin span.

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

## Defaults, platforms, limits, and version compatibility

Defaults (synthetic, not inferred from a symbol name):

- chart symbol `NASDAQ:AAPL`, timeframe `1`, minMove 1, priceScale 100,
  quantityPrecision 0, currency `USD`, point value `1`, timezone `Etc/UTC`;
- indicator reports have a null account contract; strategies disclose
  same-currency linear accounting with unit point value;
- drawing objects use the existing 50-default / 500-max (100 for polylines)
  oldest-active eviction; series history is capped at 1,000,000 committed
  values; `while` loops stop at 100,000 iterations; arrays at 100,000
  elements; strings at 40,960 characters.

Supported candidate platforms: Windows x86-64 (`win_amd64`), native Ubuntu
22.04 glibc Linux x86-64 (`manylinux_2_35_x86_64`), and the separately rebuilt
manylinux2014 `manylinux_2_17_x86_64` candidate wheel. The latter passed auditwheel
and 715 installed Python tests; see CANDIDATE_ACCEPTANCE.md for evidence.
macOS, musllinux, ARM, and free-threaded CPython are out of this matrix.

Account and data range for this candidate: standard candles, host-supplied
price grid, same-currency linear accounting, unit point value. Foreign-currency
conversion, non-unit contract multipliers, and nonstandard charts remain
unsupported and fail closed.

Dynamic input limits: `input.*` overrides apply to compiled call-site IDs from
the current compilation. Runtime-expression request arguments stay unresolved
in the inventory; they are not treated as literal keys. Call-site IDs are not
stable across recompilation.

Version compatibility: analysis schema 5, runtime schema 8, host-requirements
schema 1. The crate identity is Cargo `0.3.0-rc.1` / PEP 440 `0.3.0rc1`. Do not
mix this candidate with published `0.2.0` wheels or crates. A failed compile,
seed, or update returns an error and does not replace previously owned results
on the realtime session path.

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
unsupported account profiles it lists. The scoped profile/readiness review is
recorded below; broader profiles and final platform delivery remain separate.


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


Python realtime clock support: `seed(bars, execution_times=[...])` accepts one
integer millisecond timestamp per history bar. `update_forming(bar,
execution_time=...)` and `update_confirmed(bar, execution_time=...)` accept an
optional timestamp for that execution. These are keyword-only additions;
omission retains the existing missing-clock behavior. Core timestamped seeding
preserves batch context and commits only on success. Invalid types, mismatched
counts and missing required clocks leave session state unchanged.

Windows validation passes 6646 Rust tests, 710 fresh installed-wheel Python tests,
115 tool tests and actual WASM smoke. The retained new wheel passes all 12 Python
realtime-session tests. Original before-fix API failures remain under
`.local/delivery-20260909/host-contracts/python-clock-before.log`.

A separate frozen real-update oracle was then replayed through the installed
wheel. All 896 values were compared: 62 mismatches remain in position and trade
history fields, while time, EMA/SMA and var/varip fields match. This demonstrates
a broker-lifetime semantic gap; clock plumbing is qualified, but native realtime
strategy compatibility is not. See LIVE_TICK_REFERENCE_AUDIT.md.

Qualification evidence: `host-contracts/source-provenance-full-gate-v3.log`.
The first gate exposed one additional v1/v2 HIR comparison with shifted byte
offsets; the saved failure audit verifies that only `call_site_sources` differed.
The second gate exposed existing file-size limits. Source-call types, allocation
logic and the original module tests were extracted into their respective files;
no limit was loosened. Original execution goldens remain unchanged.

## Scoped configured-input review

The existing boundary behavior was reviewed in the request provider/timeframe
code and current price-grid, quantity, point-value, execution-clock and session
tests. Ten additional checks through the retained e7435e036 release wheel passed:
same-context request without a provider; unreached request and clock reads;
explicit reached missing-provider/clock errors; lower and nonintegral request
timeframe rejection; consumption of supplied external data; rejection of nonunit
point value and an invalid price grid. Evidence is in
`host-contracts/readiness-audit.py`, `.log` and `.json`.

For the frozen standard-candle, same-currency unit-point profile, the host can
discover potential inputs, locate their source, select explicit metadata, and
handle missing/unsupported capabilities at the documented evaluation boundary.
No additional unconditional preflight gate is required for D3. This conclusion
does not certify dataset completeness or branch reachability, expand modern
dynamic-request admission, or remove optional fallbacks. Those are deliberately
outside the inventory contract, not unimplemented promises. Final Linux/Windows
distribution qualification remains D5; numerical and resource evidence remain
D2 and D4 respectively.

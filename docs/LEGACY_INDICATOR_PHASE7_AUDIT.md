# Legacy Indicator Phase 7 Audit

## Outcome

Phase 7 makes the fixture-backed legacy `security` family executable through
the existing host-neutral request-data boundary. It adds versioned call
binding, explicit gaps/lookahead policies, separate historical and realtime
alignment, requested-context isolation, original-span provider failures, and
chart-context inputs for every public runtime host.

The modern `request.security` surface is unchanged: its public optional merge
arguments still accept only the existing default `gaps_off`/`lookahead_off`
subset. `study(resolution=...)` remains one precise unsupported feature because
whole-program execution on another timeframe is not equivalent to evaluating
one expression in a requested child context. No legacy strategy analysis,
lowering, broker behavior, or migration path is enabled.

## Historical Background

The behavior was checked against TradingView's official
[v2-to-v3 migration guide](https://www.tradingview.com/pine-script-docs/migration-guides/to-pine-version-3/),
archived
[Pine v3 security documentation](https://www.tradingview.com/pine-script-docs/v3/essential/context-switching-the-security-function/),
archived
[Pine v3 release notes](https://www.tradingview.com/pine-script-docs/v3/release-notes/),
archived
[Pine v4 security documentation](https://www.tradingview.com/pine-script-docs/v4/essential/context-switching-the-security-function/),
and archived
[Pine v4 release notes](https://www.tradingview.com/pine-script-docs/v4/release-notes/).

Those sources establish the result-affecting boundaries used here:

- the fifth `lookahead` parameter appeared in Pine v3;
- keyword arguments for built-in functions appeared in Pine v3, so v1/v2
  `security` calls remain positional;
- Pine v1/v2 behavior corresponds to `barmerge.lookahead_on`;
- Pine v3/v4 default to `barmerge.lookahead_off`;
- `gaps_off` carries an eligible requested value while `gaps_on` preserves
  unmapped chart bars as `na`;
- on historical bars, lookahead-on exposes a higher-timeframe value from the
  requested bar's opening boundary, while lookahead-off waits for its closing
  boundary;
- realtime lookahead-on does not expose the historical future-value behavior;
- `study(resolution=...)` was added as declaration-level execution context,
  not as an ordinary scalar call.

The mutable-variable restriction in historical `security` expressions is
preserved by reusing the existing closed requested-expression analyzer instead
of accepting arbitrary expressions.

## Phase Plan And Decisions

The implementation followed six gates:

1. verify historical signatures, defaults, and realtime handoff;
2. audit the existing request provider, cache, child-runtime, alignment, and
   host boundaries;
3. bind legacy calls by dialect and preserve a structured lowering decision;
4. implement and fixture gaps/lookahead alignment plus warning/error behavior;
5. expose chart identity consistently through CLI, Python, and WASM;
6. measure the unchanged corpus, update the conformance contract, and run the
   complete release gate before committing.

Two designs were deliberately rejected. A plain name alias would erase version
defaults, and widening the modern `request.security` call would make modern
scripts accept behavior that has not been claimed for them. Wrapping the entire
program in a fake request call was also rejected for declaration-level
timeframes because it would not prove state, output, cache, gaps, or realtime
equivalence.

## Versioned Binder

The focused binder runs only after lexical values and user-defined functions
fail to resolve, so a user function named `security` retains ordinary source
precedence.

| Dialect | Accepted shape | Named arguments | Default gaps | Default lookahead |
| --- | --- | --- | --- | --- |
| v1/v2 | 3 required arguments, optional fourth `gaps` | no | off | on |
| v3/v4 | 3 required arguments, optional `gaps`, optional `lookahead` | yes | off | off |

Required roles are `symbol`, `resolution`, and `expression`. The v3/v4 names
are bound before canonical request validation, including reordered named
arguments. Duplicates, unknown names, positional arguments after named
arguments, missing required roles, and excess arguments use the established
call diagnostics. A gaps/lookahead value must resolve at analysis time to a
bool or the corresponding `barmerge` constant; otherwise
`E_LEGACY_SECURITY_MERGE` stops lowering.

The first three arguments reuse the request-specific type, qualifier,
same-context, provider-context, lower-timeframe, side-effect, and expression
checks. Accepted calls record a `security -> request.security`
`signatureReshape` translation and a `security.merge` emulation whose behavior
states the selected policy.

## HIR And Runtime Routing

Lowering emits one of four source-inaccessible callees:

```text
$legacy.security.gaps_off.lookahead_off
$legacy.security.gaps_on.lookahead_off
$legacy.security.gaps_off.lookahead_on
$legacy.security.gaps_on.lookahead_on
```

The symbol, timeframe, and expression remain ordinary HIR arguments. Two
hidden integer arguments, `$legacy_span_start` and `$legacy_span_end`, retain
the full original call span. The runtime dispatches the four internal names to
the shared request engine with an explicit merge policy; modern
`request.security` always dispatches with its existing off/off policy.

The translator revision is `6`, preventing semantic compile-cache reuse across
the new binding, lowering, and execution contract.

## Alignment Contract

For the supported same-or-higher-timeframe provider subset:

| Context | `gaps_off` | `gaps_on` |
| --- | --- | --- |
| same timeframe | latest requested open not after chart open | exact requested/chart open |
| higher timeframe, lookahead off | latest requested close not after chart close | requested close equals chart close |
| higher timeframe, historical lookahead on | latest requested open not after chart open | requested open equals chart open |

Realtime forming and confirmed updates always use confirmed lookahead-off
alignment, including a legacy call whose historical policy is lookahead on.
This produces an intentional historical/realtime handoff rather than exposing
future data during a live bar. Chart bars before the first eligible requested
value return `na`.

A reached lookahead-on call records
`W_LEGACY_SECURITY_LOOKAHEAD` once per distinct callsite. Warnings are sorted by
callsite and returned as non-error runtime diagnostics. The warning therefore
documents repaint risk only after the corresponding behavior is implemented;
it is not used to disguise an unsupported approximation.

## Requested Context And Provider Contract

Provider evaluation clones the immutable provider but replaces child chart
metadata with the requested symbol and timeframe. The child historical runtime
owns its histories, `var` values, arrays, drawings, outputs, and stateful
callsites. Cached results remain keyed by callsite, requested key, and
expression identity. Child legacy warnings are merged into the outer runtime;
mutable execution state is not.

The host supplies chart bars, chart symbol/timeframe, and every requested
symbol/timeframe stream:

- CLI: repeated `--request-bars SYMBOL:TIMEFRAME=bars.csv` plus optional
  `--chart-symbol` and `--chart-timeframe`;
- Python: the existing `request_bars` mapping plus optional final
  `chart_symbol` and `chart_timeframe` keywords on `run_script` and
  `Program.run`;
- WASM: the existing request-bars JSON mapping plus an optional reserved
  `$chart` object containing string `symbol` and `timeframe` fields.

Omitting chart metadata preserves the deterministic default. Empty symbols,
invalid timeframes, malformed WASM metadata, unsorted streams, duplicate
timestamps, and duplicate request keys fail at the host/provider boundary.
Missing data keeps the provider's stable key text. Legacy calls prefix the core
runtime error with `legacy security at source span START..END:`; CLI and WASM
then retain their normal `runtime failed:` wrapper, while Python raises the
same core text as `ValueError`.

## `study(resolution=...)` Boundary

The existing focused declaration diagnostic remains the correct Phase 7
result. A future accepted subset needs a separate program-level coordinator
that owns:

- requested execution timeframe and chart/execution metadata;
- whole-program evaluation on execution bars;
- alignment of every normalized output back to chart bars;
- `resolution_gaps` behavior;
- historical, forming, and confirmed update transitions;
- provider lookup and cache identity;
- stable missing-data errors.

None of those responsibilities is silently approximated. The v4 study binder
continues to produce one focused unsupported diagnostic before HIR when
`resolution` or `resolution_gaps` is present.

## Fixture And Host Evidence

Persisted Phase 7 assets include:

- `tests/fixtures/legacy/v4/runtime/security_same_context_legacy.pine`;
- `tests/fixtures/legacy/v4/runtime/security_provider_legacy.pine`;
- `tests/fixtures/legacy/v4/runtime/security_chart_bars.csv`;
- `tests/fixtures/legacy/v4/runtime/security_request_5m.csv`;
- `tests/snapshots/runtime_legacy_v4_security_same_context.json`.

A later corpus-ranked follow-up adds
`tests/fixtures/legacy/v4/runtime/security_pure_udf_legacy.pine` and the paired
`tests/fixtures/legacy/v4/unsupported/security_mutable_udf.pine`. They cover
nested pure requested UDFs, immutable UDF locals, legacy source-input defaults,
and the retained mutable-state boundary.

A subsequent integer-division follow-up originally added a v4 fixture that
assumed all integer divisions truncate.
The initial implementation covered integer-compatible call parameters. The
later corpus follow-up originally applied integer truncation to every Pine
v1-v4 `int / int` expression, including ordinary values, aliases, history
offsets, built-in calls, and untyped UDF arguments. The independent Pine v4
Gaussian Channel comparison later disproved this for input integers: its
`2 / N`, with `N` from `input()`, must retain `0.5` when `N=4`. The v4 rule
now truncates only `const int / const int`, while v1-v3 retain the older
behavior. The corrected v4 fixture is
`tests/fixtures/legacy/v4/runtime/contextual_integer_division_v4_native_legacy.pine`.
A separate version-boundary follow-up adds
`tests/fixtures/runtime/v5_const_integer_division.pine` and its explicit-v6
rewrite: v5 truncates only when both operands are `const int`, while input,
simple, or series integers preserve fractions.

A later bool-call follow-up adds
`tests/fixtures/legacy/v4/runtime/numeric_bool_call_arguments_legacy.pine`.
It extends the existing Pine v1-v5 numeric-to-bool conversion to explicitly
bool-compatible built-in parameters, preserves qualifier bounds, and keeps
Pine v6 strict.

A subsequent array-index follow-up adds
`tests/fixtures/legacy/v4/runtime/array_series_index_legacy.pine`. It aligns
`array.get` and `array.set` with their integer-compatible index contract,
including a per-bar `series int` in Pine v4. Namespace and method calls use the
same signatures; non-integer indexes remain analysis errors, and bounds remain
runtime-checked.

A subsequent drawing-enum follow-up adds
`tests/fixtures/legacy/v4/runtime/dynamic_drawing_enums_legacy.pine`. It admits
per-bar `line` style/extend and `label` style expressions only when their
complete string domain is statically proven to contain supported enum values.
Explicit string-input `options` bound that domain; unbounded inputs and any
invalid branch remain analysis errors. Pine v4's
`label.style_labelup` / `label.style_labeldown` spellings lower to the current
underscored constants and remain unavailable in v5/v6.

Semantic tests cover v1/v2 versus v3/v4 defaults, explicit bool and barmerge
values, named/reordered arguments, invalid versioned signatures, dynamic merge
rejection, user-function shadowing, request-expression reuse, canonical report
spans, and inaccessible HIR callees. Runtime tests cover same-context identity,
v2/v3 intentionally different snapshots, both gaps modes, requested callsite
isolation, requested `syminfo.tickerid`/`timeframe.period` metadata,
historical/incremental/realtime agreement, realtime confirmation, and
source-spanned missing data. A modern same-timeframe gaps-off regression also
fixes the shared forward-fill contract explicitly.

CLI, Python, and WASM test the same persisted same-context golden. Each host
also executes provider-backed legacy security and asserts the stable missing
key/span fragment. CLI parses explicit chart flags, Python tests both direct and
compiled-program keyword entry points, and WASM tests `$chart` parsing plus
validation. Analysis projections in all three hosts assert the same
translation/emulation report.

## Corpus Effect

The unchanged 29-item Phase 0 manifest was run twice at build revision
`phase7`; the reports were byte-for-byte identical with SHA-256:

```text
19812f2920d85c99d42c2511706b9e617f273a835d7ab451f423f6442325fc4e
```

The manifest SHA-256 remained:

```text
775dd5361a4cbfff954cacb78dc3b66bcd02d5bd6c6689657b8374b7cab0d879
```

Rates retain the denominator of 22 eligible legacy indicators:

| Stage | Passed | Attempted | Eligible denominator | Rate |
| --- | ---: | ---: | ---: | ---: |
| Parse | 22 | 22 | 22 | 100% |
| Analyze | 12 | 22 | 22 | 54.55% |
| Lower | 12 | 12 | 22 | 54.55% of eligible; 100% of attempted |
| Historical run | 12 | 12 | 22 | 54.55% of eligible; 100% of attempted |

Within v4, analysis, lowering, and historical execution are now 12 of 12
(100%). `legacy_v4_security` is the only newly passing corpus item; its chart
context and request-data manifest are both consumed by the CLI runtime. The ten
remaining failures are all the expected v1-v3 declaration cluster reserved for
Phases 8 and 9. There are no unknown diagnostics, scope mismatches, or missing
required inputs.

The corpus analyzer still does not claim incremental, realtime, or reference
output comparison. Those stages remain `notRun`; dedicated executable tests
supply the Phase 7 incremental/realtime evidence.

## Deferred Boundary

- Declaration-level `study(resolution=...)` remains fail-closed until the
  program-context contract above is implemented and fixture-backed.
- Lower-timeframe scalar/array requests remain unsupported.
- Pure scalar requested UDFs with immutable local declarations are supported by
  the later bounded follow-up. Persistent or reassigned UDF state, recursion,
  provider-local aliases outside that lexical UDF subset, side effects, arrays,
  arbitrary mutable expressions, and request families outside the current
  whitelist remain unsupported.
- Pine v1-v3 whole indicator declarations and their wider name/constant/type
  surfaces remain Phases 8 and 9, even though the shared security binder and
  runtime policies are versioned now.
- Pine v1-v3 division of two integers discards the fractional remainder.
  Pine v4-v5 truncate only `const int / const int`; input, simple, or series
  integer divisions retain fractional results. Pine v6 always retains the
  fractional result. Explicit `int(...)` remains available for integer
  history offsets and function parameters.
- Pre-v6 numeric-to-bool call conversion applies only to explicitly
  bool-compatible built-in parameters. Generic inferred collection element
  types and unrelated argument families are not widened by that follow-up.
- Series integer indexes are admitted only for `array.get` and `array.set` in
  this follow-up. Other indexed array helpers retain their separately
  fixture-backed qualifier contracts.
- The legacy compatibility path remains indicator-only; legacy strategies are
  permanently out of scope.

## Verification

Targeted semantic, runtime, CLI, Python-wheel, WASM, provider-validation,
corpus, and host-projection tests passed. The complete `scripts/verify.sh`
release gate then passed, including formatting, warning-free workspace Clippy,
the entire Rust workspace, all 534 WASM tests, all 508 installed-wheel Python
tests, the 300-file structural guard, corpus-analyzer tests, host parity over
729 registered CLI runtime snapshots and 433 required runtime plus five
required legacy-analysis Python/WASM golden assertions, and the generated Node
WASM smoke test.

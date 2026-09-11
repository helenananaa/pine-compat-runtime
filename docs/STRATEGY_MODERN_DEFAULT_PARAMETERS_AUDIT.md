# Modern function default parameters audit

Date: 2026-09-08. Status: closed for this default-parameter slice.
Baseline HEAD: `45cbd6586a9e0b6d7e9f137945bacb2eaf9ab346` plus the qualified
worktree from [the previous cycle](STRATEGY_MODERN_NEXT_CYCLE_AUDIT.md).
All files in that cycle's implementation manifest matched at entry. Existing
staged changes remain separate; this task does not commit, push or publish.

## Problem and resulting behavior

TechnicalRating v3 failed to parse its `ratingStatus` header at the defaults
`strongBound = 0.5` and `weakBound = 0.1`. The modern function subset now parses,
validates and binds optional scalar parameters in local and host-provided
imported functions, including private helper calls and separate import aliases.

Supported default forms are scalar literals, signed numeric literals, named
built-in constants, and predeclared scalar inputs such as close, time and na.
Explicit type checks apply even when the default is unused or overridden. An
untyped na default and a v6 bool na default fail. Reference/method defaults,
calls, calculations and user-variable defaults remain explicitly rejected;
dynamic dotted built-in defaults are outside this slice. Existing implicit
qualifier inference is not replaced by a new general inference engine.

Arguments supplied by the caller retain their source order. Omitted defaults
are appended as named arguments and pass through the same argument mapper used
for required parameters. Required parameters can follow optional ones. Duplicate,
unknown, excess, missing-required and positional-after-named errors remain errors.
An untyped default does not freeze the type of explicit overrides.

Defaults must name a built-in at definition time, but an omitted built-in-name
argument is bound in the **caller** scope. TradingView demonstrated that
`feed(source=close)` inside `wrapper(close)` uses the wrapper's close parameter.
An initial unshadowable-input interpretation was disproved by the independent
oracle and removed. Conversely, `close=42; f(x=close)=>x` is rejected because the
name is already a user variable at definition time. Per-call zero-width source
locations prevent one expansion from overwriting another call's name binding.

Typed scalar parameter binding also retains int-to-float promotion and the
declared kind of a typed na. This keeps an omitted default and the same explicit
argument equivalent, including v5 floating-point division. Explicit series
parameters stay series when the supplied/default value is constant.

## Evidence and tests

The [official function syntax](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/)
describes optional parameters and type requirements. Additional constraints were
verified in the authenticated TradingView Chrome editor with original probes:

- `f(x=1)` accepts `f()` and a float override; `g(x=2,y)` accepts `g(y=3)`.
- `close`, signed literals and typed na work as defaults.
- User variables, arithmetic and function-call defaults produce CE10132,
  CE10134 and CE10133 respectively. An already-shadowed close produces CE10132.
- The official TechnicalRating library returns the expected Buy/Neutral/Strong
  Sell values for omitted thresholds and a named weakBound override.
- A time-gated derivative of the original runtime fixture exercises the same
  defaults over fixed real OHLCV in v5/v6. Only accumulator start time and display
  precision differ, so exporting a finite chart window has reproducible state.

Public tests cover positional/named omission, independent callsite state,
typed-na/numeric conversion, caller shadowing and separate callsite bindings,
imported/private helpers, UDT/array passthrough with optional scalar parameters,
pure-expression history identity, incremental execution and forming rollback.
Negative tests preserve declaration, binding, qualifier and recursion failures.
CLI, Python and real WASM/Node share a new original runtime golden.

`FunctionParam.default_value` is an added Rust AST field. Code constructing AST
parameters directly must initialize it (usually None). Public runtime/analysis
JSON schemas and HIR wire structures are unchanged. New diagnostics are
E_FUNCTION_DEFAULT and E_FUNCTION_DEFAULT_TYPE. Parameter metadata and pure-call
argument helpers were split into their existing module families to satisfy the
repository's structural limits without raising thresholds.

## TechnicalRating follow-up

The complete frozen TechnicalRating v3 source now parses with zero diagnostics.
Loading the whole library still fails with E_IMPORT_MISSING_LIBRARY for
TradingView/ta/9 and E_IMPORT_FUNCTION_SIDE_EFFECT for calcRatingAll. Neither
guard was disabled. This is not a claim that the entire library can execute.

The exact ratingStatus export was isolated into a clearly labeled **derived
validation library**, preserving its source and defaults. Local omitted/named
calls return the same three results observed from the full official library.
That isolates this language slice without disguising the full-library blockers.

Follow-up: [function-local array audit](LOCAL_FUNCTION_ARRAYS_AUDIT.md) records
the subsequent exact ta/9 capture, local clear/push implementation and remaining
`simple` qualifier/dependency blockers. The conclusions below describe this
default-parameter cycle's original acceptance.

The next library work needs the exact ta/9 source and an ownership-aware review
of calcRatingAll's array mutations; it must distinguish local deterministic
collection work from forbidden host or global side effects.

Raw source, browser observations, oracle CSVs, reports, hashes and verification
logs remain under `.local/default-parameters-20260908/`. Public CI does not depend
on those local files. Previous G3/r3 references and their independent denominators
remain unchanged; B1 stays UNVERIFIED_INTERNAL_ORDER.

## Final acceptance

The final non-update `scripts/verify.ps1` run exited 0 (`verify-final.log`):
Rust 6,555 tests, a freshly built Python wheel with 676 tests, 101 tool tests,
and real WASM/Node smoke all passed. Host parity covers 878 CLI runtime goldens,
582 required runtime goldens and 5 legacy analysis goldens.

| Independent reference | Closed bars | Series | Compared values | Differences |
| --- | ---: | ---: | ---: | ---: |
| v6 original default-parameter oracle | 509 | 13 | 6,617 | 0 |
| v5 version control | 509 | 13 | 6,617 | 0 |
| Total | | 26 | 13,234 | 0 |

The chart was OKX:BTCUSDT, 15-minute standard candles, with native UTC epoch
seconds converted to runtime milliseconds. All 509 bars up to the preselected
1788785100000 cutoff were compared, without skipped warmup bars. The accumulator
begins at 1788480000000 in both executions. Numeric absolute/relative tolerances
remain 1e-9; source hashes are checked before reproducing the comparison.
This is a language-value reference, not a new strategy-trade accuracy denominator.

Fresh Python/WASM artifacts additionally match the CLI's complete output in four
checks. Ten mode checks cover true append, realtime historical seed, forming
replacement, and the first two active accumulator bars. Previous frozen G3/r3
outputs pass 18 separate regression checks without overwriting the old artifacts.

The target ratingStatus validation slice yields the same three results seen in
the full official TradingView library. The complete library still has the two
documented blockers; no guard or unsupported capability was silently removed.

Only the new default-parameter runtime golden and the matrix changed in this
cycle. Existing runtime and legacy-analysis goldens were retained. An intermediate
legacy analysis regression was corrected in code rather than refreshed away.
All original staged contents remain byte-for-byte unchanged. No commit, push,
merge or release was performed. Final manifests, implementation archive and
retained host artifacts are in the ignored evidence directory named above.

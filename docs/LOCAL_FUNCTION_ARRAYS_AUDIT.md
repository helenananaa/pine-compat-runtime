# Function-local array mutation — 2026-09-08

Status: bounded implementation validated; full TechnicalRating compatibility remains open.

This follows [the default-parameter audit](STRATEGY_MODERN_DEFAULT_PARAMETERS_AUDIT.md).
Pine v5/v6 functions can now use direct `array.clear` and `array.push` statements
on scalar arrays constructed inside the function. This also applies to exported
library functions. The admission check and function-call analyzer use the same
ownership proof. Runtime storage, execution and host contracts are unchanged.

The proof recognizes straight-line declarations and direct namespace calls.
It does not extend support to array parameters, global receivers, aliases,
methods, indirect receivers or mutations inside control-flow blocks. An array
binding that may be reassigned anywhere in the body is excluded: a persistent
`var` may otherwise hold an external array on the next bar, even when the
reassignment appears after the mutation. Unknown control flow invalidates the
proof. Argument expressions retain their ordinary effect and type checks.
These are current implementation limits, not claims that Pine forbids all such
collection operations.

## Evidence

`scripts/verify.ps1` exited 0 with 6,558 Rust tests, 676 tests against a fresh
Python wheel, 101 tool tests, real WASM/Node smoke, and the unchanged host-parity
set (878 registered CLI runtime snapshots; 582 required runtime and 5 required
legacy-analysis goldens). Existing goldens were not refreshed.

The new runtime test covers v5/v6 and local/imported functions, independent
call-site accumulators, clear/push/average, incremental append and repeated
forming-bar replacement followed by confirmation. With closes 10, 20, 30,
the three independent outputs are `[10,30,60]`, `[1,2,3]`, and `[11,21,31]`.
Negative tests retain rejection of external receivers, reassignment before or
after mutation, branch reassignment, direct/nested plot effects and effects
inside call arguments. Eight additional fresh Python/WASM comparisons match
the complete CLI output for those four local/imported version combinations.

## Exact upstream source and remaining work

The [official ta page](https://www.tradingview.com/script/BICzyhq0-ta/) currently
shows version 14. The reference used here was selected through its Pine Editor
version history: **9, Nov 26, 2024, 19:42**, with source comment
`// v9, 2024.11.26`. The complete 842-line clipboard source and MPL notice were
retained locally, without replacing it with the current version or committing
third-party code to the repository.

With the original TechnicalRating/3 source, the `calcRatingAll` import-effect
diagnostic is now removed. Without ta/9, the missing-library diagnostic remains.
Supplying the exact ta/9 source exposes the next parser gap at line 85:
`simple int shortLength = 5` in `ao`. The resulting 121 diagnostics include
parser recovery cascades and do not represent 121 independent defects.
The source also imports `TradingView/RelativeValue/3`, which still needs its
exact source supplied. No complete TechnicalRating runtime/reference acceptance
is claimed, and no new strategy accuracy denominator is inferred from this work.

Next: implement explicit scalar `simple` parameter qualifiers with qualifier
validation, supply the exact remaining dependency, then reassess the unmodified
library before expanding array support or making end-to-end compatibility claims.

Raw sources, diagnostic JSON, fresh host artifacts, comparison scripts, hashes
and gate logs are under `.local/library-arrays-20260908/`. Public tests do not
depend on this ignored evidence directory. The original staged patch was
preserved byte-for-byte. No commit, push, merge or release was performed.

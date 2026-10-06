# Legacy Indicator Corpus Seeds

These sources are original minimal indicators written for the legacy
compatibility corpus. They are not copied from third-party or protected
scripts.

`corpus.tsv` is the deterministic Phase 0 manifest consumed by
`scripts/analyze_legacy_corpus.py`. The public seed corpus intentionally covers
common legacy failure families rather than claiming to represent the user's
private indicator library. Authorized whole-script samples can be added to a
private manifest without changing the analyzer.

`legacy_strategy_excluded` rows keep strategy sources outside the indicator
denominator and corpus compiler path. Pine v4 strategies now have a separate,
measured strategy execution slice, as do v1-v3 strategy controls; this
indicator corpus does not count them.
The analyzer derives strategy mode from source so an incorrect manifest scope
cannot silently count a strategy as an indicator.

Five `invalid_control` rows exercise lexer, parser, unknown-name, call-shape,
and type failures. `control_modern_v6` must still analyze and run successfully.
Together these controls distinguish legacy compatibility failures from a
broken analyzer or CLI.

After building the CLI, generate the deterministic report with:

```text
cargo build -p pine-cli
python3 scripts/analyze_legacy_corpus.py --output /tmp/legacy-corpus.json
```

Corpus report schema 4 retains schema 3's privacy-preserving `executionTimes`
input availability and adds three-mode resource and provider-cache evidence.
The optional manifest
`execution_times_path` is forwarded to CLI `--execution-times`; reports expose
only whether the file was supplied, passed preflight, or was missing, never its
path or timestamp values. `eligibleSuccessRate` always uses every eligible
script in the profile as the denominator; later-stage `not_run` and
`missing_input` rows therefore cannot inflate the promotion measurements.
Failure clusters expose both diagnostic occurrence counts and affected-script
shares. `requiresDisposition` becomes true at the provisional 2% per-profile
threshold, but only qualifying unknown clusters block the automated baseline
assessment.

Passing `stableBaseline.thresholdsMet` is not a stable release claim. The
release registry must still prove incremental, realtime, provider, resource,
cache, and host-parity behavior.

Before combining a private v4 selection with the 12 committed public v4 seeds,
run the version-aware dedup audit. Its JSON contains one-way fingerprints and
opaque private ids, not source text, paths, or titles:

```text
python3 scripts/audit_legacy_corpus_dedup.py \
  --candidate-manifest /absolute/path/to/corpus-r2.tsv \
  --build-revision corpus-r2-dedup \
  --output /absolute/path/to/corpus-r2-dedup.json
```

Exact, normalized-text, and token-equivalent matches are reported separately.
The token fingerprint removes comments and trivia but remains version-bound;
it does not claim that differently written programs are semantically equal.

For a private or user-authorized R2 corpus, keep sources outside the repository
and point the analyzer at an external root:

```text
python3 scripts/analyze_legacy_corpus.py \
  --manifest /absolute/path/to/corpus-r2.tsv \
  --root /absolute/path/to/corpus-root \
  --build-revision corpus-r2-pre-code \
  --output /absolute/path/to/corpus-r2-pre-code.json
```

The importer defaults to private user-authorized intake. For a corpus sourced
from a permissively licensed repository, record the immutable upstream origin,
revision, and SPDX-style license id instead of reusing that private label:

```text
python3 scripts/import_legacy_corpus.py \
  --source-dir /absolute/path/to/selected-sources \
  --output-dir /absolute/path/to/corpus-r3 \
  --id-prefix gh-v4-r3 \
  --license-class permissive \
  --source-origin https://github.com/owner/repository \
  --source-revision <full-commit-sha> \
  --license-id MIT
```

Permissive imports fail before writing unless all three provenance fields are
present. The public report still exposes only `licenseClass`; the ignored local
`intake-summary.json` owns the upstream URL, revision, and license id.

When one R3 selection spans several independently imported repositories, merge
their manifests without losing the individual file roots:

```text
python3 scripts/merge_legacy_corpus_manifests.py \
  --manifest /absolute/path/to/first/corpus.tsv \
  --root /absolute/path/to/first/root \
  --manifest /absolute/path/to/second/corpus.tsv \
  --root /absolute/path/to/second/root \
  --exclude-id audited-non-standalone-id \
  --expected-version 4 \
  --expected-scope legacy_indicator \
  --output /absolute/path/to/combined-v4.tsv
```

When any explicit `--root` is needed, supply one root for every manifest in the
same order. Without `--root`, each manifest defaults to its own parent. The
merger converts only file-input columns to absolute paths, sorts opaque ids,
rejects collisions, and never copies source text into its output.
For rows with request data, it also writes deterministic JSON sidecars under
`<output>.request-data/`; nested CSV paths are made absolute so independently
rooted corpora remain executable after the merge. Sidecar ids are constrained
to filename-safe opaque ids, and existing sidecar directories are never
overwritten.
`--exclude-id` is repeatable and fails on unknown ids; use it only when intake
evidence shows that an upstream file is incomplete or requires an unavailable
preprocessor/dependency, never merely because the interpreter rejects valid
Pine semantics.

The committed, redacted Phase 0 result is recorded in
`docs/LEGACY_INDICATOR_PHASE0_BASELINE.md`.

Versioned `v*/syntax`, `v*/sema`, `v*/runtime`, and `v*/unsupported`
directories own paired compatibility fixtures added after the baseline. Phase 3
adds v4 declaration and exact-alias pairs under `v4/sema` and `v4/runtime`;
these files are original project fixtures and are also referenced by
`conformance.tsv`.

The chart-context declaration slice adds the paired
`v4/runtime/study_empty_resolution_*` fixtures. The legacy source proves that
the exact `study(resolution="")` form inherits the host chart symbol and
timeframe without requesting provider data; non-empty and dynamic resolution
forms remain unsupported until a whole-program execution coordinator exists.

Phase 4 adds the paired `v4/runtime/inputs_*` fixtures for all supported Pine v4
input type constants, metadata, callsites, default values, and scalar host
overrides. `v4/sema/input_constant_alias.pine` owns the local const-alias case.

Phase 5 adds paired `v4/runtime/outputs_*` fixtures for all ten initial output
families, primitive plot/hline styles, transparency defaults and alpha
precedence, visual metadata, normalized colors, and historical execution.
`v4/unsupported/output_arguments.pine` keeps later-only output arguments behind
an analysis-time diagnostic.

The corpus-ranked pre-v4 output slice adds paired `v1/runtime/outputs_*`
fixtures for the same ten output families using the documented v3 parameter
tables, including historical transparency defaults and context-specific style
ordinals. `v1/unsupported/output_arguments.pine` keeps later `display` and
`fillgaps` roles outside the v1-v3 surface.

The next corpus-ranked request slice adds
`v1/runtime/security_aliases_legacy.pine` for dynamic input resolution,
immutable requested-series alias expansion, const/input capture, provider
alignment, and v1 historical lookahead. The release profile verifies batch,
incremental, realtime, provider, and resource behavior;
`v1/unsupported/security_mutable_alias.pine` preserves the historical mutable
variable rejection. The follow-up pure-function slice adds
`v4/runtime/security_pure_udf_legacy.pine` for nested, immutable UDF
recomputation and legacy `input.source` default-source selection in the
requested context;
`v4/unsupported/security_mutable_udf.pine` keeps reassigned UDF-local state
outside that bounded subset.

The next Pine v4 request fixture pair,
`v4/runtime/security_udf_local_dependencies_legacy.pine` and its explicit
canonical rewrite, covers a `security` call placed directly in a UDF body. Its
scalar parameters and normal immutable scalar locals are dependency nodes:
series nodes recompute in the requested context, while const/input/simple nodes
are captured. An earlier three-positional-argument legacy request may feed a
later request only when symbol, timeframe, gaps, and lookahead are identical.
`v4/unsupported/security_udf_local_dependency_mismatch.pine` keeps a different
requested symbol fail-closed, while
`v4/unsupported/security_udf_control_flow_local.pine` covers a request nested
under a local `if`; reassignment, persistence, recursion, and the modern
provider-local boundary are unchanged.

The integer-division call-shape slice first covered v1-v3 truncation, including
aliases, history offsets, integer-compatible calls, and untyped UDF arguments.
Native Pine v4 Gaussian Channel output then established a different boundary:
`2 / N` retains a fraction when `N` comes from `input()`. The focused
`v4/runtime/contextual_integer_division_v4_native_legacy.pine` fixture and its
explicit-v6 canonical rewrite cover v4 `const int / const int` truncation and
fractional input division. Float operands remain fractional. The separate
version-boundary fixture pair,
`../runtime/v5_const_integer_division.pine` and
`../runtime/v6_fractional_integer_division.pine`, proves that v5 truncates only
two `const int` operands; input and series integers retain fractional results,
and v6 requires an explicit `int(...)` cast when truncation is intended.

The next call-shape slice adds
`v4/runtime/numeric_bool_call_arguments_legacy.pine`. It extends the existing
Pine v1-v5 numeric-to-bool rule to bool-compatible built-in parameters using an
explicit canonical `bool(...)` lowering. The source qualifier is preserved,
zero and `na` are false, nonzero numerics are true, and v6 remains strict.

The following array-index slice adds
`v4/runtime/array_series_index_legacy.pine`. It proves that `array.get()` and
`array.set()` accept a per-bar `series int` index in Pine v4, matching the
general array contract. Modern namespace and method forms share the same
integer-compatible signature, while float and string indexes remain rejected.

The later insert-index slice adds
`v4/runtime/array_insert_series_index_legacy.pine` plus
`tests/fixtures/runtime/array_insert_series_index.pine`. It extends the same
integer-compatible index contract to `array.insert()` / `.insert()`, so a
per-bar `series int` index inserts before that slot. `array.remove` remains
simple-int-only, and non-int insert indexes stay rejected.

The following request-expression slice adds
`v4/runtime/security_time_alias_legacy.pine` plus
`tests/fixtures/runtime/request_security_time_function.pine`. It admits
positional `time(timeframe)` calls, so a v4 top-level `time("D")` alias and a
`valuewhen` graph that depends on that alias can be requested. `time_close()`
and named `time()` arguments remain rejected.

The later barstate slice adds
`v4/runtime/security_barstate_islast_legacy.pine` plus
`tests/fixtures/runtime/request_security_barstate_islast.pine`. Provider and
legacy UDF-local `security` expressions may read `barstate.islast` against the
requested stream. Other `barstate.*` flags remain provider-rejected.

The subsequent drawing-enum slice adds
`v4/runtime/dynamic_drawing_enums_legacy.pine`. It proves that supported
`line` style/extend and `label` style values can vary by bar, including the
historical v4 `label.style_labelup` / `label.style_labeldown` spellings.
Static enum-domain checks keep unbounded strings and invalid branches rejected.

The later plot-style slice adds
`v4/runtime/plot_dynamic_style_legacy.pine` plus
`tests/fixtures/runtime/plot_dynamic_style.pine`. It admits `plot()` style
values whose proven domain is a documented `plot.style_*` enum, including
per-bar ternaries. The emitted plot metadata keeps the last evaluated style.
Unbounded strings and series integer ordinals remain rejected.

The following request and output-enum slices add
`v4/runtime/security_time_close_legacy.pine`,
`v4/runtime/security_barstate_flags_legacy.pine`, and
`v4/runtime/plotshape_hline_dynamic_style_legacy.pine`. Requested expressions
may call `time_close()`, pass documented named `time()` / `time_close()`
arguments, and read the remaining `barstate.*` flags against the requested
stream. `plotshape` style and `hline` linestyle follow the same proven enum
domain as `plot` style. Named arguments on other requested calls stay
rejected.

The following `na`-origin slice adds
`v2/runtime/bool_numeric_comparisons_legacy.pine` and
`v4/runtime/udf_source_order_builtin_aliases_legacy.pine`. The first proves
that Pine v1/v2 bool-versus-numeric comparisons use the same explicit
boolean-to-float lowering as their arithmetic profile, while v3 stays strict.
The second proves that, from v3 onward, a global declared after a UDF body does
not retroactively hide an unqualified historical built-in call in that body.
`v4/unsupported/udf_earlier_legacy_alias_shadow.pine` keeps an earlier lexical
collision rejected. Bare or failure-derived `na` call arguments are not
contextually accepted by this slice.

The following output-offset slice adds
`v4/runtime/series_output_offset_legacy.pine` and its explicit constant-offset
v6 rewrite. Pine v4/v5 `plot`, `plotchar`, `plotshape`, `plotarrow`,
`bgcolor`, and `barcolor` accept `series int` offsets and apply the final
evaluated value to the complete output. The v3 and v6 negative fixtures keep
their simple-int boundary, and ordinary `expr[offset]` history access is not
changed.

The next UDF-return slice adds
`v4/runtime/udf_final_statements_legacy.pine` and an explicit-expression v6
rewrite. A final local declaration or reassignment returns its bound value,
branch-final declarations participate in conditional results, and a final
conditional without `else` returns `na` on the missing path.
The following reference-side-effect slice adds
`v4/runtime/udf_reference_side_effects_legacy.pine` and an explicitly expanded
v6 rewrite. Pine v4 UDFs may use the corpus-backed namespace calls
`array.set/pop/unshift/clear`, `label.new/delete`, `line.new/delete`, and the
separately corpus-proven `line.set_x2/set_extend` pair, including final
side-effect-only conditionals and loops. The paired runtime fixtures cover
historical, incremental, realtime historical, forming replacement, rollback,
and confirmation; `v4/unsupported/udf_other_reference_side_effects.pine` keeps
`array.push`, `label.set_text`, `line.set_x1`, and all other collection or
drawing mutations fail-closed.

The following parser slice adds
`v1/runtime/ternary_continuation_legacy.pine` and a single-line v6 rewrite.
Only a no-directive v1 source at global scope may treat exactly four ASCII
spaces as a continuation when the line boundary is adjacent to ternary `?` or
`:` punctuation. The paired fixture covers both operator-at-end and
operator-at-start forms. Explicit versions, tabs, local blocks, ordinary
multiple-of-four indentation, and consumer typing remain unchanged.

The next declaration-graph slice adds
`v1/runtime/graph_source_order_prerequisite_legacy.pine` and its explicit v6
rewrite. An earlier scalar `input()` needed only to infer a later self-history
chain remains in ordinary source order instead of becoming a predeclared graph
node. The same pair proves the exact v1-v4 `rising` / `falling` mappings to
`ta.rising` / `ta.falling`. The v2 unsafe-initializer fixture still rejects an
input that is itself a graph target, and
`v2/unsupported/forward_reference_unsafe_initializer_barrier.pine` prevents a
current-value forward edge from moving across an input declaration.

The following version-annotation slice adds
`v4/runtime/spaced_version_annotation_legacy.pine` and its canonical v6
rewrite. Horizontal whitespace around the equals sign in `//@version = 4`
selects an explicit v4 dialect instead of falling back to implicit v1. The
prefix remains exact: `// @version=6` is still an ordinary comment. The paired
fixture exercises v4-only qualified colors, size constants, `study`
`max_bars_back`, output metadata, and alert declarations through historical,
incremental, realtime, and resource release gates.

The next conditional-result slice adds
`v4/runtime/nested_if_expression_legacy.pine` and its canonical v6 pair. A
complete nested `if`/`else-if`/`else` statement at the end of an enclosing
value-producing `if` block recursively supplies that branch's result.
`sema/unsupported_nested_if_branch_return.pine` keeps a nested leaf ending in
a reassignment behind `E_BRANCH_RETURN`.

The following input-runtime slice adds
`v4/runtime/named_input_default_legacy.pine` and its canonical v6 pair. Input
defaults are selected by the canonical `defval` parameter instead of raw source
position, so a named `title` or other metadata argument may precede `defval`
without becoming the runtime value. Callsite allocation, metadata, host
overrides, and the positional-input path remain unchanged.

Phase 8 adds paired `v3/runtime/core_*` fixtures for the executable v3 name,
constant, declaration, input, output, chart-metadata, and untyped-`na` slice.
`v3/sema/shadowing.pine` proves that source declarations retain precedence over
fallback aliases, while `v3/unsupported` owns stable fixtures for ambiguous
`na` inference and later-only call parameters.

Phase 9 adds the implicit-v1/explicit-v2 shared pair and the paired
`v2/runtime/core_*` fixtures for self-history, current and historical forward
references, bool arithmetic, and numeric conditions. `v2/unsupported` owns the
stable graph cycle, statement-barrier, and unsafe-initializer diagnostics;
v3/v6 negative controls prove the conversion version boundaries. The
host-neutral `runtime_legacy_v2_core.json` golden is generated by the CLI and
required by Python and WASM parity tests.

Phase 10 adds the dedicated implicit-v1 and v4-input runtime goldens plus five
complete `analysis_legacy_*` reports spanning v1-v4 and a v2 graph failure.
`scripts/host_parity_required.txt` and
`scripts/legacy_analysis_parity_required.txt` are the explicit two-host policy
manifests; `scripts/check_host_parity.py` verifies the CLI registrations and
both Python/WASM assertions.

The execution-clock slice adds
`v4/runtime/timenow_execution_clock_legacy.pine` and its canonical v6 pair.
`timenow_execution_times.txt` provides the deterministic per-execution UNIX
millisecond inputs used by the `deterministic_clock` release profile. The gate
proves historical, incremental, forming replacement/rollback, and confirmation
behavior without substituting bar time or a process wall clock.

# EMA initialization candidate

2026-09-09. Status: independent modern references match; regression review
and integration pending. Candidate worktree:
`E:/projects/pine-interpreter-delivery-ema`, based on ad2063878. Main has not
adopted this candidate. No release or existing golden refresh.

## Evidence and implementation

The separate EMA reference batch starts at TradingView bar_index=0 and uses
21,133 frozen closed OKX:BTCUSDT 15-minute bars. All OHLCV and timestamps match
the supplied runtime input; no warmup rows are skipped. Numeric tolerance
remains absolute/relative 1e-9. Before correction, EMA3 differed at 22 values
and EMA200 at 1,704; SMA3 and bar_index matched.

v5/v6 probes confirm an arithmetic mean of the first length non-na samples
as the seed. Missing inputs are not counted toward that seed and yield na.
After seeding, missing inputs preserve the internal previous value: following
three na inputs, the next EMA3 is half the next source plus half the previous
non-na EMA. Length 1 returns the available source.

The candidate changes only eval_ema: use its checkpointed callsite rolling
window during seeding; afterward retain the existing recursive update. No
changes to the shared ema_next helper, MACD, KC, TSI, DEMA or TEMA. Their own
initialization contracts are not inferred from standalone ta.ema evidence.

| Independent batch | Compared values | Differences |
| --- | ---: | ---: |
| EMA3/EMA200 with SMA3 and bar_index controls | 84,532 | 0 |
| v5 initial/interspersed missing values, lengths 1/3, real EMA200 | 105,665 | 0 |
| Same probe in v6 | 105,665 | 0 |
| v6 recovery after missing values | 63,399 | 0 |
| legacy v4 EMA3/EMA200 and bar_index | 63,399 | 0 |

Original runtime tests verify manual seed arithmetic, callsite independence,
batch/append equivalence and repeated forming replacement on/before the seed
bar. These targeted tests pass. They are not a full regression gate.

Grok's eight-turn implementation task exhausted its bound without editing.
Codex confirmed terminal status, implemented the candidate and ran tests and
comparisons. No unbounded approval or subagents were enabled.

## Downstream review still required

An isolated temporary test harness captured all 932 runtime outputs into an
ignored candidate directory. It did not overwrite existing golden files and
its successful capture is not a passing regression test. The test harness was
restored byte-for-byte afterward. Ten outputs differ:

- runtime_ema_rma_edge_cases.json
- runtime_legacy_v3_core.json
- runtime_legacy_v4_expressions.json
- runtime_loop_state_interactions.json
- runtime_request_security_same_context.json
- runtime_simple_scalar_parameters.json
- runtime_ta.json
- runtime_ta_named_reordered_remaining_averages.json
- runtime_typed_declaration_qualifiers.json
- runtime_math.json (three platform last-bit differences; do not refresh)

The other nine differences are confined to plot values downstream of EMA;
complete field-level deltas were retained. v4 reference passes; v3 and affected
loop/request assumptions must be reviewed before selecting any golden updates.
Then update only reviewed expectations, run full Windows and installed-wheel /
real WASM gates, repeat frozen modern references, and integrate a qualified
commit. The current candidate is not yet deliverable.

Evidence in `.local/delivery-20260909/`: ema-candidate-manifest.json,
ema-comparison-before.json, ema-comparison-candidate.json,
ema-control-comparison.json, ema-golden-field-deltas.json,
ema-targeted-v2.log. This batch does not alter older G3 or public r1 denominators.

# EMA/SMA initialization and repeated-call qualification

2026-09-09. Scope: standalone EMA initialization and SMA/EMA repeated calls
on one executed bar. Core implementation and full Windows gate are qualified;
final source/host artifact receipts accompany the local integration. No release.

## Observable contract

- EMA seeds from the mean of its first `length` non-na executed-bar samples.
- Repeated SMA/EMA calls on one bar replace that bar's tentative sample, rather
  than appending extra historical samples. EMA always recalculates from the
  previous committed EMA, not the preceding iteration's output.
- A final na EMA input discards any earlier tentative sample for that bar and
  preserves committed history. Conditional non-execution does not advance it.
- Same-bar window length shrink/grow restores the pre-bar tail before replacing
  the sample. Ordinary variable accumulation inside loops remains unchanged.
- Callsites and requested runtimes remain independent. Existing forming and
  strategy-evaluation checkpoints include the new state.

The window stores only evicted elements plus exact prior aggregate values for
undo; it does not clone its whole deque on each update. Profile rolling-window
value/capacity totals include this retained undo storage. Other TA algorithms
keep their existing paths; their repeated-call semantics are not qualified by
inference. Runtime and analysis JSON schemas are unchanged.

## Independent evidence

All controls use 21,133 frozen closed bars, begin at TradingView bar index 0,
skip no warmup bars and retain absolute/relative tolerance 1e-9. Raw captures,
source files, normalized input, before/after outputs and comparisons are under
`.local/delivery-20260909/` in the main checkout.

| Control | Compared values | Final differences |
| --- | ---: | ---: |
| EMA3/EMA200, SMA3 and bar index | 84,532 | 0 |
| v5 missing inputs, lengths 1/3 and real EMA200 | 105,665 | 0 |
| Same missing-input control in v6 | 105,665 | 0 |
| v6 recovery after missing values | 63,399 | 0 |
| v4 EMA3/EMA200 and bar index | 63,399 | 0 |
| v3 EMA3/EMA200 and native n counter | 63,399 | 0 |
| Full original loop calculation plus same-context request/direct EMA | 105,665 | 0 |
| Repeated EMA, including final na | 63,399 | 0 |
| Repeated SMA, final na and changing length | 84,532 | 0 |

These are control scenarios sharing a frozen market window, not nine distinct
strategy families or an expanded public-r1 denominator. The original v3 probe
incorrectly used bar_index; its compile error/source were retained separately,
and the native-v3 wrapper uses n without changing the EMA calculation.

Initially EMA3/EMA200 differed at 22/1,704 values. A seed-only candidate fixed
those but exposed a pre-existing loop error: both baseline and seed-only
candidate differed at 21,124 loop outputs and all 21,133 carried values.
That failure was retained and led to the repeated-call correction; the loop
golden was not approved merely because the candidate produced a new value.

Nine final scenarios also undergo 18 exact complete-output comparisons between
CLI and a newly installed Python wheel / actual generated WASM module. The CLI
outputs are independently compared to the captured TradingView columns. The
existing G3 real-strategy batch remains separate: all 65 trades and its prior
series comparisons pass unchanged in the retained rerun.

## Regression and delivery receipts

- Final synchronized Windows gate: 6,588 Rust tests, 677 tests against a freshly
  installed Python wheel, 101 tool tests, real WASM/Node, structural and host
  parity checks. Log: ema-final-synced-verify.log (exit 0).
- Original tests cover seed arithmetic, separate callsites, same-bar overwrite,
  final-na discard, changing lengths, exact aggregate restoration, checkpoint
  clones, append equivalence and forming replacements around the seed bar.
- All 932 runtime outputs were captured to a separate ignored directory. Twelve
  reviewed plot-value-only goldens changed; runtime_math last-bit differences
  were excluded. Exact reviewed deltas: ema-approved-golden-deltas.json.
- Related unit/CLI/Python/WASM hard-coded EMA expectations were derived from
  the new seed rule. The nested legacy request test preserves its original
  six-bar prefix and adds subsequent bars to test populated post-warmup output.
  No tolerance or resource ceiling was relaxed.
- Only ta.sma and ta.ema matrix entries changed, to describe the tested rules.
- Candidate artifacts are under ema-final-hosts/ (inside the evidence directory),
  including the installed-wheel environment and generated WASM module. These
  are local debug qualification artifacts, not a production release matrix.

Grok supplied a bounded read-only delta review, repeated-call design and the
window undo implementation. Codex obtained the independent evidence, rejected
unsupported review assumptions, implemented EMA state integration, reviewed
all changes and ran the actual tests. Grok's earlier no-edit turn-limit failure
and all intermediate regression failures were retained. No subagents, blanket
auto-approval, push or publication were enabled.

Full TechnicalRating dependency-chain execution, other strategy/timeframe
reference expansion, long-session resource qualification and final cross-platform
release delivery remain open in DELIVERY_ROADMAP.md.

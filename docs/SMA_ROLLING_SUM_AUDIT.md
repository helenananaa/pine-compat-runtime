# SMA rolling replacement audit

Baseline 115d76ab4 had one remaining TechnicalRating MA decision difference at
index 15460, also affecting the total rating. Native/local time and close agree
at 1783818000000 / 63922.6. Only the SMA(10) component differs.

The native SMA is above close by about 1.30967237055e-10; the previous local
rolling sum puts it below close. Recomputing the ten exact binary inputs gives
the same mean as close. math.sign correctly preserves the raw sign, so changing
its semantics or loosening rating comparison tolerance would conceal the issue.

Independent evidence in `.local/delivery-20260909/`:

- sma-order-control: native SMA versus add/remove, remove/add and fresh sum.
- sma-input-precision-control: all ten prices have zero difference from frozen
  input literals; native math.sum/10 agrees with SMA at this point.
- sma-price-roundtrip-control: zero price roundtrip differences through index
  15460, ruling out observed loss at the market's price grid.
- sma-recurrence-control: none of three tested raw double recurrences exactly
  reproduces every native step. We do not claim TradingView's internal algorithm.
- sma-nearby-replacement-control: positive and negative four-sample controls
  both have zero residual against +/-1056.175 after the replacement. This
  independently agrees with the candidate's additional small-window regression.

The candidate forms incoming-minus-outgoing before updating the accumulated sum
for one-for-one, same-sign replacements whose magnitudes are within a factor of
two. Sterbenz's exact-subtraction condition avoids losing low bits in two large
sum updates. Initialization, resizing, missing transitions, opposite signs and
large magnitude changes retain their existing paths. Exact pre-bar aggregate
restoration remains in force for repeated calls and discard; no whole-window
clone, new history allocation, price normalization or comparison tolerance is
introduced. This improves the measured behavior; it is not a universal bitwise
native summation guarantee or a claim of improved accuracy for the fallback path.

The bounded candidate matches all three frozen original-library outputs:
21,133 bars, 63,399 values, zero skipped warmup, unchanged absolute/relative 1e-9
tolerances. Artifact: technical-bounded-delta-candidate-result.json. Original
sources and references remain unchanged. Twelve window tests and EMA/MACD
state-mode tests pass. Candidate capture of 936 existing snapshots shows only
the known unrelated runtime_math last-bit delta; no old golden is updated.

The new six-bar sma_nearby_replacement fixture tests the active replacement in
CLI, installed Python and WASM, including real Node instantiation. Full gate:
sma-delta-full-verify.log exited 0: 6,621 Rust tests, 682 installed-wheel Python
tests, 103 tool tests and real generated WASM/Node. Complete-library mode/host acceptance
and the overall resource/release goals remain separate requirements.

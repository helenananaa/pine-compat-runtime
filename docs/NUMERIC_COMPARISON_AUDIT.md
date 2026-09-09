# Numeric comparison qualification

Baseline: 5dfeaf4cc. Full TechnicalRating had 129 remaining oscillator rating
differences, first at bar 487. Independent component controls matched time/close
and isolated a Stochastic RSI K/D comparison: local values differed by about
7.1e-14, producing a sell rating where the native component was neutral.

The [official type-system documentation](https://www.tradingview.com/pine-script-docs/language/type-system/)
describes limited precision for float comparisons. We did not implement decimal
quantization from that description alone: native boundary controls distinguish
the observable behavior from separately rounding each operand. An absolute
difference at most 1e-10 compares equal; a larger difference does not. This is
not relative error, and the original values remain unchanged. math.sign still
observes the raw difference. Array includes/indexof share numeric equality.

Native controls in `.local/delivery-20260909/`, each with retained Pine source
and DOM receipt:

- float-comparison-control: six-operation examples, crossing decimal rounding
  buckets, raw signs and series comparisons.
- float-comparison-sweep: differences around 1e-10 at zero and 100, plus larger
  magnitudes. Floating representation matters at the exact boundary: the actual
  difference between 100 and 100+1e-10 exceeds the threshold, unlike 0 and 1e-10.
- float-operator-contract-v6 and -v4: six operators, inclusive/exclusive boundary,
  raw arithmetic/sign; v6 additionally checks array lookup.
- float-qualifier-v4 and -v5: v4 constant comparisons are exact, while input/series
  comparisons use the runtime boundary. v5 constants also use the boundary.

Implementation uses one host-neutral IR helper in runtime comparison, array
lookup and modern constant/history evaluation. Pre-v5 constant comparisons retain
exact folding; the v4 native control proves that distinction, while older profile
claims remain unchanged. No price/broker comparison, arithmetic value, math.sign,
or reference acceptance tolerance is modified.

New tests cover versioned const/series behavior, all six operators, array lookup,
preserved raw sign and a history bound whose constant conditional must agree
with runtime. One old unit expected 0.1+0.2 != 0.3 in the modern test dialect;
native v5/v6 evidence disproves it. The original operands are preserved and its
expected branch is corrected. Initial failure: numeric-comparison-regressions.log.
Candidate capture of 935 existing runtime snapshots shows no new semantic delta;
the sole runtime_math difference is the previously identified platform last bit
and is not updated. The new numeric_comparison fixture covers CLI, installed
Python, WASM Rust tests and actual generated WASM/Node.

Full original root candidate comparison is now 1/0/1 mismatches across total,
oscillator and MA outputs, all at index 15460: 21,133 bars / 63,399 values,
zero skipped warmup, unchanged absolute/relative 1e-9 tolerances. This is still
a failed full-library oracle. Artifacts: technical-comparison-candidate-runtime.json
and technical-comparison-candidate-result.json. Final Windows gate is
numeric-comparison-full-verify.log exited 0: 6,618 Rust tests, 681 fresh
installed-wheel Python tests, 103 tool tests and real generated WASM/Node.
This is Windows qualification, not final Linux/release-artifact acceptance.

The separate MA component probe at index 15460 matches native/local time and
close (1783818000000 / 63922.6). Only MA component 0, SMA(10), differs: native
-1 versus local +1. All other 14 MA component ratings agree. This remaining
calculation/sign difference is not corrected by the comparison rule, because
math.sign deliberately retains raw-value behavior. Source/runtime/DOM receipts
are technical-component-probe-15460.pine, technical-component-local-15460.json
and technical-component-native-15460-dom.txt in the same local evidence folder.

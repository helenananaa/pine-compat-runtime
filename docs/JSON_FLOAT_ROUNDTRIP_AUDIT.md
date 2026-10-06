# Exact JSON floating-point round trips

Retained Pivot qualification at `9f9a24aad` found two distinct issues. First,
historical lookahead-on annual snapshots cannot be treated as a realtime feed:
the current annual bar must be delivered as forming, separate from completed
provider years. The failed static-input four-mode equality check is retained in
`.local/product-completion-20260912/pivot-retained-reference-v2.log`. Corrected
feed simulation seeds completed years, then explicitly supplies the captured
current annual sample as forming. End-of-history projection can create drawings
earlier than a one-shot historical run, so native numerical reference comparison
and full-output comparison across identical execution profiles are separate.

That explicit feed exposed a real serialization defect: WASM session and replica
results differed in 18 label/line coordinates by one or a few least-significant
floating-point bits. Examples include 96118.61666666665 becoming
96118.61666666664. Failed full results and a field-level diff are retained under
`pivot-hosts-9f9a24aad/pivot-original-live-payload.json.*` and
`pivot-wasm-live-replica-differences.json`.

The runtime now enables serde_json's `float_roundtrip` feature. Its upstream
[feature contract](https://docs.rs/crate/serde_json/1.0.150/source/Cargo.toml.orig)
provides exact f64-to-JSON-to-f64 parsing instead of default best-effort precision.
The runtime dependency activates this behavior for its native and WASM consumers.
This does not change the output schema or numerical comparison tolerance.

The new Rust regression covers complete snapshot parsing, plotted values,
label/line coordinates, repeated forming updates, confirmation and serialized
incremental replica application. It compares public wire outputs, because the
wire contract intentionally represents colors as numbers rather than retaining
the internal Color enum variant. `json-float-fixed-v2.log` passes. A new actual
WASM Node smoke assertion covers the same representable-float loss.

Candidate bindings in `json-float-wasm/` pass that smoke and the original full
Pivot live-feed scenario: WASM session, WASM replica and the retained Python
live result agree exactly (`pivot-json-float-live-result.json`). This is a repair
check across builds, not a new same-commit distribution receipt. Retained
`9f9a24aad` artifacts remain unchanged and are not relabeled as repaired.

Full regression and rebuilt commit-bound artifacts remain required. JSON float
parsing cost must be included in resource qualification; upstream documents a
parsing-performance tradeoff, which is not a measurement of this runtime's
end-to-end workload. Native chronological realtime-feed and visual-drawing
qualification remain separate from the frozen-snapshot simulation.

The full repaired Windows gate now passes in `json-float-full-verify-v2.log`:
6791 Rust tests, 771 installed-wheel Python tests, 130 tool tests and actual WASM.
One older WASM assertion suite relied on lossy JSON parsing to round CCI/other
indicator values. Before changing its expectations, the retained old and repaired
WASM modules produced byte-identical raw output for the complete request fixture
(`request-host-wire-before.json` / `request-host-wire-after.json`). Twelve array
expectations were corrected from those unchanged wire values; CCI's expected
value is expressed by its underlying 1/(0.015*(2/3)) formula. The diff receipt is
`request-host-json-expectation-updates.json`. Calculation algorithms were unchanged.

Additional native Pivot drawing observations discovered a separate outstanding
defect: timeframe.in_seconds("12M") returns 31104000 locally versus 31536036 on
TradingView. The frozen six-column drawing probe has 213 mismatches among 654
confirmed values (seconds and line-end coordinates). v5/v6 DOM controls agree
on 2628003 seconds per month, scaled linearly for 2M/3M/6M/12M; day/week/minute/
second controls match their usual durations. See `corpus/pivot-drawing-*` and
`corpus/timeframe-seconds-*`. This calendar-duration defect is not repaired by
the JSON change and prevents claiming full Pivot drawing qualification.

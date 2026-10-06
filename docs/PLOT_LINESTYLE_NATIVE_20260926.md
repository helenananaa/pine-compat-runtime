# Plot line styles: native admission and output contract

2026-09-26 local-source slice, continuing the
[TASC admission audit](OPTIONAL_ALERT_NATIVE_20260926.md).

Native Chrome probes confirm that v6 `plot(linestyle=...)` accepts
`plot.linestyle_solid`, `plot.linestyle_dotted`, and `plot.linestyle_dashed`,
including an input-bool conditional. Replacing the input with `close > open`
is rejected by TradingView: series plot_line_style cannot satisfy input
plot_line_style. The runtime now accepts at most input-qualified values and
validates known line-style constants. The public fixture is
`tests/fixtures/runtime/plot_linestyle.pine`.

The runtime does not draw pixels. It preserves a `linestyle` property on plot
snapshots and streamed series headers for host renderers. Solid is the default
and is omitted on the wire; missing fields decode as solid. The optional field
is additive within runtime schema 9/render metadata 1, so old default-style
results keep their existing JSON. Consumers wanting dashed/dotted plots must
honor this field. Rust snapshot parsing, streaming reconstruction, JSON and
Python dictionary bridges all carry it; WASM shares the core JSON path.

Native evidence in ignored `.local/continued-popular-20260926`:

- `plot-linestyle-input-probe.pine` and `plot-linestyle-input-native-dom.txt`:
  input-qualified success.
- `plot-linestyle-series-probe.pine` and
  `plot-linestyle-series-native-error.txt`: native qualifier rejection.
- `plot-linestyle-native.png`: native dotted/dashed chart appearance.
- `plot-linestyle-local.json`: 4,283 daily bars with two preserved line styles.
- `check-linestyle.log`, `test-linestyle-final.log`: validation receipts.

Focused tests verify initial streamed headers, forming/confirmed updates,
replica reconstruction, snapshot JSON roundtrips, invalid constants and series
rejection. `cargo check --workspace --locked` passed, including Python and WASM
bridge compilation. Installed-wheel/browser-WASM execution and host pixel
rendering were not tested in this slice. The full builtins/sema/runtime/CLI
regression suite passed 6,093 tests. Formatting and diff checks pass.

The original TASC indicator now has only its display-mask subtraction admission
error. Benchmark data and full native numeric qualification remain pending.
No complete-script parity is claimed for TASC at this point. The temporary
native probe is removed and the previous editor contents restored.

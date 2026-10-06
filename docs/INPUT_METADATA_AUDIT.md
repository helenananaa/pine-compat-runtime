# Versioned input/render metadata — 2026-09-12

This slice follows `f9cd5377c` in the local product-completion branch. The
unmodified official v6 RSI had 22 admission diagnostics. Declaration and
metadata repairs reduce these to six gradient-fill diagnostics; it is still
not an accepted complete script.

## Native evidence and implementation

Chrome/TradingView accepted the v6 `input_active_metadata.pine` control and
displayed length 7, scale 2.5 and string comparison 1 with `active=false`.
The same source in v5 rejected `active` as an unknown input argument. A separate
v5 control with active removed still rejected input-qualified `editable`;
replacing editable with a const bool succeeded while input-qualified display
remained accepted. Raw captures are in `.local/product-completion-20260912/`:
`input-metadata-v6-dom.txt`, `input-metadata-v5-dom.txt`,
`input-metadata-v5-no-active-compiled-dom.txt`, and `input-display-v5-dom.txt`.

- Typed input families accept v6 `active` with a const/input bool bound. It is
  UI metadata and does not disable evaluation or replace supplied overrides.
  The current input-discovery output does not expose an activation-dependency
  graph; a host UI cannot infer one from this admission claim.
- Modern `indicator(timeframe="", timeframe_gaps=...)` inherits the supplied
  chart context. Non-empty or unresolved program timeframes remain explicitly
  rejected instead of silently executing at the chart timeframe.
- `plot`, `plotshape` and `fill` display parameters accept const/input strings
  in v5/v6. Their editable parameters accept const/input bool in v6 and preserve
  the v5 const requirement. Series qualifiers remain rejected.
- Output metadata is already evaluated and serialized by the existing runtime.
  This slice does not add output fields or change the public schema versions.

## Verification

`verify-input-metadata.log` records 6,724 Rust tests, 130 tool tests, strict
format/clippy/structure and host-parity checks, actual generated WASM, and 758
tests against a freshly installed Windows wheel. Added native-value and
override tests confirm that inactive inputs retain their computation values.
Initial overly broad v5 admission was corrected from the independently observed
native rejection before this gate. This remains local worktree qualification,
not a retained optimized Windows/Linux distribution.

The official source and its 316-row one-minute CSV are frozen separately. That
CSV starts after the indicator's initialization history and is not a complete
from-origin oracle. A separate monthly export has now been frozen with indexes
0 through 109. The forming September 2026 bar is excluded, leaving 109 confirmed
bars and 218 RSI/RSI-based-MA values with no skipped warmup. The original source,
raw CSV, derived inputs and expected values have SHA-256 identities in
`.local/product-completion-20260912/corpus/manifest.json`. Absolute and relative
tolerances are frozen at 1e-9 before implementation comparison. The full source
remains blocked by gradient-fill admission; a ready reference is not a pass.

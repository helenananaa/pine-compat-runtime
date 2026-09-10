# TradingView reference closure

Started 2026-09-10 from clean `c292e5ef591a43c42260423f4a6b1f92d768d680`.
The user authorized goal execution and Chrome-based TradingView reference
collection. Earlier task-specific restrictions on new captures no longer apply.

## Work order

1. Reproduce and explain the native realtime exit-price difference. Preserve
   the original 16/896 failed comparison and collect separate controls with
   source, timestamps, chart settings and comparison tolerance frozen first.
2. Establish observable B1 acceptance cases. Do not infer private internal
   sequencing from final trades or aggregate position snapshots.
3. Expand independent modern-strategy reference coverage. Keep original r1
   inputs/denominator distinct from new native-data scenarios; no synthetic
   runtime output may become the external oracle.
4. Fix demonstrated runtime defects and verify affected execution modes and
   installed artifacts. Record unresolved external-input boundaries explicitly.

## Current work

- Chrome is connected to the authenticated TradingView chart.
- A second four-update capture is frozen under
  `.local/tv-goal-20260910/live-r2/`. Only the capture start constant differs
  from the original script. The original editor content is backed up.
- Warmup starts at 2026-09-10 07:59 UTC; two capture bars start at 08:03 UTC.
- Original reference values and 1e-9 absolute/relative tolerances are unchanged.
- Live-r2 completed: 896/896 values pass on the unchanged candidate wheel.
  The original CSV was independently reconstructed and remains 16/896 failed.
- Six native capital/ordering controls exposed a same-side long stop admission
  defect. The scoped repair and regression evidence are tracked in
  `STRATEGY_LONG_STOP_MARGIN_ADMISSION_AUDIT.md`; Windows development-artifact
  qualification passes 6670 Rust / 715 Python / 130 tool tests plus actual WASM,
  and all 9460 native values agree after the repair.
- An unmodified r1 commission script now has a separate native-data companion
  capture from bar zero (24988 closed bars). Before repair, 49974/99952 plot
  values differ, in initial-capital and absent-open-trade commission behavior.
  These defects are now corrected: 99952/99952 values pass, and the separate
  v5/v6 controls bring the new reference total to 624806 passing values.
  Full Windows checks pass 6672 Rust / 715 Python / 130 tools and actual WASM.
  See `STRATEGY_NATIVE_DEFAULTS_AUDIT.md`. The original r1 source and synthetic
  scenario are unchanged; this is one new native companion scenario.
- No publication is performed. B1 private ordering and original r1 coverage
  are not upgraded by these separate captures.
- Source-version margin defaults are corrected and Windows-qualified in
  `STRATEGY_VERSIONED_MARGIN_AUDIT.md`: 6674 Rust / 715 Python / 130 tool tests,
  actual WASM, and three native controls totaling 225171 matching values.
- Linux native verification of commit 906e04592 passes 6672 Rust / 715
  installed-wheel Python / actual WASM. Of 130 tool tests, the Windows-only
  memory probe is skipped on Linux. Evidence is retained in
  `.local/tv-goal-20260910/linux-906e04592-gate-v3.log` and the pinned source path.
- Five repeated live round trips with concurrent browser market-event capture
  are retained under `live-r3-quotes`. Native closed counts progress 1 through 5.
  The replay compares 1856 values and retains 14 mismatches in one repeated
  entry-price observation (78175.3 versus 78175.2). Finer input evidence is
  being investigated; no price heuristic has been applied.

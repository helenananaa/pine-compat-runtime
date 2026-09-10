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

- Chrome was used for native captures; the original chart script is restored.
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
- Current access boundary: the user has no additional trade-level source,
  proxy, or Ultimate access. Native one-second data also aggregates the
  relevant changes; public OKX API attempts did not yield data. See
  `REALTIME_PRICE_INPUT_BOUNDARY_AUDIT.md`. The goal is not complete.
- Linux native verification of `a6a28528e` passes 6674 Rust / 715 installed
  Python / actual WASM; the tool suite has the same one Windows-only skip.
  Release-profile Windows artifact validation is in progress separately.
- Windows and Ubuntu-native optimized wheels at a6a28528e each pass 715
  installed tests and the three repaired reference groups. The Linux wheel
  is tagged manylinux_2_35; it is not the manylinux2014 distribution gate.
- A 64-execution native trace exposed an additional late-attachment context
  gap. Rust/Python now accept optional opening-update metadata; the full
  trace passes 975/975 values when its known mid-bar attachment is supplied.
  Windows checks pass 6676 Rust / 717 Python / 130 tools and actual WASM.
  This does not resolve either price-precision capture.

## Final qualification for this batch

Source `6c31e2b22c1c7f27a42db4d6df3da76ee425d791` passes full Windows and
Ubuntu-native checks: 6676 Rust, 717 installed-wheel Python and actual WASM.
Windows passes 130 tool tests; Linux runs 130 with one Windows-only skip.
Both optimized release wheels separately pass 717 installed tests and the
975-value late-attachment trace. Their release CLIs produce identical complete
output for the native r1 commission companion. The Linux wheel targets
manylinux_2_35, not a newly qualified manylinux2014 distribution.

Four local implementation commits repair occupied-long stop margin admission,
default initial capital, absent open-trade commission, versioned margin defaults,
and mid-bar opening context. The earlier reference groups retain their original
source commits and denominators; they are not relabeled as fresh full reference
qualification of the final source. Local artifacts and receipts are indexed in
`.local/tv-goal-20260910/final-evidence-manifest.json`.

The original 16/896 and newer 14/1856 realtime price comparisons still fail on
the final Windows release wheel. The 64-execution trace adds passing coverage
but contains no equivalent same-close/new-extreme discriminator. The user has
confirmed no additional feed, proxy or Ultimate access. Further price correction
requires evidence that distinguishes the competing explanations. The goal remains
incomplete; private B1 ordering, original r1 coverage and general product release
readiness are not implied by this batch. Nothing was published or pushed.

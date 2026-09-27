# Legacy Squeeze Momentum: native qualification

2026-09-26. Unchanged public LazyBear source, 40 lines, no version directive:
https://www.tradingview.com/script/nqQ1DT5a-Squeeze-Momentum-Indicator-LazyBear/
Chrome showed 116,522 boosts. The published BB multiplier typo is preserved;
this qualification concerns the exact displayed source, not the linked revision.
The local analyzer classifies it as implicit v1; TradingView editor displays v3.
Native explicit v5 probes provide a separate modern legacy-language check.

## Results

Local admission succeeds with zero diagnostics. Four appended state plots are
neutral to the original complete output. Across 23,970 confirmed BTCUSD hourly
bars, all 95,880 state cells agree after the logical-NA correction: momentum,
squeeze-on, squeeze-off and no-squeeze. Input timestamp/OHLCV matches exactly.
The final forming bar is excluded. Float comparison tolerance is relative and
absolute 1e-9. Rendered colors/styles are not part of this numerical receipt.

Before the fix, 20 initial no-squeeze values differed. Native minimal probes
show that numeric comparisons with missing history remain NA, but AND/OR with
NA operands treat them as false and NOT NA returns true. Explicit v5 reproduces
the same results; all 311,610 exported v5 probe values agree after correction.

## Implementation and regression boundaries

Runtime logical operators now treat missing boolean operands as false. Numeric
comparison version rules stay intact, as does v6 short-circuit evaluation versus
legacy eager evaluation. A local source-level regression iterates v1 through v6,
covering missing comparisons, AND/OR combinations, NOT and subsequent valid bars.
Native exports directly establish implicit-source and explicit-v5 behavior;
this does not claim six independent native compiler-version qualifications.

Original and instrumented full outputs agree after removing the added plots.
Mode and older-script regression receipts are `squeeze-modes.log`; complete
workspace test evidence is `test-logical-na-full.log`.

## Reproduction evidence

Local receipts are in `.local/continued-popular-20260926/`, including the
squeeze-evidence-manifest.json, original/probe sources, downloaded native CSVs,
squeeze-fixed-comparison.json and v5-na-logical-comparison.json. Original native
exports are retained in the user's download directory. Public sources and large
native data are not vendored. No host-specific dependency was added to the core.

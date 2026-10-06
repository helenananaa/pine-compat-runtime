# SSL Hybrid native cases rerun after the extrema repair

Verified 2026-09-29 on the current local core patch. Nine previously captured
original Pine v5 SSL Hybrid Strategy cases were executed again in **all
three historical modes**. The native chart/trade comparators were also
rerun against their frozen Chrome exports. All complete runtime outputs
are byte-identical to the preceding broker-patch outputs, with no diagnostics
or native mismatches. Original evidence directories and receipts remain intact.

This is current-source requalification of existing native captures.
It does not represent nine new browser captures or nine new settings.

## Current-source results

All cases use the unchanged public source SHA-256
`1b90bead338682dcc17b8c1063637e7b8354c91b35d9d2a741ae41b5274c658c`,
HL2, and the input/property/metadata contracts frozen in their original reports.
USDJPY account currency is JPY with grid `1/1000`; AUDUSD is USD with
grid `1/100000`. Both use integer quantities and point value 1.

| Symbol | Period | Baseline | Confirmed bars | Chart observations | Closed trades | Explicit exits |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| USDJPY | Four hours | HMA 20 | 21,345 | 149,285 | 1,088 | 592 |
| USDJPY | Four hours | HMA 30 | 21,345 | 149,241 | 892 | 581 |
| USDJPY | Four hours | EMA 30 | 21,345 | 149,255 | 470 | 319 |
| USDJPY | Four hours | DEMA 30 | 21,345 | 149,141 | 658 | 480 |
| USDJPY | Four hours | TEMA 30 | 21,345 | 149,025 | 768 | 507 |
| USDJPY | Daily | TEMA 30 | 14,316 | 99,822 | 134 | 77 |
| USDJPY | Weekly | TEMA 30 | 2,906 | 19,952 | 28 | 24 |
| AUDUSD | Four hours | TEMA 30 | 21,344 | 149,018 | 3,452 | 2,031 |
| AUDUSD | Four hours | LSMA 30 | 21,344 | 149,248 | 3,185 | 2,075 |

Observations count nonblank values in eight named exported columns; missing
positions are checked too. Native comparisons check entry/exit IDs,
direction, prices, quantities, durations, display monetary fields, open
entry records and explicit exits. Intraday times use UTC+8 minute timestamps;
daily/weekly exports use their native session dates. Forming bars and open
unrealized PnL are excluded according to each original capture contract.
Overlapping histories must not be summed as independent evidence.

## Source and evidence chain

Base HEAD: `2f9a83a0c333092167c7253571f56ee82a7ec0d0`.
Current core patch SHA-256:
`827e195c3a7db4422bbd657a5634990adb9d8d1db69658f6331df41a97cb2636`.
Immutable CLI SHA-256:
`584ae27bb25fed15c038c5c55a9356720c1bff90b8db29ce8b403e8a08010f36`.
This includes the earlier broker fixes and the later highest/lowest NA repair.

Evidence is under `.local/extrema-requalification-20260929/`. Each case
has copied immutable native chart/trade files, original source and bars,
settings/screenshot artifacts covered by the original capture hash manifest,
the unchanged comparator, **fresh** batch/incremental/historical-realtime
outputs and terminal receipts, and a fresh native comparison. Capture
provenance identifies the original directory, original source version,
original verification/comparison hashes, and hashes of every copied file.
New `current-verification.json` files bind those captures to the new CLI
and current source. The aggregate receipt is `matrix-verification.json`.

`.local/requalify_extrema_matrix_20260929.py` performed 27 actual mode runs
and nine native comparator runs. It checked frozen files before and after,
the current core diff and source files, mode exits/stderr, whole output
equality with the original receipts, and comparison equality.
`.local/audit_extrema_requalification_20260929.py` provides a read-only
current-state audit of the matrix, exact commands/metadata, artifact hashes,
original capture chain, comparisons, source and CLI. It rejects later core
changes rather than carrying this result to unverified versions.

The current source is identical to the freshly gated VAMA window 10 source,
so its passing full release gate is reused without rerunning it: 1,984
runtime tests, 242 CLI tests, 130 tool tests, WASM Node smoke, host parity
and 774 wheel tests. Gate log SHA-256:
`0d85d41cdf71fa4a60f166d8737dd6207af6b7f755a15fa96f4de8a2f9f2aec9`.

The [USDJPY matrix](SSL_HYBRID_USDJPY_COVERAGE_20260929.md) keeps its original
broker-patch audit as historical evidence and links this current rerun.
The new [AUDUSD four-hour VAMA window 60](SSL_HYBRID_FX_AUDUSD_FOURHOUR_VAMA30_VOL60_HL2_20260929.md)
and [daily VAMA window 60](SSL_HYBRID_FX_AUDUSD_DAILY_VAMA30_VOL60_HL2_20260929.md)
are separate fresh Chrome captures. Together these records qualify named
settings on their exact histories; arbitrary Pine compatibility and native
forming-tick behavior are not established.

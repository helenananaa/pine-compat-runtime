# Historical candidate acceptance checklist (`0.3.0-rc.1`)

This record binds the older b9cae7ea5 / 2792a0950 artifacts only. For the repaired
implementation and current artifact inventory, use [DELIVERY_ROADMAP.md](DELIVERY_ROADMAP.md).
The failures and TV-blocked classification below are historical; original evidence
and missing-log disclosures are preserved. Resource limits still constrain claims.

TV-blocked local prerelease; not a stable release or full Pine compatibility.
No tag, push or publication is authorized by this checklist.

## Identity and evidence

- Original Windows artifacts: `b9cae7ea5343f284e6f453b6c88c678d59617056`.
- Candidate test baseline and new manylinux wheel source:
  `2792a095075d58e4a7145d3b97ce7d1286eb2800`; its change from the Windows
  artifact source is a `cfg(test)` CLI snapshot registration.
- Review baseline: `c03cf77bb355113909e3798b67d7d859a8b1b080`. Intervening
  commits add documentation and a platform test, not runtime semantics.
- Reconciliation changes are working-tree documentation, evidence and delivery
  test changes. They do not relabel binaries as built from a later HEAD.
- Artifact root: `.local/candidate-0.3.0-rc.1/`. `inventory.json` and
  `SHA256SUMS` bind artifacts; `evidence/manifest.json` binds evidence by
  relative path, size and SHA-256. These local files are intentionally ignored.

The original goal scratch directory was removed. Original combined Windows/Linux
gate logs and candidate frozen-reference comparison logs were not recovered.
Their old summary counts (6668 Rust / 715 Python / 122 tools plus WASM) remain
reported historical results, not newly verified original receipts. Fresh checks
and earlier retained reference receipts are identified separately below.

## Acceptance table

Evidence paths below are relative to the artifact root.

| Item | Evidence | Qualification |
| --- | --- | --- |
| Four non-trend workloads | `evidence/d4/original/` original plans, pilot/formal reports and receipts; `evidence/d4/*-recheck.json` | Four budgets independently recomputed and passed; 10k history + 1k tail, two repeats |
| Trend 100k/10k | `evidence/trend/` frozen plan, full report, original receipt and recheck | Reused Windows qualification at 3ca746976; no scale extrapolation |
| Output costs | `evidence/d4/original/summary.json` and raw reports | Snapshot, serialization, drop and process memory measured; see budget interpretation |
| Windows installed candidate wheel | `evidence/reconciliation/windows-installed-wheel-tests.log` | Fresh 715-test check against retained candidate environment |
| Retained WASM and examples | `evidence/reconciliation/wasm-smoke.log`, `launches.json` | Fresh execution evidence, distinct from lost original gate logs |
| manylinux2014 candidate | `evidence/reconciliation/manylinux-qualification.json`, `manylinux-build-test-v3.log`, `auditwheel.txt` | Fresh offline release build, actual installed module identity and 715 tests passed; manylinux_2_17 |
| Current delivery tools | `evidence/reconciliation/tool-tests.log` | Fresh tool regression check |
| TechnicalRating full graph | `evidence/technical-recheck/technical-qualification.json` and complete outputs | Fresh 63,399 reference values passed at original 1e-9 tolerances; CLI four modes, installed Python and retained WASM have identical complete output |
| Earlier independent references | `evidence/prior-references/` | Historical receipts only; source identities and limitations retained |
| Known TV gaps | `LIVE_TICK_REFERENCE_AUDIT.md` | 16/896 exit-price differences remain failed; B1 unverified; r1 0/482 is missing coverage, not 482 failed comparisons |

## Resource budget interpretation

The four non-trend plans were frozen from pilot measurements using 2.5x timing
and 2x process-memory headroom, then evaluated on separate formal runs. These
are reproducible regression baselines, not a host-independent product SLA.
No evidence establishes that those multipliers derive from an embedding client's
latency or memory requirements. Original plans and thresholds remain unchanged.

The candidate qualifies the recorded single-workload sizes on the measured
Windows host. It does not qualify concurrency, every workload at 100k/10k,
Linux resource ceilings, or indefinite bounded output retention. Future host
acceptance must specify update frequency, concurrent sessions, output retention,
end-to-end latency and process-memory budgets before its acceptance run.
That dependency is host requirements / resource qualification, not TradingView.

## Remaining boundaries

- Exact realtime fill pricing and broader broker/account semantics require
  independent evidence. No tolerance or reference was changed.
- Dynamic request discovery cannot always return literal context keys. Widening
  runtime admission/resolution is an engineering scope decision; provider data
  completeness belongs to the host. Neither automatically needs new TV output.
- WASM exposes historical execution only. Python uses confirmed session updates
  for streaming; no separate incremental API is exported.
- Stable publication remains blocked. Use the candidate for evaluation within
  its declared scope. See `DELIVERY_SURFACES.md` and `CANDIDATE_CLOSEOUT.md`.

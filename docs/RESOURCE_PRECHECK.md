# Resource precheck

Use `scripts/resource_precheck.py` before starting the full resource matrix.
It runs fresh workers from existing release artifacts and selects complete cases
from an existing frozen `plan.json`. It does not build artifacts or reuse old
worker results.

```powershell
python scripts/resource_precheck.py `
  --input-root .local/resource-follow-up-20261002/matrix-v14 `
  --artifacts .local/current-resource-artifacts `
  --output .local/resource-precheck-new
```

The defaults run Rust once for each of `rsi-default-1024-1`,
`rsi-alternate-1024-1`, and `pivot-original-1024-1`. Each case retains its complete
1,024-bar seed, 128-bar tail, provider events, outputs, replicas and native
historical controls. The output directory must be new.

Select additional surfaces or a larger case explicitly:

```powershell
python scripts/resource_precheck.py `
  --input-root .local/resource-follow-up-20261002/matrix-v14 `
  --artifacts .local/current-resource-artifacts `
  --output .local/resource-precheck-pivot-new `
  --surface rust --surface python --surface wasm `
  --case pivot-original-100000-4 --repetitions 1 --timeout 600
```

Artifacts must contain release `build-provenance.json` with current core-file
hashes. A Rust run additionally requires a resource probe built from the current
collector source. Python uses the artifact environment's installed module; WASM
uses its built module through the current Node collector. The precheck records
current artifact and collector hashes separately from the original input-plan
hashes and checks for changes after execution. Workers must emit version 2
reports referencing complete sibling spool files and version 1 metadata.

`precheck-results.json` records execution failures, exact metric sample counts,
worker wall time, output hashes and collector overhead metrics. A timeout kills
the worker process tree and preserves its log. Failed cases remain in the
receipt. A completed precheck means the selected workers finished and their
reported invariants and complete-output references passed validation. It does
not establish cross-platform parity, a growth ratio, or resource acceptance.

Every precheck explicitly sets `diagnosticPrecheck: true`, `fullMatrix: false`
and `qualification: "notEvaluated"`. The original frozen plan and budgets are
unchanged. Run the full acceptance matrix and its independent audit when a
candidate is ready for qualification.

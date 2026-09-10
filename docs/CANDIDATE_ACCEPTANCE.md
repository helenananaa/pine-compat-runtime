# Candidate acceptance checklist (`0.3.0-rc.1`)

TV-blocked local prerelease. Not a stable release. Not full Pine compatibility.
No GitHub tag, push, or publish.

Fill every row with the source commit, artifact path, and evidence path after
the corresponding verification. Empty evidence means not yet judged on this
identity.

| Item | Commit | Artifact | Evidence | Result |
| --- | --- | --- | --- | --- |
| Baseline inventory | da55025dc then this candidate | n/a | `{scratch}/baseline-inventory.json` | recorded |
| Trend 100k/10k Windows (reuse) | 3ca746976 | `trend-100k-probe-v5.exe` | `.local/delivery-20260909/resources/trend-100k-acceptance-v5.json` | passed (reused) |
| Every-update 10k+1k | *this candidate* | release `long_session_benchmark` | `{scratch}/d4/every-update-acceptance.json` | pending |
| Dense orders 10k+1k | *this candidate* | same | `{scratch}/d4/dense-orders-acceptance.json` | pending |
| Collection 10k+1k | *this candidate* | same | `{scratch}/d4/collection-acceptance.json` | pending |
| Complete-output costs | *this candidate* | same reports | `{scratch}/d4/*-formal.json` snapshot/serialization/drop | pending |
| Magnifier 10k+1k historical-only | *this candidate* | same | `{scratch}/d4/magnifier-acceptance.json` | pending |
| Resource over-limit / atomicity / owned results | *this candidate* | rustc test + python session | `crates/pine-runtime/tests/resource_limit_atomicity.rs`, `python/tests/test_candidate_embedding.py` | unit-tested |
| Rust embedding example | *this candidate* | `embed_runtime` | `{scratch}/launch-rust.log` | pending |
| CLI installed binary | *this candidate* | `pine-compat` | `{scratch}/launch-cli.log` | pending |
| Python installed wheel | *this candidate* | `pine_compat_runtime-0.3.0rc1-*.whl` | `{scratch}/launch-python.log` | pending |
| WASM Node bindings | *this candidate* | `pine_wasm.js` + `pine_wasm_bg.wasm` | `{scratch}/launch-wasm.log` | pending |
| Windows verify.ps1 | *this candidate* | wheel + tests | `{scratch}/verify-windows.log` | pending |
| Linux installed artifacts | *this candidate* | Linux wheel/CLI | `{scratch}/verify-linux.log` or `linux-launcher-failure.log` | pending |
| Frozen TechnicalRating / G3 / HTF / Magnifier / margin | *this candidate* | candidate CLI/wheel/WASM | `{scratch}/frozen-refs/` | pending |
| Known failures still visible | *this candidate* | n/a | live-tick 16/896, B1, r1 0/482 | must remain visible |

## Product claims still blocked on new TradingView evidence

- Live-tick exit-price mismatches (16/896) — blocks “native realtime broker fill parity”.
- B1 `UNVERIFIED_INTERNAL_ORDER` — blocks independent broker-evidence completion.
- Public r1 independent-reference 0/482 — blocks raising the public independent-reference denominator.
- Broader account/tick profiles and unresolved dynamic request arguments — blocks those host-profile claims.

## Classification

completed-and-verified / implemented-but-not-finally-qualified / explicit-failure /
waiting-on-TV-or-external — see the closeout report written at the end of this
candidate run.

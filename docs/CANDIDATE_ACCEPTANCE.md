# Candidate acceptance checklist (`0.3.0-rc.1`)

TV-blocked local prerelease. Not a stable release. Not full Pine compatibility.
No GitHub tag, push, or publish. HEAD `2792a0950`; Windows release binaries
were produced at `b9cae7ea5` (the follow-up commit is `cfg(test)` only).

| Item | Commit | Artifact | Evidence | Result |
| --- | --- | --- | --- | --- |
| Baseline inventory | da55025dc then this candidate | n/a | scratch `baseline-inventory.json` | recorded |
| Trend 100k/10k Windows (reuse) | 3ca746976 | `trend-100k-probe-v5.exe` | `.local/delivery-20260909/resources/trend-100k-acceptance-v5.json` | passed (reused) |
| Every-update 10k+1k | b9cae7ea5 | release `long_session_benchmark` sha256 `f8333ca5…` | scratch `d4/every-update-acceptance.json` | passed |
| Dense orders 10k+1k | b9cae7ea5 | same | scratch `d4/dense-orders-acceptance.json` | passed |
| Collection 10k+1k | b9cae7ea5 | same | scratch `d4/collection-acceptance.json` | passed |
| Complete-output costs | b9cae7ea5 | same reports | scratch `d4/summary.json` snapshot/serialization/drop | recorded |
| Magnifier 10k+1k historical-only | b9cae7ea5 | same | scratch `d4/magnifier-acceptance.json` | passed |
| Resource over-limit / atomicity / owned results | 2792a0950 | rustc + python session | `crates/pine-runtime/tests/resource_limit_atomicity.rs`, `python/tests/test_candidate_embedding.py`, scratch `resource-unit-tests.log` | passed |
| Rust embedding example | b9cae7ea5 | `embed_runtime.exe` | scratch `launch-rust.log` | passed (two launches) |
| CLI installed binary | b9cae7ea5 | `bin/pine-compat.exe` | scratch `launch-cli.log` | passed (two launches) |
| Python installed wheel | b9cae7ea5 | `pine_compat_runtime-0.3.0rc1-cp310-abi3-win_amd64.whl` | scratch `launch-python.log`; verify.ps1 715 tests | passed |
| WASM Node bindings | b9cae7ea5 | `wasm/pine_wasm.js` + `pine_wasm_bg.wasm` | scratch `launch-wasm.log` | passed (two launches) |
| Windows verify.ps1 | 2792a0950 | wheel + tests | scratch `verify-windows.log` | passed: 6668 rust / 715 installed Python / 122 tools + WASM |
| Linux native verify.sh | 2792a0950 | Ubuntu 22.04 WSL wheel `manylinux_2_35` + CLI | scratch `verify-linux.log`; `.local/candidate-0.3.0-rc.1/linux/` | passed: 6668 rust / 715 installed Python / 122 tools + WASM |
| Linux manylinux2014 image | n/a | n/a | scratch `linux-launcher-failure.log` | unverifiable here (Docker engine down) |
| Frozen TechnicalRating | b9cae7ea5 CLI | `pine-compat.exe` | scratch `frozen-refs/technical.json` 63399 values, original hashes/1e-9 | passed |
| Frozen G3 r3/r2 | b9cae7ea5 CLI | same | scratch `frozen-refs/summary.json` 65+41 trades, prior runtime equal | passed |
| Frozen HTF / Magnifier / margin | b9cae7ea5 CLI | same | scratch `frozen-refs/summary.json` 422660 HTF; 2960 Magnifier plots; 7992 margin values | passed |
| Known failures still visible | 2792a0950 | n/a | live-tick 16/896; B1 `UNVERIFIED_INTERNAL_ORDER`; r1 0/482 | visible failures |

## Product claims still blocked on new TradingView evidence

- Live-tick exit-price mismatches (16/896) — blocks “native realtime broker fill parity”.
- B1 `UNVERIFIED_INTERNAL_ORDER` — blocks independent broker-evidence completion.
- Public r1 independent-reference 0/482 — blocks raising the public independent-reference denominator.
- Broader account/tick profiles and unresolved dynamic request arguments — blocks those host-profile claims.

## Classification

See `docs/CANDIDATE_CLOSEOUT.md`.

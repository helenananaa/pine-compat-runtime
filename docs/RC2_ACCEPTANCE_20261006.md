# 0.3.0-rc.2 acceptance

The current RC qualifies the named compatibility and install scope. It does
not establish stable resource acceptance or arbitrary Pine/native live-Tick
compatibility. See [migration and limits](RC2_MIGRATION.md).

## Source and reference identity

- Qualified implementation: `83193323cdb1a763ee112183a7448bf839c06231`.
- CI-only toolchain commit: `9f04a19d7ae6fd080ec23538443ed73cd5d05495`; core, manifests, lockfile, binding inputs
  and acceptance runners are byte-identical to the implementation commit.
- The release tag includes the later documentation record and release-note
  selection in the publisher (wheel build steps are unchanged). Its exact commit
  is recorded by the published release manifest. Core/build inputs are checked
  again before tagging; no binary is relabeled from an earlier implementation.
- Fifteen script/settings/control cases reuse the unchanged 75 hash-pinned
  reference files from the retained 2026-10-01 plan. Native data is not recaptured,
  numerical tolerances are not widened, and earlier reports remain on their pins.
- [Machine-readable index](RC2_ACCEPTANCE_RESULTS_20261006.json) binds plan,
  matrix, binary provenance, output-index and installation receipt hashes.
  Raw local receipts are retained under `.local/release-0.3.0-rc.2-20261006`.

## Fresh validation

Windows x86-64 and Ubuntu 22.04 x86-64 under WSL each pass the complete
canonical gate: 7,569 Rust tests, 798 installed-wheel Python tests, 166 tooling
tests run (platform-dependent skips retained), strict Clippy/format checks,
structural/host-parity guards and actual Node/WASM smoke checks. Local toolchains
are Rust 1.97.1 on Windows and 1.95.0 on Linux. Remote canonical CI pins the
declared Rust 1.95.0 minimum instead of floating Clippy rules.

Fresh `--release` CLI, direct Rust probe, installed Python wheel and generated
Node/WASM run all fifteen cases on both platforms. CLI batch, incremental and
realtime-history are compared separately: all 180 complete outputs agree, with
absolute tolerance 1e-9 and relative tolerance 1e-12, preserving keys, lengths,
types and null positions. The original native comparators for Hull, UT Bot,
SSL, commissions, RSI and Pivot, including their realtime controls, pass.
Hull's retained displayed-PnL residual remains explicitly open.

The [pre-tag wheel workflow](https://github.com/helenananaa/pine-compat-runtime/actions/runs/37460105095) builds optimized
Windows `cp310-abi3-win_amd64` and Linux
`cp310-abi3-manylinux_2_17_x86_64.manylinux2014_x86_64` wheels and runs 798
installed-wheel tests per platform. Each wheel is separately installed offline
into a clean environment and compared against all fifteen qualified complete
outputs, an independent SMA oracle and the Python embedding example.
The Linux check runs under actual glibc 2.17 / CPython 3.10 with networking
disabled. The native Linux acceptance wheel is glibc 2.35; it is kept separate
and is not substituted for the official manylinux2014 asset.

The tag workflow rebuilds both wheels at the final release commit, tests them,
and publishes only those wheels plus manifest/checksums as an opt-in prerelease.
Actual published bytes are downloaded, hashed and freshly installed after
publication; the pre-tag wheel hashes above identify pre-tag evidence only.
Rust, CLI and WASM are usable from source but are not separate release assets.

## Stable-release work still open

The earlier 216-trial resource matrix remains `notPassed` on its original
source/artifact pin. Linux Python Pivot four-session growth was
4.288 / 4.578 / 4.456 against the unchanged limit of four. This RC does not
reuse those measurements as current qualification. Its deterministic per-bar
allowances are not RSS or long-session guarantees. Rerun the fixed matrix on
the selected final core and resolve any reproducible budget failures before
declaring stable resource acceptance.

Native live-Tick, exchange-session calendars, visual geometry and unlisted
Pine/strategy behavior remain outside the named scope. macOS, ARM, musllinux
and free-threaded CPython are not part of the wheel matrix.

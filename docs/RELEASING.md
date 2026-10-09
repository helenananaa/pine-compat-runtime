# Releasing Binary Wheels

GitHub Actions builds the Python extension wheels for the supported
desktop platforms in the current source workflow:

- glibc Linux x86-64: `cp310-abi3-manylinux_2_17_x86_64`
- Windows x86-64: `cp310-abi3-win_amd64`
- macOS Apple Silicon: `cp310-abi3-macosx_*_arm64` on `macos-15`
- macOS Intel: `cp310-abi3-macosx_*_x86_64` on `macos-15-intel`

The `abi3-py310` PyO3 feature makes each platform wheel compatible with
ordinary GIL-enabled CPython 3.10 and newer. macOS wheels are built and tested
natively on each architecture using Rust 1.95.0. The macOS deployment version
is encoded in the generated wheel filename. Linux/Windows ARM, musllinux, and
free-threaded CPython remain outside the release matrix.

The macOS jobs use standard GitHub-hosted runners, which are free for public
repositories. Only wheels are uploaded; the workflow does not cache `target/`.
Workflow artifacts expire after 14 days. Each job installs its generated wheel
and runs `python/tests` before uploading it.

The published `v0.3.1` release contains Linux and Windows wheels only. Adding
macOS to the workflow does not add assets to that existing release; macOS build
and installed-wheel test results must pass before claiming macOS validation.

The current stable release is `0.3.2`. See
[stable acceptance](STABLE_ACCEPTANCE_20261006.md),
[migration and limits](STABLE_MIGRATION.md), and the
[delivery ledger](DELIVERY_ROADMAP.md).
Earlier `.local/candidate-0.3.0-rc.1/`, streaming and delivery inventories
preserve their own source pins. Never substitute an older wheel for a newer
tag, or treat a native glibc 2.35 build as manylinux2014 qualification.

## Workflow Behavior

`.github/workflows/wheels.yml` runs for pull requests, pushes to `main`, version
tags, and manual dispatches.

- Pull requests and ordinary `main` pushes build, install, test, and retain the
  wheels as 14-day workflow artifacts.
- A `v*` tag performs the same build and then creates a durable GitHub Release.
- New releases require all four wheels, `manifest.json`, and `SHA256SUMS`.
- PEP 440 prerelease versions publish with `--prerelease --latest=false`.
  They do not change the release consumed through `/releases/latest`.

Workflow artifacts are test evidence. Applications must consume only GitHub
Release assets.

## Version Contract

Before tagging, keep these versions synchronized:

- `pyproject.toml` project version
- every workspace crate package version under `crates/*/Cargo.toml`
- workspace package versions recorded in `Cargo.lock`
- Git tag without its leading `v`

The manifest builder reads the wheel metadata and fails the release if the tag
does not match the packaged version or if the four expected wheels are missing.

## Release Checklist

1. Commit a unique, synchronized version. Freeze the selected implementation
   and native-reference plan. Preserve prior receipts on their original pins.
2. Run the canonical gates on Windows and Linux, build `--release` artifacts,
   and rerun the frozen native/cross-surface acceptance matrix:

   ```text
   scripts/verify.sh
   ```

3. For a stable release, rerun the unchanged fixed resource matrix on the
   selected implementation; all trials, output audits and budgets must pass.
   Push the candidate branch and manually dispatch `wheels.yml` at that branch.
   Confirm the run's `headSha`, all four installed-wheel test jobs, actual wheel
   versions, and Linux manylinux2014 floor. Download and freshly install the
   artifacts, checking the named reference cases before tagging.
4. Complete release notes and migration/known-limit documentation. If the final
   commit changes only documentation, verify every qualified core/build input
   remains identical and record the implementation and release commits separately.
5. Create and push the annotated tag at the selected release commit:

   ```text
   git tag -a v0.3.2 -m "Pine Compat Runtime v0.3.2"
   git push origin v0.3.2
   ```

6. Confirm the tag workflow succeeds and the GitHub Release contains exactly
   four wheels plus manifest/checksums, with matching version and release commit.
   Compare/download the actual assets and verify their SHA-256 digests.
7. Freshly install each published wheel and run the binding examples and named
   reference cases. Record the actual release bytes; workflow artifacts alone
   are not final asset verification.

A stable release additionally needs fresh qualification of the unchanged fixed
resource matrix and explicit strategy/capability scope. Deterministic per-bar
allowances do not prove RSS or indefinite-session resource bounds.

## Application Update Contract

An application updater should query the repository's latest stable GitHub
Release, download `manifest.json`, choose a wheel using Python compatibility
tags, verify its SHA-256 digest, and install it into a versioned plugin
directory. It must keep the previous version for rollback and activate a new
native extension only on process restart.

Do not install release wheels into a global Python environment, replace a
loaded `.pyd` or `.so` in place, consume pull-request artifacts, or silently
fall back to building from source on an end-user machine.

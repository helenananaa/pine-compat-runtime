# Host interface acceptance - 2026-10-07

Version 0.3.1 migrates the host-neutral extension at d9e2fe146 onto main's
390e93afd base. Official 0.3.0 never contained Program.run_external or
Program.historical_session; this was a divergent unpublished extension, not a
stable-wheel packaging omission. These interfaces now belong to the normal
Rust/Python build and installed-wheel gate. No CandleScope dependencies were
introduced into the runtime.

## Changes and contracts

External execution consumes authoritative account frames and emits order
intents. It does not run native matching or expose a native ledger on owned,
borrowed or Python result surfaces. External position history uses feedback;
entry quantity expressions execute once. calc_bars_count slices feedback and
pass groups with the executed chart window. Unsupported settings/fields fail.

Fixed-history sessions preserve complete-dataset endpoint flags, frozen input
and request data, magnifier, overrides, explicit clocks and session windows.
Forward advance and independent forks match the final full batch. Failed
sessions reject further use, while healthy saved forks and prior output remain
valid. Invalid cursor requests do not change execution state.

See [external feedback](EXTERNAL_BROKER_V1.md) and
[fixed history](FIXED_HISTORY_SESSION.md). Analysis/output/changes schemas stay
6/9/4. CLI and WASM retain their existing APIs; the new interfaces are exposed
on Rust/Python.

## Current validation

| Gate | Result |
| --- | --- |
| Windows canonical scripts/verify.ps1 | 7,573 Rust; 166 scripts; 829 installed Python; clippy/fmt, structure, host parity and WASM/Node pass |
| Ubuntu 22.04 canonical scripts/verify.sh | Same 7,573 Rust, 166 scripts and 829 installed Python; remaining checks pass |
| Optimized Windows ABI3 wheel | All 829 Python tests pass, including 31 external/fixed-history cases |
| Unchanged CandleScope native/external/replay suite | 107 pass, zero failures/skips; real native engines |
| Isolated host installation | Core 0.3.1, Pyne 0.4.1, SDK 0.2.0; pip check and module/version/API provenance pass |

Counts describe separate runs and contain repeated cases; they are not a sum
of unique test identities. Local Windows uses Rust 1.97; Linux uses Rust 1.95
and Python 3.10. The host suite uses Windows/Python 3.12. The host adapter is a
local candidate with only its version and exact core dependency repinned to
0.3.1; adapter runtime logic and host assertions are unchanged. No production
registry or active host installation was modified.

The initial host run started before dependency installation finished and had
15 import failures. Its receipt remains under .local/host-interfaces-20261007;
after installation completed, the 14 engine foundation cases and the complete
107-case optimized-wheel run passed. The failed run is not acceptance evidence.

[Machine receipt](HOST_INTERFACES_ACCEPTANCE_20261007.json) binds the local
artifacts, qualified-input digest and successful/failure receipts. Source/input
hash inventory and raw logs remain under .local/host-interfaces-20261007.
GitHub release CI separately builds and tests the official Windows and
manylinux2014/ABI3 assets from the release commit.

## Limits and host migration

The earlier complete 216-trial resource matrix belongs to 0.3.0. This change
reruns canonical budgets/atomicity checks and host isolation/replay cases, but
does not claim a fresh full resource matrix or long-session qualification of
these interfaces. Full native live-tick parity, broader external account/order
profiles and the earlier unsupported platforms remain outside scope.

Hosts should repin their adapter dependency and content lock to the official
0.3.1 artifact, check both APIs in the installed wheel, and rebuild sessions
from authoritative input transcripts. The isolated candidate adapter is not a
published CandleScope plugin asset. Matching, persistence, process management
and production host activation stay in the host repository.

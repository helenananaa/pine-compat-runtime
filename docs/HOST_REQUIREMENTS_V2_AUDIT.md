# Host discovery correction and platform checkpoint

Full-script readiness inspection found that the original Pivot's explicit
lookahead_on request was incorrectly reported as lookaheadOff. The runtime
executed the policy correctly; host discovery only decoded legacy callee names
and defaulted modern calls. Its integer-multiple relation also omitted calendar
months after the nominal-month correction.

Host requirements schema 2 reads modern merge arguments and constant alias
initializers without executing Pine. Unknown expressions are explicit
runtimeExpression values. Legacy encoded policies remain supported. The relation
now names its calendar-month exception. Runtime result schema 9 and changes
schema 4 are unchanged. Consumer migration is documented in HOST_REQUIREMENTS.md.
The original Pivot candidate report in `corpus/pivot-host-requirements-v2.json`
now reports lookaheadOn and the calendar-aware relation. Fourteen Rust inventory
tests pass; Python and actual-WASM assertions were added. The full gate is
`host-requirements-v2-full-verify.log`; no final acceptance is claimed yet.

Separately, committed 4f32668a7 debug artifacts are retained on Windows and Linux.
Both platforms passed 6793 Rust and 772 installed Python tests and actual WASM,
with both full RSI references, prior member/tuple/UDT controls, v5/v6 drawing-array
and matrix controls, and full Pivot numeric/live-simulation/coordinate checks.
Linux used Rust 1.95, Python 3.10.12 and Node 24.18.0. Its wheel is actually
cp310-abi3-manylinux_2_35_x86_64 and contains an ELF extension; do not relabel it
as the earlier candidate's platform tag. The old candidate-path tool test was
skipped on Linux, and the current wheel was inspected separately. Windows ran
all 130 tool tests; Linux ran 130 with that one skip.

Receipts and artifact hashes are registered in PRODUCT_COMPLETION_ARTIFACTS.json.
Those artifacts predate the discovery-schema repair and remain labeled with their
real source identity. Resource acceptance has not been run; its finite workloads
and budgets are frozen in PRODUCT_STANDARD_CANDLE_RESOURCE_PLAN.json. Broader
Pivot inputs, chronological native realtime feed, full visual appearance,
optimized artifacts and final consumption/closeout remain required.

The original Pivot readiness control now also verifies the execution boundary:
its default daily-based branch fails explicitly when the annual provider is
absent; selecting input 4=false executes local calculation without a provider
and produces 99 lines and 99 labels. The conservative inventory still records
the potential request. `full-script-host-readiness.json` retains the exact
missing-provider error and local-run receipt. This is a host-input contract
check, not native numerical qualification of that alternate input configuration.

The full schema-2 Windows gate now passes in
`host-requirements-v2-full-verify-final.log`: 6794 Rust tests, 773 installed-wheel
Python tests, 130 tool tests and actual WASM. The first attempt stopped on an
embedding test's old version-1 assertion; its remaining lifecycle checks passed
after the explicit version expectation was updated. Linux's previously skipped
candidate-path tag test was also rerun unchanged with the current retained wheel
directory and passed (`linux-current-wheel-tag-test.log`). Current schema-2
commit-bound artifacts and resource measurements still remain to be produced.

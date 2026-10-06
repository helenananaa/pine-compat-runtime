# Array checkpoint cost under complete Pivot workload

The release artifacts at 200dbb76d passed targeted native references before
resource measurement. The frozen input generator produced twelve hashed minute
datasets (three history sizes and four independent streams), with completed
daily seed bars and an explicit cumulative forming-feed rule. Sources and plan
hashes are recorded under `resource-inputs-200dbb76d/`.

The Python public-API collector is currently partial and awards no acceptance.
Its append phase means confirmed realtime observations, since Python exposes no
separate historical append API. Native historical incrementality and independent
session controls still require their own checks. The 1024-history Pivot run
completed with 128 tail bars, two repetitions and verified operation counts;
confirmation P95 was about 1.25 ms and peak RSS about 29 MB.

The 100000-history run was stopped for diagnosis after logging 2001 tail
observations in its first phase. The first two blocks each averaged about
107 ms per append. The actual Python worker was around 391 MB RSS at inspection;
an unrelated larger Python process was not attributed to this experiment. The
interruption receipt explicitly says incomplete/notEvaluated, retains the log,
and does not claim a full P95 or memory-budget failure. No budget was relaxed.

Code inspection identified whole-store cloning: runtime checkpoints deep-copied
HashMap<u32, Vec<PineValue>>, while the complete Pivot creates temporary request
arrays and matrix-row arrays on successive bars. Its array value/kind/type/slice
stores now use a sparse persistent ID tree. Checkpoints share closed entries;
insertion/mutation copies a bounded leaf and branch path, and only a changed
payload is cloned. Sparse large IDs do not allocate all intervening IDs. No
third-party persistent-collection dependency or host policy was added.

Two structural tests prove sparse/checkpoint isolation and that checkpoint or
insertion does not clone untouched payloads. Ninety-four existing array tests
pass. Full verification is running in `id-store-full-verify.log`. New optimized
measurements and complete native-reference checks are still required; a speedup
has not yet been measured. Physical arrayCapacity now counts allocated ID-leaf
slots; payload capacity remains reported separately, and these counts are not
an RSS estimate.

The full verifier now passes: 6796 Rust tests, 773 tests against a freshly
installed Python wheel, 130 tool tests, structural/host parity checks and actual
WASM. Release rebuilding, native-reference rechecks and the unchanged large
resource trial are still pending. The collector now persists incomplete timing
checkpoints during long runs so future interrupted trials retain raw samples;
that instrumentation change does not alter the workload or acceptance budgets.

Release 5c775fb87 passed all prior native-reference gates. Its unchanged
100000-history trial improved markedly but still missed budgets: the first
complete 10000-append phase had P95 5.3098 ms and its replica applications had
P95 4.2504 ms. Raw samples were retained in the interrupted checkpoint; the full
multi-phase/repetition case remains incomplete, not accepted. No memory-limit
failure is claimed.

The next cost was unconditional empty drawing SetTail records plus quadratic
ID searches. The output cursor now retains both stable-prefix and total snapshot
lengths, so it can omit genuinely unchanged objects while still emitting empty
tails that retract forming snapshots. Tables retain their updates because their
metadata lives outside snapshots. Drawing ID lookups use ordered maps and
missing-ID detection avoids quadratic scans. A focused test verifies omission,
mutation, preview retraction and confirmation against a replica; it passes.
The full verifier passed on September 12; its raw output is retained in
`drawing-delta-full-verify.log`. Performance after this second change has not
yet been measured.

# Version annotation spacing

2026-09-09. Locally verified slice; no release. Baseline ad2063878.

The exact RelativeValue/3 library source uses `// @version=6`. Previously the
lexer only recognized `//@version`, so that source was incorrectly classified
as implicit v1 and triggered a root/library language-version conflict.
The lexer now allows ASCII horizontal spaces/tabs between `//` and `@version`.
Payload validation, token spans, duplicate/placement checks and version ranges
remain unchanged. Ordinary comments with later mentions and extra prefixes
are not version declarations. No runtime, host, schema or snapshot changes.

Chrome independently compiled an original spaced-version v6 indicator using
a simple-qualified function; Pine Editor displayed v6 and plot value 3.
Exact RelativeValue/3 was supplied without editing its bytes, and its
E_LANGUAGE_VERSION_CONFLICT disappeared. The full dependency chain still fails
on more complex declarations/calls; this is not complete library acceptance.

Grok implemented the lexer change and seven integration tests in an isolated
worktree. Codex fixed a test-only extra reference that prevented compilation,
formatted the tests, and corrected two older parser/sema expectations that
classified the exact spaced v6 input as implicit v1. Those original inputs
remain in tests; the changed classification is explicit. Duplicate and
misplaced-version tests continue to run unchanged.

Final `scripts/verify.ps1 -Python python` exited 0:
6,575 Rust tests, 677 tests against a fresh installed Python wheel, 101 tool
tests, structural/host checks and real WASM/Node smoke. No golden was refreshed.
Logs, original probes and full-library diagnostics are retained under
`.local/delivery-20260909/` in the main checkout (`version-full-verify-v2.log`,
`spaced-version-dom.txt`, `technical-after-version.json`).

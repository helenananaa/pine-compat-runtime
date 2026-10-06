# Contributing

This project is a clean-room Pine-compatible indicator and strategy runtime
for an explicitly supported subset. The core owns deterministic Pine semantics;
hosts own data acquisition, services, persistence and application policy. See
[AGENTS.md](AGENTS.md) for the architectural boundary.

## Rules

- Do not copy TradingView source code, private APIs, proprietary data, UI, icons,
  branding, or error text.
- Prefer original fixtures written for this project.
- Include license and source metadata for non-original fixtures.
- Unsupported language behavior must produce diagnostics instead of panics.
- Keep host-specific adapters outside the core runtime crates.

## Development

Use Rust 1.95+, Python 3.10+, Node.js, the `wasm32-unknown-unknown`
Rust target, and `maturin>=1.13,<2.0` plus pytest. Before submitting runtime
or binding changes, run the complete gate:

```bash
scripts/verify.sh
```

It covers formatting, clippy, workspace and tool tests, source structure,
host parity, actual generated WASM/Node execution, and a freshly built wheel
installed into a disposable test environment. `cargo check` alone does not
qualify the JavaScript module or installed Python extension. Documentation-only
changes can use focused source, example and link checks instead.

Windows contributors can run the equivalent full gate with:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify.ps1 -Python python
```

Python binding changes should also be checked in an active virtual environment
where the extension module can be installed:

```bash
python -m pip install "maturin>=1.13,<2.0" pytest
maturin develop --manifest-path crates/pine-python/Cargo.toml
python -m pytest python/tests
```

## Module Ownership

New code should go to the crate and module that owns the behavior:

- `pine-syntax`: source files, lexer, parser, AST, and syntax diagnostics.
- `pine-builtins`: semantic signatures, built-in constants, namespace registry
  data, and shared return specifications.
- `pine-ir`: host-neutral HIR and shared intermediate model contracts.
- `pine-sema`: compatibility reports, resolver/scope rules, type acceptance,
  call validation, HIR lowering, and semantic history requirements.
- `pine-runtime`: bar execution, runtime values/storage, built-in execution,
  output collection, profiling, retention, and realtime update behavior.
- `pine-cli`, `pine-python`, and `pine-wasm`: thin host adapters only.

Built-in additions must move through all applicable owners in one change:

- semantic signature or constant metadata in `pine-builtins`
- type, argument, or unsupported-feature validation in `pine-sema` when needed
- runtime implementation in `pine-runtime`
- fixture coverage under `tests/fixtures`
- conformance metadata in `tests/fixtures/conformance.tsv` when capability
  reporting changes

Keep cross-crate behavior fixtures in `tests/fixtures`. Keep small edge-case
tests near the module that owns private helper behavior.

## Source Size Guardrail

`python3 scripts/check_structure.py` checks production Rust source files before
the binding checks in `scripts/verify.sh`. The default limits are:

- Facade files such as crate-root `lib.rs` and declaration-only `mod.rs`: 300
  lines.
- Model/helper modules: 800 lines.
- Implementation modules: 1,500 lines.

Large table-heavy files must be listed in the script allowlist with an owner,
reason, and split plan. Do not widen production visibility just to move tests;
prefer module-local tests, `pub(super)`, or `pub(crate)` only when there is a
real internal API boundary.

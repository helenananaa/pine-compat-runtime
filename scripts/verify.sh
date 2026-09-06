#!/bin/sh
set -eu

run() {
    printf '+ %s\n' "$*"
    "$@"
}

run cargo fmt --check
run cargo clippy --workspace --all-targets -- -D warnings
run cargo test --workspace
run python3 scripts/check_structure.py
run python3 -m unittest scripts/tests/test_check_host_parity.py
run python3 -m unittest scripts/tests/test_build_wheel_manifest.py
run python3 -m unittest scripts/tests/test_analyze_legacy_corpus.py
run python3 -m unittest scripts/tests/test_analyze_modern_strategy_corpus.py
run python3 -m unittest scripts/tests/test_audit_legacy_corpus_dedup.py
run python3 -m unittest scripts/tests/test_import_legacy_corpus.py
run python3 -m unittest scripts/tests/test_merge_legacy_corpus_manifests.py
run python3 -m unittest scripts/tests/test_compare_tradingview_outputs.py
run python3 -m unittest scripts/tests/test_compare_strategy_reference_outputs.py
run python3 -m unittest scripts/tests/test_normalize_tradingview_bars.py
run python3 -m unittest scripts/tests/test_profile_legacy_release.py
run python3 scripts/check_host_parity.py
run scripts/check_wasm_node.sh

# Build and install inside one disposable gate root. This keeps a release check
# from replacing the caller's active pine_compat extension or package stamp.
gate_root=$(mktemp -d "${TMPDIR:-/tmp}/pine-python-release-gate.XXXXXX")
wheel_output_dir="$gate_root/wheels"
wheel_test_venv="$gate_root/venv"
mkdir -p "$wheel_output_dir"
trap 'rm -rf "$gate_root"' EXIT
trap 'exit 1' HUP INT TERM
run maturin build --manifest-path crates/pine-python/Cargo.toml --out "$wheel_output_dir"
set -- "$wheel_output_dir"/*.whl
if [ "$#" -ne 1 ] || [ ! -f "$1" ]; then
    printf 'error: expected exactly one freshly built wheel in %s\n' \
        "$wheel_output_dir" >&2
    exit 1
fi
run python3 -m venv --system-site-packages "$wheel_test_venv"
run "$wheel_test_venv/bin/python" -m pip install --no-deps --force-reinstall "$1"
run "$wheel_test_venv/bin/python" -m pytest python/tests

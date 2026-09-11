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
# Match Windows discovery so newly added tool tests are not silently omitted.
run python3 -m unittest discover -s scripts/tests -p 'test_*.py'
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

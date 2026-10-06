#!/bin/sh
set -eu
# Run in a Linux checkout of the pinned commit; .local reference corpus must exist.
output=$1
plan=$2
test ! -e "$output"
test -z "$(git diff HEAD -- crates Cargo.toml Cargo.lock)"
mkdir -p "$output/wasm" "$output/wheels" "$output/rust-probe"
source_commit=$(git rev-parse HEAD)
sh scripts/verify.sh
cargo build --locked --release -p pine-cli
cp "${CARGO_TARGET_DIR:-target}/release/pine-compat" "$output/pine-compat.exe"
cargo build --locked --release -p pine-wasm --target wasm32-unknown-unknown
host=$(rustc -vV | sed -n 's/^host: //p')
cargo run --locked --quiet -p pine-wasm --example generate_node_bindings --target "$host" -- "${CARGO_TARGET_DIR:-target}/wasm32-unknown-unknown/release/pine_wasm.wasm" "$output/wasm"
maturin build --locked --release --manifest-path crates/pine-python/Cargo.toml --out "$output/wheels"
python3 -m venv --system-site-packages "$output/venv"
set -- "$output/wheels"/*.whl
test "$#" -eq 1
"$output/venv/bin/python" -m pip install --no-deps --force-reinstall "$1"
"$output/venv/bin/python" -m pytest python/tests
# The retained reference helpers use these neutral artifact names on both OSes.
mkdir "$output/venv/Scripts"
ln -s ../bin/python "$output/venv/Scripts/python.exe"
cp scripts/core_script_probe.rs "$output/rust-probe/main.rs"
cp Cargo.lock "$output/rust-probe/Cargo.lock"
python3 - "$output/rust-probe/Cargo.toml" <<'PY'
import sys
from pathlib import Path
deps='\n'.join(f'{n}={{path="{Path.cwd()/"crates"/n}"}}' for n in ['pine-runtime','pine-sema','pine-syntax'])
Path(sys.argv[1]).write_text('[package]\nname="core-script-probe"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[[bin]]\nname="core-script-probe"\npath="main.rs"\n[dependencies]\n'+deps+'\nserde_json="1"\n')
PY
cargo build --offline --release --manifest-path "$output/rust-probe/Cargo.toml"
cp "${CARGO_TARGET_DIR:-$output/rust-probe/target}/release/core-script-probe" "$output/core-script-probe.exe"
test "$(git rev-parse HEAD)" = "$source_commit"
python3 scripts/requalify_core_scripts.py --artifacts "$output" --plan "$plan" --profile release

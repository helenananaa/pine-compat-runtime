"""Build hash-pinned working-tree resource candidates without committing user work.

Run inside an initialized native toolchain. The output must be a fresh directory.
This produces resource artifacts, not a release qualification or a Git commit.
"""
import argparse
import hashlib
import json
import os
import platform
import shutil
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(*command):
    print('+', ' '.join(map(str, command)), flush=True)
    subprocess.run(list(map(str, command)), cwd=REPO, check=True)


def build(output, identity):
    source = json.loads(identity.read_text(encoding='utf-8-sig'))
    for path, digest in source['coreFiles'].items():
        assert sha(REPO/path) == digest, f'source differs: {path}'
    output.mkdir(parents=True, exist_ok=False)
    release = Path(os.environ.get('CARGO_TARGET_DIR', REPO/'target'))/'release'
    wasm_release = release.parent/'wasm32-unknown-unknown/release'
    windows = os.name == 'nt'
    for directory in ['wasm', 'wheels', 'rust-probe']:
        (output/directory).mkdir()
    run('cargo', 'build', '--locked', '--release', '-p', 'pine-wasm', '--target', 'wasm32-unknown-unknown')
    host = next(line[6:] for line in subprocess.check_output(['rustc','-vV'],text=True).splitlines() if line.startswith('host: '))
    run('cargo','run','--locked','--quiet','-p','pine-wasm','--example','generate_node_bindings','--target',host,'--',wasm_release/'pine_wasm.wasm',output/'wasm')
    run('maturin','build','--locked','--release','--manifest-path','crates/pine-python/Cargo.toml','--out',output/'wheels')
    run(sys.executable,'-m','venv','--system-site-packages',output/'venv')
    python = output/('venv/Scripts/python.exe' if windows else 'venv/bin/python')
    wheels = list((output/'wheels').glob('*.whl'))
    assert len(wheels)==1
    run(python,'-m','pip','install','--no-deps','--force-reinstall',wheels[0])
    run(python,'-m','pytest','python/tests')
    run('node','scripts/tests/wasm_node_smoke.cjs',output/'wasm/pine_wasm.js')
    probe = output/'rust-probe'
    shutil.copyfile(REPO/'scripts/product_resource_probe.rs',probe/'main.rs')
    shutil.copyfile(REPO/'Cargo.lock',probe/'Cargo.lock')
    manifest = '[package]\nname="resource-probe"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[[bin]]\nname="resource-probe"\npath="main.rs"\n[dependencies]\n'
    for crate in ['pine-runtime','pine-sema','pine-syntax']:
        manifest += f'{crate}={{path={json.dumps((REPO/"crates"/crate).as_posix())}}}\n'
    (probe/'Cargo.toml').write_text(manifest+'serde_json="1"\n',encoding='utf-8')
    # The workspace lock has extra packages; allow Cargo to prune them offline.
    run('cargo','build','--offline','--release','--manifest-path',probe/'Cargo.toml')
    probe_target = Path(os.environ.get('CARGO_TARGET_DIR',probe/'target'))/'release'
    binary = 'resource-probe.exe' if windows else 'resource-probe'
    shutil.copyfile(probe_target/binary,output/binary)
    for path, digest in source['coreFiles'].items():
        assert sha(REPO/path)==digest, f'source changed during build: {path}'
    source.update(profile='release',platform=platform.platform(),sourceState='workingTreeHashPinned',
        resourceProbeSourceSha256=sha(REPO/'scripts/product_resource_probe.rs'),
        rustc=subprocess.check_output(['rustc','-V'],text=True).strip(),installedWheelTests='passed',actualWasmNode='passed')
    (output/'build-provenance.json').write_text(json.dumps(source,indent=2),encoding='utf-8')


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--source-identity',type=Path,required=True)
    args=parser.parse_args()
    build(args.output.resolve(),args.source_identity.resolve())

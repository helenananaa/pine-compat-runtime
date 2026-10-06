"""Assemble local optimized artifacts with an auditable payload manifest."""
import argparse
import json
import shutil
import subprocess
import zipfile
from pathlib import Path

from requalify_core_scripts import read, write, sha

REPO = Path(__file__).resolve().parents[1]


def package(root, output):
    output.mkdir(exist_ok=False)
    commits=set(); platforms=[]
    for label in ('windows','linux'):
        artifacts=root/label
        results=read(artifacts/'matrix-results.json')
        provenance=read(artifacts/'build-provenance.json')
        assert results['completed'] and not results['failures']
        assert results['crossSurfaceCasesPassed']==15
        assert provenance['profile']=='release'
        commits.add(provenance['sourceCommit'])
        dest=output/label;dest.mkdir()
        (dest/'bin').mkdir();(dest/'wasm').mkdir();(dest/'wheels').mkdir();(dest/'examples').mkdir()
        shutil.copyfile(artifacts/'pine-compat.exe',dest/'bin'/('pine-compat.exe' if label=='windows' else 'pine-compat'))
        for p in (artifacts/'wasm').glob('*'):shutil.copyfile(p,dest/'wasm'/p.name)
        for p in (artifacts/'wheels').glob('*.whl'):shutil.copyfile(p,dest/'wheels'/p.name)
        (dest/'examples/sma.pine').write_text('//@version=6\nindicator("Offline example")\nplot(ta.sma(close, 3))\n')
        (dest/'examples/bars.csv').write_text('time,open,high,low,close,volume\n0,100,101,99,100,1\n60000,100,102,99,101,1\n120000,101,103,100,102,1\n180000,102,104,101,103,1\n')
        (dest/'examples/run.py').write_text('''import csv,json
from pathlib import Path
import pine_compat
base=Path(__file__).resolve().parent
with (base/'bars.csv').open() as f:
    bars=[{k:int(v) if k=='time' else float(v) for k,v in r.items()} for r in csv.DictReader(f)]
result=pine_compat.run_script((base/'sma.pine').read_text(),bars)
assert result['schemaVersion']==9 and not result['diagnostics']
print(json.dumps(result))
''')
        (dest/'examples/run.cjs').write_text('''const fs=require('node:fs'),path=require('node:path');
const pine=require('../wasm/pine_wasm.js');
const result=JSON.parse(pine.runScriptCsv(fs.readFileSync(path.join(__dirname,'sma.pine'),'utf8'),fs.readFileSync(path.join(__dirname,'bars.csv'),'utf8')));
if(result.schemaVersion!==9||result.diagnostics.length)throw Error('smoke failed');
console.log(JSON.stringify(result));
''')
        write(dest/'qualification.json',dict(coreCommit=provenance['sourceCommit'],profile='release',platform=provenance['platform'],matrixSha256=sha(artifacts/'matrix-results.json'),provenanceSha256=sha(artifacts/'build-provenance.json'),resourceQualification='See separate resource receipt; compatibility does not imply resource acceptance'))
        shutil.copyfile(REPO/'LICENSE',dest/'LICENSE')
        platforms.append(dict(name=label,files={p.relative_to(dest).as_posix():sha(p) for p in sorted(dest.rglob('*')) if p.is_file()}))
    assert len(commits)==1
    commit=commits.pop()
    evidence=output/'evidence';evidence.mkdir()
    shutil.copyfile(root/'plan.json',evidence/'compatibility-plan.json')
    cross=read(root/'cross-platform-audit.json');assert cross['status']=='passed' and cross['outputsCompared']==180
    shutil.copyfile(root/'cross-platform-audit.json',evidence/'cross-platform-audit.json')
    write(evidence/'qualification-scope.json',dict(coreCommit=commit,profile='release',platforms=['Windows x86_64','Ubuntu 22.04 x86_64 under WSL'],compatibilityCases=15,completeOutputs=180,planSha256=sha(root/'plan.json'),inheritedPlanScope=read(root/'plan.json')['platformScope'],scopeNote='The frozen workload plan retains its prior debug/platform status label. This current qualification scope and built-artifact provenance supersede that inherited label; original receipts remain unchanged.'))
    for label in ('windows','linux'):
        shutil.copyfile(root/label/'matrix-results.json',evidence/f'compatibility-{label}.json')
        shutil.copyfile(root/label/'build-provenance.json',evidence/f'build-provenance-{label}.json')
    references=output/'references'
    for relative,digest in read(root/'plan.json')['frozenFiles'].items():
        source=REPO/relative;assert sha(source)==digest
        dest=references/relative;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(source,dest)
    resource=REPO/'.local/resource-20261001-final'
    assert (resource/'summary.json').exists(),'Complete and audit resource measurements before packaging'
    resource_result=read(resource/'summary.json');assert resource_result['coreCommit']==commit
    shutil.copyfile(resource/'summary.json',evidence/'resources.json')
    shutil.copyfile(resource/'plan.json',evidence/'resource-plan.json')
    for name in ['sampler-repair.json','worker-frozen.py','worker-repaired-frozen.py','generator-frozen.py','execution-interruption.json','probe-buffering-repair.json','probe-unbuffered-frozen.rs','probe-buffered-frozen.rs']:
        shutil.copyfile(resource/name,evidence/name)
    tools=output/'resource-tools';tools.mkdir()
    for name in ['product_resource_acceptance.py','product_resource_wasm.cjs','product_resource_probe.rs','requalify_core_scripts.py']:
        source=resource/'worker-repaired-frozen.py' if name=='product_resource_acceptance.py' else REPO/'scripts'/name
        shutil.copyfile(source,tools/name)
    verification=output/'verification-tools';verification.mkdir()
    for name in ['verify_local_delivery.py','requalify_core_scripts.py']:
        shutil.copyfile(REPO/'scripts'/name,verification/name)
    subprocess.run(['git','archive','--format=tar','--output',str(output/'source.tar'),commit],cwd=REPO,check=True)
    (output/'README.md').write_text(f'''# Local optimized prerelease

Source commit: `{commit}`. Runtime result schema 9, changes schema 4,
realtime session schema 1, render metadata version 1.
Compatibility qualification: passed for the frozen fifteen-case corpus on both
platforms. Resource qualification: **{resource_result['status']}**; see
`evidence/resources.json` for measured failures and missing controls.

Choose the matching platform directory. Python requires CPython 3.10+;
the wheel filename declares the exact ABI/platform tag. Install into a fresh
virtual environment with `python -m pip install wheels/<wheel filename>` and
run `python examples/run.py`. Node runs `node examples/run.cjs`.

Windows CLI: `bin/pine-compat.exe run examples/sma.pine --bars examples/bars.csv`.
Linux CLI: `chmod +x bin/pine-compat` then
`bin/pine-compat run examples/sma.pine --bars examples/bars.csv`.

`source.tar` contains the pinned Rust workspace, contracts, tests and build
scripts. Build the native CLI with `cargo build --release --locked -p pine-cli`.
Embedding code uses pine-syntax/pine-sema/pine-runtime without a concrete host.
Cargo dependencies must be available to the build environment.
Documents inside the source archive retain that commit's historical checkpoint.
This README and `evidence/qualification-scope.json` define the current packaged
qualification; inherited debug/Linux status labels in the frozen workload plan
do not override the optimized platform receipts.

`MANIFEST.json` hashes every distributed payload. Platform qualification records
bind the compatibility receipts. Frozen source/reference inputs are retained
under `references`, with their original corpus-relative paths and hashes.
Verify extracted bytes and fresh installation with
`python verification-tools/verify_local_delivery.py . windows <new receipt directory>`
(use `linux` on Linux). This creates a fresh venv, installs the local wheel
without network access and checks CLI/Python/Node outputs against an SMA oracle.
The measured resource tools are retained separately from the committed source
archive and have their own hashes. Resource evidence is separate and may contain
failed or unverified budgets. The qualified script/settings corpus does not
claim arbitrary Pine compatibility or native live Tick parity.
This artifact is local and has not been published.
''')
    files={p.relative_to(output).as_posix():sha(p) for p in sorted(output.rglob('*')) if p.is_file()}
    write(output/'MANIFEST.json',dict(coreCommit=commit,profile='release',platforms=platforms,files=files,packageToolSha256=sha(Path(__file__))))
    with zipfile.ZipFile(str(output)+'.zip','w',zipfile.ZIP_DEFLATED) as z:
        for p in sorted(output.rglob('*')):
            if p.is_file():z.write(p,p.relative_to(output).as_posix())
    write(Path(str(output)+'.archive.json'),dict(archiveSha256=sha(Path(str(output)+'.zip')),manifestSha256=sha(output/'MANIFEST.json'),coreCommit=commit))
    print(output)


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('root',type=Path);parser.add_argument('output',type=Path)
    args=parser.parse_args();package(args.root.resolve(),args.output.resolve())

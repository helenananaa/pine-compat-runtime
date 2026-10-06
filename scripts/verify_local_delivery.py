"""Verify packaged bytes and installation examples in a fresh environment."""
import argparse
import os
import subprocess
import sys
from pathlib import Path

from requalify_core_scripts import read, write, sha, differences


def verify(bundle, platform, output):
    output.mkdir(exist_ok=False)
    manifest=read(bundle/'MANIFEST.json')
    for relative,digest in manifest['files'].items():assert sha(bundle/relative)==digest,relative
    payload=bundle/platform
    subprocess.run([sys.executable,'-m','venv',str(output/'venv')],check=True)
    py=output/('venv/Scripts/python.exe' if os.name=='nt' else 'venv/bin/python')
    wheels=list((payload/'wheels').glob('*.whl'));assert len(wheels)==1
    subprocess.run([str(py),'-m','pip','install','--no-index','--no-deps',str(wheels[0])],check=True)
    cli=payload/'bin'/('pine-compat.exe' if platform=='windows' else 'pine-compat')
    if os.name!='nt':cli.chmod(cli.stat().st_mode|0o111)
    commands={
        'cli':[str(cli),'run',str(payload/'examples/sma.pine'),'--bars',str(payload/'examples/bars.csv')],
        'python':[str(py),str(payload/'examples/run.py')],
        'wasm':['node',str(payload/'examples/run.cjs')],
    }
    baseline=None;receipts={}
    for surface,command in commands.items():
        output_file=output/(surface+'.json')
        with output_file.open('wb') as f:subprocess.run(command,stdout=f,check=True)
        result=read(output_file)
        assert result['schemaVersion']==9 and result['renderMetadataVersion']==1
        assert not result['diagnostics']
        assert result['plots'][0]['values']==[None,None,101,102]
        if baseline is None:baseline=result
        assert not differences(baseline,result),surface
        receipts[surface]=dict(command=command,outputSha256=sha(output_file))
    module=subprocess.check_output([str(py),'-c','import pine_compat;print(pine_compat.__file__)'],text=True).strip()
    assert Path(module).is_relative_to(output/'venv'),'wheel import is outside the fresh environment'
    versions={'python':subprocess.check_output([str(py),'--version'],text=True).strip(),'node':subprocess.check_output(['node','--version'],text=True).strip(),'wheel':wheels[0].name,'wheelSha256':sha(wheels[0])}
    receipt=dict(status='passed',coreCommit=manifest['coreCommit'],platform=platform,manifestSha256=sha(bundle/'MANIFEST.json'),module=module,runtimeVersions=versions,surfaces=receipts,oracle='SMA(3) of 100/101/102/103 is null/null/101/102',verifierSha256=sha(Path(__file__)))
    write(output/'receipt.json',receipt);print('passed',platform)


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('bundle',type=Path);parser.add_argument('platform',choices=['windows','linux']);parser.add_argument('output',type=Path)
    a=parser.parse_args();verify(a.bundle.resolve(),a.platform,a.output.resolve())

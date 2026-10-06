"""Independently audit frozen outputs across both optimized platforms."""
import argparse
from pathlib import Path

from requalify_core_scripts import read, write, sha, differences, MODES


def audit(root):
    plan=read(root/'plan.json');reports={};cases=[]
    for system in ('windows','linux'):
        report=read(root/system/'matrix-results.json')
        assert report['completed'] and not report['failures']
        assert report['coreCommit']==plan['coreCommit']
        assert report['planSha256']==sha(root/'plan.json')
        reports[system]=report
    for case in plan['cases']:
        baseline=None;outputs={};issues=[]
        names=[(m,case['outputs'][m]) for m in MODES]+[(s,case['id']+suffix) for s,suffix in [('installedPython','-python.json'),('actualNodeWasm','-wasm.json'),('directRust','-rust.json')]]
        for system,report in reports.items():
            row=next(r for r in report['cases'] if r['id']==case['id'])
            assert row['status']=='cross-surface-passed'
            for surface,filename in names:
                path=root/system/case['group']/filename;record=row['surfaces'][surface]
                digest=sha(path)
                assert digest==record.get('outputSha256',record.get('receipt',{}).get('outputSha256'))
                value=read(path)
                if baseline is None:baseline=value
                diff=differences(baseline,value)
                if diff:issues.append(dict(system=system,surface=surface,count=len(diff),examples=diff[:10]))
                outputs[system+'/'+surface]=digest
        cases.append(dict(id=case['id'],outputs=outputs,mismatches=issues))
    result=dict(coreCommit=plan['coreCommit'],planSha256=sha(root/'plan.json'),status='failed' if any(c['mismatches'] for c in cases) else 'passed',cases=cases,outputsCompared=sum(len(c['outputs']) for c in cases),toolSha256=sha(Path(__file__)),scope='frozen full result JSON; prior native comparators retain their named assertions')
    write(root/'cross-platform-audit.json',result)
    print(result['status'],result['outputsCompared'])


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('root',type=Path)
    audit(p.parse_args().root.resolve())

"""Close resource receipts without converting missing cases or budget failures to passes."""
import argparse
from pathlib import Path

from requalify_core_scripts import read, write, sha, differences


def audit(root):
    plan=read(root/'plan.json');reports={};failed=[];unverified=[];comparisons=[];isolation=[]
    recipes={sha(root/'worker-frozen.py'),sha(root/'worker-repaired-frozen.py')}
    repair=read(root/'sampler-repair.json')
    assert repair['collectorSha256']==sha(root/'worker-repaired-frozen.py')
    buffering=read(root/'probe-buffering-repair.json')
    assert buffering['oldProbeSourceSha256']==sha(root/'probe-unbuffered-frozen.rs')
    assert buffering['bufferedProbeSourceSha256']==sha(root/'probe-buffered-frozen.rs')
    probe_recipes={buffering['oldProbeSourceSha256'],buffering['bufferedProbeSourceSha256']}
    for case in plan['cases']:assert sha(root/case['payload'])==case['payloadSha256']
    for system in ('Windows','Linux'):
        for surface in ('rust','python','wasm'):
            path=root/system/surface/'results.json'
            if not path.exists():
                unverified.append(f'{system}/{surface}: no receipt');continue
            report=read(path);assert report['completed']
            assert report['planSha256']==sha(root/'plan.json')
            assert report['coreCommit']==plan['coreCommit']
            assert report['runnerSha256'] in recipes
            assert report['wasmRunnerSha256']==plan['workerSourceHashes']['scripts/product_resource_wasm.cjs']
            assert report['rustProbeSourceSha256'] in probe_recipes
            if surface=='rust':assert report['rustProbeSourceSha256']==buffering['bufferedProbeSourceSha256' if system=='Linux' else 'oldProbeSourceSha256']
            artifacts=root.parents[0]/'delivery-20261001-v2'/system.lower()
            assert report['buildProvenanceSha256']==sha(artifacts/'build-provenance.json')
            for relative,digest in report['artifactHashes'].items():assert sha(artifacts/relative)==digest
            if system=='Linux' and surface=='rust':assert report['artifactHashes']['resource-probe']==buffering['bufferedLinuxProbeSha256']
            assert len(report['rows'])==len(plan['cases'])*plan['repetitions']
            assert len({r['id'] for r in report['rows']})==len(report['rows'])
            for row in report['rows']:
                base=path.parent
                assert sha(base/(row['id']+'.progress.log'))==row['progressSha256']
                if row['status']=='measured':assert sha(base/(row['id']+'.json'))==row['outputSha256']
                else:unverified.append(f'{system}/{surface}/{row["id"]}: {row.get("error")}')
            reports[system+'/'+surface]=dict(receiptSha256=sha(path),rows=len(report['rows']),measured=sum(r['status']=='measured' for r in report['rows']),failures=report['failures'],growth=report['growth'],host=report['host'],artifactHashes=report['artifactHashes'],runnerSha256=report['runnerSha256'],wasmRunnerSha256=report['wasmRunnerSha256'],rustProbeSourceSha256=report['rustProbeSourceSha256'])
            failed.extend(dict(surface=system+'/'+surface,failure=f) for f in report['failures'])
    for case in plan['cases']:
        baseline=None;row=dict(case=case['id'],outputs={},mismatches=[],missing=[])
        for system in ('Windows','Linux'):
            for surface in ('rust','python','wasm'):
                for repeat in range(plan['repetitions']):
                    tag=case['id']+f'-{repeat}';report_path=root/system/surface/'results.json'
                    if not report_path.exists():continue
                    saved=next(r for r in read(report_path)['rows'] if r['id']==tag)
                    if saved['status']!='measured':row['missing'].append(f'{system}/{surface}/{repeat}');continue
                    output=root/system/surface/(tag+'.json')
                    value=read(output)['results']
                    if baseline is None:baseline=value
                    issues=differences(baseline,value)
                    row['outputs'][f'{system}/{surface}/{repeat}']=sha(output)
                    if issues:row['mismatches'].append(dict(surface=f'{system}/{surface}/{repeat}',count=len(issues),examples=issues[:10]))
                    del value
                    if surface=='rust' and saved.get('historicalAppendMatches') is False:
                        unverified.append(f'{system}/{tag}: seed/append and one-shot batch have distinct dataset-end contexts and unequal full outputs')
        comparisons.append(row);print(case['id'],'missing',len(row['missing']),'mismatches',len(row['mismatches']),flush=True)
        if case['sessions']==4 and baseline is not None:
            singleton=case['id'].rsplit('-',1)[0]+'-1'
            reference=None
            for system in ('Windows','Linux'):
                for surface in ('rust','python','wasm'):
                    path=root/system/surface/(singleton+'-0.json')
                    if path.exists():
                        saved=next(r for r in read(path.parent/'results.json')['rows'] if r['id']==singleton+'-0')
                        if saved['status']=='measured':reference=read(path)['results'][0];break
                if reference is not None:break
            if reference is not None:
                issues=differences(reference,baseline[0]);isolation.append(dict(case=case['id'],scope='session zero alone versus interleaved with three distinct streams',mismatches=len(issues),examples=issues[:10]))
                if issues:failed.append(dict(case=case['id'],error='session zero isolation comparison failed'))
    mismatches=sum(len(r['mismatches']) for r in comparisons)
    summary=dict(coreCommit=plan['coreCommit'],planSha256=sha(root/'plan.json'),status='notPassed' if failed or unverified or mismatches else 'passed',reports=reports,budgetAndExecutionFailures=failed,unverified=unverified,crossSurfaceComparisons=comparisons,sessionZeroIsolation=isolation,crossSurfaceMismatchGroups=mismatches,auditToolSha256=sha(Path(__file__)),samplerRepair=repair,samplerRepairSha256=sha(root/'sampler-repair.json'),probeBufferingRepair=buffering,probeBufferingRepairSha256=sha(root/'probe-buffering-repair.json'),scope=plan['scope'],qualificationBoundary='Finite synthetic offline workload; no native live Tick or indefinite-memory claim')
    write(root/'summary.json',summary)
    return summary


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('root',type=Path)
    args=parser.parse_args();result=audit(args.root.resolve());print(result['status'])

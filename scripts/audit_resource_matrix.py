"""Audit a hash-pinned resource matrix, including complete spooled public outputs."""
import argparse
import json
import math
import time
from pathlib import Path
from requalify_core_scripts import REPO, read, write, sha, differences
from resource_receipt_reuse import verify_reuse, receipt_root
from product_resource_acceptance import same_files, resource_metadata
from resource_report import expected_spool_names, validate_report


def validate_json_output(path):
    def finite_float(text):
        value=float(text)
        if not math.isfinite(value):raise ValueError('nonfinite public JSON number')
        return value
    def reject_constant(text):
        raise ValueError(f'invalid public JSON constant: {text}')
    with Path(path).open(encoding='utf-8-sig') as stream:
        value=json.load(stream,parse_float=finite_float,parse_constant=reject_constant)
    assert isinstance(value,dict), 'public runtime output must be a JSON object'


def compare_outputs(first, second, validated=None, known_hashes=None):
    validated=set() if validated is None else validated
    known_hashes={} if known_hashes is None else known_hashes
    # Every distinct, integrity-checked content still undergoes complete JSON
    # validation once. Equal empty/truncated files cannot pass the byte path.
    for path in (first,second):
        digest=known_hashes.get(path)
        if digest is None:digest=sha(path)
        if digest not in validated:
            validate_json_output(path)
            validated.add(digest)
    # Exact complete-byte agreement is sufficient. Only formatting/numeric
    # differences need the existing tolerant whole-output comparison.
    if same_files(first, second):
        return []
    return differences(read(first), read(second))


def audit(root):
    audit_started=time.monotonic()
    plan=read(root/'plan.json')
    assert plan['schemaVersion']==2
    frozen=REPO/'docs/PRODUCT_STANDARD_CANDLE_RESOURCE_PLAN.json'
    assert plan['budgetSha256']==sha(frozen) and plan['budgets']==read(frozen)
    assert plan['repetitions']==plan['budgets']['repetitions']==2
    identity_path=REPO/plan['sourceIdentityFile']
    assert sha(identity_path)==plan['sourceIdentitySha256']
    identity=read(identity_path)
    reuse_proof=verify_reuse(REPO,plan)
    for name,digest in plan.get('workerSourceHashes',{}).items():assert sha(REPO/name)==digest
    for name,digest in plan.get('workerAuxiliarySourceHashes',{}).items():assert sha(REPO/name)==digest
    assert {'Cargo.toml','Cargo.lock','crates/pine-runtime/src/lib.rs'}<=identity['coreFiles'].keys()
    expected_cases={f'{script}-{size["historyBars"]}-{count}':(size['historyBars'],size['tailBars'],count)
        for script in ['rsi-default','rsi-alternate','pivot-original']
        for size in plan['budgets']['historyCases'] for count in plan['budgets']['independentSessionCounts']}
    assert len(plan['cases'])==len(expected_cases)
    assert {case['id']:(case['history'],case['tail'],case['sessions']) for case in plan['cases']}==expected_cases
    expected={case['id']+f'-{repeat}' for case in plan['cases'] for repeat in range(plan['repetitions'])}
    reports={};failures=[];missing=[];comparisons=[];isolation=[];known_hashes={};validated=set()
    for case in plan['cases']:assert sha(root/case['payload'])==case['payloadSha256']
    for system in ['Windows','Linux']:
        artifacts=REPO/plan['artifactPaths'][system]
        for surface in ['rust','python','wasm']:
            key=system+'/'+surface;selected_root=receipt_root(REPO,root,plan,system,surface)
            selected_plan=read(selected_root/'plan.json')
            path=selected_root/system/surface/'results.json'
            if not path.exists():missing.append(key);continue
            report=read(path)
            assert report.get('completed') is True,key
            assert report['planSha256']==sha(selected_root/'plan.json')
            assert report['coreCommit']==plan['coreCommit']
            if report.get('collectorContractVersion')==2:
                for name,digest in report['workerHashBindings'].items():assert sha(REPO/name)==digest
            assert report['runnerSha256']==selected_plan['workerSourceHashes']['scripts/product_resource_acceptance.py']
            assert report['wasmRunnerSha256']==selected_plan['workerSourceHashes']['scripts/product_resource_wasm.cjs']
            assert report['rustProbeSourceSha256']==selected_plan['workerSourceHashes']['scripts/product_resource_probe.rs']
            assert report['buildProvenanceSha256']==sha(artifacts/'build-provenance.json')
            provenance=read(artifacts/'build-provenance.json')
            assert provenance['sourceState']=='workingTreeHashPinned'
            assert provenance['coreFiles']==identity['coreFiles']
            assert all(sha(REPO/name)==digest for name,digest in provenance['coreFiles'].items())
            assert len(report['rows'])==len(expected)
            assert {row['id'] for row in report['rows']}==expected
            for name,digest in report['artifactHashes'].items():assert sha(artifacts/name)==digest
            for row in report['rows']:
                assert row['progressSha256']==sha(path.parent/(row['id']+'.progress.log'))
                if row['status']!='measured':missing.append(key+'/'+row['id']);continue
                assert row['outputSha256']==sha(path.parent/(row['id']+'.json'))
                metadata=path.parent/(row['id']+'.json.metadata.json')
                assert row['metadataSha256']==sha(metadata)
                values=read(metadata)
                assert values['resourceMetadataVersion']==1
                if surface=='python' and selected_plan.get('workerAuxiliarySourceHashes'):
                    assert values['serializerSourceSha256']==selected_plan['workerAuxiliarySourceHashes']['scripts/resource_json.py']
                parts=path.parent/(row['id']+'.json.parts')
                case=next(item for item in plan['cases'] if item['id']==row['case'])
                assert values['resultCount']==case['sessions']
                assert values['confirmedBars']==case['history']+case['tail']
                assert {f'live-{stream}.json' for stream in range(case['sessions'])}<=row['spoolOutputSha256'].keys()
                for name,digest in row['spoolOutputSha256'].items():
                    assert sha(parts/name)==digest
                    known_hashes[parts/name]=digest
                if row.get('resourceReportVersion')==2:
                    validate_report(path.parent/(row['id']+'.json'),case['sessions'],
                        case['history']+case['tail'],surface=='rust',case['history'])
                    resource_metadata(path.parent/(row['id']+'.json'),case['sessions'],case['history']+case['tail'])
                    assert set(row['spoolOutputSha256'])==expected_spool_names(case['sessions'],surface=='rust')
                elif report.get('collectorContractVersion')==2:
                    raise AssertionError('current collector row lacks a compact report version')
                if surface=='rust':
                    assert row['historicalSameContextMatches'] is True
                    assert values['historicalSameContextMatches'] is True
            failures.extend(dict(surface=key,failure=item) for item in report['failures'])
            reports[key]=dict(receiptSha256=sha(path),receiptPath=path.relative_to(REPO).as_posix(),
                sourcePlanSha256=sha(selected_root/'plan.json'),retained=selected_root!=root,rows=len(report['rows']),
                measured=sum(row['status']=='measured' for row in report['rows']),
                growth=report['growth'],failures=report['failures'],host=report['host'],artifactHashes=report['artifactHashes'])
    integrity_finished=time.monotonic()
    for case in plan['cases']:
        group=dict(case=case['id'],streams=case['sessions'],outputs=[],mismatches=[],missing=[])
        for stream in range(case['sessions']):
            baseline=None
            for system in ['Windows','Linux']:
                for surface in ['rust','python','wasm']:
                    for repeat in range(plan['repetitions']):
                        tag=case['id']+f'-{repeat}'
                        receipt=receipt_root(REPO,root,plan,system,surface)/system/surface/'results.json'
                        if not receipt.exists():continue
                        row=next(item for item in read(receipt)['rows'] if item['id']==tag)
                        if row['status']!='measured':continue
                        output=receipt.parent/(tag+f'.json.parts/live-{stream}.json')
                        assert output.exists(),output
                        if baseline is None:baseline=output
                        delta=compare_outputs(baseline,output,validated,known_hashes)
                        group['outputs'].append(dict(surface=system+'/'+surface,repeat=repeat,stream=stream,sha256=sha(output)))
                        if delta:group['mismatches'].append(dict(surface=system+'/'+surface,repeat=repeat,stream=stream,count=len(delta),examples=delta[:10]))
                        if case['sessions']==4 and stream==0:
                            single=case['id'].rsplit('-',1)[0]+'-1'
                            solo=receipt.parent/(single+f'-{repeat}.json.parts/live-0.json')
                            if solo.exists():
                                delta=compare_outputs(solo,output,validated,known_hashes)
                                isolation.append(dict(case=case['id'],surface=system+'/'+surface,repeat=repeat,mismatches=len(delta)))
            if baseline is None:group['missing'].append(stream)
            del baseline
        comparisons.append(group)
        print(case['id'],'mismatch groups',len(group['mismatches']),flush=True)
    mismatches=sum(len(group['mismatches']) for group in comparisons)
    isolation_mismatches=sum(row['mismatches'] for row in isolation)
    result=dict(schemaVersion=2,status='notPassed' if failures or missing or mismatches or isolation_mismatches else 'passed',
        sourceState=plan['sourceState'],planSha256=sha(root/'plan.json'),budgetSha256=sha(frozen),
        reports=reports,receiptReuseProof=reuse_proof,
        retainedTrials=sum(value['rows'] for value in reports.values() if value['retained']),
        freshTrials=sum(value['rows'] for value in reports.values() if not value['retained']),budgetAndExecutionFailures=failures,unverified=missing,
        crossSurfaceComparisons=comparisons,crossSurfaceMismatchGroups=mismatches,sessionZeroIsolation=isolation,
        historicalContext='Known-dataset steps versus batch must match; seed-then-discovery controls remain separate and retained',
        auditTimingsMs=dict(integrity=(integrity_finished-audit_started)*1000,
            comparisons=(time.monotonic()-integrity_finished)*1000),
        qualificationBoundary='Finite frozen synthetic offline workload; working-tree hash-pinned candidate; no release or indefinite-session claim',
        auditToolSha256=sha(Path(__file__)))
    write(root/'summary.json',result)
    return result


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('root',type=Path)
    args=parser.parse_args();print(audit(args.root.resolve())['status'])

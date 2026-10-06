import json
import contextlib
import io
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch
import sys

sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import audit_resource_matrix as resource


class ResourceMatrixAuditTests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory()
        self.repo=Path(self.temp.name);self.root=self.repo/'matrix';self.root.mkdir()
        docs=self.repo/'docs';docs.mkdir()
        original=resource.REPO/'docs/PRODUCT_STANDARD_CANDLE_RESOURCE_PLAN.json'
        frozen=docs/original.name;frozen.write_bytes(original.read_bytes())
        budgets=resource.read(frozen)
        identity=self.repo/'identity.json'
        resource.write(identity,dict(coreFiles={name:'unused' for name in ['Cargo.toml','Cargo.lock','crates/pine-runtime/src/lib.rs']}))
        payload=self.root/'payload.json';payload.write_text('{}')
        self.plan=dict(schemaVersion=2,budgets=budgets,budgetSha256=resource.sha(frozen),
            sourceIdentityFile='identity.json',sourceIdentitySha256=resource.sha(identity),
            sourceState='workingTreeHashPinned',artifactPaths={'Windows':'windows','Linux':'linux'},cases=[],repetitions=2)
        for script in ['rsi-default','rsi-alternate','pivot-original']:
            for size in budgets['historyCases']:
                for count in budgets['independentSessionCounts']:
                    self.plan['cases'].append(dict(id=f'{script}-{size["historyBars"]}-{count}',
                        history=size['historyBars'],tail=size['tailBars'],sessions=count,
                        payload=payload.name,payloadSha256=resource.sha(payload)))
        self.mock=patch.object(resource,'REPO',self.repo);self.mock.start()

    def tearDown(self):
        self.mock.stop();self.temp.cleanup()

    def audit(self):
        resource.write(self.root/'plan.json',self.plan)
        with contextlib.redirect_stdout(io.StringIO()):
            return resource.audit(self.root)

    def complete_manifest_matrix(self):
        """Small synthetic receipts exercise the real auditor, not runtime budgets."""
        self.plan['coreCommit']='a'*40
        core={}
        for name in ['Cargo.toml','Cargo.lock','crates/pine-runtime/src/lib.rs']:
            path=self.repo/name;path.parent.mkdir(parents=True,exist_ok=True)
            path.write_text('fixture core '+name,encoding='utf-8');core[name]=resource.sha(path)
        resource.write(self.repo/'identity.json',dict(coreFiles=core))
        self.plan['sourceIdentitySha256']=resource.sha(self.repo/'identity.json')
        worker_names=['scripts/product_resource_acceptance.py','scripts/product_resource_wasm.cjs',
                      'scripts/product_resource_probe.rs']
        helper_names=['scripts/resource_json.py','scripts/resource_report.py']
        hashes={}
        for name in worker_names+helper_names:
            path=self.repo/name;path.parent.mkdir(parents=True,exist_ok=True)
            path.write_text('fixture worker '+name,encoding='utf-8');hashes[name]=resource.sha(path)
        self.plan['workerSourceHashes']={name:hashes[name] for name in worker_names}
        self.plan['workerAuxiliarySourceHashes']={name:hashes[name] for name in helper_names}
        resource.write(self.root/'plan.json',self.plan)
        plan_hash=resource.sha(self.root/'plan.json')
        for system in ['Windows','Linux']:
            artifacts=self.repo/self.plan['artifactPaths'][system]
            artifacts.mkdir()
            artifact_hashes={}
            for name in ['resource-probe.exe' if system=='Windows' else 'resource-probe',
                         'wasm/pine_wasm.js','wasm/pine_wasm_bg.wasm','wheels/fixture.whl']:
                path=artifacts/name;path.parent.mkdir(parents=True,exist_ok=True)
                path.write_bytes(('fixture artifact '+system+'/'+name).encode())
                artifact_hashes[name]=resource.sha(path)
            resource.write(artifacts/'build-provenance.json',dict(sourceState='workingTreeHashPinned',
                sourceCommit=self.plan['coreCommit'],profile='release',coreFiles=core,
                resourceProbeSourceSha256=hashes['scripts/product_resource_probe.rs']))
            for surface in ['rust','python','wasm']:
                directory=self.root/system/surface;directory.mkdir(parents=True)
                rows=[]
                for case in self.plan['cases']:
                    for repeat in range(self.plan['repetitions']):
                        tag=case['id']+f'-{repeat}';output=directory/(tag+'.json')
                        parts=output.with_name(output.name+'.parts');parts.mkdir()
                        count=case['sessions'];confirmed=case['history']+case['tail']
                        # Output is intentionally tiny, yet every stream/control has
                        # its own full file, reference and independently checked hash.
                        for stream in range(count):
                            value=dict(schemaVersion=9,diagnostics=[],plots=[dict(id=0,
                                title=case['id'].rsplit('-',1)[0],
                                values=[case['history'],case['tail'],stream])])
                            kinds=['live','replica']+(['batch','incremental','same-context'] if surface=='rust' else [])
                            for kind in kinds:resource.write(parts/f'{kind}-{stream}.json',value)
                        metadata=dict(resourceMetadataVersion=1,resultCount=count,confirmedBars=confirmed,
                            metrics={'compile':[0.01],'snapshot':[0.01]*count,'serialization':[0.01]*count},
                            overheadMetrics={'reportWrite':[0.01]})
                        manifest=dict(resourceReportVersion=2,metadata={'path':output.name+'.metadata.json'},
                            results=[{'path':output.name+f'.parts/live-{stream}.json'} for stream in range(count)],
                            confirmedBars=confirmed)
                        if surface=='rust':
                            metadata.update(historicalAppendMatches=True,historicalSameContextMatches=True)
                            manifest.update(historicalAppendMatches=True,historicalSameContextMatches=True,
                                historicalContexts=[dict(stream=stream,matches=True,sameContextMatches=True,
                                    batchDatasetEnd=confirmed-1,initialSeedDatasetEnd=case['history']-1,
                                    batch={'path':output.name+f'.parts/batch-{stream}.json'},
                                    incremental={'path':output.name+f'.parts/incremental-{stream}.json'},
                                    sameContextIncremental={'path':output.name+f'.parts/same-context-{stream}.json'})
                                    for stream in range(count)])
                        else:
                            metadata['historicalAppend']=manifest['historicalAppend']='not exposed by this binding; separately checked by native probe'
                        if surface=='python':metadata['serializerSourceSha256']=hashes['scripts/resource_json.py']
                        resource.write(output,manifest)
                        sidecar=output.with_name(output.name+'.metadata.json');resource.write(sidecar,metadata)
                        progress=directory/(tag+'.progress.log');progress.write_text('fixture worker completed\n')
                        rows.append(dict(id=tag,case=case['id'],repeat=repeat,status='measured',
                            resourceReportVersion=2,outputSha256=resource.sha(output),
                            metadataSha256=resource.sha(sidecar),progressSha256=resource.sha(progress),
                            historicalSameContextMatches=surface=='rust',
                            spoolOutputSha256={name:resource.sha(parts/name)
                                for name in resource.expected_spool_names(count,surface=='rust')}))
                report=dict(completed=True,collectorContractVersion=2,workerHashBindings=hashes,
                    coreCommit=self.plan['coreCommit'],planSha256=plan_hash,
                    runnerSha256=hashes[worker_names[0]],wasmRunnerSha256=hashes[worker_names[1]],
                    rustProbeSourceSha256=hashes[worker_names[2]],
                    buildProvenanceSha256=resource.sha(artifacts/'build-provenance.json'),
                    artifactHashes=artifact_hashes,rows=rows,failures=[],growth=[],host={'fixture':True})
                resource.write(directory/'results.json',report)

    def test_complete_v2_matrix_and_tampered_receipts(self):
        self.complete_manifest_matrix()
        result=self.audit()
        self.assertEqual(result['status'],'passed')
        self.assertEqual(result['freshTrials'],216)
        self.assertEqual(result['retainedTrials'],0)
        self.assertEqual(len(result['reports']),6)
        self.assertTrue(all(report['measured']==36 for report in result['reports'].values()))
        self.assertEqual(len(result['crossSurfaceComparisons']),18)
        self.assertEqual(sum(len(group['outputs']) for group in result['crossSurfaceComparisons']),540)
        self.assertEqual(len(result['sessionZeroIsolation']),108)
        self.assertEqual(result['crossSurfaceMismatchGroups'],0)
        self.assertEqual(result['unverified'],[])
        directory=self.root/'Windows/rust'
        receipt=directory/'results.json';original_receipt=receipt.read_bytes()
        tag=self.plan['cases'][0]['id']+'-0';output=directory/(tag+'.json')
        original_manifest=output.read_bytes();parts=output.with_name(output.name+'.parts')
        live=parts/'live-0.json';original_live=live.read_bytes()
        control=parts/'same-context-0.json';original_control=control.read_bytes()

        for mutation in ['manifestReference','controlEndpoint','outputBytes','missingSpool','rehashMismatch']:
            with self.subTest(mutation=mutation):
                try:
                    report=resource.read(receipt);row=next(value for value in report['rows'] if value['id']==tag)
                    if mutation in ('manifestReference','controlEndpoint'):
                        manifest=resource.read(output)
                        if mutation=='manifestReference':
                            manifest['results'][0]={'path':output.name+'.parts/replica-0.json'}
                        else:manifest['historicalContexts'][0]['initialSeedDatasetEnd']+=1
                        resource.write(output,manifest)
                        # Rehash deliberately: validation must reject the changed
                        # contract, rather than stopping at a stale receipt digest.
                        row['outputSha256']=resource.sha(output);resource.write(receipt,report)
                    elif mutation=='outputBytes':live.write_bytes(original_live+b' ')
                    elif mutation=='missingSpool':control.unlink()
                    else:
                        changed=resource.read(live);changed['plots'][0]['values'][-1]=999
                        resource.write(live,changed)
                        row['spoolOutputSha256'][live.name]=resource.sha(live);resource.write(receipt,report)
                    if mutation=='rehashMismatch':
                        rejected=self.audit()
                        self.assertEqual(rejected['status'],'notPassed')
                        self.assertGreater(rejected['crossSurfaceMismatchGroups'],0)
                    else:
                        with self.assertRaises((AssertionError,FileNotFoundError)):self.audit()
                finally:
                    receipt.write_bytes(original_receipt);output.write_bytes(original_manifest)
                    live.write_bytes(original_live);control.write_bytes(original_control)
        self.assertEqual(self.audit()['status'],'passed')

    def test_missing_surface_receipts_cannot_pass(self):
        result=self.audit()
        self.assertEqual(result['status'],'notPassed')
        self.assertEqual(len(result['unverified']),6)

    def test_empty_or_duplicate_case_plan_is_rejected(self):
        cases=self.plan['cases']
        for invalid in [[],cases+[cases[0]]]:
            self.plan['cases']=invalid
            with self.assertRaises(AssertionError):self.audit()

    def test_changed_frozen_budget_is_rejected(self):
        self.plan['budgets']['budgets']['peakRssBytesPerSingleSessionProcess']*=10
        with self.assertRaises(AssertionError):self.audit()

    def test_reduced_repetition_count_is_rejected(self):
        self.plan['repetitions']=1
        with self.assertRaises(AssertionError):self.audit()

    def test_incomplete_receipt_is_rejected(self):
        path=self.root/'Windows/rust/results.json';path.parent.mkdir(parents=True)
        resource.write(path,dict(completed=False))
        with self.assertRaises(AssertionError):self.audit()

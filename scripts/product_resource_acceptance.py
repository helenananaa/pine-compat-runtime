"""Finite offline full-script resource acceptance; frozen budgets never change."""
import argparse
import json
import math
import os
import platform
import statistics
import subprocess
import sys
import threading
import time
from pathlib import Path

from requalify_core_scripts import read, write, sha, differences

REPO = Path(__file__).resolve().parents[1]
MINUTE = 60000
DAY = 1440 * MINUTE


def aggregate(bars):
    return dict(time=bars[0]['time']//DAY*DAY, open=bars[0]['open'],
                high=max(b['high'] for b in bars), low=min(b['low'] for b in bars),
                close=bars[-1]['close'], volume=sum(b['volume'] for b in bars))


def prepare(root):
    root.mkdir(exist_ok=False)
    budget = read(REPO/'docs/PRODUCT_STANDARD_CANDLE_RESOURCE_PLAN.json')
    cases = []
    source_root = REPO/'.local/product-completion-20260912/corpus'
    for name, source, overrides in [
        ('rsi-default', 'relative-strength-index.pine', {}),
        ('rsi-alternate', 'relative-strength-index.pine', {'23':'SMA + Bollinger Bands','3':True}),
        ('pivot-original', 'pivot-points-standard.pine', {}),
    ]:
        for size in budget['historyCases']:
            n, tail = size['historyBars'], size['tailBars']
            start = 1767225600000 - (n % 1440)*MINUTE
            bars = []
            for i in range(n+tail):
                o = 100+8*math.sin(i/17)+2*math.sin(i/113)
                c = 100+8*math.sin((i+1)/17)+2*math.sin((i+1)/113)
                bars.append(dict(time=start+i*MINUTE, open=o, high=max(o,c)+1,
                                 low=min(o,c)-1, close=c, volume=100+i%31))
            seed = bars[:n]
            daily = []
            if name.startswith('pivot'):
                for j in range(-32,0):
                    o=100+8*math.sin(j/7);c=100+8*math.sin((j+1)/7)
                    daily.append(dict(time=start//DAY*DAY+j*DAY,open=o,high=max(o,c)+1,low=min(o,c)-1,close=c,volume=1440))
            group = []
            for b in seed:
                if group and group[0]['time']//DAY != b['time']//DAY:
                    daily.append(aggregate(group)); group=[]
                group.append(b)
            daily.append(aggregate(group))
            assert (seed[-1]['time']+MINUTE)%DAY == 0
            events=[];group=[]
            for b in bars[n:]:
                for k, price in enumerate([b['open'], b['high']]):
                    partial = {**b,'close':price}
                    if name.startswith('pivot'):
                        events.append(dict(kind='request_forming', bar=aggregate(group+[partial]), phase='request'))
                    events.append(dict(kind='forming',bar=partial,phase='forming' if k==0 else 'replacement'))
                group.append(b)
                if (b['time']+MINUTE)%DAY == 0:
                    if name.startswith('pivot'):
                        events.append(dict(kind='request_confirmed',bar=aggregate(group),phase='request'))
                    group=[]
                events.append(dict(kind='confirmed',bar=b,phase='confirmation'))
            payload={'source':(source_root/source).read_text(encoding='utf-8-sig'),
                     'bars':seed, 'tail':bars[n:], 'events':events, 'overrides':overrides,
                     'request':{'$chart':{'symbol':'OFFLINE:RESOURCE','timeframe':'1','minMove':1,'priceScale':1000},
                                **({'OFFLINE:RESOURCE:1D':daily} if name.startswith('pivot') else {})}}
            filename=f'{name}-{n}.json';write(root/filename,payload)
            for sessions in budget['independentSessionCounts']:
                cases.append(dict(id=f'{name}-{n}-{sessions}',script=name,payload=filename,
                                  payloadSha256=sha(root/filename),history=n,tail=tail,sessions=sessions))
    plan=dict(budgets=budget,budgetSha256=sha(REPO/'docs/PRODUCT_STANDARD_CANDLE_RESOURCE_PLAN.json'),
              coreCommit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=REPO,text=True).strip(),
              generatorSha256=sha(Path(__file__)),cases=cases,repetitions=2,
              processTimeoutSeconds=600,providerWarmupDays=32,scope='untrimmed outputs; interleaved independent sessions; synthetic minute input')
    write(root/'plan.json',plan)


def shifted(bar, offset):
    delta=offset*((bar['time']//60000)%17-8)
    return {k:v+delta if k in ('open','high','low','close') else v for k,v in bar.items()}


def shifted_request(request, offset):
    return {k:v if k=='$chart' else [shifted(b,offset) for b in v] for k,v in request.items()}


def timed(metrics, key, fn):
    start=time.perf_counter_ns();value=fn()
    metrics.setdefault(key,[]).append((time.perf_counter_ns()-start)/1e6)
    return value


def python_worker(payload, count, output):
    import pine_compat as pine
    p=read(payload);metrics={};program=timed(metrics,'compile',lambda:pine.compile_script(p['source']))
    sessions=[];replicas=[]
    for i in range(count):
        request=shifted_request(p['request'],i*0.125);chart=request.pop('$chart')
        request['$chart']={k:v for k,v in chart.items() if k not in ('symbol','timeframe')}
        session=program.realtime_session(request_bars=request,input_overrides={int(k):v for k,v in p['overrides'].items()},chart_symbol=chart['symbol'],chart_timeframe=chart['timeframe'])
        timed(metrics,'seed',lambda:session.seed([shifted(b,i*0.125) for b in p['bars']]))
        sessions.append(session);replicas.append(session.replica())
    for index,event in enumerate(p['events']):
        for i,(session,replica) in enumerate(zip(sessions,replicas)):
            before=session.confirmed_bars;b=shifted(event['bar'],i*0.125)
            if event['kind'].startswith('request'):
                fn=lambda:getattr(session,'apply_'+event['kind'])('OFFLINE:RESOURCE','1D',b)
            else:fn=lambda:getattr(session,'apply_'+event['kind'])(b)
            changes=timed(metrics,event['phase'],fn)
            if changes is not None:timed(metrics,'replica',lambda:replica.apply(changes))
            assert session.confirmed_bars==before+(event['kind']=='confirmed')
        if index%1024==0:print(f'events {index}/{len(p["events"])}',file=sys.stderr,flush=True)
    results=[]
    for session,replica in zip(sessions,replicas):
        result=timed(metrics,'snapshot',session.result)
        assert not differences(result,replica.result())
        assert not result['diagnostics'] and not (result.get('strategy') or {}).get('diagnostics')
        timed(metrics,'serialization',lambda:json.dumps(result,allow_nan=False))
        assert session.confirmed_bars==len(p['bars'])+len(p['tail'])
        results.append(result)
    if count>1:assert all(differences(results[0],r) for r in results[1:]),'independent streams must produce distinct outputs'
    write(output,dict(metrics=metrics,results=results,confirmedBars=len(p['bars'])+len(p['tail']),
                      historicalAppend='not exposed by this binding; separately checked by native probe'))


def sample_peak(process, box, stop):
    if sys.platform=='win32':
        import psutil
        # The Windows venv launcher spawns the actual interpreter. Track every
        # descendant and retain each OS high-water mark after process exit.
        peaks={};tracked={process.pid:psutil.Process(process.pid)}
        while not stop.is_set():
            for pid, item in list(tracked.items()):
                try:
                    for descendant in item.children(recursive=True):
                        tracked.setdefault(descendant.pid,descendant)
                    peaks[pid]=max(peaks.get(pid,0),item.memory_info().peak_wset)
                except (psutil.NoSuchProcess,psutil.AccessDenied):pass
            box[0]=max(box[0],sum(peaks.values()))
            stop.wait(.02)
    else:
        while not stop.is_set():
            try:
                for line in Path(f'/proc/{process.pid}/status').read_text().splitlines():
                    if line.startswith('VmHWM:'):box[0]=max(box[0],int(line.split()[1])*1024)
            except (FileNotFoundError,ProcessLookupError):pass
            stop.wait(.02)


def measure(root, artifacts, surface, resume=False):
    plan=read(root/'plan.json');outdir=root/platform.system()/surface;outdir.mkdir(parents=True,exist_ok=True)
    provenance=read(artifacts/'build-provenance.json')
    assert provenance['sourceCommit']==plan['coreCommit'] and provenance['profile']=='release'
    assert all(sha(REPO/p)==h for p,h in provenance['coreFiles'].items()),'core differs from built artifact source'
    initial_tool_hashes={p:sha(REPO/p) for p in ['scripts/product_resource_acceptance.py','scripts/product_resource_wasm.cjs','scripts/product_resource_probe.rs']}
    receipt=outdir/'results.json'
    report=read(receipt) if resume and receipt.exists() else dict(platform=platform.platform(),processor=platform.processor(),surface=surface,coreCommit=plan['coreCommit'],planSha256=sha(root/'plan.json'),rows=[],failures=[])
    assert report['planSha256']==sha(root/'plan.json')
    budgets=plan['budgets']['budgets']
    for case in plan['cases']:
        for repeat in range(plan['repetitions']):
            tag=case['id']+f'-{repeat}'
            if any(r['id']==tag for r in report['rows']):continue
            payload=root/case['payload'];assert sha(payload)==case['payloadSha256']
            output=outdir/(tag+'.json');progress=outdir/(tag+'.progress.log')
            if surface=='python':
                py=artifacts/('venv/Scripts/python.exe' if os.name=='nt' else 'venv/bin/python')
                command=[str(py),str(Path(__file__).resolve()),'--python-worker',str(payload),'--sessions',str(case['sessions']),'--output',str(output)]
            elif surface=='wasm':command=['node',str(REPO/'scripts/product_resource_wasm.cjs'),str(artifacts/'wasm/pine_wasm.js'),str(payload),str(case['sessions']),str(output)]
            else:command=[str(artifacts/('resource-probe.exe' if os.name=='nt' else 'resource-probe')),str(payload),str(case['sessions']),str(output)]
            row=dict(id=tag,case=case['id'],repeat=repeat,status='failed',command=command);report['rows'].append(row)
            start=time.monotonic();box=[0];stop=threading.Event()
            try:
                with progress.open('wb') as log:
                    child=subprocess.Popen(command,cwd=REPO,stdout=log,stderr=log)
                    sampler=threading.Thread(target=sample_peak,args=(child,box,stop));sampler.start()
                    try:code=child.wait(timeout=plan['processTimeoutSeconds'])
                    except subprocess.TimeoutExpired:
                        if os.name=='nt':
                            import psutil
                            try:
                                for descendant in psutil.Process(child.pid).children(recursive=True):
                                    try:descendant.kill()
                                    except psutil.NoSuchProcess:pass
                            except psutil.NoSuchProcess:pass
                        child.kill();child.wait();raise RuntimeError('frozen process timeout exceeded')
                    finally:stop.set();sampler.join()
                assert code==0,f'worker exit {code}; see {progress.name}'
                value=read(output);samples=value['metrics'];issues=[]
                counts={k:sum(e['phase']==k for e in read(payload)['events'])*case['sessions'] for k in ('forming','replacement','confirmation','request')}
                counts.update(compile=1,seed=case['sessions'],snapshot=case['sessions'],serialization=case['sessions'])
                counts['replica']=(case['tail']*3+(counts['request']//case['sessions']-case['tail'] if case['script'].startswith('pivot') else 0))*case['sessions']
                if surface=='rust':counts['append']=case['tail']*case['sessions']
                for key,count in counts.items():assert len(samples.get(key,[]))==count,(key,count)
                stats={k:dict(count=len(v),p50Ms=statistics.median(v),p95Ms=sorted(v)[math.ceil(.95*len(v))-1],maxMs=max(v)) for k,v in samples.items() if v}
                for key,budget in [('append','appendP95Ms'),('forming','formingP95Ms'),('replacement','replacementP95Ms'),('confirmation','confirmationP95Ms'),('request','requestUpdateP95Ms'),('replica','replicaApplyP95Ms')]:
                    if key in stats and stats[key]['p95Ms']>budgets[budget]:issues.append(f'{key} p95 exceeds {budgets[budget]} ms')
                for key,limit in [('compile',budgets['compileMaxSeconds']*1000),('seed',budgets['historicalSeedMaxSecondsPerSession']*1000)]:
                    if stats[key]['maxMs']>limit:issues.append(f'{key} exceeds budget')
                if any(a+b>budgets['fullSnapshotAndSerializationMaxSecondsPerSession']*1000 for a,b in zip(samples['snapshot'],samples['serialization'])):issues.append('snapshot and serialization exceeds budget')
                memory=budgets['peakRssBytesPerSingleSessionProcess' if case['sessions']==1 else 'peakRssBytesPerFourSessionProcess']
                if box[0]==0 or box[0]>memory:issues.append('peak RSS unavailable or exceeds budget')
                if repeat==1:
                    prior=read(outdir/(case['id']+'-0.json'));assert not differences(prior['results'],value['results']),'repeat output differs'
                row.update(status='measured',metrics=stats,peakRssBytes=box[0],budgetFailures=issues,outputSha256=sha(output),historicalAppendMatches=value.get('historicalAppendMatches'))
                if issues:report['failures'].append(dict(id=tag,errors=issues))
            except Exception as exc:
                row['error']=str(exc);report['failures'].append(dict(id=tag,errors=[str(exc)]))
            row['wallSeconds']=time.monotonic()-start;row['peakRssBytes']=box[0]
            row['progressSha256']=sha(progress)
            write(receipt,report);print(surface,tag,row['status'],row.get('budgetFailures',row.get('error')),flush=True)
    report['completed']=True
    assert initial_tool_hashes=={p:sha(REPO/p) for p in initial_tool_hashes},'benchmark tools changed during measurement'
    assert all(sha(REPO/p)==h for p,h in provenance['coreFiles'].items()),'core changed during measurement'
    report['buildProvenanceSha256']=sha(artifacts/'build-provenance.json')
    report['host']={'uname':list(platform.uname()),'python':sys.version,'memoryCounters':'windowsSumOfProcessTreePeakWorkingSetUpperBound' if os.name=='nt' else 'linuxVmHWM','samplingIntervalMs':20,'note':'process-wide peaks include input conversion and verification; not per-runtime retained allocation'}
    growth=[]
    for name in {c['script'] for c in plan['cases']}:
        for sessions in (1,4):
            small=[r for r in report['rows'] if r['case']==f'{name}-1024-{sessions}' and r['status']=='measured']
            large=[r for r in report['rows'] if r['case']==f'{name}-100000-{sessions}' and r['status']=='measured']
            if len(small)!=2 or len(large)!=2:
                growth.append(dict(script=name,sessions=sessions,status='unverified'));continue
            for phase in ('append','forming','replacement','confirmation','request','replica'):
                if phase not in small[0]['metrics'] or phase not in large[0]['metrics']:continue
                ratio=statistics.median(r['metrics'][phase]['p50Ms'] for r in large)/max(statistics.median(r['metrics'][phase]['p50Ms'] for r in small),1e-12)
                growth.append(dict(script=name,sessions=sessions,phase=phase,ratio=ratio))
                if ratio>budgets['maxP50TailGrowthRatio1024To100000']:report['failures'].append(dict(script=name,sessions=sessions,error=f'{phase} median growth {ratio} exceeds frozen budget'))
    report['growth']=growth
    report['qualification']='notPassed' if report['failures'] else 'livePhasesPassed'
    report['historicalAppendQualification']='Requires native per-script append/batch evidence; not inferred from live confirmation'
    report['artifactHashes']={p.relative_to(artifacts).as_posix():sha(p) for p in [artifacts/'wasm/pine_wasm.js',artifacts/'wasm/pine_wasm_bg.wasm',*sorted((artifacts/'wheels').glob('*.whl')),artifacts/('resource-probe.exe' if os.name=='nt' else 'resource-probe')] if p.exists()}
    report['runnerSha256']=sha(Path(__file__));report['wasmRunnerSha256']=sha(REPO/'scripts/product_resource_wasm.cjs');report['rustProbeSourceSha256']=sha(REPO/'scripts/product_resource_probe.rs');write(receipt,report)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--prepare',type=Path);parser.add_argument('--root',type=Path);parser.add_argument('--artifacts',type=Path)
    parser.add_argument('--surface',choices=['python','wasm','rust']);parser.add_argument('--resume',action='store_true')
    parser.add_argument('--python-worker',type=Path);parser.add_argument('--sessions',type=int);parser.add_argument('--output',type=Path)
    a=parser.parse_args()
    if a.prepare:prepare(a.prepare.resolve())
    elif a.python_worker:python_worker(a.python_worker,a.sessions,a.output)
    else:measure(a.root.resolve(),a.artifacts.resolve(),a.surface,a.resume)


if __name__=='__main__':main()

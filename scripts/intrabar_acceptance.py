"""Offline, host-neutral event replay. Synthetic inputs are not native tick evidence."""
import argparse
import hashlib
import json
import math
import subprocess
import sys
from pathlib import Path

from requalify_core_scripts import differences, read, sha, write
from requalify_core_scripts import run

REPO=Path(__file__).resolve().parents[1]
STEP=4*3600*1000
DAY=6*STEP

STATE_SOURCE='''//@version=6
indicator("State and daily request control")
var int ordinary = 0
varip int persistent = 0
ordinary += 1
persistent += 1
plot(ordinary, title="ordinary")
plot(persistent, title="persistent")
plot(request.security(syminfo.tickerid, "1D", close), title="daily")
'''
BROKER_SOURCE='''//@version=6
strategy("Intrabar broker control", calc_on_every_tick=true, calc_on_order_fills=true, initial_capital=10000)
varip int executions = 0
executions += 1
if barstate.isrealtime and strategy.position_size == 0 and barstate.isnew
    strategy.entry("L", strategy.long, qty=4)
if strategy.position_size == 4
    strategy.exit("partial", "L", qty=2, limit=strategy.position_avg_price+1)
    strategy.exit("rest", "L", qty=2, stop=strategy.position_avg_price-1)
if strategy.position_size == 2
    strategy.close("L", immediately=true)
plot(executions, title="executions")
plot(strategy.position_size, title="position")
plot(request.security(syminfo.tickerid, "1D", close), title="daily")
'''

def aggregate(bars):
    return {'time':bars[0]['time']//DAY*DAY,'open':bars[0]['open'],'high':max(b['high'] for b in bars),
            'low':min(b['low'] for b in bars),'close':bars[-1]['close'],'volume':sum(b['volume'] for b in bars)}


def prepare(root):
    provenance=read(REPO/'.local/core-requalification-20260930/build-provenance.json')
    assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=REPO,text=True).strip()==provenance['sourceCommit']
    for p,h in provenance['coreFiles'].items(): assert sha(REPO/p)==h,p
    for a in provenance['artifacts']: assert sha(Path(a['path']))==a['sha256']
    source=REPO/'.local/core-requalification-20260930/ssl-kijun/ssl-hybrid-original.pine'
    ssl=source.read_text(encoding='utf-8-sig')
    cli=REPO/'.local/core-requalification-20260930/pine-compat.exe'
    analysis=json.loads(subprocess.check_output([str(cli),'analyze',str(source),'--format','json']))
    selected={'SSL1 / Baseline Type':'EMA','SSL1 / Baseline Length':5,'Use Current Chart Resolution?':False,
              'Use Different Timeframe? Uncheck Box Above':'1D','Moving Average Length - LookBack Period':3,
              'Optional 2nd Moving Average':True,'Moving Average Length - Optional 2nd MA':5,
              'Use CF Ultimate MA MTF':True,'SL ATR Multiplier':0.2,'TP1 ATR Multiplier':0.2,
              'TP2 ATR Multiplier':0.3,'TP3 ATR Multiplier':0.4,'TP4 ATR Multiplier':0.5,'TP5 ATR Multiplier':0.6}
    overrides={}
    for title,value in selected.items():
        found=[i for i in analysis['inputs'] if i['title']==title]
        if len(found)!=1: raise ValueError((title,[i['title'] for i in analysis['inputs'] if i['title'] and 'Baseline' in i['title']]))
        overrides[str(found[0]['callSiteId'])]=value
    all_bars=[]; start=1767225600000
    for i in range(204):
        o=100+12*math.sin(i/7);c=100+12*math.sin((i+1)/7)
        all_bars.append({'time':start+i*STEP,'open':o,'high':max(o,c)+2,'low':min(o,c)-2,'close':c,'volume':1000+i})
    seed=all_bars[:192]; daily=[aggregate(seed[i:i+6]) for i in range(0,192,6)]
    cases=[]
    for ordering in ['high-first-provider-before','low-first-provider-after']:
        events=[]; daybars=[]
        for j,bar in enumerate(all_bars[192:]):
            o=bar['open']; upper=o+8;lower=o-8
            prices=[o,upper,lower,bar['close']] if ordering.startswith('high') else [o,lower,upper,bar['close']]
            high=low=o
            for k,price in enumerate(prices):
                high=max(high,price);low=min(low,price)
                partial={**bar,'high':high,'low':low,'close':price,'volume':bar['volume']*(k+1)/4}
                provider=aggregate(daybars+[partial])
                request={'kind':'request_forming','bar':provider}
                chart={'kind':'forming','bar':partial,'context':{'execution_time':bar['time']+k*STEP//4,'opening_update':k==0}}
                events.extend([request,chart] if ordering.startswith('high') else [chart,request])
                if k==2:
                    events.append({**chart,'context':{**chart['context'],'opening_update':False}})
            final={**bar,'high':high,'low':low};daybars.append(final)
            confirmation={'kind':'confirmed','bar':final,'context':{'execution_time':bar['time']+STEP-1,'opening_update':False}}
            if j%6==5:
                provider_confirm={'kind':'request_confirmed','bar':aggregate(daybars)}
                events.extend([provider_confirm,confirmation] if ordering.startswith('high') else [confirmation,provider_confirm])
                daybars=[]
            else: events.append(confirmation)
        for name,src,inputs in [('ssl',ssl,overrides),('state',STATE_SOURCE,{}),('broker',BROKER_SOURCE,{})]:
            payload={'source':src,'bars':seed,'request':{'$chart':{'symbol':'OFFLINE:TEST','timeframe':'240','minMove':1,'priceScale':1000,'quantityPrecision':6,'pointValue':1},'OFFLINE:TEST:1D':daily},'overrides':inputs,'symbol':'OFFLINE:TEST','requestTimeframe':'1D','events':events}
            filename=f'{name}-{ordering}.json';write(root/filename,payload)
            cases.append({'id':filename[:-5],'payload':filename,'sha256':sha(root/filename),'script':name,'events':len(events)})
    plan={'coreCommit':provenance['sourceCommit'],'sourceSha256':sha(source),'artifacts':provenance['artifacts'],
          'kind':'synthetic offline acceptance','nativeTickQualified':False,'cases':cases,'seedBars':192,'tailBars':12}
    write(root/'plan.json',plan)
    return plan


def python_replay(payload,out):
    import pine_compat
    p=read(payload);chart=p['request']['$chart']
    request={k:v for k,v in p['request'].items() if k!='$chart'}
    request['$chart']={k:v for k,v in chart.items() if k not in ['symbol','timeframe']}
    session=pine_compat.compile_script(p['source']).realtime_session(request_bars=request,input_overrides={int(k):v for k,v in p['overrides'].items()},chart_symbol=chart['symbol'],chart_timeframe=chart['timeframe'])
    session.seed(p['bars']);replica=session.replica();trace=[]
    for e in p['events']:
        before=session.confirmed_bars
        if e['kind'].startswith('request'):
            changes=getattr(session,'apply_'+e['kind'])(p['symbol'],p['requestTimeframe'],e['bar'])
        else:
            changes=getattr(session,'apply_'+e['kind'])(e['bar'],**e['context'])
        if changes is not None: replica.apply(changes)
        result=session.result()
        assert not differences(result,replica.result())
        assert session.confirmed_bars==before+(e['kind']=='confirmed')
        assert not result['diagnostics'] and not (result.get('strategy') or {}).get('diagnostics'), (len(trace), e['kind'], result['diagnostics'], (result.get('strategy') or {}).get('diagnostics'))
        trace.append({'result':result,'confirmed':session.confirmed_result(),'confirmedBars':session.confirmed_bars})
    write(out,trace)


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('root',type=Path);parser.add_argument('--python-replay',type=Path);parser.add_argument('--output',type=Path);parser.add_argument('--artifacts',type=Path);parser.add_argument('--generate-probe',action='store_true');args=parser.parse_args()
    if args.python_replay: python_replay(args.python_replay,args.output);return
    root=args.root.resolve()
    if args.generate_probe:
        generate_probe(root);return
    if args.artifacts:
        qualify(root,args.artifacts.resolve());return
    root.mkdir(exist_ok=False)
    plan=prepare(root); print('prepared',len(plan['cases']),'cases',flush=True)


def generate_probe(root):
    original=(REPO/'scripts/core_script_probe.rs').read_text()
    original=original.replace('Bar, ChartContext, HistoricalRuntime,','Bar, BarUpdate, ChartContext, RealtimeRuntime, RealtimeUpdateContext,')
    start=original.index('    let mut runtime =')
    original=original[:start]+'''    let mut runtime = RealtimeRuntime::with_request_environment_and_input_overrides(&hir, env, overrides);
    runtime.seed_historical(&bars(&p["bars"])).map_err(|e| e.message)?;
    let mut replica = runtime.replica();
    let mut trace = Vec::new();
    for e in p["events"].as_array().unwrap() {
        let kind = e["kind"].as_str().unwrap();
        let b = bars(&serde_json::json!([e["bar"].clone()]))[0];
        let before = runtime.confirmed_bar_count();
        let changes = if kind.starts_with("request") {
            runtime.apply_request_update(RequestKey::new(p["symbol"].as_str().unwrap(), RequestTimeframe::parse(p["requestTimeframe"].as_str().unwrap()).map_err(|e|e.to_string())?),
                if kind == "request_forming" { BarUpdate::forming(b) } else { BarUpdate::confirmed(b) }).map_err(|e|e.message)?
        } else {
            Some(runtime.apply_update_with_context(if kind == "forming" { BarUpdate::forming(b) } else { BarUpdate::confirmed(b) },
                RealtimeUpdateContext { execution_time: e["context"]["execution_time"].as_i64(), opening_update: e["context"]["opening_update"].as_bool() }).map_err(|e|e.message)?)
        };
        if let Some(changes) = changes { replica.apply(&changes).map_err(|e|e.message)?; }
        assert_eq!(runtime.confirmed_bar_count(), before + usize::from(kind == "confirmed"));
        let result = public_runtime_result_json(&runtime.result());
        assert_eq!(result, public_runtime_result_json(&replica.result()));
        trace.push(serde_json::json!({"result":serde_json::from_str::<Value>(&result)?,"confirmed":serde_json::from_str::<Value>(&public_runtime_result_json(&runtime.confirmed_result()))?,"confirmedBars":runtime.confirmed_bar_count()}));
    }
    println!("{}",serde_json::to_string(&trace)?);
    Ok(())
}
'''
    probe=root/'rust-probe';probe.mkdir(exist_ok=True)
    (probe/'main.rs').write_text(original)
    (probe/'Cargo.lock').write_bytes((REPO/'Cargo.lock').read_bytes())
    deps='\n'.join(f'{n}={{path="{(REPO/"crates"/n).as_posix()}"}}' for n in ['pine-runtime','pine-sema','pine-syntax'])
    (probe/'Cargo.toml').write_text('[package]\nname="intrabar-probe"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[[bin]]\nname="intrabar-probe"\npath="main.rs"\n[dependencies]\n'+deps+'\nserde_json="1"\n')


def qualify(root,artifacts):
    plan=read(root/'plan.json')
    patch=subprocess.check_output(['git','diff','--binary','HEAD','--','crates'],cwd=REPO)
    (root/'core.patch').write_bytes(patch)
    source_hashes={p.relative_to(REPO).as_posix():sha(p) for p in (REPO/'crates').rglob('*.rs')}
    results={'baseCommit':plan['coreCommit'],'corePatchSha256':hashlib.sha256(patch).hexdigest(),'sourceHashes':source_hashes,
             'nativeTickQualified':False,'cases':[],'failures':[],
             'artifactHashes':{p.relative_to(artifacts).as_posix():sha(p) for p in [artifacts/'intrabar-probe.exe',artifacts/'wasm/pine_wasm.js',artifacts/'wasm/pine_wasm_bg.wasm',*list((artifacts/'wheels').glob('*.whl'))]},
             'comparisonTolerance':{'absolute':1e-9,'relative':1e-12}}
    for case in plan['cases']:
        payload=root/case['payload']; assert sha(payload)==case['sha256']
        row={'id':case['id'],'events':case['events'],'status':'failed'};results['cases'].append(row)
        try:
            baseline=None;row['outputs']={}
            for repeat in range(2):
                for surface in ['python','wasm','rust']:
                    out=root/f'{case["id"]}-{surface}-{repeat}.trace.json'
                    if surface=='python':
                        cmd=[str(artifacts/'venv/Scripts/python.exe'),str(Path(__file__).resolve()),str(root),'--python-replay',str(payload),'--output',str(out)]
                        run(cmd,root/f'{case["id"]}-{surface}-{repeat}.log')
                    elif surface=='wasm':
                        run(['node',str(REPO/'scripts/intrabar_replay.cjs'),str(artifacts/'wasm/pine_wasm.js'),str(payload),str(out)],root/f'{case["id"]}-{surface}-{repeat}.log')
                    else:
                        run([str(artifacts/'intrabar-probe.exe')],out,payload.read_bytes())
                    trace=read(out);assert len(trace)==case['events']
                    if baseline is None:baseline=trace
                    issues=differences(baseline,trace)
                    assert not issues,issues[:10]
                    row['outputs'][f'{surface}-{repeat}']=sha(out)
            p=read(payload);counts={'forming':0,'confirmed':0,'request_forming':0,'request_confirmed':0}
            for e,t in zip(p['events'],baseline):
                counts[e['kind']]+=1
                assert not t['result']['diagnostics'] and not (t['result'].get('strategy') or {}).get('diagnostics')
            assert baseline[-1]['confirmedBars']==len(p['bars'])+12
            strategy=baseline[-1]['result'].get('strategy')
            row.update({'eventCounts':counts,'finalConfirmedBars':baseline[-1]['confirmedBars'],
                        'finalOrders':len(strategy['orders']) if strategy else 0,'finalClosedTrades':len(strategy['trades']) if strategy else 0,
                        'intrabarOrderChangingUpdates':sum(a['confirmedBars']==b['confirmedBars'] and a['result'].get('strategy') is not None and len(a['result']['strategy']['orders'])!=len(b['result']['strategy']['orders']) for a,b in zip(baseline,baseline[1:]))})
            if strategy:
                row['newClosedTradesAfterFirstEvent']=len(strategy['trades'])-len(baseline[0]['result']['strategy']['trades'])
            if case['script']=='state':
                for i,(event,t) in enumerate(zip(p['events'],baseline)):
                    plots={x['title']:x['values'] for x in t['result']['plots']}
                    assert plots['ordinary'][-1]==t['confirmedBars']+(event['kind']!='confirmed' and len(plots['ordinary'])>t['confirmedBars'])
                assert max(t['result']['plots'][1]['values'][-1]-t['result']['plots'][0]['values'][-1] for t in baseline)>1
            if case['script']=='broker':
                assert row['intrabarOrderChangingUpdates']>0 and row['finalClosedTrades']>0
                executions=[next(p['values'][-1] for p in t['result']['plots'] if p['title']=='executions') for t in baseline]
                row['maxExecutionsBetweenHostEvents']=max(b-a for a,b in zip(executions,executions[1:]))
                assert row['maxExecutionsBetweenHostEvents']>1
            if case['script']=='ssl':assert row['intrabarOrderChangingUpdates']>0 and row['newClosedTradesAfterFirstEvent']>0
            row['status']='passed'
        except Exception as exc:results['failures'].append({'case':case['id'],'error':str(exc)})
        write(root/'results.json',results);print(case['id'],row['status'],flush=True)
    assert source_hashes=={p.relative_to(REPO).as_posix():sha(p) for p in (REPO/'crates').rglob('*.rs')}
    assert patch==subprocess.check_output(['git','diff','--binary','HEAD','--','crates'],cwd=REPO)
    results['completed']=True
    results['toolHashes']={p:sha(REPO/p) for p in ['scripts/intrabar_acceptance.py','scripts/intrabar_replay.cjs','scripts/core_script_probe.rs']}
    write(root/'results.json',results)
    if results['failures']:raise RuntimeError(results['failures'])


if __name__=='__main__':main()

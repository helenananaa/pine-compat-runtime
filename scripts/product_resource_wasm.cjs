// Actual optimized WASM benchmark. Each invocation owns a fresh process.
const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');
const pine = require(path.resolve(process.argv[2]));
const p = JSON.parse(fs.readFileSync(process.argv[3], 'utf8'));
const count = Number(process.argv[4]);
const metrics = {};
function timed(key, fn) {
  const start = performance.now(); const value = fn();
  (metrics[key] ??= []).push(performance.now()-start); return value;
}
function shift(b, i) {
  const d=i*.125*(Math.floor(b.time/60000)%17-8);
  return {...b,open:b.open+d,high:b.high+d,low:b.low+d,close:b.close+d};
}
const columns = ['time','open','high','low','close','volume'];
const csv = bars => columns.join(',')+'\n'+bars.map(b=>columns.map(k=>b[k]).join(',')).join('\n')+'\n';
const program = timed('compile', ()=>pine.compileScript(p.source));
const sessions=[],replicas=[];
for(let i=0;i<count;i++) {
  const request = Object.fromEntries(Object.entries(p.request).map(([k,v])=>[k,k==='$chart'?v:v.map(b=>shift(b,i))]));
  const session=program.realtimeSessionWithRequestBarsAndInputOverrides(JSON.stringify(request),JSON.stringify(p.overrides));
  const input=csv(p.bars.map(b=>shift(b,i)));
  timed('seed',()=>session.seed(input)); sessions.push(session);replicas.push(session.replica());
}
for(let n=0;n<p.events.length;n++) {
  const event=p.events[n];
  for(let i=0;i<count;i++) {
    const session=sessions[i], before=session.confirmedBars, bar=JSON.stringify(shift(event.bar,i));
    const changes=timed(event.phase,()=>event.kind.startsWith('request')
      ? session[event.kind==='request_forming'?'applyRequestForming':'applyRequestConfirmed']('OFFLINE:RESOURCE','1D',bar)
      : session[event.kind==='forming'?'applyForming':'applyConfirmed'](bar));
    if(changes!=='null')timed('replica',()=>replicas[i].apply(changes));
    assert.equal(session.confirmedBars,before+Number(event.kind==='confirmed'));
  }
  if(n%1024===0)process.stderr.write(`events ${n}/${p.events.length}\n`);
}
const results=[];
for(let i=0;i<count;i++) {
  const result=timed('snapshot',()=>JSON.parse(sessions[i].result()));
  assert.deepEqual(result,JSON.parse(replicas[i].result()));
  assert.equal(result.diagnostics.length,0);
  assert.equal((result.strategy?.diagnostics??[]).length,0);
  timed('serialization',()=>JSON.stringify(result));
  assert.equal(sessions[i].confirmedBars,p.bars.length+p.tail.length);
  results.push(result);
}
if(count>1)for(let i=1;i<count;i++)assert.notDeepEqual(results[0],results[i]);
fs.writeFileSync(process.argv[5],JSON.stringify({metrics,results,confirmedBars:p.bars.length+p.tail.length,
  historicalAppend:'not exposed by this binding; separately checked by native probe'}));

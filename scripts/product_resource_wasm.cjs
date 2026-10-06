// Actual optimized WASM benchmark. Each invocation owns a fresh process.
const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');
const pine = require(path.resolve(process.argv[2]));
let p = JSON.parse(fs.readFileSync(process.argv[3], 'utf8'));
const count = Number(process.argv[4]);
const metrics = {};
const overheadMetrics = {};
function timed(key, fn) {
  const start = performance.now(); const value = fn();
  (metrics[key] ??= []).push(performance.now()-start); return value;
}
function timedOverhead(key, fn) {
  const start = performance.now(); const value = fn();
  (overheadMetrics[key] ??= []).push(performance.now()-start); return value;
}
function shift(b, i) {
  const d=i*.125*(Math.floor(b.time/60000)%17-8);
  return {...b,open:b.open+d,high:b.high+d,low:b.low+d,close:b.close+d};
}
const columns = ['time','open','high','low','close','volume'];
const csv = bars => columns.join(',')+'\n'+bars.map(b=>columns.map(k=>b[k]).join(',')).join('\n')+'\n';
// Inspect only diagnostic arrays. The full public String is spooled and audited
// separately, so verification need not materialize a second JavaScript forest.
function valueEnd(source, start) {
  let depth=0, quoted=false, escaped=false;
  for(let i=start;i<source.length;i++) {
    const c=source[i];
    if(quoted) {
      if(escaped)escaped=false;
      else if(c==='\\')escaped=true;
      else if(c==='"') {
        quoted=false;
        if(depth===0)return i+1;
      }
    } else if(c==='"')quoted=true;
    else if(c==='{'||c==='[')depth++;
    else if(c==='}'||c===']') {
      if(depth===0)return i;
      if(--depth===0)return i+1;
    } else if(c===','&&depth===0)return i;
  }
  throw new Error('unterminated public JSON value');
}
function objectFields(source, start, keys) {
  assert.equal(source[start],'{');
  const fields={};let pos=start+1;
  while(source[pos]!=='}') {
    assert.equal(source[pos],'"');
    const keyEnd=valueEnd(source,pos),key=JSON.parse(source.slice(pos,keyEnd));
    assert.equal(source[keyEnd],':');
    const begin=keyEnd+1,end=valueEnd(source,begin);
    if(keys.includes(key)) {assert(!(key in fields));fields[key]=[begin,end];}
    pos=end;
    if(source[pos]===',')pos++;
    else assert.equal(source[pos],'}');
  }
  return fields;
}
function assertDiagnostics(source) {
  const fields=objectFields(source,0,['diagnostics','strategy']);
  function check(range) {
    assert(range,'missing diagnostics');
    const diagnostics=JSON.parse(source.slice(...range));
    assert(Array.isArray(diagnostics));assert.equal(diagnostics.length,0);
  }
  check(fields.diagnostics);
  if(fields.strategy)check(objectFields(source,fields.strategy[0],['diagnostics']).diagnostics);
}
function writeString(file, text) {
  const fd=fs.openSync(file,'w'),buffer=Buffer.allocUnsafe(65536);
  try {
    for(let start=0;start<text.length;) {
      let end=Math.min(start+16384,text.length);
      const last=text.charCodeAt(end-1);
      if(end<text.length&&last>=0xd800&&last<=0xdbff)end--;
      const size=buffer.write(text.slice(start,end),0,buffer.length,'utf8');
      for(let offset=0;offset<size;) {
        const written=fs.writeSync(fd,buffer,offset,size-offset);
        assert(written>0);offset+=written;
      }
      start=end;
    }
  } finally {fs.closeSync(fd);}
}
const program = timed('compile', ()=>pine.compileScript(p.source));
const sessions=[],replicas=[];
for(let i=0;i<count;i++) {
  const request = Object.fromEntries(Object.entries(p.request).map(([k,v])=>[k,k==='$chart'?v:v.map(b=>shift(b,i))]));
  const session=program.realtimeSessionWithRequestBarsAndInputOverrides(JSON.stringify(request),JSON.stringify(p.overrides));
  const input=csv(i===0?p.bars:p.bars.map(b=>shift(b,i)));
  timed('seed',()=>session.seed(input)); sessions.push(session);
  replicas.push(timedOverhead('replicaSnapshot',()=>session.replica()));
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
const parts=process.argv[5]+'.parts';fs.mkdirSync(parts,{recursive:true});
const expectedCount=p.bars.length+p.tail.length;
p=null;
function sameFiles(first,second) {
  const left=fs.openSync(first,'r'),right=fs.openSync(second,'r');
  const a=Buffer.allocUnsafe(65536),b=Buffer.allocUnsafe(65536);
  try {
    while(true) {
      const n=fs.readSync(left,a,0,a.length,null),m=fs.readSync(right,b,0,b.length,null);
      if(n!==m||!a.subarray(0,n).equals(b.subarray(0,m)))return false;
      if(n===0)return true;
    }
  } finally {fs.closeSync(left);fs.closeSync(right);}
}
for(let i=0;i<count;i++) {
  let encoded=timedOverhead('replicaExport',()=>replicas[i].intoResult());
  timedOverhead('replicaWrite',()=>writeString(path.join(parts,`replica-${i}.json`),encoded));
  encoded=null;
  replicas[i]=null;
}
for(let i=0;i<count;i++) {
  let encoded;
  timed('snapshot',()=>{encoded=sessions[i].result();assertDiagnostics(encoded);});
  const file=path.join(parts,`live-${i}.json`);
  timed('serialization',()=>writeString(file,encoded));
  encoded=null;
  timedOverhead('comparison',()=>assert(sameFiles(file,path.join(parts,`replica-${i}.json`)),'complete replica output differs'));
  assert.equal(sessions[i].confirmedBars,expectedCount);
  if(i>0)timedOverhead('comparison',()=>assert(!sameFiles(results[0],file),'independent streams must produce distinct outputs'));
  results.push(file);
}
for(const session of sessions)session.free();
const reportPath=process.argv[5];
const relativePath=file=>path.relative(path.dirname(reportPath),file).split(path.sep).join('/');
const historicalAppend='not exposed by this binding; separately checked by native probe';
timedOverhead('reportWrite',()=>writeString(reportPath,JSON.stringify({resourceReportVersion:2,
  metadata:{path:relativePath(reportPath+'.metadata.json')},
  results:results.map(file=>({path:relativePath(file)})),confirmedBars:expectedCount,historicalAppend})));
writeString(reportPath+'.metadata.json',JSON.stringify({resourceMetadataVersion:1,metrics,overheadMetrics,
  resultCount:count,confirmedBars:expectedCount,
  historicalAppend}));

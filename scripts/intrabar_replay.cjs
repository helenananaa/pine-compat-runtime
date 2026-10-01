const fs = require('node:fs');
const assert = require('node:assert/strict');
const pine = require(require('node:path').resolve(process.argv[2]));
const p = JSON.parse(fs.readFileSync(process.argv[3], 'utf8'));
const session = pine.compileScript(p.source).realtimeSessionWithRequestBarsAndInputOverrides(JSON.stringify(p.request), JSON.stringify(p.overrides));
const fields = ['time','open','high','low','close','volume'];
session.seed(fields.join(',')+'\n'+p.bars.map(b=>fields.map(k=>b[k]).join(',')).join('\n')+'\n');
const replica=session.replica();
const trace=[];
for (const e of p.events) {
  const before=session.confirmedBars;
  let changes;
  if(e.kind.startsWith('request')) {
    changes=JSON.parse(session[e.kind==='request_forming'?'applyRequestForming':'applyRequestConfirmed'](p.symbol,p.requestTimeframe,JSON.stringify(e.bar)));
  } else {
    const context={executionTime:e.context.execution_time,openingUpdate:e.context.opening_update};
    changes=JSON.parse(session[e.kind==='forming'?'applyFormingWithContext':'applyConfirmedWithContext'](JSON.stringify(e.bar),JSON.stringify(context)));
  }
  if(changes!==null) replica.apply(JSON.stringify(changes));
  const result=JSON.parse(session.result());
  assert.deepEqual(JSON.parse(replica.result()),result);
  assert.equal(session.confirmedBars,before+(e.kind==='confirmed'?1:0));
  trace.push({result,confirmed:JSON.parse(session.confirmedResult()),confirmedBars:session.confirmedBars});
}
fs.writeFileSync(process.argv[4],JSON.stringify(trace));

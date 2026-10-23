'use strict';
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');

module.exports = function(modulePath) {
  const pine = require(path.resolve(modulePath));
  const root = path.resolve(__dirname,'../..');
  const cases = JSON.parse(fs.readFileSync(path.join(root,'docs/evidence/strategy-library-20261010/array-persistence-oracles-42.json'),'utf8')).records;
  const bars = [0,1,2].map(i=>({time:i*60000,open:10,high:10,low:10,close:10,volume:1}));
  const csv = rows=>'time,open,high,low,close,volume\n'+rows.map(b=>`${b.time},10,10,10,10,1\n`).join('');
  const values = wire=>JSON.parse(wire).plots.map(p=>p.values.at(-1));
  const forming = (test,t,h)=> {
    switch(test.intrabar_profile) {
      case 'opening':return [t,t,0,1,0,t===1?-1:0];
      case 'late_history':return [t,t,0,Number(t>=2)];
      case 'forming_reassign':return [t,t===1?2:1,t+1,Number(t>=2)];
      case 'transient':return [1,h+t,h+t];
      default:return test.forming_expected;
    }
  };
  for(const test of cases) {
    const raw=fs.readFileSync(path.join(root,test.source));
    assert.equal(crypto.createHash('sha256').update(raw).digest('hex'),test.source_sha256);
    const script=raw.toString('utf8');
    const report=JSON.parse(pine.analyzeScript(script));
    if(test.native_status==='analysis_rejected') {
      assert.equal(report.executable,false,test.name);
      assert.throws(()=>pine.compileScript(script));
      continue;
    }
    assert.equal(report.executable,true,test.name);
    const prepared=pine.compileScript(script);
    try {
      if(test.native_status==='runtime_error') {
        const error=new RegExp(test.runtime_error_pattern);
        assert.throws(()=>pine.runScriptCsv(script,csv(bars)),error,test.name);
        assert.throws(()=>prepared.runCsvWithRequestBars(csv(bars),'{}'),error,test.name);
        const session=prepared.realtimeSession();
        try {assert.throws(()=>session.seed(csv(bars.slice(0,2))),error,test.name);} finally {session.free();}
        continue;
      }
      assert.deepEqual(values(pine.runScriptCsv(script,csv(bars))),test.historical_expected,test.name);
      assert.deepEqual(values(prepared.runCsvWithRequestBars(csv(bars),'{}')),test.historical_expected,test.name);
      for(const history of test.execution_kind==='intrabar'?[1,2]:[2]) {
        const session=prepared.realtimeSession();
        try {
          session.seed(csv(bars.slice(0,history)));
          const replica=session.replica();
          try {
            for(let tick=1;tick<=3;tick++) {
              assert.equal(replica.apply(session.applyForming(JSON.stringify(bars[history]))),true);
              assert.deepEqual(values(session.result()),forming(test,tick,history),test.name);
              assert.equal(replica.result(),session.result(),test.name);
            }
            assert.equal(replica.apply(session.applyConfirmed(JSON.stringify(bars[history]))),true);
            assert.deepEqual(values(session.confirmedResult()),forming(test,4,history),test.name);
            assert.equal(replica.result(),session.confirmedResult(),test.name);
          } finally {replica.free();}
        } finally {session.free();}
      }
    } finally {prepared.free();}
  }
};

'use strict';
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');

module.exports = function(modulePath) {
  const pine = require(path.resolve(modulePath));
  const root = path.resolve(__dirname,'../..');
  const cases = JSON.parse(fs.readFileSync(path.join(root,'docs/evidence/strategy-library-20261010/request-reference-oracles-43.json'),'utf8')).records;
  const bars = [0,1,2].map(i=>({time:i*60000,open:10,high:10,low:10,close:10,volume:1}));
  const csv = rows=>'time,open,high,low,close,volume\n'+rows.map(b=>`${b.time},10,10,10,10,1\n`).join('');
  const values = wire=>JSON.parse(wire).plots.map(p=>p.values.at(-1));
  for(const test of cases) {
    const raw=fs.readFileSync(path.join(root,test.source));
    assert.equal(crypto.createHash('sha256').update(raw).digest('hex'),test.source_sha256);
    const script=raw.toString('utf8');
    if(test.native_status==='analysis_rejected') {
      assert.equal(JSON.parse(pine.analyzeScript(script)).executable,false,test.name);
      assert.throws(()=>pine.compileScript(script));
      continue;
    }
    assert.equal(JSON.parse(pine.analyzeScript(script)).executable,true,test.name);
    const prepared=pine.compileScript(script);
    try {
      assert.deepEqual(values(pine.runScriptCsv(script,csv(bars))),test.historical_expected,test.name);
      assert.deepEqual(values(prepared.runCsvWithRequestBars(csv(bars),'{}')),test.historical_expected,test.name);
      const session=prepared.realtimeSession();
      try {
        session.seed(csv(bars.slice(0,2)));
        const replica=session.replica();
        try {
          for(let tick=1;tick<=3;tick++) {
            assert.equal(replica.apply(session.applyForming(JSON.stringify(bars[2]))),true);
            assert.deepEqual(values(session.result()),test.forming_expected,test.name);
            assert.equal(replica.result(),session.result(),test.name);
          }
          assert.equal(replica.apply(session.applyConfirmed(JSON.stringify(bars[2]))),true);
          assert.deepEqual(values(session.confirmedResult()),test.historical_expected,test.name);
          assert.equal(replica.result(),session.confirmedResult(),test.name);
        } finally {replica.free();}
      } finally {session.free();}
    } finally {prepared.free();}
  }
};

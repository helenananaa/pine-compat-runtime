'use strict';
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');

module.exports = function(modulePath) {
  const pine = require(path.resolve(modulePath));
  const root = path.resolve(__dirname,'../..');
  const cases = JSON.parse(fs.readFileSync(path.join(root,'docs/evidence/strategy-library-20261010/point-reference-oracles-41.json'),'utf8')).records;
  const csv = 'time,open,high,low,close,volume\n0,10,10,10,10,1\n60000,10,10,10,10,1\n120000,10,10,10,10,1\n';
  const seed = csv.slice(0,csv.lastIndexOf('120000'));
  const bar = JSON.stringify({time:120000,open:10,high:10,low:10,close:10,volume:1});
  for(const test of cases) {
    const raw = fs.readFileSync(path.join(root,test.source));
    assert.equal(crypto.createHash('sha256').update(raw).digest('hex'),test.source_sha256);
    const script = raw.toString('utf8');
    const report = JSON.parse(pine.analyzeScript(script));
    if(test.native_status === 'analysis_rejected') {
      assert.equal(report.executable,false,test.name);
      assert.throws(()=>pine.compileScript(script));
      continue;
    }
    assert.equal(report.executable,true,test.name);
    const prepared = pine.compileScript(script);
    try {
      if(test.native_status === 'runtime_error') {
        assert.throws(()=>pine.runScriptCsv(script,csv),/E_CHART_POINT_NA_FIELD/,test.name);
        assert.throws(()=>prepared.runCsvWithRequestBars(csv,'{}'),/E_CHART_POINT_NA_FIELD/,test.name);
        const session=prepared.realtimeSession();
        try {assert.throws(()=>session.seed(seed),/E_CHART_POINT_NA_FIELD/,test.name);} finally {session.free();}
        continue;
      }
      const check = (wire,tick) => {
        let values=JSON.parse(wire).plots.map(p=>p.values.at(-1));
        let expected=test.native_values;
        if(test.execution_kind==='intrabar')expected=test.intrabar_expected.map(v=>v==='tick'?tick:v);
        else if(test.native_marker_suffix){expected=test.native_marker_suffix;values=values.slice(-expected.length);}
        assert.deepEqual(values,expected,test.name);
      };
      if(test.execution_kind==='static') {
        check(pine.runScriptCsv(script,csv));
        check(prepared.runCsvWithRequestBars(csv,'{}'));
      }
      const session=prepared.realtimeSession();
      try {
        session.seed(seed);
        const replica=session.replica();
        try {
          for(let tick=1;tick<=3;tick++) {
            replica.apply(session.applyForming(bar));
            check(session.result(),tick);
            assert.equal(replica.result(),session.result(),test.name);
          }
          replica.apply(session.applyConfirmed(bar));
          check(session.confirmedResult(),4);
          assert.equal(replica.result(),session.confirmedResult(),test.name);
        } finally {replica.free();}
      } finally {session.free();}
    } finally {prepared.free();}
  }
};

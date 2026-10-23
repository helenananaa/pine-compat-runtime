"use strict";
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');

module.exports = function(modulePath) {
  const pine = require(path.resolve(modulePath));
  const root = path.resolve(__dirname,'../..');
  const cases = JSON.parse(fs.readFileSync(path.join(root,'docs/evidence/strategy-library-20261010/table-capacity-oracles-36.json'),'utf8')).records;
  const csv = 'time,open,high,low,close,volume\n0,10,10,10,10,1\n60000,10,10,10,10,1\n120000,10,10,10,10,1\n';
  const seed = csv.slice(0,csv.lastIndexOf('120000'));
  for (const test of cases.filter(t=>!t.name.startsWith('original-')&&!t.name.includes('populated'))) {
    const raw = fs.readFileSync(path.join(root,test.source));
    assert.equal(crypto.createHash('sha256').update(raw).digest('hex'),test.source_sha256);
    const script = raw.toString('utf8');
    assert.equal(JSON.parse(pine.analyzeScript(script)).executable,true,test.name);
    if(test.native_status==='analysis_or_runtime_rejected') {
      assert.throws(()=>pine.runScriptCsv(script,csv),/table cell count cannot exceed 10000/);
      continue;
    }
    const check = wire => {
      const result = JSON.parse(wire);
      assert.deepEqual(result.plots.map(p=>p.values.at(-1)),test.native_values,test.name);
      assert.ok(result.tables.length,test.name);
    };
    check(pine.runScriptCsv(script,csv));
    const prepared = pine.compileScript(script);
    try {
      check(prepared.runCsvWithRequestBars(csv,'{}'));
      const session = prepared.realtimeSession();
      session.seed(seed);
      const replica = session.replica();
      try {
        const bar=JSON.stringify({time:120000,open:10,high:10,low:10,close:10,volume:1});
        replica.apply(session.applyForming(bar));
        check(session.result());
        assert.equal(replica.result(),session.result());
        replica.apply(session.applyConfirmed(bar));
        check(session.confirmedResult());
        assert.equal(replica.result(),session.confirmedResult());
      } finally {replica.free();session.free();}
    } finally {prepared.free();}
  }
};

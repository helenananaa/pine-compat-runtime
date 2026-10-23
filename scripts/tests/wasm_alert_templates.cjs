'use strict';
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
module.exports = function(modulePath) {
  const pine = require(modulePath);
  const evidence = path.join(__dirname,'../../docs/evidence/strategy-library-20261010');
  const csv = 'time,open,high,low,close,volume\n0,1,1,1,1,1\n';
  const hosts = JSON.stringify({$executionTimes:[1700000000123]});
  for (const version of [4,5,6]) {
    const source = fs.readFileSync(path.join(evidence,`alert-template-v${version}-oracle-26.pine`),'utf8');
    const prepared = pine.compileScript(source);
    assert.throws(()=>prepared.runCsvWithRequestBars(csv,'{}'),/explicit execution timestamp/);
    for (const result of [JSON.parse(pine.runScriptCsvWithRequestBars(source,csv,hosts)),JSON.parse(prepared.runCsvWithRequestBars(csv,hosts))]) {
      assert.deepEqual(result.diagnostics,[]);
      assert.equal(result.alerts[0].message,'P0=1|P19=20|NOW=2023-11-14T22:13:20Z');
      assert.deepEqual(result.plots.map(p=>p.values),Array.from({length:20},(_,i)=>[i+1]));
    }
  }
  const lazy='//@version=6\nindicator("lazy")\nalertcondition(false,"no clock","{{timenow}}")\nalertcondition(true,"bar time","{{time}}")\nplot(close)\n';
  assert.equal(JSON.parse(pine.runScriptCsv(lazy,csv)).alerts[0].message,'1970-01-01T00:00:00Z');
};

'use strict';
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
module.exports = function(modulePath) {
  const pine = require(modulePath);
  const evidence = path.join(__dirname,'../../docs/evidence/strategy-library-20261010');
  const csv = 'time,open,high,low,close,volume\n0,1,1,1,1,1\n';
  for (const version of [4,5,6]) {
    const source = fs.readFileSync(path.join(evidence,`literal-alert-braces-v${version}-oracle-25.pine`),'utf8');
    for (const result of [JSON.parse(pine.runScriptCsv(source,csv)),JSON.parse(pine.compileScript(source).runCsvWithRequestBars(csv,'{}'))]) {
      assert.deepEqual(result.alerts.map(a=>a.message),['{{ticker}}|{{close}}|{{unknown}}|{{','{{ticker}}|{{close}}|{{unknown}}|{{x','C=1']);
      assert.equal(result.alerts[2].source,'{{ticker}}|{{unknown}}|{{');
    }
  }
  assert.throws(()=>pine.compileScript('//@version=5\nindicator("negative")\nalertcondition(true,"title","{{unknown}}")\nplot(1)\n'),/unknown/);
};

'use strict';
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
module.exports = function(modulePath) {
  const pine = require(modulePath);
  const evidence = path.join(__dirname,'../../docs/evidence/strategy-library-20261010');
  const csv = 'time,open,high,low,close,volume\n1700000000000,1,1,1,1,1\n';
  for (const version of [4,5]) {
    const source = fs.readFileSync(path.join(evidence,`bool-numeric-ternary-expanded-v${version}-oracle-24.pine`),'utf8');
    for (const result of [JSON.parse(pine.runScriptCsv(source,csv)),JSON.parse(pine.compileScript(source).runCsvWithRequestBars(csv,'{}'))]) {
      assert.deepEqual(result.plots.map(p=>p.values),[1,1,1,0,0,1,1,1,0,0,0,0].map(x=>[x]));
    }
  }
  assert.throws(()=>pine.compileScript(fs.readFileSync(path.join(evidence,'bool-int-ternary-v6-negative-24.pine'),'utf8')),/incompatible types/);
  assert.throws(()=>pine.compileScript(fs.readFileSync(path.join(evidence,'bool-numeric-ternary-type-negative-v5-24.pine'),'utf8')),/bool/);
};

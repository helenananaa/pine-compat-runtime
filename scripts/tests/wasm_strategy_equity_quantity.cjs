'use strict';
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
module.exports = function(modulePath) {
  const pine = require(path.resolve(modulePath));
  const cases = JSON.parse(fs.readFileSync(path.join(__dirname,'../../docs/evidence/strategy-library-20261010/equity-quantity-binding-cases-30.json'),'utf8'));
  const csv = 'time,open,high,low,close,volume\n'+[100,100,300,300].map((p,i)=>`${1700000000000+i*60000},${p},${p},${p},${p},1\n`).join('');
  for (const record of cases) {
    const prepared = pine.compileScript(record.source);
    try {
      for (const run of [()=>pine.runScriptCsv(record.source,csv),()=>prepared.runCsvWithRequestBars(csv,'{}')]) {
        if (record.error) {
          assert.throws(run,new RegExp(record.error),record.name);
        } else {
          const result = JSON.parse(run());
          assert.equal(result.plots.length,record.values.length,record.name);
          for (let i=0;i<record.values.length;i++) {
            assert.equal(result.plots[i].values.length,record.values[i].length,record.name);
            for (let j=0;j<record.values[i].length;j++) {
              // Pine numeric equality (also used by Rust/Python) equates IEEE
              // negative zero with zero; JS deepStrictEqual uses Object.is.
              assert.ok(result.plots[i].values[j]===record.values[i][j],`${record.name} plot ${i} bar ${j}`);
            }
          }
          assert.equal(result.strategy.orders.length,record.orders,record.name);
        }
      }
    } finally { prepared.free(); }
  }
};

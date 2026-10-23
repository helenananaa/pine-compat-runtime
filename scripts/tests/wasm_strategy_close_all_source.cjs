'use strict';
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
module.exports = function(modulePath) {
  const pine = require(modulePath);
  const evidence = path.join(__dirname,'../../docs/evidence/strategy-library-20261010');
  const records = JSON.parse(fs.readFileSync(path.join(evidence,'close-all-oracles-29.json'),'utf8'));
  const csv = 'time,open,high,low,close,volume\n' + Array.from({length:6},(_,i)=>`${1700000000000+i*60000},10,10,10,10,1\n`).join('');
  for (const record of records) {
    const source = fs.readFileSync(path.join(evidence,record.stem+'.pine'),'utf8');
    if (record.native_status==='analysis_rejected') {
      assert.throws(()=>pine.compileScript(source),undefined,record.stem);
      continue;
    }
    const prepared = pine.compileScript(source);
    try {
      for (const wire of [pine.runScriptCsv(source,csv),prepared.runCsvWithRequestBars(csv,'{}')]) {
        const result = JSON.parse(wire);
        assert.deepEqual(result.plots.map(p=>p.values.at(-1)),record.native_values,record.stem);
        if (record.expected_exit_bar!==null) {
          assert.deepEqual(result.plots[0].values,[0,1,1,1,0,0]);
          assert.equal(result.strategy.trades.length,1);
          assert.equal(result.strategy.alerts.length,record.expected_alert_count);
        } else {
          assert.deepEqual(result.plots[0].values,Array(6).fill(record.native_values[0]));
          assert.equal(result.strategy.trades.length,0);
        }
      }
    } finally { prepared.free(); }
  }
};

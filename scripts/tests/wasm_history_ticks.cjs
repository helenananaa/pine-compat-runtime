'use strict';
const assert = require('node:assert/strict');
const path = require('node:path');
module.exports = function(modulePath) {
  const pine = require(path.resolve(modulePath));
  assert.equal(pine.externalTickScheduleSchemaVersion(),1);
  assert.equal(pine.maxStrategyObservationsPerBar(),100000);
  const csv='time,open,high,low,close,volume\n0,10,12,9,11,100\n60000,11,14,8,12,200\n';
  const source='//@version=6\nstrategy("History ticks",calc_on_every_history_tick=true)\nvar int ordinary=0\nvarip int retained=0\nordinary+=1\nretained+=1\nplot(ordinary)\nplot(retained)\nplot(ta.cum(volume))\n';
  const prepared=pine.compileScript(source);
  try {
    assert.equal(JSON.parse(prepared.hostRequirements()).execution.calcOnEveryHistoryTick,true);
    for(const output of [pine.runScriptCsv(source,csv),prepared.runCsvWithRequestBars(csv,'{}')]) {
      assert.deepEqual(JSON.parse(output).plots.map(row=>row.values),[[1,2],[4,8],[100,300]]);
    }
  } finally {prepared.free();}
  assert.throws(()=>pine.compileScript(source.replace('version=6','version=5')));
};

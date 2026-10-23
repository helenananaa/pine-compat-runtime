'use strict';
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
module.exports = function(modulePath) {
  const pine = require(modulePath);
  const evidence = path.join(__dirname,'../../docs/evidence/strategy-library-20261010');
  const csv = 'time,open,high,low,close,volume\n' + Array.from({length:6},(_,i)=>`${1700000000000+i*60000},10,10,10,10,1\n`).join('');
  for(const [filename,expected] of [
    ['legacy-direction-v4-oracle-27.pine', [1,0,1,1,1].map(n=>Array(6).fill(n)).concat([[0,1,1,1,0,0]])],
    ['close-comment-v5-oracle-27.pine', [[1,1,1,1,1,1],[0,1,1,1,0,0]]],
    ['close-bool-v5-oracle-27.pine', [[1,1,1,1,1,1],[0,1,1,1,1,1]]],
  ]) {
    let source=fs.readFileSync(path.join(evidence,filename),'utf8');
    if(filename.startsWith('close-')) source+='\nplot(strategy.position_size)\n';
    for(const result of [JSON.parse(pine.runScriptCsv(source,csv)),JSON.parse(pine.compileScript(source).runCsvWithRequestBars(csv,'{}'))]) {
      assert.deepEqual(result.plots.map(p=>p.values),expected);
    }
  }
  for(const version of [5,6]) {
    assert.throws(()=>pine.compileScript(`//@version=${version}\nstrategy("guard")\nd=strategy.long\nf(x)=>x\nstrategy.close("L",f(d))\n`),/strategy_direction/);
  }
};

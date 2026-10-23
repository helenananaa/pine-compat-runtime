'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
module.exports=function(modulePath) {
  const pine=require(modulePath);
  const evidence=path.join(__dirname,'../../docs/evidence/strategy-library-20261010');
  const csv='time,open,high,low,close,volume\n'+Array.from({length:6},(_,i)=>`${1700000000000+i*60000},10,10,10,10,1\n`).join('');
  for(const version of [1,2,3]) {
    const source=fs.readFileSync(path.join(evidence,`legacy-direction-truth-v${version}-oracle-28.pine`),'utf8');
    for(const result of [JSON.parse(pine.runScriptCsv(source,csv)),JSON.parse(pine.compileScript(source).runCsvWithRequestBars(csv,'{}'))]) {
      assert.deepEqual(result.plots.map(p=>p.values),[1,1,0,0,0,0,1,1,0,1,0,0,1,1,1,1,1,1].map(n=>Array(6).fill(n)));
    }
  }
  for(const [variant,positions,orders,trades] of [['orders',[0,1,1,1,-1,0],3,2],['when',[0,0,1,1,1,0],2,1]]) {
    const source=fs.readFileSync(path.join(evidence,`legacy-direction-${variant}-v3-oracle-28.pine`),'utf8');
    for(const result of [JSON.parse(pine.runScriptCsv(source,csv)),JSON.parse(pine.compileScript(source).runCsvWithRequestBars(csv,'{}'))]) {
      assert.deepEqual(result.plots[0].values,positions);
      assert.equal(result.strategy.orders.length,orders);
      assert.equal(result.strategy.trades.length,trades);
    }
  }
};

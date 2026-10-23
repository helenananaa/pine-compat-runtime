'use strict';
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
module.exports = function(modulePath) {
  const pine = require(path.resolve(modulePath));
  const dir = path.join(__dirname,'../../docs/evidence/strategy-library-20261010');
  const records = JSON.parse(fs.readFileSync(path.join(dir,'fx-native-ledger-oracles-31.json'),'utf8'));
  const rates = JSON.parse(fs.readFileSync(path.join(dir,'fx-native-rates-input-31.json'),'utf8'));
  for (const record of records) {
    const csv = fs.readFileSync(path.join(dir,`fx-native-chart-${record.chart_timeframe.toLowerCase()}-31.csv`),'utf8');
    const host = JSON.stringify({$chart:{symbol:'BINANCE:BTCUSDT',timeframe:record.chart_timeframe,currency:'USDT',minMove:1,priceScale:100,quantityPrecision:5},$currencyRates:rates});
    const source = fs.readFileSync(path.join(dir,record.stem+'.pine'),'utf8');
    const prepared = pine.compileScript(source);
    try {
      for (const wire of [pine.runScriptCsvWithRequestBars(source,csv,host),prepared.runCsvWithRequestBars(csv,host)]) {
        const actual = JSON.parse(wire).plots.map(p=>p.values.at(-1));
        assert.equal(actual.length,record.native_values.length,record.stem);
        actual.forEach((value,i)=>{
          const expected=record.native_values[i];
          if(expected===null)assert.equal(value,null,record.stem);
          else assert.ok(typeof value==='number' && Math.abs(value-expected)<=5.1e-9,`${record.stem} plot ${i}: ${value} != ${expected}`);
        });
      }
    } finally { prepared.free(); }
  }
  const nameSource = fs.readFileSync(path.join(dir,'fx-name-controls-v6-oracle-31.pine'),'utf8');
  const nameBars = 'time,open,high,low,close,volume\n1792195200000,1,1,1,1,0\n';
  const nameRates = {schemaVersion:1,pairs:[
    {from:'USD',to:'EUR',rates:[{effectiveTime:1792195200000,rate:0.891}]},
    {from:'USD',to:'ZZZ',unavailable:true},
  ]};
  const nameResult=JSON.parse(pine.runScriptCsvWithRequestBars(nameSource,nameBars,JSON.stringify({$currencyRates:nameRates})));
  assert.deepEqual(nameResult.plots.map(p=>p.values.at(-1)),[1,0.891,1,null]);
  for (const invalid of [{schemaVersion:2,pairs:[]},{schemaVersion:1,pairs:[{from:'USD',to:'EUR'}]}]) {
    assert.throws(()=>pine.runScriptCsvWithRequestBars('//@version=6\nindicator("input")\nplot(close)\n',nameBars,JSON.stringify({$currencyRates:invalid})));
  }
  const realtimeRecords=JSON.parse(fs.readFileSync(path.join(dir,'fx-native-realtime-oracles-31.json'),'utf8'));
  for(const record of realtimeRecords){
    const compiled=pine.compileScript(fs.readFileSync(path.join(dir,record.stem+'.pine'),'utf8'));
    const host=JSON.stringify({$chart:{symbol:'BINANCE:BTCUSDT',timeframe:record.chart_timeframe,currency:'USDT',minMove:1,priceScale:100,quantityPrecision:5},$currencyRates:rates});
    const session=compiled.realtimeSessionWithRequestBars(host);
    session.seed('time,open,high,low,close,volume\n');
    const replica=session.replica();
    try {
      const expected=record.native_values;
      const bar=JSON.stringify({time:expected[4],open:100,high:100,low:100,close:100,volume:0});
      for(const clock of [expected[5],expected[5]+60000]){
        replica.apply(session.applyFormingWithContext(bar,JSON.stringify({executionTime:clock})));
        const result=JSON.parse(session.result());
        assert.deepEqual(JSON.parse(replica.result()),result);
        result.plots.forEach((plot,i)=>assert.ok(Math.abs(plot.values.at(-1)-(i===5?clock:expected[i]))<=5.1e-9,`${record.stem} realtime plot ${i}`));
      }
    } finally {replica.free();session.free();compiled.free();}
  }
};

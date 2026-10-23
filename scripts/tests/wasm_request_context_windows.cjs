'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const crypto=require('node:crypto');

module.exports=function(modulePath) {
  const pine=require(path.resolve(modulePath));
  const root=path.resolve(__dirname,'../..');
  const cases=JSON.parse(fs.readFileSync(path.join(root,'docs/evidence/strategy-library-20261010/request-window-oracles-44.json'),'utf8')).records;
  const bar=time=>({time,open:10,high:10,low:10,close:10,volume:1});
  const bars=Array.from({length:3350},(_,i)=>bar(1502928000000+i*86400000));
  const csv='time,open,high,low,close,volume\n'+bars.map(b=>`${b.time},10,10,10,10,1\n`).join('');
  const requests=JSON.stringify({$chart:{symbol:'BINANCE:BTCUSDT',timeframe:'D'},'BINANCE:BTCUSDT:W':Array.from({length:479},(_,i)=>bar(1502668800000+i*604800000)),'NASDAQ:AAPL:D':bars});
  for(const test of cases) {
    const raw=fs.readFileSync(path.join(root,test.source));
    assert.equal(crypto.createHash('sha256').update(raw).digest('hex'),test.source_sha256);
    const source=raw.toString('utf8');
    if(test.native_status==='analysis_rejected') {
      assert.equal(JSON.parse(pine.analyzeScript(source)).executable,false,test.name);
      assert.throws(()=>pine.compileScript(source));
      continue;
    }
    const prepared=pine.compileScript(source);
    const session=prepared.realtimeSessionWithRequestBars(requests);
    try {
      if(test.native_status==='runtime_error') {
        for(const run of [()=>pine.runScriptCsvWithRequestBars(source,csv,requests),()=>prepared.runCsvWithRequestBars(csv,requests),()=>session.seed(csv)]) {
          assert.throws(run,error=>String(error).includes(test.runtime_error_message),test.name);
        }
        continue;
      }
      for(const wire of [pine.runScriptCsvWithRequestBars(source,csv,requests),prepared.runCsvWithRequestBars(csv,requests),session.seed(csv)]) {
        const result=JSON.parse(wire);
        assert.deepEqual(result.plots.map(p=>p.values.at(-1)),test.native_values,test.name);
        assert.equal(result.plots[0].values.length,test.native_values.at(-1),test.name);
      }
      const replica=session.replica();
      try { assert.equal(replica.result(),session.result(),test.name); } finally { replica.free(); }
    } finally { session.free(); prepared.free(); }
  }
  require('./wasm_request_window_live_origin.cjs')(modulePath);
  require('./wasm_simple_declaration_names.cjs')(modulePath);
};

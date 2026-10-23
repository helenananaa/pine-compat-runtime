'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const crypto=require('node:crypto');

module.exports=function(modulePath) {
  const pine=require(path.resolve(modulePath));
  const root=path.resolve(__dirname,'../..');
  const cases=JSON.parse(fs.readFileSync(path.join(root,'docs/evidence/strategy-library-20261010/simple-declaration-name-oracles-44.json'),'utf8')).records;
  const rows=[0,60000,120000].map(time=>({time,open:10,high:10,low:10,close:10,volume:1}));
  const csv=bars=>'time,open,high,low,close,volume\n'+bars.map(b=>`${b.time},10,10,10,10,1\n`).join('');
  for(const test of cases) {
    const raw=fs.readFileSync(path.join(root,test.source));
    assert.equal(crypto.createHash('sha256').update(raw).digest('hex'),test.source_sha256);
    const source=raw.toString('utf8');const prepared=pine.compileScript(source);const session=prepared.realtimeSession();
    const expected=[2,2,2];
    try {
      for(const wire of [pine.runScriptCsv(source,csv(rows)),prepared.runCsvWithRequestBars(csv(rows),'{}')]) {
        assert.deepEqual(JSON.parse(wire).plots[0].values,expected,test.name);
      }
      session.seed(csv(rows.slice(0,2)));const replica=session.replica();
      try {
        assert.equal(replica.apply(session.applyForming(JSON.stringify(rows[2]))),true);
        assert.deepEqual(JSON.parse(replica.result()).plots[0].values,expected,test.name);
        assert.equal(replica.apply(session.applyConfirmed(JSON.stringify(rows[2]))),true);
        assert.deepEqual(JSON.parse(replica.result()).plots[0].values,expected,test.name);
        assert.equal(replica.result(),session.confirmedResult(),test.name);
      } finally {replica.free();}
    } finally {session.free();prepared.free();}
  }
};

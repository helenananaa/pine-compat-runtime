'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const crypto=require('node:crypto');

module.exports=function(modulePath) {
  const pine=require(path.resolve(modulePath));
  const root=path.resolve(__dirname,'../..');
  const [initial,continued]=JSON.parse(fs.readFileSync(path.join(root,'docs/evidence/strategy-library-20261010/request-window-live-origin-oracles-44.json'),'utf8')).records;
  const raw=fs.readFileSync(path.join(root,initial.source));
  assert.equal(crypto.createHash('sha256').update(raw).digest('hex'),initial.source_sha256);
  const prepared=pine.compileScript(raw.toString('utf8'));
  const session=prepared.realtimeSessionWithRequestBars(JSON.stringify({$chart:{symbol:'BINANCE:BTCUSDT',timeframe:'1'}}));
  const time=initial.native_values[3];
  const bar=time=>JSON.stringify({time,open:10,high:10,low:10,close:10,volume:1});
  const values=wire=>JSON.parse(wire).plots.map(p=>p.values.at(-1));
  try {
    assert.deepEqual(values(session.seed(`time,open,high,low,close,volume\n${time-60000},10,10,10,10,1\n${time},10,10,10,10,1\n`)),initial.native_values);
    const replica=session.replica();
    try {
      for(let tick=0;tick<3;tick++) {
        assert.equal(replica.apply(session.applyForming(bar(time+60000))),true);
        assert.deepEqual(values(replica.result()),[2,2,3,time+60000]);
        assert.equal(replica.result(),session.result());
      }
      assert.equal(replica.apply(session.applyConfirmed(bar(time+60000))),true);
      for(let tick=0;tick<3;tick++) {
        assert.equal(replica.apply(session.applyForming(bar(time+120000))),true);
        assert.deepEqual(values(replica.result()),continued.native_values);
        assert.equal(replica.result(),session.result());
      }
      assert.equal(replica.apply(session.applyConfirmed(bar(time+120000))),true);
      assert.deepEqual(values(replica.result()),continued.native_values);
      assert.equal(replica.result(),session.confirmedResult());
    } finally {replica.free();}
  } finally {session.free();prepared.free();}
};

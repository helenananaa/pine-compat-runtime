'use strict';
const assert=require('node:assert/strict');
const path=require('node:path');
module.exports=function(modulePath) {
  const pine=require(path.resolve(modulePath));
  assert.equal(pine.realtimeRequestFeedSchemaVersion(),1);
  const bar=(time,price=10)=>({time,open:price,high:price,low:price,close:price,volume:1});
  const update=(time,price=10)=>({symbol:'B',timeframe:'1',bar:bar(time,price),kind:'confirmed'});
  const code='//@version=6\nindicator("feed")\nvarip int n=0\nn+=1\nplot(n)\na=request.security_lower_tf("B","1",close)\nplot(array.size(a))\nplot(array.sum(a))';
  const program=pine.compileScript(code);
  const session=program.realtimeSessionWithRequestBars(JSON.stringify({$chart:{symbol:'A',timeframe:'5'},'B:1':Array.from({length:5},(_,i)=>bar(i*60000)),'C:1':[bar(0)]}));
  let replica;
  try {
    assert.throws(()=>session.applyRequestFeedUpdates('[]'),/seeded/);
    const initial=session.seed('time,open,high,low,close,volume\n0,10,10,10,10,1\n');
    session.applyRequestFeedUpdates(JSON.stringify([update(300000)]));
    assert.equal(session.result(),initial);
    session.applyForming(JSON.stringify(bar(300000)));
    replica=session.replica();
    const before=session.result(),revision=session.revision,changes=session.lastChanges();
    assert.throws(()=>session.applyRequestFeedUpdates(JSON.stringify([update(360000,20),{...update(0,999),symbol:'C'}])));
    assert.equal(session.result(),before);
    assert.equal(session.applyRequestFeedUpdates(JSON.stringify([update(360000,20),update(420000,30)])),undefined);
    assert.equal(session.result(),before);
    assert.equal(session.revision,revision);
    assert.equal(session.lastChanges(),changes);
    const delta=session.applyForming(JSON.stringify(bar(300000,11)));
    assert.equal(replica.apply(delta),true);
    const result=JSON.parse(session.result());
    assert.deepEqual(result.plots.map(p=>p.values.at(-1)),[3,3,60]);
    assert.deepEqual(JSON.parse(replica.result()),result);
    assert.equal(session.requestBarClose('B','30S','0'),'30000');
    assert.equal(session.requestBarClose('B','1M','1706745600000'),'1709251200000');
    assert.throws(()=>session.requestBarClose('B','1','true'));
    assert.throws(()=>session.requestBarClose('B','1','60000.5'));
  } finally {if(replica)replica.free();session.free();program.free();}
};

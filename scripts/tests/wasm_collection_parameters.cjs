'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const crypto=require('node:crypto');

module.exports=function(modulePath) {
  const pine=require(path.resolve(modulePath));
  const root=path.resolve(__dirname,'../..');
  const cases=JSON.parse(fs.readFileSync(path.join(root,'docs/evidence/strategy-library-20261010/collection-parameter-oracles-45.json'),'utf8')).records;
  const supplement=JSON.parse(fs.readFileSync(path.join(root,'docs/evidence/strategy-library-20261010/collection-import-supplementary-45.json'),'utf8')).records;
  cases.push(...supplement.filter(c=>c.native_status==='compiled_and_executed'));
  const bars=[0,60000,120000].map(time=>({time,open:10,high:10,low:10,close:10,volume:1}));
  const csv=rows=>'time,open,high,low,close,volume\n'+rows.map(b=>`${b.time},10,10,10,10,1\n`).join('');
  const values=wire=>JSON.parse(wire).plots.map(p=>p.values.at(-1));
  for(const test of cases) {
    const raw=fs.readFileSync(path.join(root,test.source));
    assert.equal(crypto.createHash('sha256').update(raw).digest('hex'),test.source_sha256);
    const source=raw.toString('utf8');
    if(test.native_status==='analysis_rejected') {
      assert.equal(JSON.parse(pine.analyzeScript(source)).executable,false,test.name);
      assert.throws(()=>pine.compileScript(source));continue;
    }
    const prepared=pine.compileScript(source);const session=prepared.realtimeSession();
    try {
      assert.deepEqual(values(pine.runScriptCsv(source,csv(bars))),test.local_three_bar_values,test.name);
      assert.deepEqual(values(prepared.runCsvWithRequestBars(csv(bars),'{}')),test.local_three_bar_values,test.name);
      session.seed(csv(bars.slice(0,2)));const replica=session.replica();
      try {
        for(let i=0;i<3;i++) {
          assert.equal(replica.apply(session.applyForming(JSON.stringify(bars[2]))),true);
          assert.deepEqual(values(replica.result()),test.local_three_bar_values,test.name);
        }
        assert.equal(replica.apply(session.applyConfirmed(JSON.stringify(bars[2]))),true);
        assert.deepEqual(values(replica.result()),test.local_three_bar_values,test.name);
        assert.equal(replica.result(),session.confirmedResult(),test.name);
      } finally {replica.free();}
    } finally {session.free();prepared.free();}
  }
  for(const test of supplement.filter(c=>c.native_status==='compiled_library')) {
    const raw=fs.readFileSync(path.join(root,test.source));
    assert.equal(crypto.createHash('sha256').update(raw).digest('hex'),test.source_sha256);
    const array=test.name.includes('mutable-array');
    const body=array?'var values=array.new<float>(1,10)\nplot(c.step(values))\nplot(values.size())':'var m=map.new<string,int>()\nvar x=matrix.new<int>(1,1,1)\nif barstate.isfirst\n    m.put("x",1)\nplot(c.bump(m))\nplot(m.get("x"))\nplot(c.step(x))\nplot(x.get(0,0))';
    const source=`//@version=${test.version}\nimport local/Contract/1 as c\nindicator("import contract")\n${body}\n`;
    const expected=array?[[2,3,4],[2,3,4]]:Array.from({length:4},()=>[1.5,2,2.5]);
    const prepared=pine.compileScriptWithLibraries(source,JSON.stringify({'local/Contract/1':raw.toString('utf8')}));
    const session=prepared.realtimeSession();
    try {
      assert.deepEqual(JSON.parse(prepared.runCsvWithRequestBars(csv(bars),'{}')).plots.map(p=>p.values),expected,test.name);
      session.seed(csv(bars.slice(0,2)));const replica=session.replica();
      try {
        for(let i=0;i<3;i++) {
          assert.equal(replica.apply(session.applyForming(JSON.stringify(bars[2]))),true);
          assert.deepEqual(JSON.parse(replica.result()).plots.map(p=>p.values),expected,test.name);
        }
        assert.equal(replica.apply(session.applyConfirmed(JSON.stringify(bars[2]))),true);
        assert.deepEqual(JSON.parse(replica.result()).plots.map(p=>p.values),expected,test.name);
        assert.equal(replica.result(),session.confirmedResult(),test.name);
      } finally {replica.free();}
    } finally {session.free();prepared.free();}
  }
};

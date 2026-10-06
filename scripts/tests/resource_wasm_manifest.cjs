'use strict';

// Run against an actual wasm-bindgen Node artifact, never a mock runtime.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const {spawnSync} = require('node:child_process');

if(process.argv.length!==3)throw new Error('usage: node resource_wasm_manifest.cjs <generated-module.js>');
const modulePath=path.resolve(process.argv[2]);
const worker=path.resolve(__dirname,'../product_resource_wasm.cjs');
const directory=fs.mkdtempSync(path.join(os.tmpdir(),'pine-resource-manifest-'));
const bars=Array.from({length:19},(_,i)=>({time:i*60000,open:100+i,high:102+i,
  low:99+i,close:101+i,volume:10+i}));
const tail=bars.slice(16);
const payload={source:'//@version=6\nindicator("manifest")\nplot(close)\n',
  bars:bars.slice(0,16),tail,overrides:{},
  request:{$chart:{symbol:'OFFLINE:RESOURCE',timeframe:'1',minMove:1,priceScale:1000}},
  events:tail.flatMap(bar=>[
    {kind:'forming',phase:'forming',bar:{...bar,close:bar.open}},
    {kind:'forming',phase:'replacement',bar:{...bar,close:bar.high}},
    {kind:'confirmed',phase:'confirmation',bar},
  ])};
const payloadPath=path.join(directory,'payload.json');
fs.writeFileSync(payloadPath,JSON.stringify(payload));

function read(file){return JSON.parse(fs.readFileSync(file,'utf8'));}
function checkMetrics(value,counts){
  assert.deepEqual(Object.keys(value).sort(),Object.keys(counts).sort());
  for(const [key,count] of Object.entries(counts)){
    assert.equal(value[key].length,count,key);
    assert(value[key].every(x=>Number.isFinite(x)&&x>=0),key);
  }
}
function resolveReference(reportPath,ref){
  assert.deepEqual(Object.keys(ref),['path']);
  assert.equal(typeof ref.path,'string');
  assert(!path.isAbsolute(ref.path));
  assert(!ref.path.includes('\\'));
  assert(!ref.path.split('/').includes('..'));
  return path.resolve(path.dirname(reportPath),ref.path);
}
try {
  const outputDirectory=path.join(directory,'nested outputs');
  fs.mkdirSync(outputDirectory);
  for(const count of [1,4]){
    const reportPath=path.join(outputDirectory,`result-${count} 你好.json`);
    // Cover both absolute and relative report arguments on the current host.
    const reportArgument=count===1?reportPath:path.relative(directory,reportPath);
    const child=spawnSync(process.execPath,[worker,modulePath,payloadPath,String(count),reportArgument],
      {cwd:directory,encoding:'utf8',timeout:30000});
    assert.equal(child.error,undefined);
    assert.equal(child.status,0,child.stderr);
    const report=read(reportPath);
    assert.deepEqual(Object.keys(report).sort(),
      ['resourceReportVersion','metadata','results','confirmedBars','historicalAppend'].sort());
    assert.equal(report.resourceReportVersion,2);
    assert.equal(report.confirmedBars,bars.length);
    assert.equal(report.results.length,count);
    assert.equal(report.metadata.path,path.basename(reportPath)+'.metadata.json');
    const metadata=read(resolveReference(reportPath,report.metadata));
    assert.equal(metadata.resourceMetadataVersion,1);
    assert.equal(metadata.resultCount,count);
    assert.equal(metadata.confirmedBars,bars.length);
    assert.equal(metadata.historicalAppend,report.historicalAppend);
    checkMetrics(metadata.metrics,{compile:1,seed:count,forming:tail.length*count,
      replacement:tail.length*count,confirmation:tail.length*count,
      replica:payload.events.length*count,snapshot:count,serialization:count});
    checkMetrics(metadata.overheadMetrics,{replicaSnapshot:count,replicaExport:count,
      replicaWrite:count,comparison:2*count-1,reportWrite:1});
    let previous;
    for(let i=0;i<count;i++){
      assert.equal(report.results[i].path,`${path.basename(reportPath)}.parts/live-${i}.json`);
      const livePath=resolveReference(reportPath,report.results[i]);
      const liveBytes=fs.readFileSync(livePath);
      const replicaBytes=fs.readFileSync(path.join(path.dirname(livePath),`replica-${i}.json`));
      assert(liveBytes.equals(replicaBytes),'complete output differs from replica');
      const live=JSON.parse(liveBytes);
      assert.deepEqual(live.diagnostics,[]);
      assert.deepEqual(live.plots[0].values,bars.map(bar=>bar.close+i*.125*(Math.floor(bar.time/60000)%17-8)));
      if(previous)assert(!previous.equals(liveBytes),'streams must remain independent');
      previous=liveBytes;
    }
    // A manifest must stay small; full results live only in their spool files.
    assert(fs.statSync(reportPath).size<2048);
  }
} finally {
  // Only remove the exact fresh temporary directory created by this test.
  assert(path.dirname(directory)===path.resolve(os.tmpdir()));
  assert(path.basename(directory).startsWith('pine-resource-manifest-'));
  fs.rmSync(directory,{recursive:true,force:true});
}
console.log('WASM resource manifest passed: actual WASM, 1/4 streams, full outputs, metric counts');

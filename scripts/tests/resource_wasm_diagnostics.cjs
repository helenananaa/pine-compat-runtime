const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const worker = fs.readFileSync(path.join(__dirname, '../product_resource_wasm.cjs'), 'utf8');
const code = worker.slice(worker.indexOf('function valueEnd('), worker.indexOf('const program ='));
const context = vm.createContext({assert,fs,Buffer});
vm.runInContext(worker.slice(worker.indexOf('const columns ='),worker.indexOf('// Inspect only')),context);
const csv=vm.runInContext('csv',context);
for(const count of [0,1,1023,1024,1025,2048,2049]) {
  const bars=Array.from({length:count},(_,i)=>({time:i*60000,open:-0,high:i+0.5,low:-i,close:1/3,volume:i}));
  const columns=['time','open','high','low','close','volume'];
  const original=columns.join(',')+'\n'+bars.map(b=>columns.map(k=>b[k]).join(',')).join('\n')+'\n';
  assert.equal(csv(bars),original,'CSV bytes changed at row/block boundary '+count);
}
vm.runInContext(code, context);
const verify = context.assertDiagnostics;
verify(JSON.stringify({plots:[{values:[1,null,true,[2]],title:'\\"diagnostics":[]{},你好'}],diagnostics:[]}));
verify(JSON.stringify({strategy:{orders:[{id:'escaped \\" } ]'}],diagnostics:[]},diagnostics:[]}));
for(const value of [
  {diagnostics:[{message:'failed'}]},
  {diagnostics:[],strategy:{diagnostics:[{code:'error'}]}},
  {diagnostics:[],strategy:{orders:[]}},
  {plots:[]},
  {diagnostics:{}},
])assert.throws(()=>verify(JSON.stringify(value)));
assert.throws(()=>verify('{"diagnostics":[],"diagnostics":[]}'));
assert.throws(()=>verify('{"plots":[{"title":"unterminated'));
const directory=fs.mkdtempSync(path.join(require('node:os').tmpdir(),'pine-resource-string-'));
const output=path.join(directory,'output.json');
try {
  for(const text of ['', 'a'.repeat(16383)+'😀你好\n'+('界'.repeat(70000)), JSON.stringify({text:'😀'.repeat(40000)})]) {
    context.writeString(output,text);
    assert(fs.readFileSync(output).equals(Buffer.from(text)));
  }
} finally {if(fs.existsSync(output))fs.unlinkSync(output);fs.rmdirSync(directory);}
console.log('WASM resource diagnostic field scanner passed');

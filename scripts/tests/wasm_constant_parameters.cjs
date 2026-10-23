"use strict";
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const crypto = require("node:crypto");

module.exports = function(modulePath) {
  const pine = require(path.resolve(modulePath));
  const root = path.resolve(__dirname,"../..");
  const cases = JSON.parse(fs.readFileSync(path.join(root,"docs/evidence/strategy-library-20261010/constant-param-oracles-35.json"),"utf8")).records;
  const candidates = cases.filter(test=>test.qualification_scope==="candidate_bounded_control");
  const source = test => {
    const raw = fs.readFileSync(path.join(root,test.source));
    assert.equal(crypto.createHash("sha256").update(raw).digest("hex"),test.source_sha256);
    return raw.toString("utf8");
  };
  const csv = "time,open,high,low,close,volume\n0,10,10,10,10,1\n60000,10,10,10,10,1\n120000,10,10,10,10,1\n";
  const seed = csv.slice(0,csv.lastIndexOf("120000"));
  for (const test of candidates) {
    const script = source(test);
    const report = JSON.parse(pine.analyzeScript(script));
    if(test.native_status==="analysis_rejected") {
      assert.equal(report.executable,false,test.name);
      continue;
    }
    assert.equal(report.executable,true,test.name);
    const check = wire => assert.deepEqual(JSON.parse(wire).plots.map(plot=>plot.values.at(-1)),test.native_values,test.name);
    check(pine.runScriptCsv(script,csv));
    const prepared = pine.compileScript(script);
    try {
      check(prepared.runCsvWithRequestBars(csv,"{}"));
      const session = prepared.realtimeSession();
      session.seed(seed);
      const replica = session.replica();
      try {
        const bar = JSON.stringify({time:120000,open:10,high:10,low:10,close:10,volume:1});
        replica.apply(session.applyForming(bar));
        check(session.result());
        assert.equal(replica.result(),session.result());
        replica.apply(session.applyConfirmed(bar));
        check(session.confirmedResult());
        assert.equal(replica.result(),session.confirmedResult());
      } finally {replica.free();session.free();}
    } finally {prepared.free();}
  }
  const script='//@version=6\nimport test/parameters/1 as lib\nindicator("helper")\nplot(lib.value(true))\n';
  const libraries=JSON.stringify({"test/parameters/1":'//@version=6\nlibrary("helper")\nhidden(const int x=2)=>x+1\nexport value(simple bool enabled)=>enabled?hidden():0\n'});
  const prepared=pine.compileScriptWithLibraries(script,libraries);
  try {assert.deepEqual(JSON.parse(prepared.runCsvWithRequestBars(csv,"{}")).plots[0].values,[3,3,3]);}
  finally {prepared.free();}
  const bad=JSON.stringify({"test/parameters/1":source(candidates.find(test=>test.library))});
  const report=JSON.parse(pine.analyzeScriptWithLibraries(script.replace("plot(lib.value(true))","plot(close)"),bad));
  assert.equal(report.executable,false);
  assert.ok(report.diagnostics.some(item=>item.code==="E_FUNCTION_PARAM_TYPE"&&item.message.includes("exported")));
};

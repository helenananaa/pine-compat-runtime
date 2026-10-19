"use strict";
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const crypto = require("node:crypto");
module.exports = function(modulePath) {
  const pine = require(path.resolve(modulePath));
  const root = path.resolve(__dirname,"../..");
  const cases = JSON.parse(fs.readFileSync(path.join(root,"docs/evidence/strategy-library-20261010/reserved-identifiers-oracles-33.json"),"utf8")).cases;
  const csv = "time,open,high,low,close,volume\n0,10,10,10,10,1\n60000,10,10,10,10,1\n";
  const implicit = JSON.parse(fs.readFileSync(path.join(root,"docs/evidence/strategy-library-20261010/reserved-implicit-import-research-33.json"),"utf8"));
  const implicitRaw = fs.readFileSync(path.join(root,implicit.source));
  assert.equal(crypto.createHash("sha256").update(implicitRaw).digest("hex"),implicit.source_sha256);
  const implicitReport = JSON.parse(pine.analyzeScript(implicitRaw.toString("utf8")));
  assert.equal(implicitReport.executable,false);
  assert.ok(implicitReport.diagnostics.some(d=>d.code==="E_IMPORT_MISSING_LIBRARY"));
  assert.ok(implicitReport.diagnostics.every(d=>d.code!=="E_RESERVED_IDENTIFIER"));
  for (const test of cases) {
    const raw = fs.readFileSync(path.join(root,test.source));
    assert.equal(crypto.createHash("sha256").update(raw).digest("hex"),test.source_sha256);
    const source = raw.toString("utf8");
    const report = JSON.parse(pine.analyzeScript(source));
    if (test.native_status === "analysis_rejected") {
      assert.equal(report.executable,false,test.name);
      assert.ok(report.diagnostics.some(d=>d.code==="E_RESERVED_IDENTIFIER"),test.name);
      assert.throws(()=>pine.compileScript(source),/reserved identifier/);
      assert.throws(()=>pine.runScriptCsv(source,csv),/reserved identifier/);
      continue;
    }
    assert.equal(report.executable,true,test.name);
    const check = (wire,count)=>assert.deepEqual(JSON.parse(wire).plots.map(p=>p.values),test.expected_plots.map(v=>Array(count).fill(v)),test.name);
    const prepared = pine.compileScript(source);
    try {
      check(pine.runScriptCsv(source,csv),2);
      check(prepared.runCsvWithRequestBars(csv,"{}"),2);
      const session = prepared.realtimeSession();
      session.seed(csv);
      const replica = session.replica();
      try {
        const bar = JSON.stringify({time:120000,open:10,high:10,low:10,close:10,volume:1});
        replica.apply(session.applyForming(bar));
        check(session.result(),3);
        assert.equal(replica.result(),session.result());
        replica.apply(session.applyConfirmed(bar));
        check(session.confirmedResult(),3);
        assert.equal(replica.result(),session.confirmedResult());
      } finally { replica.free();session.free(); }
    } finally { prepared.free(); }
  }
};

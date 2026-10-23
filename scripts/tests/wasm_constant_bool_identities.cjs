"use strict";
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const crypto = require("node:crypto");

module.exports = function(modulePath) {
  const pine = require(path.resolve(modulePath));
  const root = path.resolve(__dirname,"../..");
  const read = name => JSON.parse(fs.readFileSync(path.join(root,"docs/evidence/strategy-library-20261010",name),"utf8"));
  const source = test => {
    const raw = fs.readFileSync(path.join(root,test.source));
    assert.equal(crypto.createHash("sha256").update(raw).digest("hex"),test.source_sha256);
    return raw.toString("utf8");
  };
  const csv = "time,open,high,low,close,volume\n0,10,10,10,10,1\n60000,10,10,10,10,1\n120000,10,10,10,10,1\n";
  const seed = csv.slice(0,csv.lastIndexOf("120000"));
  const contextCases = read("constant-bool-declaration-context-oracles-34.json").records;
  for (const test of [...read("constant-bool-identity-oracles-34.json").records,...contextCases.filter(test=>test.native_status==="compiled_and_executed")]) {
    const script = source(test);
    const check = wire => assert.deepEqual(JSON.parse(wire).plots.map(p=>p.values.at(-1)),test.native_values);
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
      } finally { replica.free();session.free(); }
    } finally { prepared.free(); }
  }
  for (const test of contextCases.filter(test=>test.native_status==="analysis_rejected")) {
    const report = JSON.parse(pine.analyzeScript(source(test)));
    assert.equal(report.executable,false);
    assert.ok(report.diagnostics.some(d=>d.code==="E_CALL_ARG_TYPE"));
  }
  for (const test of read("constant-bool-research-34.json").records.filter(t=>t.group==="typeflow")) {
    const report = JSON.parse(pine.analyzeScript(source(test)));
    assert.equal(report.executable,false);
    assert.ok(report.diagnostics.some(d=>d.code==="E_CALL_ARG_TYPE"));
  }
  const groups = new Set(["declaration","not-binding","config-control","config-extra","numeric-config","na-config","config"]);
  const declarations = read("constant-bool-research-34.json").records.filter(test=>groups.has(test.group));
  assert.equal(declarations.length,13);
  for (const test of declarations) {
    const report = JSON.parse(pine.analyzeScript(source(test)));
    assert.equal(report.executable,test.native_status==="compiled_and_executed");
  }
  const metadata = read("constant-bool-input-metadata-oracle-34.json");
  const metadataSource = source(metadata);
  const report = JSON.parse(pine.analyzeScript(metadataSource));
  assert.deepEqual(report.inputs.map(input=>input.title),metadata.native_input_titles);
  assert.deepEqual(report.inputs.map(input=>input.default),metadata.native_input_defaults);
  assert.deepEqual(JSON.parse(pine.runScriptCsv(metadataSource,csv)).plots.map(plot=>plot.values.at(-1)),metadata.native_values);
};

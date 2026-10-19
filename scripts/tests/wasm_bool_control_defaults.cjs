"use strict";
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
module.exports = function(modulePath) {
  const pine = require(path.resolve(modulePath));
  for (const [filename,boolCount,otherCount] of [["bool_control_defaults_v6.pine",4,1],["bool_control_functions_history_v6.pine",6,3]]) {
  const native = fs.readFileSync(path.join(__dirname,"../../tests/fixtures/runtime",filename),"utf8");
  const csv = "time,open,high,low,close,volume\n0,10,10,10,10,1\n60000,10,10,10,10,1\n";
  for (const version of [5,6]) {
    const source = native.replace("version=6",`version=${version}`);
    const expected = [...Array(boolCount).fill(version===6?1:0),...Array(otherCount).fill(1)];
    const check = (wire,count) => assert.deepEqual(JSON.parse(wire).plots.map(p=>p.values),expected.map(value=>Array(count).fill(value)));
    const prepared = pine.compileScript(source);
    try {
      check(pine.runScriptCsv(source,csv),2);
      check(prepared.runCsvWithRequestBars(csv,"{}"),2);
      const session = prepared.realtimeSession();
      session.seed("time,open,high,low,close,volume\n0,10,10,10,10,1\n");
      const replica = session.replica();
      try {
        for (const close of [10,20]) {
          const bar = JSON.stringify({time:60000,open:close,high:close,low:close,close,volume:1});
          assert.equal(replica.apply(session.applyForming(bar)),true);
          check(session.result(),2);
          assert.equal(replica.result(),session.result());
        }
        const bar = JSON.stringify({time:60000,open:20,high:20,low:20,close:20,volume:1});
        replica.apply(session.applyConfirmed(bar));
        check(session.confirmedResult(),2);
        assert.equal(replica.result(),session.confirmedResult());
      } finally { replica.free();session.free(); }
    } finally { prepared.free(); }
  }
  }
};

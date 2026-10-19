'use strict';
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');

module.exports = function(modulePath) {
  const pine = require(path.resolve(modulePath));
  const root = path.resolve(__dirname, '../..');
  const cases = JSON.parse(fs.readFileSync(path.join(root,
    'docs/evidence/strategy-library-20261010/function-body-positive-migration-oracles-46.json'), 'utf8')).records;
  const bodyCases = ['function-body-oracles-46.json', 'function-body-shape-oracles-46.json', 'function-body-supplementary-oracles-46.json', 'function-body-method-forwarding-oracles-46.json']
    .flatMap(file => JSON.parse(fs.readFileSync(path.join(root, 'docs/evidence/strategy-library-20261010', file), 'utf8')).records)
    .filter(test => test.qualification_scope !== 'confirmed_remaining_parser_gap');
  const bars = Array.from({length: 12}, (_, i) => ({time: i * 60000, open: 10, high: 10, low: 10, close: 10, volume: 1}));
  const header = 'time,open,high,low,close,volume\n';
  const csv = items => header + items.map(b => Object.values(b).join(',') + '\n').join('');
  for (const test of bodyCases) {
    const raw = fs.readFileSync(path.join(root, test.source));
    assert.equal(crypto.createHash('sha256').update(raw).digest('hex'), test.source_sha256);
    let script = raw.toString('utf8');
    const libraries = {};
    if (test.kind === 'exported') {
      libraries['local/Unused/1'] = script;
      script = `//@version=${test.version}\nimport local/Unused/1 as c\nindicator("unused library")\nplot(1)\n`;
    }
    const report = JSON.parse(pine.analyzeScriptWithLibraries(script, JSON.stringify(libraries)));
    assert.equal(report.executable, test.native_status !== 'analysis_rejected', test.name);
    if (test.native_status === 'analysis_rejected') {
      assert.throws(() => pine.compileScriptWithLibraries(script, JSON.stringify(libraries)));
    } else {
      const program = pine.compileScriptWithLibraries(script, JSON.stringify(libraries));
      try {
        assert.deepEqual(JSON.parse(program.runCsvWithRequestBars(csv(bars), '{}')).plots.map(p => p.values.at(-1)), test.native_values, test.name);
      } finally { program.free(); }
    }
  }
  for (const test of cases) {
    const raw = fs.readFileSync(path.join(root, test.source));
    assert.equal(crypto.createHash('sha256').update(raw).digest('hex'), test.source_sha256);
    const script = raw.toString('utf8');
    const report = JSON.parse(pine.analyzeScript(script));
    if (test.native_status === 'analysis_rejected') {
      assert.equal(report.executable, false, test.name);
      assert.throws(() => pine.compileScript(script));
      continue;
    }
    assert.equal(report.executable, true, test.name);
    const check = wire => assert.deepEqual(JSON.parse(wire).plots.map(p => p.values.at(-1)), test.native_values, test.name);
    check(pine.runScriptCsv(script, csv(bars)));
    const prepared = pine.compileScript(script);
    try {
      check(prepared.runCsvWithRequestBars(csv(bars), '{}'));
      const session = prepared.realtimeSession();
      session.seed(csv(bars.slice(0, 11)));
      const replica = session.replica();
      try {
        for (let tick = 0; tick < 2; tick++) {
          replica.apply(session.applyForming(JSON.stringify(bars[11])));
          check(session.result());
          assert.equal(replica.result(), session.result());
        }
        replica.apply(session.applyConfirmed(JSON.stringify(bars[11])));
        check(session.confirmedResult());
        assert.equal(replica.result(), session.confirmedResult());
      } finally { replica.free(); session.free(); }
    } finally { prepared.free(); }
  }
};

'use strict';
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

module.exports = function (modulePath) {
  const pine = require(path.resolve(modulePath));
  const fixture = path.resolve(__dirname, '../../tests/fixtures/runtime/standard_ohlc');
  const csv = fs.readFileSync(path.join(fixture, 'chart.csv'), 'utf8');
  const source = fs.readFileSync(path.join(fixture, 'strategy.pine'), 'utf8');
  const host = {$chart: {chartType: 'heikinashi'},
    $standardOhlc: JSON.parse(fs.readFileSync(path.join(fixture, 'observations.json'), 'utf8'))};
  const compiled = pine.compileScript(source);
  const result = JSON.parse(compiled.runCsvWithRequestBars(csv, JSON.stringify(host)));
  assert.deepEqual(result.plots[0].values, [1000, 1001, 1002, 1003, 1004]);
  assert.equal(result.strategy.trades[0].entryPrice, 101);
  assert.equal(result.strategy.trades[0].exitPrice, 104);
  delete host.$standardOhlc;
  assert.throws(() => compiled.runCsvWithRequestBars(csv, JSON.stringify(host)), /E_STANDARD_OHLC_INPUT/);

  for (const command of ['entry', 'order']) {
    for (const price of ['na', 'float(na)']) {
      const script = `//@version=6\nstrategy("na")\nif bar_index==0\n    strategy.${command}("E",strategy.long,qty=1,limit=${price},stop=${price})\nif bar_index==3\n    strategy.close_all()\n`;
      const actual = JSON.parse(pine.runScriptCsv(script, csv));
      assert.equal(actual.strategy.trades.length, 1);
      assert.deepEqual(actual.strategy.diagnostics, []);
    }
  }
  const udf = '//@version=5\nindicator("udf")\nemit()=>\n    alert("inside")\n    nz(replacement=7.0,source=close[1])\nplot(emit())\n';
  const actual = JSON.parse(pine.runScriptCsv(udf, csv));
  assert.deepEqual(actual.plots[0].values, [7, 1000, 1001, 1002, 1003]);
  assert.deepEqual(actual.alerts.map(a => a.message), ['inside', 'inside', 'inside', 'inside', 'inside']);
};

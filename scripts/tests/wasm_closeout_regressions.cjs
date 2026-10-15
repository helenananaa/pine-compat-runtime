'use strict';
const assert = require('node:assert/strict');

module.exports = function(modulePath) {
  const pine = require(modulePath);
  const csv = 'time,open,high,low,close,volume\n0,1,1,1,1,1\n';
  const cases = [
    [
      '//@version=6\nindicator("typed sum")\na=matrix.new<int>(1,1,2)\nb=matrix.sum(a,float(na))\nplot(b.get(0,0))\nb.set(0,0,1.5)\nplot(b.get(0,0))\nplot(a.get(0,0))\n',
      [[null], [1.5], [2]],
    ],
    [
      '//@version=6\nindicator("timestamp arguments")\nvar calls=array.new_int()\npart(value)=>\n    array.push(calls,value)\n    value\nplot(timestamp(part(2024),part(1),part(1),part(0),part(0),part(0)))\nplot(array.size(calls))\n',
      [[1704067200000], [6]],
    ],
  ];
  for (const [source, expected] of cases) {
    const prepared = pine.compileScript(source);
    for (const wire of [pine.runScriptCsv(source,csv), prepared.runCsvWithRequestBars(csv,'{}')]) {
      assert.deepEqual(JSON.parse(wire).plots.map(plot => plot.values), expected);
    }
    prepared.free();
  }
};

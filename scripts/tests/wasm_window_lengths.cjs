'use strict';

const assert = require('node:assert/strict');
const path = require('node:path');
const { spawnSync } = require('node:child_process');

function verifyWindowLengths(modulePath) {
  const result = spawnSync(process.execPath, [__filename, path.resolve(modulePath)], {
    encoding: 'utf8', timeout: 45_000, maxBuffer: 1_048_576,
  });
  assert.equal(result.error, undefined, `Wasm window check: ${result.error}`);
  assert.equal(result.status, 0, `Wasm window check failed: ${result.stderr}`);
  assert.equal(result.stdout.trim(), 'Wasm window lengths passed');
}

function checkInWasm(modulePath) {
  const pine = require(path.resolve(modulePath));
  const bars = ['time,open,high,low,close,volume',
    ...Array.from({ length: 10 }, (_, i) => `${i * 60_000},${i + 1},${i + 2},${i},${i + 1},10`),
    ''].join('\n');
  const functions = ['math.sum', 'ta.sma', 'ta.wma', 'ta.hma', 'ta.variance', 'ta.stdev',
    'ta.range', 'ta.dev', 'ta.rci', 'ta.median', 'ta.mode', 'ta.percentrank', 'ta.cog',
    'ta.cci', 'ta.vwma', 'ta.mfi', 'ta.cmo', 'ta.change', 'ta.mom', 'ta.roc', 'ta.rma', 'ta.ema', 'ta.rsi'];
  for (const length of [4_294_967_296, 4_294_967_303]) {
    const expressions = functions.map(name => `${name}(close,${length})`);
    expressions.push(`ta.correlation(close,open,${length})`,
      `ta.covariance(close,open,${length})`, `ta.stoch(close,high,low,${length})`,
      `ta.wpr(${length})`, `ta.alma(close,${length},0.85,6)`, `ta.linreg(close,${length},0)`,
      `ta.bbw(close,${length},2)`, `ta.percentile_linear_interpolation(close,${length},50)`,
      `ta.percentile_nearest_rank(close,${length},50)`, `ta.atr(${length})`,
      `ta.pivothigh(close,${length},0)`, `ta.pivotlow(close,0,${length})`);
    const source = `//@version=6\nindicator("wide lengths")\n` +
      expressions.map(expression => `plot(${expression})`).join('\n') + '\n';
    const result = JSON.parse(pine.runScriptCsv(source, bars));
    assert.equal(result.plots.length, expressions.length);
    result.plots.forEach((plot, i) => assert.deepEqual(plot.values, Array(10).fill(null), expressions[i]));
    const shapes = JSON.parse(pine.runScriptCsv(`//@version=6\nindicator("wide shapes")\n` +
      `[b,u,l]=ta.bb(close,${length},2)\n[d,p,a]=ta.dmi(${length},${length})\n` +
      `[m,s,h]=ta.macd(close,${length},${length},9)\n[st,dir]=ta.supertrend(2,${length})\n` +
      `plot(b)\nplot(u)\nplot(l)\nplot(d)\nplot(p)\nplot(a)\n` +
      `plot(m)\nplot(s)\nplot(h)\nplot(st)\nplot(dir)\n` +
      `plot(ta.rising(close,${length})?1:0)\nplot(ta.falling(close,${length})?1:0)\n`, bars));
    shapes.plots.slice(0, 11).forEach(plot => assert.deepEqual(plot.values, Array(10).fill(null)));
    shapes.plots.slice(11).forEach(plot => assert.deepEqual(plot.values, Array(10).fill(0)));
    const floating = JSON.parse(pine.runScriptCsv(`//@version=6\nindicator("wide scalar smoothing")\n` +
      `[kc,ku,kl]=ta.kc(close,${length},2)\nplot(kc)\nplot(ku)\nplot(kl)\n` +
      `plot(ta.dema(close,${length}))\nplot(ta.tema(close,${length}))\n` +
      `plot(ta.kcw(close,${length},2))\nplot(ta.tsi(close,${length},${length}))\n`, bars));
    floating.plots.slice(0, 6).forEach(plot => assert.ok(plot.values.every(Number.isFinite)));
    assert.ok(Number.isFinite(floating.plots[6].values.at(-1)));
  }
  for (const expression of ['ta.pivothigh(close,4294967295,0)', 'ta.pivotlow(close,0,4294967295)',
    'ta.pivothigh(close,2147483647,2147483648)']) {
    const result = JSON.parse(pine.runScriptCsv(`//@version=6\nindicator("pivot overflow")\nplot(${expression})\n`, bars));
    assert.deepEqual(result.plots[0].values, Array(10).fill(null));
  }
  for (const length of [4_294_967_296, 4_294_967_303]) {
    const resumed = JSON.parse(pine.runScriptCsv(`//@version=6\nindicator("resume windows")\n` +
      `n=bar_index==2?${length}:2\ns=bar_index==2?999:close\n` +
      'plot(ta.wma(s,n))\nplot(ta.variance(s,n))\nplot(ta.sma(s,n))\nplot(math.sum(s,n))\n', bars));
    resumed.plots.forEach(plot => assert.equal(plot.values[2], null));
    assert.deepEqual(resumed.plots.map(plot => plot.values[3]), [10 / 3, 1, 3, 6]);
  }
  const largeBars = ['time,open,high,low,close,volume',
    ...Array.from({ length: 65_791 }, (_, i) => `${i * 60_000},1,1,1,1,10`), ''].join('\n');
  const result = JSON.parse(pine.runScriptCsv('//@version=6\nindicator("wide weights")\n' +
    'plot(ta.wma(close,65536))\nplot(ta.hma(close,65536))\n', largeBars));
  assert.equal(result.plots[0].values[65_534], null);
  assert.ok(result.plots[0].values.slice(65_535).every(value => value === 1));
  assert.equal(result.plots[1].values[65_789], null);
  assert.equal(result.plots[1].values[65_790], 1);
  console.log('Wasm window lengths passed');
}

module.exports = verifyWindowLengths;
if (require.main === module) checkInWasm(process.argv[2]);

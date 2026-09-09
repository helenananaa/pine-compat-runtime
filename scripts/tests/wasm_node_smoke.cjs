'use strict';

const assert = require('node:assert/strict');
const path = require('node:path');

if (process.argv.length !== 3) {
  throw new Error('usage: node wasm_node_smoke.cjs <generated-module.js>');
}

// Requiring wasm-bindgen's Node target synchronously instantiates the real
// WebAssembly.Module and wires its generated JS ABI adapters.
const pine = require(path.resolve(process.argv[2]));

for (const name of ['analyzeScript', 'runScriptCsv', 'compileScript']) {
  assert.equal(typeof pine[name], 'function', `missing Wasm export ${name}`);
}

const source = '//@version=6\nindicator("node smoke")\nplot(close * 2)\n';
const bars = [
  'time,open,high,low,close,volume',
  '0,1,1,1,1,10',
  '1,2,2,2,2,20',
  '2,3,3,3,3,30',
  '',
].join('\n');

const analysis = JSON.parse(pine.analyzeScript(source));
assert.equal(analysis.schemaVersion, 5);
assert.equal(analysis.languageVersion, 6);
assert.equal(analysis.languageVersionOrigin, 'explicit');
assert.equal(analysis.dialect, 'v6');
assert.equal(analysis.scriptMode, 'indicator');
assert.equal(analysis.executable, true);
assert.deepEqual(analysis.diagnostics, []);
assert.deepEqual(analysis.compatibility.legacyTranslations, []);
assert.deepEqual(analysis.compatibility.legacyEmulations, []);
assert.ok(
  analysis.compatibility.supported.some(({ feature }) => feature === 'plot'),
  'analysis should report plot support',
);

const direct = JSON.parse(pine.runScriptCsv(source, bars));
assert.equal(direct.schemaVersion, 8);
assert.equal(direct.renderMetadataVersion, 1);
assert.deepEqual(direct.plots[0].values, [2, 4, 6]);
assert.deepEqual(direct.diagnostics, []);

const program = pine.compileScript(source);
const gridSource = '//@version=6\nindicator("grid")\nplot(syminfo.mintick)\nplot(math.round_to_mintick(10.26))\n';
const gridResult = JSON.parse(pine.runScriptCsvWithRequestBars(gridSource, bars,
  JSON.stringify({ $chart: { minMove: 1, priceScale: 10 } })));
assert.deepEqual(gridResult.plots[0].values, [0.1, 0.1, 0.1]);
assert.ok(gridResult.plots[1].values.every(value => Math.abs(value - 10.3) < 1e-9));
assert.throws(() => pine.runScriptCsvWithRequestBars(gridSource, bars,
  JSON.stringify({ $chart: { minMove: 0, priceScale: 10 } })));
assert.equal(typeof program.runCsv, 'function');
const compiled = JSON.parse(program.runCsv(bars));
const seriesSource = require('node:fs').readFileSync(
  path.resolve(__dirname, '../../tests/fixtures/runtime/series_scalar_parameters.pine'), 'utf8');
const seriesResult = JSON.parse(pine.runScriptCsv(seriesSource, bars));
assert.deepEqual(seriesResult.plots[0].values, [null, 3, 3]);
assert.deepEqual(seriesResult.plots[1].values, [null, 1, 2]);
assert.deepEqual(seriesResult.plots[2].values, [1.5, 1.5, 1.5]);
assert.deepEqual(seriesResult.plots[3].values, [1, 3, 6]);
const defaultsSource = require('node:fs').readFileSync(
  path.resolve(__dirname, '../../tests/fixtures/runtime/function_default_parameters.pine'), 'utf8');
const defaultsResult = JSON.parse(pine.runScriptCsv(defaultsSource, bars));
assert.deepEqual(defaultsResult.plots[0].values, [1.4, 1.4, 1.4]);
assert.deepEqual(defaultsResult.plots[6].values, [2000, 2000, 2000]);
assert.deepEqual(defaultsResult.plots[11].values, [1, 3, 6]);
const invalidDefault = JSON.parse(pine.analyzeScript(
  '//@version=6\nindicator("invalid")\nf(x=1+2) => x\nplot(f())\n'));
assert.equal(invalidDefault.executable, false);
assert.ok(invalidDefault.diagnostics.some(d => d.code === 'E_FUNCTION_DEFAULT'));
const seriesRejection = JSON.parse(pine.analyzeScript(
  '//@version=6\nindicator("series")\nf(series int n) => n\nplot(ta.ema(close,f(3)))\n'));
assert.equal(seriesRejection.executable, false);
assert.ok(seriesRejection.diagnostics.some(d => d.code === 'E_CALL_ARG_TYPE'));
const zeroPyramidingSource = require('node:fs').readFileSync(
  path.resolve(__dirname, '../../tests/fixtures/runtime/strategy_pyramiding_zero.pine'), 'utf8');
const zeroPyramiding = JSON.parse(pine.runScriptCsv(zeroPyramidingSource, bars));
assert.deepEqual(zeroPyramiding.plots[0].values, [0, 1, 1]);
const absentProfitSource = require('node:fs').readFileSync(
  path.resolve(__dirname, '../../tests/fixtures/runtime/strategy_absent_trade_profit.pine'), 'utf8');
const absentProfit = JSON.parse(pine.runScriptCsv(absentProfitSource, bars));
for (const index of [0, 1, 2]) assert.deepEqual(absentProfit.plots[index].values, [0, 0, 0]);
assert.deepEqual(absentProfit.plots[3].values, [null, null, null]);
assert.deepEqual(compiled.plots[0].values, [2, 4, 6]);
assert.deepEqual(compiled, direct);
program.free();

const implicitLegacy = JSON.parse(
  pine.analyzeScript('study("legacy")\nplot(close)\n'),
);
assert.equal(implicitLegacy.languageVersion, 1);
assert.equal(implicitLegacy.languageVersionOrigin, 'implicit');
assert.equal(implicitLegacy.dialect, 'v1');
assert.equal(implicitLegacy.scriptMode, 'legacyIndicator');
assert.equal(implicitLegacy.executable, true);
assert.deepEqual(implicitLegacy.diagnostics, []);
const implicitLegacyRun = JSON.parse(
  pine.runScriptCsv('study("legacy")\nplot(close)\n', bars),
);
assert.deepEqual(implicitLegacyRun.plots[0].values, [1, 2, 3]);

const legacyStrategy = JSON.parse(
  pine.analyzeScript(
    '//@version=4\nstrategy("legacy")\nstrategy.entry("L", strategy.long)\n',
  ),
);
assert.equal(legacyStrategy.scriptMode, 'strategy');
assert.deepEqual(
  legacyStrategy.diagnostics.map(({ code }) => code),
  ['E_LEGACY_STRATEGY_OUT_OF_SCOPE'],
);

const combinedSource = [
  '//@version=6',
  'indicator("combined node smoke")',
  'import user/lib/1 as lib',
  'factor = input.float(1.0, "Factor")',
  'requested = request.security("NYSE:IBM", "1", close)',
  'plot(lib.scale(requested, factor))',
  '',
].join('\n');
const librarySources = JSON.stringify({
  'user/lib/1': [
    '//@version=6',
    'library("lib")',
    'export offset = 1.0',
    'export scale(value, factor) => value * factor + offset',
    '',
  ].join('\n'),
});
const requestBars = JSON.stringify({
  'NYSE:IBM:1': [
    { time: 0, open: 9, high: 11, low: 8, close: 10, volume: 100 },
    { time: 1, open: 19, high: 21, low: 18, close: 20, volume: 200 },
    { time: 2, open: 29, high: 31, low: 28, close: 30, volume: 300 },
  ],
});
const combinedAnalysis = JSON.parse(
  pine.analyzeScriptWithLibraries(combinedSource, librarySources),
);
assert.deepEqual(combinedAnalysis.diagnostics, []);
const factorInput = combinedAnalysis.inputs.find(({ title }) => title === 'Factor');
assert.ok(factorInput, 'combined analysis should expose the Factor input');
assert.equal(typeof factorInput.callSiteId, 'number');
const overrides = JSON.stringify({ [factorInput.callSiteId]: 3.0 });
const combined = JSON.parse(
  pine.runScriptCsvWithLibrariesAndRequestBarsAndInputOverrides(
    combinedSource,
    bars,
    librarySources,
    requestBars,
    overrides,
  ),
);
assert.deepEqual(combined.plots[0].values, [31, 61, 91]);
assert.deepEqual(combined.diagnostics, []);

assert.throws(
  () => pine.compileScript('//@version=6\nindicator("broken")\nplot(unknown_name)\n'),
  (error) => {
    // Result<_, JsValue> is intentionally thrown by wasm-bindgen. JsValue::from_str
    // crosses the boundary as a thrown string rather than an Error instance.
    assert.match(String(error), /unknown_name|unknown identifier/i);
    return true;
  },
);

assert.throws(
  () => pine.runScriptCsv(source, 'time,open,high,low,close,volume\n0,1,2\n'),
  (error) => {
    assert.match(String(error), /invalid bars CSV.*expected 6 columns/i);
    return true;
  },
);

console.log(
  'wasm Node smoke passed: instantiate, analyze, run, compile/run, combined hosts, JS exceptions',
);

'use strict';

const assert = require('node:assert/strict');
const path = require('node:path');

if (process.argv.length !== 3) {
  throw new Error('usage: node wasm_node_smoke.cjs <generated-module.js>');
}

// Requiring wasm-bindgen's Node target synchronously instantiates the real
// WebAssembly.Module and wires its generated JS ABI adapters.
const pine = require(path.resolve(process.argv[2]));

for (const name of ['analyzeScript', 'runScriptCsv', 'compileScript', 'packageVersion']) {
  assert.equal(typeof pine[name], 'function', `missing Wasm export ${name}`);
}
assert.equal(pine.packageVersion(), '0.3.0-rc.1');

const source = '//@version=6\nindicator("node smoke")\nplot(close * 2)\n';
const bars = [
  'time,open,high,low,close,volume',
  '0,1,1,1,1,10',
  '1,2,2,2,2,20',
  '2,3,3,3,3,30',
  '',
].join('\n');

const analysis = JSON.parse(pine.analyzeScript(source));
assert.equal(analysis.schemaVersion, 6);
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
assert.equal(direct.schemaVersion, 9);
assert.equal(direct.renderMetadataVersion, 1);
assert.deepEqual(direct.plots[0].values, [2, 4, 6]);
assert.deepEqual(direct.diagnostics, []);

const program = pine.compileScript(source);
const requirements = JSON.parse(program.hostRequirements());
assert.equal(requirements.schemaVersion, 2);
{
  const mergeProgram=pine.compileScript('//@version=6\nindicator("merge inventory")\ng=barmerge.gaps_on\nplot(request.security("OTHER","M",close,gaps=g,lookahead=barmerge.lookahead_on))\n');
  const report=JSON.parse(mergeProgram.hostRequirements());
  assert.equal(report.requests[0].gaps,'gapsOn');
  assert.equal(report.requests[0].lookahead,'lookaheadOn');
  assert.equal(report.requests[0].timeframeRelation,'sameOrHigherIntegerMultipleExceptCalendarMonths');
  mergeProgram.free();
}
assert.equal(requirements.chart.bars, 'hostSuppliedStandardOhlcv');
const compiledRun = JSON.parse(program.runCsv(bars));
assert.deepEqual(compiledRun.plots[0].values, [2, 4, 6]);
const owned = compiledRun.plots[0].values.slice();
compiledRun.plots[0].values[0] = 999;
assert.deepEqual(JSON.parse(program.runCsv(bars)).plots[0].values, owned);
const rejected = (() => {
  try {
    pine.runScriptCsv('//@version=6\nindicator("bad")\nplot(unknown_name)\n', bars);
    return null;
  } catch (error) {
    return String(error);
  }
})();
assert.match(rejected, /unknown_name|unknown identifier/i);
const quantitySource = require('node:fs').readFileSync(path.resolve(__dirname, '../../tests/fixtures/runtime/quantity_precision.pine'), 'utf8');
const quantityBars = require('node:fs').readFileSync(path.resolve(__dirname, '../../tests/fixtures/runtime/quantity_precision_bars.csv'), 'utf8');
const quantityExpected = JSON.parse(require('node:fs').readFileSync(path.resolve(__dirname, '../../tests/snapshots/runtime_quantity_precision.json'), 'utf8'));
assert.deepEqual(JSON.parse(pine.runScriptCsvWithRequestBars(quantitySource, quantityBars,
  JSON.stringify({$chart:{minMove:1,priceScale:10,quantityPrecision:6,pointValue:1}}))), quantityExpected);
const simpleSource = require('node:fs').readFileSync(
  path.resolve(__dirname, '../../tests/fixtures/runtime/simple_scalar_parameters.pine'), 'utf8');
const simpleBars = require('node:fs').readFileSync(
  path.resolve(__dirname, '../../tests/fixtures/runtime/bars.csv'), 'utf8');
const simpleExpected = JSON.parse(require('node:fs').readFileSync(
  path.resolve(__dirname, '../../tests/snapshots/runtime_simple_scalar_parameters.json'), 'utf8'));
assert.deepEqual(JSON.parse(pine.runScriptCsv(simpleSource, simpleBars)), simpleExpected);
const macdSource = require('node:fs').readFileSync(
  path.resolve(__dirname, '../../tests/fixtures/runtime/macd.pine'), 'utf8');
const macdExpected = JSON.parse(require('node:fs').readFileSync(
  path.resolve(__dirname, '../../tests/snapshots/runtime_macd.json'), 'utf8'));
assert.deepEqual(JSON.parse(pine.runScriptCsv(macdSource, simpleBars)), macdExpected);
const nearbySource = require('node:fs').readFileSync(
  path.resolve(__dirname, '../../tests/fixtures/runtime/sma_nearby_replacement.pine'), 'utf8');
const nearbyBars = require('node:fs').readFileSync(
  path.resolve(__dirname, '../../tests/fixtures/runtime/macd_edge_cases_bars.csv'), 'utf8');
const nearbyExpected = JSON.parse(require('node:fs').readFileSync(
  path.resolve(__dirname, '../../tests/snapshots/runtime_sma_nearby_replacement.json'), 'utf8'));
assert.deepEqual(JSON.parse(pine.runScriptCsv(nearbySource, nearbyBars)), nearbyExpected);
const comparisonSource = require('node:fs').readFileSync(
  path.resolve(__dirname, '../../tests/fixtures/runtime/numeric_comparison.pine'), 'utf8');
const comparisonExpected = JSON.parse(require('node:fs').readFileSync(
  path.resolve(__dirname, '../../tests/snapshots/runtime_numeric_comparison.json'), 'utf8'));
assert.deepEqual(JSON.parse(pine.runScriptCsv(comparisonSource, simpleBars)), comparisonExpected);
const libraryFormsSource = require('node:fs').readFileSync(
  path.resolve(__dirname, '../../tests/fixtures/runtime/library_declaration_forms.pine'), 'utf8');
const libraryFormsExpected = JSON.parse(require('node:fs').readFileSync(
  path.resolve(__dirname, '../../tests/snapshots/runtime_library_declaration_forms.json'), 'utf8'));
assert.deepEqual(JSON.parse(pine.runScriptCsv(libraryFormsSource, simpleBars)), libraryFormsExpected);
const transitiveSource = require('node:fs').readFileSync(
  path.resolve(__dirname, '../../tests/fixtures/runtime/transitive_imports.pine'), 'utf8');
const transitiveLibraries = Object.fromEntries(['inner', 'outer'].map(name => [
  `user/transitive_${name}/1`, require('node:fs').readFileSync(
    path.resolve(__dirname, `../../tests/fixtures/libraries/transitive_${name}_lib.pine`), 'utf8')
]));
const transitiveExpected = JSON.parse(require('node:fs').readFileSync(
  path.resolve(__dirname, '../../tests/snapshots/runtime_transitive_imports.json'), 'utf8'));
assert.deepEqual(JSON.parse(pine.runScriptCsvWithLibraries(transitiveSource, simpleBars, JSON.stringify(transitiveLibraries))), transitiveExpected);
const overloadSource = require('node:fs').readFileSync(
  path.resolve(__dirname, '../../tests/fixtures/runtime/scalar_overloads.pine'), 'utf8');
const overloadLibraries = { 'test/scalar_overloads/1': require('node:fs').readFileSync(
  path.resolve(__dirname, '../../tests/fixtures/libraries/scalar_overloads_lib.pine'), 'utf8') };
const overloadExpected = JSON.parse(require('node:fs').readFileSync(
  path.resolve(__dirname, '../../tests/snapshots/runtime_scalar_overloads.json'), 'utf8'));
assert.deepEqual(JSON.parse(pine.runScriptCsvWithLibraries(overloadSource, simpleBars, JSON.stringify(overloadLibraries))), overloadExpected);
const simpleRejection = JSON.parse(pine.analyzeScript(
  '//@version=6\nindicator("simple")\nf(simple float x) => x\nplot(f(close))\n'));
assert.equal(simpleRejection.executable, false);
assert.ok(simpleRejection.diagnostics.some(d => d.code === 'E_FUNCTION_ARG_TYPE'));
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

assert.equal(typeof program.realtimeSession, 'function');
assert.equal(pine.runtimeChangesSchemaVersion(), 4);
assert.equal(pine.realtimeSessionSchemaVersion(), 1);
const realtime = program.realtimeSession();
const seededRealtime = JSON.parse(realtime.seed(bars));
assert.deepEqual(seededRealtime.plots[0].values, [2, 4, 6]);
const replica = realtime.replica();
const forming = JSON.parse(realtime.applyForming(JSON.stringify({
  time: 3, open: 4, high: 4, low: 4, close: 4, volume: 1,
})));
assert.equal(forming.visibility, 'preview');
assert.equal(forming.schemaVersion, 4);
assert.equal(replica.apply(JSON.stringify(forming)), true);
assert.deepEqual(JSON.parse(replica.result()).plots[0].values, JSON.parse(realtime.result()).plots[0].values);
assert.equal(replica.apply(JSON.stringify(forming)), false);
const confirmed = JSON.parse(realtime.applyConfirmed(JSON.stringify({
  time: 3, open: 4, high: 4, low: 4, close: 4, volume: 1,
})));
assert.equal(confirmed.visibility, 'confirmed');
assert.equal(replica.apply(JSON.stringify(confirmed)), true);
assert.deepEqual(JSON.parse(realtime.result()).plots[0].values, [2, 4, 6, 8]);
const replayed = JSON.parse(realtime.replay(bars));
assert.deepEqual(replayed.plots[0].values, [2, 4, 6]);
assert.equal(realtime.formingTime, undefined);
const corrected = JSON.parse(realtime.correct(1, [
  'time,open,high,low,close,volume',
  '1,2.5,2.5,2.5,2.5,20',
  '2,3,3,3,3,30',
  '',
].join('\n')));
assert.deepEqual(corrected.plots[0].values, [2, 5, 6]);
realtime.free();
replica.free();
program.free();

// Physical prefix pruning and alert retention through the real generated module.
{
  const code = '//@version=6\nindicator("bounded")\nalert("event", alert.freq_all)\nplot(close)';
  const compiled = pine.compileScript(code);
  const session = compiled.realtimeSession();
  session.setOutputRetention(4);
  const csv = 'time,open,high,low,close,volume\n' + Array.from({length: 400}, (_, i) =>
    `${i * 60000},${i+100},${i+100},${i+100},${i+100},1`).join('\n') + '\n';
  session.seed(csv);
  const consumer = session.replica();
  for (let i = 400; i < 412; ++i) {
    const value = i + 100;
    const update = JSON.stringify({time:i*60000, open:value, high:value, low:value, close:value, volume:1});
    consumer.apply(session.applyConfirmed(update));
    const result = JSON.parse(session.result());
    assert.equal(result.plots[0].values.length, 4);
    assert.equal(result.alerts.length, 4);
    assert.deepStrictEqual(JSON.parse(consumer.result()), result);
  }
  session.clearOutputRetention();
  assert.equal(session.displayOrigin, 408);
  consumer.free(); session.free(); compiled.free();
}

// Stateful requested context: replacements reuse history but preserve results.
{
  const compiled = pine.compileScript('//@version=6\nindicator("feed")\nplot(request.security("B","5",ta.sma(close,3)))');
  const makeBar = (time, value) => ({time,open:value,high:value,low:value,close:value,volume:1});
  const session = compiled.realtimeSessionWithRequestBars(JSON.stringify({'B:5': [makeBar(0,1),makeBar(300000,2),makeBar(600000,3)]}));
  session.seed('time,open,high,low,close,volume\n0,1,1,1,1,1\n');
  const consumer = session.replica();
  assert.equal(session.applyRequestForming('B','5',JSON.stringify(makeBar(900000,4))), 'null');
  consumer.apply(session.applyForming(JSON.stringify(makeBar(1140000,99))));
  assert.equal(JSON.parse(consumer.result()).plots[0].values.at(-1), 3);
  consumer.apply(session.applyRequestForming('B','5',JSON.stringify(makeBar(900000,7))));
  assert.equal(JSON.parse(consumer.result()).plots[0].values.at(-1), 4);
  consumer.apply(session.applyRequestConfirmed('B','5',JSON.stringify(makeBar(900000,7))));
  consumer.apply(session.applyConfirmed(JSON.stringify(makeBar(1140000,99))));
  assert.deepStrictEqual(JSON.parse(consumer.result()), JSON.parse(session.result()));
  consumer.free(); session.free(); compiled.free();
}

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
assert.equal(legacyStrategy.executable, true);
assert.deepEqual(legacyStrategy.diagnostics, []);

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



const requirementsFs = require('node:fs');
const requirementsRoot = path.resolve(__dirname, '../..');
const requirementsSource = requirementsFs.readFileSync(path.join(requirementsRoot, 'tests/fixtures/host_requirements/strategy.pine'), 'utf8');
const requirementsProgram = pine.compileScript(requirementsSource);
const requirementsExpected = JSON.parse(requirementsFs.readFileSync(path.join(requirementsRoot, 'tests/snapshots/host_requirements.json'), 'utf8'));
const requirementsSpans = JSON.parse(requirementsFs.readFileSync(path.join(requirementsRoot, 'tests/fixtures/host_requirements/source_spans.json'), 'utf8'));
const requirementsBytes = Buffer.from(requirementsSource, 'utf8');
requirementsExpected.callSites = requirementsSpans.map(({callSiteId, text}) => {
  const bytes = Buffer.from(text, 'utf8');
  const start = requirementsBytes.indexOf(bytes);
  assert(start >= 0);
  assert.equal(requirementsBytes.indexOf(bytes, start + 1), -1);
  return {callSiteId, source: {sourceId: 0, libraryKey: null, start, end: start + bytes.length}};
});
assert.deepStrictEqual(JSON.parse(requirementsProgram.hostRequirements()), requirementsExpected);
requirementsProgram.free();

for (const newline of ['\n', '\r\n']) {
  const expression = 'request.security("REMOTE","60",close)';
  const original = ['//@version=6', '// 中文', 'indicator("origin")', `plot(${expression})`, ''].join(newline);
  const compiled = pine.compileScript(original);
  const report = JSON.parse(compiled.hostRequirements());
  const start = Buffer.from(original, 'utf8').indexOf(Buffer.from(expression));
  assert.deepStrictEqual(report.callSites, [{callSiteId: report.requests[0].callSiteId,
    source: {sourceId: 0, libraryKey: null, start, end: start + Buffer.byteLength(expression)}}]);
  compiled.free();
}

for (const pointValue of [0, -1, 0.5, 5, 1.0000000001, true, null, '1']) {
  assert.throws(() => pine.runScriptCsvWithRequestBars(source, bars, JSON.stringify({$chart: {pointValue}})), /pointValue/);
}

// A function mutates caller-owned arrays through namespace, method and nested
// alias paths. Expected terminal values were observed in native v5/v6 charts.
for (const version of [5, 6]) {
  const pushSource = requirementsFs.readFileSync(path.join(requirementsRoot, 'tests/fixtures/runtime/udf_array_push.pine'), 'utf8').replace('version=6', `version=${version}`);
  const pushBars = 'time,open,high,low,close,volume\n' + Array.from({length: 8}, (_, i) => `${i * 60000},10,10,10,10,1`).join('\n') + '\n';
  const pushResult = JSON.parse(pine.runScriptCsv(pushSource, pushBars));
  assert.deepEqual(pushResult.plots.map(plot => plot.values.at(-1)), [1, 3, 19, 9, 10, 5]);
}

const activeMetadataSource = requirementsFs.readFileSync(path.join(requirementsRoot, 'tests/fixtures/runtime/input_active_metadata.pine'), 'utf8');
const activeMetadataResult = JSON.parse(pine.runScriptCsv(activeMetadataSource, bars));
assert.deepEqual(activeMetadataResult.plots.map(plot => plot.values[0]), [7, 2.5, 1]);
assert.equal(activeMetadataResult.plots[0].editable, false);
assert.equal(JSON.parse(pine.analyzeScript(activeMetadataSource.replace('version=6', 'version=5'))).executable, false);

// Exercise the actual generated module through physical gradient pruning.
const gradientSource = requirementsFs.readFileSync(path.join(requirementsRoot, 'tests/fixtures/runtime/gradient_fill.pine'), 'utf8');
const gradientProgram = pine.compileScript(gradientSource);
const gradientSession = gradientProgram.realtimeSession();
gradientSession.setOutputRetention(7);
gradientSession.seed('time,open,high,low,close,volume\n0,10,10,10,10,1\n');
const gradientReplica = gradientSession.replica();
for (let i = 1; i < 270; i++) {
  for (const [method, price] of [['applyForming', 20], ['applyForming', 30], ['applyConfirmed', 40]]) {
    const encoded = gradientSession[method](JSON.stringify({time: i * 60000, open: price, high: price, low: price, close: price, volume: 1}));
    gradientReplica.apply(encoded);
    const result = JSON.parse(gradientSession.result());
    assert.deepEqual(JSON.parse(gradientReplica.result()), result);
    assert.ok(result.fills[0].gradient.length <= 8);
    assert.equal(result.fills[0].gradient.at(-1).bottomValue, price - 1);
    if (i === 1 && price === 20) {
      const old = JSON.parse(encoded);
      old.schemaVersion = 3;
      assert.throws(() => gradientReplica.apply(JSON.stringify(old)), /schema/);
    }
  }
}
gradientReplica.free();
gradientSession.free();
gradientProgram.free();

const pivotNaSource = requirementsFs.readFileSync(path.join(requirementsRoot, 'tests/fixtures/runtime/pivot_na_boundaries.pine'), 'utf8');
const pivotNaExpected = JSON.parse(requirementsFs.readFileSync(path.join(requirementsRoot, 'tests/fixtures/pivot_na_native_values.json'), 'utf8'));
const pivotNaBars = 'time,open,high,low,close,volume\n' + Array.from({length: 109}, (_, i) => `${i * 60000},10,10,10,10,1`).join('\n') + '\n';
for (const version of [5, 6]) {
  const pivotNaResult = JSON.parse(pine.runScriptCsv(pivotNaSource.replace('version=6', `version=${version}`), pivotNaBars));
  for (const plot of pivotNaResult.plots) assert.deepEqual(plot.values, pivotNaExpected[plot.title]);
}

const memberAccessSource = requirementsFs.readFileSync(path.join(requirementsRoot, 'tests/fixtures/runtime/member_access.pine'), 'utf8');
for (const version of [5, 6]) {
  const memberAccessResult = JSON.parse(pine.runScriptCsv(memberAccessSource.replace('version=6', `version=${version}`), bars));
  assert.deepEqual(memberAccessResult.plots[0].values,[1,2,3]);
  assert.deepEqual(memberAccessResult.plots[1].values,[1,2,3]);
  assert.deepEqual(memberAccessResult.plots[2].values,[null,1,2]);
  assert.deepEqual(memberAccessResult.plots[4].values,[1,2,3]);
  assert.deepEqual(memberAccessResult.plots[5].values,[null,1,2]);
  assert.equal(memberAccessResult.lines.length,3);
  const undefinedMemberSource = `//@version=${version}\nindicator("undefined")\ntype Item\n    float value\nItem item=na\nplot(item.value)\n`;
  assert.throws(() => pine.runScriptCsv(undefinedMemberSource, bars), /E_UDT_NA_FIELD/);
}

const tupleFinalSource = requirementsFs.readFileSync(path.join(requirementsRoot, 'tests/fixtures/runtime/tuple_final_declaration.pine'), 'utf8');
for (const version of [5, 6]) {
  const result = JSON.parse(pine.runScriptCsv(tupleFinalSource.replace('version=6', `version=${version}`), bars));
  assert.deepEqual(result.plots[4].values, [1,2,3]);
  assert.deepEqual(result.plots[8].values, [1,2,3]);
  assert.deepEqual(result.plots[9].values, [10,20,30]);
}

const udtIdentitySource = requirementsFs.readFileSync(path.join(requirementsRoot, 'tests/fixtures/runtime/udt_reference_identity.pine'), 'utf8');
for (const version of [5, 6]) {
  const result = JSON.parse(pine.runScriptCsv(udtIdentitySource.replace('version=6', `version=${version}`), bars));
  assert.deepEqual(result.plots[0].values, [2,3,4]);
  assert.deepEqual(result.plots[3].values, [4,5,6]);
  assert.deepEqual(result.plots[5].values, [null,1,2]);
}
const udtVaripProgram = pine.compileScript(requirementsFs.readFileSync(path.join(requirementsRoot, 'tests/fixtures/runtime/udt_field_varip.pine'), 'utf8'));
const udtVaripSession = udtVaripProgram.realtimeSession();
udtVaripSession.seed('time,open,high,low,close,volume\n0,10,10,10,10,1\n');
const udtVaripReplica = udtVaripSession.replica();
for (let tick=1; tick<=3; tick++) {
  udtVaripReplica.apply(udtVaripSession.applyForming(JSON.stringify({time:60000,open:11,high:11,low:11,close:11,volume:1})));
  const result = JSON.parse(udtVaripSession.result());
  assert.deepEqual(JSON.parse(udtVaripReplica.result()), result);
  assert.deepEqual(result.plots.map(plot=>plot.values[1]), [2,1+tick,2,1+tick,tick,tick,1]);
}
udtVaripReplica.free(); udtVaripSession.free(); udtVaripProgram.free();

{
  const precisionProgram = pine.compileScript('//@version=6\nindicator("wire precision")\nplot(close)\nlabel.new(bar_index,close)\nline.new(bar_index,close,bar_index+1,close)\n');
  const session = precisionProgram.realtimeSession();
  session.seed('time,open,high,low,close,volume\n0,1,1,1,1,1\n');
  const consumer = session.replica();
  const close = 96118.61666666665;
  const update = JSON.stringify({time:60000,open:close,high:close,low:close,close,volume:1});
  consumer.apply(session.applyForming(update));
  assert.deepEqual(JSON.parse(consumer.result()), JSON.parse(session.result()));
  consumer.apply(session.applyConfirmed(update));
  assert.deepEqual(JSON.parse(consumer.result()), JSON.parse(session.result()));
  const complete = consumer.result();
  assert.equal(consumer.intoResult(), complete);
  assert.throws(()=>consumer.result());
  session.free(); precisionProgram.free();
}

{
  const output = JSON.parse(pine.runScriptCsvWithRequestBars(
    '//@version=6\nindicator("month context")\nplot(timeframe.in_seconds())\nplot(timeframe.in_seconds(""))\nplot(timeframe.in_seconds("12M"))\nplot(timeframe.from_seconds(2628003)=="1M"?1:0)\nplot(timeframe.from_seconds(61)=="2"?1:0)\n',
    'time,open,high,low,close,volume\n1704067200000,1,1,1,1,1\n',
    JSON.stringify({$chart:{symbol:'BTC',timeframe:'1M',minMove:1,priceScale:100}}),
  ));
  assert.deepEqual(output.plots.map(p=>p.values),[[2628003],[2628003],[31536036],[1],[1]]);
}

console.log(
  'wasm Node smoke passed: instantiate, analyze, run, compile/run, combined hosts, JS exceptions',
);

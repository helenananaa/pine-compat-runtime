'use strict';

const assert = require('node:assert/strict');
const path = require('node:path');

if (process.argv.length !== 3) {
  throw new Error('usage: node wasm_node_smoke.cjs <generated-module.js>');
}

// Requiring wasm-bindgen's Node target synchronously instantiates the real
// WebAssembly.Module and wires its generated JS ABI adapters.
const pine = require(path.resolve(process.argv[2]));
assert.equal(pine.externalScheduleSchemaVersion(), 1);
assert.equal(pine.externalStreamSchemaVersion(), 1);
assert.equal(pine.externalRequestWindowPlanSchemaVersion(), 1);
assert.equal(pine.externalSizingSchemaVersion(), 1);
assert.equal(pine.externalOrderMetadataSchemaVersion(), 1);
require('./wasm_history_ticks.cjs')(process.argv[2]);
require('./wasm_realtime_request_feed.cjs')(process.argv[2]);
require('./wasm_closeout_regressions.cjs')(path.resolve(process.argv[2]));
require('./wasm_bool_control_defaults.cjs')(process.argv[2]);
require('./wasm_reserved_identifiers.cjs')(process.argv[2]);
require('./wasm_constant_bool_identities.cjs')(process.argv[2]);
require('./wasm_constant_parameters.cjs')(process.argv[2]);
require('./wasm_table_capacity.cjs')(process.argv[2]);
require('./wasm_missing_strings.cjs')(process.argv[2]);
require('./wasm_method_receivers.cjs')(process.argv[2]);
require('./wasm_alert_qualifiers.cjs')(process.argv[2]);
require('./wasm_point_references.cjs')(process.argv[2]);
require('./wasm_array_persistence.cjs')(process.argv[2]);
require('./wasm_request_reference_captures.cjs')(process.argv[2]);
require('./wasm_request_context_windows.cjs')(process.argv[2]);
require('./wasm_collection_parameters.cjs')(process.argv[2]);
require('./wasm_function_body_declarations.cjs')(process.argv[2]);

// Namespace resolution is shared by direct and prepared WASM executions.
for (const version of [3,4]) {
  const source=`//@version=${version}\nstudy("namespaces")\nmacd=7.0\natr=11.0\nsma=19.0\nf(sma)=>\n    ema=13.0\n    sma(close,2)+ema(close,2)+sma+ema\n[m,s,h]=macd(close,2,3,2)\nplot(m+s+h)\nplot(atr(2))\nplot(f(17))\nplot(macd+atr+sma)\n`;
  let canonical=source.replace(`version=${version}`,'version=5').replace('study(','indicator(');
  for(const name of ['sma','ema','macd','atr']) canonical=canonical.replaceAll(`${name}(`,`ta.${name}(`);
  const csv='time,open,high,low,close,volume\n0,10,10,10,10,1\n60000,20,20,20,20,1\n120000,30,30,30,30,1\n';
  const expected=JSON.parse(pine.runScriptCsv(canonical,csv)).plots;
  const prepared=pine.compileScript(source);
  assert.deepEqual(JSON.parse(pine.runScriptCsv(source,csv)).plots,expected);
  assert.deepEqual(JSON.parse(prepared.runCsvWithRequestBars(csv,'{}')).plots,expected);
}
{
  const source='//@version=4\nstudy("pvt")\nplot(pvt)\nplot(pvt[1])\n';
  const canonical=source.replace('version=4','version=5').replace('study(','indicator(').replaceAll('pvt','ta.pvt');
  const csv='time,open,high,low,close,volume\n0,10,10,10,10,1\n60000,20,20,20,20,1\n120000,30,30,30,30,1\n';
  const expected=JSON.parse(pine.runScriptCsv(canonical,csv)).plots;
  assert.deepEqual(JSON.parse(pine.runScriptCsv(source,csv)).plots,expected);
  assert.deepEqual(JSON.parse(pine.compileScript(source).runCsvWithRequestBars(csv,'{}')).plots,expected);
}
for (const timeframe of ['D','W','M','5']) {
  const source='//@version=6\nindicator("lower close")\nplot(time_close("1S")-time)\nplot(time_close("30S","0000-0001","UTC")-time)\nplot(time_close("30S",timeframe_bars_back=-1)-time)\nplot(time_close("30S",timeframe_bars_back=1)-time)\n';
  const csv='time,open,high,low,close,volume\n1704067200000,10,10,10,10,1\n';
  const requests=JSON.stringify({$chart:{symbol:'UTC:TEST',timeframe}});
  const expected=timeframe==='5'?[[1000],[30000],[60000],[0]]:[[86400000],[60000],[86400000],[0]];
  assert.deepEqual(JSON.parse(pine.runScriptCsvWithRequestBars(source,csv,requests)).plots.map(p=>p.values),expected);
  assert.deepEqual(JSON.parse(pine.compileScript(source).runCsvWithRequestBars(csv,requests)).plots.map(p=>p.values),expected);
}
require('./wasm_window_lengths.cjs')(process.argv[2]);
require('./wasm_bar_calendars.cjs')(process.argv[2]);
require('./wasm_output_transparency.cjs')(process.argv[2]);
require('./wasm_legacy_ternary_bool.cjs')(process.argv[2]);
require('./wasm_legacy_strategy_constants.cjs')(process.argv[2]);
require('./wasm_legacy_direction_truth.cjs')(process.argv[2]);
require('./wasm_strategy_close_all_source.cjs')(process.argv[2]);
require('./wasm_strategy_equity_quantity.cjs')(process.argv[2]);
require('./wasm_strategy_currency_native.cjs')(process.argv[2]);
require('./wasm_literal_alert_braces.cjs')(process.argv[2]);
require('./wasm_alert_templates.cjs')(process.argv[2]);

for (const name of ['analyzeScript', 'runScriptCsv', 'compileScript', 'packageVersion']) {
  assert.equal(typeof pine[name], 'function', `missing Wasm export ${name}`);
}
assert.equal(pine.packageVersion(), '0.3.2');

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
assert.equal(requirements.schemaVersion, 4);
{
  const mergeProgram=pine.compileScript('//@version=6\nindicator("merge inventory")\ng=barmerge.gaps_on\nplot(request.security("OTHER","M",close,gaps=g,lookahead=barmerge.lookahead_on))\n');
  const report=JSON.parse(mergeProgram.hostRequirements());
  assert.equal(report.requests[0].gaps,'gapsOn');
  assert.equal(report.requests[0].lookahead,'lookaheadOn');
  assert.equal(report.requests[0].timeframeRelation,'sameOrLowerOrHigher');
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

// State-only seeding crosses the generated JS ABI without returning a snapshot.
// Snapshot envelopes preserve the exact result bytes, including UTF-8 and escapes.
{
  const text = '图🚀 café "quoted"\\path\nnext';
  const code = [
    '//@version=6',
    'indicator("state seed unicode")',
    'var total = 0.0',
    'total += close',
    `plot(total, title=${JSON.stringify(text)})`,
    `label.new(bar_index, close, text=${JSON.stringify(text)})`,
    `alert(${JSON.stringify(text)}, alert.freq_all)`,
    '',
  ].join('\n');
  const compiled = pine.compileScript(code);
  const ordinary = compiled.realtimeSession();
  const stateOnly = compiled.realtimeSession();
  ordinary.setOutputRetention(2);
  stateOnly.setOutputRetention(2);
  assert.equal(typeof stateOnly.seedState, 'function');
  const ordinarySeed = ordinary.seed(bars);
  assert.equal(stateOnly.seedState(bars), undefined);
  assert.equal(stateOnly.isSeeded, true);
  assert.equal(stateOnly.confirmedBars, 3);
  assert.equal(stateOnly.lastConfirmedTime, 2);
  assert.equal(stateOnly.formingTime, undefined);
  assert.equal(stateOnly.lastChanges(), 'null');
  assert.equal(stateOnly.revision, ordinary.revision);
  assert.equal(stateOnly.displayOrigin, 1);
  assert.equal(stateOnly.result(), ordinarySeed);
  assert.equal(stateOnly.confirmedResult(), ordinary.confirmedResult());
  assert.throws(() => stateOnly.seedState(bars), /already been seeded/);
  const consumer = stateOnly.replica();
  assert.equal(consumer.result(), stateOnly.result());

  const assertEnvelope = () => {
    const result = stateOnly.result();
    const encoded = stateOnly.streamSnapshot();
    const expected = `{"revision":${stateOnly.revision},"retainedFrom":${stateOnly.displayOrigin},"result":${result}}`;
    assert.deepEqual(Buffer.from(encoded, 'utf8'), Buffer.from(expected, 'utf8'));
    const snapshot = JSON.parse(encoded);
    assert.deepEqual(Object.keys(snapshot), ['revision', 'retainedFrom', 'result']);
    assert.equal(snapshot.revision, Number(stateOnly.revision));
    assert.equal(snapshot.retainedFrom, stateOnly.displayOrigin);
    assert.deepEqual(snapshot.result, JSON.parse(result));
    assert.equal(snapshot.result.plots[0].title, text);
    assert.equal(snapshot.result.alerts.at(-1).message, text);
    assert.equal(snapshot.result.labels.at(-1).snapshots.at(-1).text, text);
    assert.ok(Buffer.byteLength(encoded, 'utf8') > encoded.length);
  };
  assertEnvelope();
  for (const [method, close] of [['applyForming', 4], ['applyForming', 5], ['applyConfirmed', 5]]) {
    const update = JSON.stringify({time:3, open:close, high:close, low:close, close, volume:1});
    const changes = stateOnly[method](update);
    assert.equal(changes, ordinary[method](update));
    assert.equal(consumer.apply(changes), true);
    assert.equal(consumer.result(), stateOnly.result());
    assert.equal(stateOnly.result(), ordinary.result());
    assertEnvelope();
  }
  consumer.free(); stateOnly.free(); ordinary.free(); compiled.free();
}

// Execution timestamps supplied with state-only seed remain the replay clock.
{
  const compiled = pine.compileScript('//@version=6\nindicator("state seed clock")\nplot(timenow)\nplot(timenow-time)\n');
  const ordinary = compiled.realtimeSession();
  const stateOnly = compiled.realtimeSession();
  assert.equal(typeof stateOnly.seedStateWithExecutionTimes, 'function');
  assert.throws(() => stateOnly.seedState(bars), /explicit execution timestamp/);
  assert.equal(stateOnly.isSeeded, false);
  assert.equal(stateOnly.confirmedBars, 0);
  assert.throws(() => stateOnly.seedStateWithExecutionTimes(bars, '[101]'), /count 1.*bar count 3/);
  assert.equal(stateOnly.isSeeded, false);
  assert.equal(stateOnly.confirmedBars, 0);
  const times = '[101,202,303]';
  const ordinarySeed = ordinary.seedWithExecutionTimes(bars, times);
  assert.equal(stateOnly.seedStateWithExecutionTimes(bars, times), undefined);
  assert.equal(stateOnly.result(), ordinarySeed);
  assert.deepEqual(JSON.parse(ordinarySeed).plots.map(plot => plot.values), [[101,202,303], [101,201,301]]);
  assert.equal(stateOnly.revision, ordinary.revision);
  const consumer = stateOnly.replica();
  const update = JSON.stringify({time:3, open:4, high:4, low:4, close:4, volume:1});
  const context = JSON.stringify({executionTime:707, openingUpdate:true});
  const changes = stateOnly.applyFormingWithContext(update, context);
  assert.equal(changes, ordinary.applyFormingWithContext(update, context));
  assert.equal(consumer.apply(changes), true);
  assert.equal(consumer.result(), stateOnly.result());
  assert.equal(JSON.parse(stateOnly.result()).plots[0].values.at(-1), 707);
  const envelope = JSON.parse(stateOnly.streamSnapshot());
  assert.deepEqual(envelope.result, JSON.parse(consumer.result()));
  assert.equal(envelope.revision, Number(consumer.revision));
  consumer.free(); stateOnly.free(); ordinary.free(); compiled.free();
}

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

for (const pointValue of [0, -1, true, null, '1']) {
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


// Frontend resource rejection and library origins cross the real Wasm ABI.
{
  const makeBar = (time, close) => ({time, open:close, high:close, low:close, close, volume:1});
  const chart = Array.from({length:8}, (_,i) => makeBar(i*60000,10+i));
  const csv = 'time,open,high,low,close,volume\n' + chart.map(b=>Object.values(b).join(',')).join('\n')+'\n';
  const supplied = JSON.stringify({$chart:{symbol:'A',timeframe:'1'},'B:4':[makeBar(0,100),makeBar(240000,200)]});
  for (const version of [5,6]) {
    for (const [dynamic,tail] of [[false,[13,13,13,13,17]],[true,[100,100,100,100,200]]]) {
      const source = `//@version=${version}\nindicator("nested",dynamic_requests=${dynamic})\ninner=request.security("","",close)\nplot(request.security("B","4",inner))\n`;
      assert.deepEqual(JSON.parse(pine.runScriptCsvWithRequestBars(source,csv,supplied)).plots[0].values,[null,null,null,...tail]);
      const prepared = pine.compileScript(source);
      assert.deepEqual(JSON.parse(prepared.runCsvWithRequestBars(csv,supplied)).plots[0].values,[null,null,null,...tail]);
      prepared.free();
    }
  }
}

// Named nested requests retain static/dynamic context through direct/prepared APIs.
{
  const bar = time => ({time,open:100,high:100,low:100,close:100,volume:1});
  const csv = 'time,open,high,low,close,volume\n' + Array.from({length:4},(_,i)=>Object.values(bar(i*60000)).join(',')).join('\n')+'\n';
  const supplied = JSON.stringify({$chart:{symbol:'A',timeframe:'1'},'B:4':[bar(0)]});
  for (const version of [5,6]) {
    for (const dynamic of [false,true]) {
      const source = `//@version=${version}\nindicator("named nested",dynamic_requests=${dynamic})\ninner=request.security(expression=timeframe.multiplier,timeframe="",symbol=syminfo.tickerid)\nmerged=request.security(gaps=barmerge.gaps_on,symbol=syminfo.tickerid,timeframe="",expression=timeframe.multiplier,lookahead=barmerge.lookahead_off)\nplot(inner)\nplot(request.security(expression=inner,timeframe="4",symbol="B"))\nplot(merged)\nplot(request.security(expression=merged,timeframe="4",symbol="B"))\n`;
      const outer = [null,null,null,dynamic?4:1];
      const expected = [[1,1,1,1],outer,[1,1,1,1],outer];
      assert.deepEqual(JSON.parse(pine.runScriptCsvWithRequestBars(source,csv,supplied)).plots.map(p=>p.values),expected);
      const prepared = pine.compileScript(source);
      assert.deepEqual(JSON.parse(prepared.runCsvWithRequestBars(csv,supplied)).plots.map(p=>p.values),expected);
      prepared.free();
    }
  }
}

// Non-integral higher periods preserve merge events and static series at the ABI.
{
  const bar = time => ({time,open:100,high:100,low:100,close:100,volume:1});
  const csv = 'time,open,high,low,close,volume\n' + Array.from({length:4},(_,i)=>Object.values(bar(i*300000)).join(',')).join('\n')+'\n';
  const supplied = JSON.stringify({$chart:{symbol:'A',timeframe:'5'},'A:6':Array.from({length:4},(_,i)=>bar(i*360000))});
  for (const version of [5,6]) {
    const source = `//@version=${version}\nindicator("uneven",dynamic_requests=false)\ninner=request.security("","",time)\nplot(request.security("A","6",time))\nplot(request.security("A","6",time,gaps=barmerge.gaps_on))\nplot(request.security("A","6",time,gaps=barmerge.gaps_on,lookahead=barmerge.lookahead_on))\nplot(request.security("A","6",inner))\n`;
    const expected = [[null,0,360000,720000],[null,0,360000,720000],[0,null,360000,720000],[null,0,300000,600000]];
    assert.deepEqual(JSON.parse(pine.runScriptCsvWithRequestBars(source,csv,supplied)).plots.map(p=>p.values),expected);
    const prepared = pine.compileScript(source);
    assert.deepEqual(JSON.parse(prepared.runCsvWithRequestBars(csv,supplied)).plots.map(p=>p.values),expected);
    prepared.free();
  }
}

// Source input selections are evaluated against requested OHLC through both APIs.
{
  const chart = Array.from({length:4},(_,i)=>({time:i*60000,open:4,high:5,low:3,close:4,volume:1}));
  const csv = 'time,open,high,low,close,volume\n' + chart.map(b=>Object.values(b).join(',')).join('\n')+'\n';
  const supplied = JSON.stringify({$chart:{symbol:'A',timeframe:'1'},'B:2':[
    {time:0,open:30,high:40,low:20,close:30,volume:1},
    {time:120000,open:70,high:80,low:60,close:70,volume:1},
  ]});
  for (const version of [5,6]) {
    for (const callee of ['input','input.source']) {
      const source = `//@version=${version}\nindicator("requested source")\nsrc=${callee}(title="Source",defval=hl2)\nplot(src)\nplot(request.security("B","2",src))\n`;
      const id = JSON.parse(pine.analyzeScript(source)).inputs[0].callSiteId;
      const prepared = pine.compileScript(source);
      for (const [overrides,expected] of [[{},[[4,4,4,4],[null,30,30,70]]],[{[id]:'high'},[[5,5,5,5],[null,40,40,80]]]]) {
        const input = JSON.stringify(overrides);
        const direct = JSON.parse(pine.runScriptCsvWithLibrariesAndRequestBarsAndInputOverrides(source,csv,'{}',supplied,input));
        assert.deepEqual(direct.plots.map(p=>p.values),expected);
        assert.deepEqual(JSON.parse(prepared.runCsvWithRequestBarsAndInputOverrides(csv,supplied,input)).plots.map(p=>p.values),expected);
      }
      prepared.free();
    }
  }
}

// Parameter specialization and legacy tuple UDFs cross direct/prepared Wasm APIs.
{
  const bar = (time,close) => ({time,open:close,high:close,low:close,close,volume:1});
  const csv = 'time,open,high,low,close,volume\n' + Array.from({length:4},(_,i)=>Object.values(bar(i*60000,1+i)).join(',')).join('\n')+'\n';
  const supplied = JSON.stringify({$chart:{symbol:'A',timeframe:'1'},'B:2':[bar(0,10),bar(120000,20)]});
  for (const version of [4,5,6]) {
    const body = version === 4
      ? 'study("tuple")\nf(src)=>\n    alias=src+timeframe.multiplier\n    [alias,time]\n[a,b]=security("B","2",f(close))\nplot(a)\nplot(b)\n'
      : 'indicator("parameters")\nf(src)=>\n    alias=src+timeframe.multiplier\n    request.security("B","2",alias)\ng(src)=>f(src)\nplot(g(close))\nplot(f(src=10))\n';
    const source = `//@version=${version}\n${body}`;
    const expected = [[null,12,12,22],version===4?[null,0,0,120000]:[null,12,12,12]];
    assert.deepEqual(JSON.parse(pine.runScriptCsvWithRequestBars(source,csv,supplied)).plots.map(p=>p.values),expected);
    const prepared = pine.compileScript(source);
    assert.deepEqual(JSON.parse(prepared.runCsvWithRequestBars(csv,supplied)).plots.map(p=>p.values),expected);
    prepared.free();
  }
}

// Requested local state and loop history stay isolated across direct/prepared APIs.
{
  const bar = (time,close) => ({time,open:close,high:close,low:close,close,volume:1});
  const csv = 'time,open,high,low,close,volume\n' + Array.from({length:8},(_,i)=>Object.values(bar(i*60000,1+i)).join(',')).join('\n')+'\n';
  const supplied = JSON.stringify({$chart:{symbol:'A',timeframe:'1'},'B:2':Array.from({length:4},(_,i)=>bar(i*120000,10*(i+1)))});
  for (const version of [4,5,6]) {
    const body = version===4
      ? 'study("iff history")\nf()=>\n    pos=0.0\n    pos:=iff(close>0,nz(pos[1])+1,0)\n    pos\na=f()\nplot(security("B","2",a[1]))\n'
      : 'indicator("requested loops")\nf(src)=>\n    total=0.0\n    for i=0 to 2\n        total+=src[i]\n    total\ng(src)=>\n    total=0.0\n    i=0\n    while i<3\n        total+=src[i]\n        i+=1\n    total\nplot(request.security("B","2",f(close)))\nplot(request.security("B","2",g(close)))\n';
    const source = `//@version=${version}\n${body}`;
    const expected = version===4 ? [[null,null,null,1,1,2,2,3]] : Array.from({length:2},()=>[null,null,null,null,null,60,60,90]);
    assert.deepEqual(JSON.parse(pine.runScriptCsvWithRequestBars(source,csv,supplied)).plots.map(p=>p.values),expected);
    const prepared = pine.compileScript(source);
    assert.deepEqual(JSON.parse(prepared.runCsvWithRequestBars(csv,supplied)).plots.map(p=>p.values),expected);
    prepared.free();
  }
}

// Scalar inputs and fill-NA state replay with provider values, including legacy aliases.
{
  const bar=(time,close)=>({time,open:close,high:close,low:close,close,volume:1});
  const csv='time,open,high,low,close,volume\n'+Array.from({length:6},(_,i)=>Object.values(bar(i*60000,i+1)).join(',')).join('\n')+'\n';
  const supplied=JSON.stringify({$chart:{symbol:'A',timeframe:'1'},'B:2':Array.from({length:3},(_,i)=>bar(i*120000,10*(i+1)))});
  for (const version of [4,5,6]) {
    const request=version===4?'security':'request.security';
    const sma=version===4?'sma':'ta.sma';
    const vwap=version===4?'vwap':'ta.vwap';
    const input=version===4?'input':'input.int';
    const source=`//@version=${version}\n${version===4?'study':'indicator'}("scalar inputs")\nf(src)=>fixnan(src>15 ? na : src)\nplot(${request}("B","2",${sma}(close,${input}(title="Length",defval=2))))\nplot(${request}("B","2",f(close)))\nplot(${request}("B","2",${vwap}))\n`;
    const expected=[[null,null,null,15,15,25],[null,10,10,10,10,10],[null,10,10,15,15,20]];
    assert.deepEqual(JSON.parse(pine.runScriptCsvWithRequestBars(source,csv,supplied)).plots.map(p=>p.values),expected);
    const prepared=pine.compileScript(source);
    assert.deepEqual(JSON.parse(prepared.runCsvWithRequestBars(csv,supplied)).plots.map(p=>p.values),expected);
    prepared.free();
  }
}

// Distinct bindings and static wrappers preserve requested value graphs.
{
  const bar=(time,close)=>({time,open:close,high:close,low:close,close,volume:1});
  const csv='time,open,high,low,close,volume\n'+Array.from({length:6},(_,i)=>Object.values(bar(i*60000,i+1)).join(',')).join('\n')+'\n';
  const supplied=JSON.stringify({$chart:{symbol:'A',timeframe:'1'},'B:2':Array.from({length:3},(_,i)=>bar(i*120000,10*(i+1)))});
  for (const version of [4,5,6]) {
    const request=version===4?'security':'request.security';
    const source=`//@version=${version}\n${version===4?'study':'indicator'}("shadow graph")\nf(src)=>\n    value=0.0\n    value:=nz(value[1])+src\n    value\nvalue=f(close)\nplot(value)\nplot(${request}("B","2",value))\n`;
    const expected=[[1,3,6,10,15,21],[null,10,10,30,30,60]];
    assert.deepEqual(JSON.parse(pine.runScriptCsvWithRequestBars(source,csv,supplied)).plots.map(p=>p.values),expected);
    const prepared=pine.compileScript(source);
    assert.deepEqual(JSON.parse(prepared.runCsvWithRequestBars(csv,supplied)).plots.map(p=>p.values),expected);
    prepared.free();
  }
  for (const version of [5,6]) {
    const source=`//@version=${version}\nindicator("value graph",dynamic_requests=false)\nf(src)=>request.security("B","2",src)\na=f(close)\nb=f(a)\nplot(a)\nplot(b)\n[x,y,z]=request.security("B","2",[math.pi,color.r(color.new(color.rgb(12,34,56),25)),minute(time)+second(time_close)])\nplot(x)\nplot(y)\nplot(z)\n`;
    const expected=[[null,10,10,20,20,30],[null,10,10,20,20,30],[null,...Array(5).fill(Math.PI)],[null,12,12,12,12,12],[null,0,0,2,2,4]];
    assert.deepEqual(JSON.parse(pine.runScriptCsvWithRequestBars(source,csv,supplied)).plots.map(p=>p.values),expected);
    const prepared=pine.compileScript(source);
    assert.deepEqual(JSON.parse(prepared.runCsvWithRequestBars(csv,supplied)).plots.map(p=>p.values),expected);
    prepared.free();
  }
}

const deepStatementSource = '//@version=6\nindicator("depth")\nif true\n    x = 1\n'
  + 'else if false\n    x = 2\n'.repeat(256);
const deepStatementReport = JSON.parse(pine.analyzeScript(deepStatementSource));
assert.equal(deepStatementReport.executable, false);
assert.ok(deepStatementReport.diagnostics.some(({ code }) =>
  code === 'E_PARSE_STMT_DEPTH' || code === 'E_PARSE_EXPR_DEPTH'));
const diagnosticRoot = '//@version=6\nindicator("根")\nimport audit/Library/1 as lib\nplot(lib.f(close))\n';
const diagnosticLibrary = '//@version=6\nlibrary("库")\n// 中文\nexport f(float x) => str.length("中文") + missingName + x\n';
const diagnosticLibraries = JSON.stringify({ 'audit/Library/1': diagnosticLibrary });
const libraryReport = JSON.parse(pine.analyzeScriptWithLibraries(diagnosticRoot, diagnosticLibraries));
const libraryDiagnostic = libraryReport.diagnostics.find(({ code }) => code === 'E_UNKNOWN_SYMBOL');
assert.ok(libraryDiagnostic);
const libraryOffset = diagnosticLibrary.indexOf('missingName');
assert.deepEqual(libraryDiagnostic.span, {
  start: Buffer.byteLength(diagnosticLibrary.slice(0, libraryOffset), 'utf8'),
  end: Buffer.byteLength(diagnosticLibrary.slice(0, libraryOffset + 'missingName'.length), 'utf8'),
  line: 4,
  column: [...diagnosticLibrary.split('\n')[3].split('missingName')[0]].length + 1,
  sourceId: 1,
  sourceName: '<wasm:audit/Library/1>',
  libraryKey: 'audit/Library/1',
});
assert.throws(() => pine.compileScriptWithLibraries(diagnosticRoot, diagnosticLibraries), /audit\/Library\/1.*:4:/);
const metadataSource = `//@version=6
indicator("const metadata")
const int base = 3
const string caption = "Length"
length = input.int(base + 2, caption, minval=base, maxval=base * 3, step=base - 2, options=[1, base + 2, 7])
pi = input.float(math.pi, "Pi")
root = input.float(math.sqrt(9), "Root")
plot(length + pi + root)
`;
const metadataReport = JSON.parse(pine.analyzeScript(metadataSource));
assert.deepEqual(metadataReport.diagnostics, []);
assert.equal(metadataReport.inputs[0].title, 'Length');
assert.equal(metadataReport.inputs[0].default, 5);
assert.equal(metadataReport.inputs[0].min, 3);
assert.equal(metadataReport.inputs[0].max, 9);
assert.equal(metadataReport.inputs[0].step, 1);
assert.deepEqual(metadataReport.inputs[0].options, [1, 5, 7]);
assert.equal(metadataReport.inputs[1].default, Math.PI);
assert.equal(metadataReport.inputs[2].default, 3);
const metadataResult = JSON.parse(pine.runScriptCsv(metadataSource, bars));
assert.deepEqual(metadataResult.plots[0].values, [8 + Math.PI, 8 + Math.PI, 8 + Math.PI]);


// UDF inputs retain source declaration identities through direct/prepared APIs.
{
  for (const version of [3,4,5,6]) {
    const declaration=version<=4?'study':'indicator';
    const number=version===3?'input(2,"Length",type=integer)':version===4?'input(2,"Length",type=input.integer)':'input.int(2,"Length")';
    const sourceInput=version===3?'input(close,"Source",type=source)':version===4?'input(close,"Source",type=input.source)':'input.source(close,"Source")';
    const source=`//@version=${version}\n${declaration}("shared input")\nf()=>\n    n=${number}\n    s=${sourceInput}\n    n+s\na=f()\nb=f()\nplot(a)\nplot(b)\n`;
    const report=JSON.parse(pine.analyzeScript(source));
    assert.deepEqual(report.diagnostics,[]);
    assert.equal(report.inputs.length,2);
    const length=report.inputs.find(x=>x.title==='Length').callSiteId;
    const selected=report.inputs.find(x=>x.title==='Source').callSiteId;
    const csv='time,open,high,low,close,volume\n0,1,3,0,1,1\n60000,2,4,1,2,1\n120000,3,5,2,3,1\n';
    const input=JSON.stringify({[length]:7,[selected]:'high'});
    const direct=JSON.parse(pine.runScriptCsvWithLibrariesAndRequestBarsAndInputOverrides(source,csv,'{}','{}',input));
    assert.deepEqual(direct.plots.map(p=>p.values),[[10,11,12],[10,11,12]]);
    const prepared=pine.compileScript(source);
    assert.deepEqual(JSON.parse(prepared.runCsvWithRequestBarsAndInputOverrides(csv,'{}',input)).plots.map(p=>p.values),[[10,11,12],[10,11,12]]);
    prepared.free();
  }
}

// Numeric history indices and date/session compatibility cross the real ABI.
{
  const source='//@version=6\nindicator("history and dates")\nplot(close[2.8])\nplot(close[-0.8])\nplot(close[close%3+0.8])\nplot(timestamp(999999,12,31,23,59))\nplot(timestamp("Jan 19 2024 00:00:01"))\nplot(time("", "0000-2400:1234567", "UTC"))\nplot(time("", "2400-1600:1234567", "UTC"))\nplot(time("30S"))\nplot(time("S"))\nplot(timestamp("2021-01-13:05:00"))\nplot(timestamp("01 Jan 2023 00:00 -3000"))\n';
  const csv='time,open,high,low,close,volume\n0,1,1,1,1,1\n60000,2,2,2,2,1\n120000,3,3,3,3,1\n';
  const expected=[[null,null,1],[1,2,3],[null,null,3],Array(3).fill(31494784780740000),Array(3).fill(1705622401000),[0,60000,120000],[0,60000,120000],[0,60000,120000],[0,60000,120000],Array(3).fill(1610514000000),Array(3).fill(1672639200000)];
  const direct=JSON.parse(pine.runScriptCsvWithLibrariesAndRequestBarsAndInputOverrides(source,csv,'{}','{}','{}'));
  assert.deepEqual(direct.plots.map(p=>p.values),expected);
  const prepared=pine.compileScript(source);
  assert.deepEqual(JSON.parse(prepared.runCsvWithRequestBarsAndInputOverrides(csv,'{}','{}')).plots.map(p=>p.values),expected);
  prepared.free();
}

// Legacy source binding crosses both direct and prepared real Wasm APIs.
{
  const csv='time,open,high,low,close,volume\n0,10,10,10,10,1\n60000,11,11,11,11,1\n120000,12,12,12,12,1\n180000,13,13,13,13,1\n';
  const source='//@version=4\nstudy("legacy input and alias")\na=input(2,defval=1,title="A")\nc=input(1,title="First",title="Last",group="First group",group="Last group")\nf(x)=>x+1\ng()=>f(input(2,"Nested")+1)\nplot(a)\nplot(c)\nplot(g())\nplot(g()+g())\nplot(swma(close))\nplot(tonumber("2.5"))\n';
  const report=JSON.parse(pine.analyzeScript(source));
  assert.deepEqual(report.diagnostics,[]);
  assert.equal(report.inputs.length,3);
  assert.ok(report.inputs.some(x=>x.title==='Last'));
  const expected=[[2,2,2,2],[1,1,1,1],[4,4,4,4],[8,8,8,8],[null,null,null,11.5],[2.5,2.5,2.5,2.5]];
  const cases=[[source,expected]];
  for (const version of [4,5,6]) {
    const prefix=version<=5?'true,':'';
    const tail=version>=5?',false,false':'';
    cases.push([`//@version=${version}\nstrategy("close source positions")\nif bar_index==0\n    strategy.entry("E",strategy.long,qty=2)\nif bar_index==2\n    strategy.close("E",${prefix}"half",1,na,"half alert"${tail})\nplot(strategy.position_size)\n`,[[0,2,2,1]]]);
  }
  cases.push(['//@version=4\nstrategy("numeric direction")\nif bar_index==0\n    strategy.entry("L",1,qty=2)\nif bar_index==2\n    strategy.entry("S",0,qty=1)\nplot(strategy.position_size)\n',[[0,2,2,-1]]]);
  for (const [text,values] of cases) {
    const direct=JSON.parse(pine.runScriptCsvWithLibrariesAndRequestBarsAndInputOverrides(text,csv,'{}','{}','{}'));
    assert.deepEqual(direct.plots.map(p=>p.values),values);
    const prepared=pine.compileScript(text);
    assert.deepEqual(JSON.parse(prepared.runCsvWithRequestBarsAndInputOverrides(csv,'{}','{}')).plots.map(p=>p.values),values);
    prepared.free();
  }
}

// Represented integer EMA aliases retain fractions outside the length slot.
{
  const csv='time,open,high,low,close,volume\n0,10,10,10,10,1\n60000,11,11,11,11,1\n120000,12,12,12,12,1\n180000,13,13,13,13,1\n';
  for (const version of [2,3,4]) {
    const source=`//@version=${version}\nstrategy("legacy order",default_qty_value=2)\nif bar_index==0\n    strategy.order("L",long=true)\nif bar_index==2\n    strategy.order("S",0,qty=3)\nplot(strategy.position_size)\n`;
    const prepared=pine.compileScript(source);
    for (const wire of [pine.runScriptCsv(source,csv),prepared.runCsvWithRequestBarsAndInputOverrides(csv,'{}','{}')]) {
      assert.deepEqual(JSON.parse(wire).plots[0].values,[0,2,2,-1]);
    }
    prepared.free();
    for (const modern of [5,6]) assert.equal(JSON.parse(pine.analyzeScript(source.replace(`version=${version}`,`version=${modern}`))).executable,false);
  }
  for (const version of [4,5,6]) {
    const declaration=version===4?'study':'indicator';
    const source=`//@version=${version}\n${declaration}("table defaults")\nvar t=table.new(position.top_right,3,3)\nif bar_index==0\n    for c=0 to 2\n        for r=0 to 2\n            table.cell(t,c,r,"keep")\n    table.clear(t,0,0)\n    table.clear(t,1,1,2)\n    table.clear(t,0,1,end_row=2)\nplot(close)\n`;
    const prepared=pine.compileScript(source);
    for (const wire of [pine.runScriptCsv(source,csv),prepared.runCsvWithRequestBarsAndInputOverrides(csv,'{}','{}')]) {
      assert.deepEqual(JSON.parse(wire).tables[0].snapshots.at(-1).cells.map(c=>[c.column,c.row]),[[1,0],[2,0],[1,2],[2,2]]);
    }
    prepared.free();
  }
}

// Represented integer EMA aliases retain fractions outside the length slot.
{
  const csv='time,open,high,low,close,volume\n0,10,10,10,10,1\n60000,11,11,11,11,1\n120000,12,12,12,12,1\n180000,13,13,13,13,1\n';
  for (const version of [4,5,6]) {
    const declaration=version===4?'study':'indicator';
    const ema=version===4?'ema':'ta.ema';
    const source=`//@version=${version}\n${declaration}("represented EMA")\nn=input(3)\na=n/2\nb=a*2\nplot(a)\nplot(${ema}(close,a))\nplot(${ema}(close,1))\nplot(${ema}(close,b))\nplot(${ema}(close,3))\n`;
    const prepared=pine.compileScript(source);
    for (const wire of [pine.runScriptCsvWithLibrariesAndRequestBarsAndInputOverrides(source,csv,'{}','{}','{}'),prepared.runCsvWithRequestBarsAndInputOverrides(csv,'{}','{}')]) {
      const values=JSON.parse(wire).plots.map(p=>p.values);
      assert.deepEqual(values[0],[1.5,1.5,1.5,1.5]);
      assert.deepEqual(values[1],values[2]);
      assert.deepEqual(values[3],values[4]);
      assert.notEqual(values[3].at(-1),null);
    }
    prepared.free();
    assert.equal(JSON.parse(pine.analyzeScript(`//@version=${version}\n${declaration}("bad length")\nn=input(3)\na=n/2.0\nplot(${ema}(close,a))\n`)).executable,false);
  }
  const source='//@version=4\nstudy("resolution")\ntf=input("60",title="Timeframe",type=input.resolution,options=["1","60","D"])\nplot(tf=="60"?1:0)\n';
  assert.deepEqual(JSON.parse(pine.analyzeScript(source)).inputs[0].options,['1','60','D']);
  assert.deepEqual(JSON.parse(pine.runScriptCsv(source,csv)).plots[0].values,[1,1,1,1]);
}

// Compact extreme floats preserve their IEEE-754 values through the real Wasm
// ABI, including borrowed realtime snapshots and parsed change replicas.
{
  const extremeSource = '//@version=6\nindicator("compact floats")\nplot(close)\nlabel.new(bar_index,close)\n';
  const values = [Number.MAX_VALUE, -Number.MAX_VALUE, Number.MIN_VALUE, -Number.MIN_VALUE, 1e-308, -0];
  const bits = value => {
    const view = new DataView(new ArrayBuffer(8));
    view.setFloat64(0, value);
    return view.getBigUint64(0);
  };
  const checkValues = (wire, expected) => {
    const result = JSON.parse(wire);
    assert.equal(result.schemaVersion, 9);
    assert.equal(result.renderMetadataVersion, 1);
    assert.deepEqual(result.diagnostics, []);
    assert.deepEqual(result.plots[0].values.map(bits), expected.map(bits));
    const tokens = wire.match(/"values":\[([^\]]*)\]/)[1].split(',');
    assert.ok(tokens.every(token => token.length <= 24));
    return result;
  };
  const csv = ['time,open,high,low,close,volume', ...values.map((value, time) => {
    const text = Object.is(value, -0) ? '-0' : String(value);
    return `${time},${text},${text},${text},${text},1`;
  }), ''].join('\n');
  const directWire = pine.runScriptCsv(extremeSource, csv);
  checkValues(directWire, values);
  assert.ok(directWire.includes('1.7976931348623157e+308'));
  assert.ok(directWire.includes('-5e-324'));
  const extremeProgram = pine.compileScript(extremeSource);
  assert.equal(extremeProgram.runCsv(csv), directWire);
  const session = extremeProgram.realtimeSession();
  assert.equal(session.seed(csv), directWire);
  const consumer = session.replica();
  const confirmedBefore = session.confirmedResult();
  for (const close of [1e308, -1e308, Number.MIN_VALUE]) {
    const bar = JSON.stringify({ time: values.length, open: close, high: close, low: close, close, volume: 1 });
    const changes = session.applyForming(bar);
    assert.equal(consumer.apply(changes), true);
    assert.equal(consumer.apply(changes), false);
    const visible = session.result();
    const result = checkValues(visible, [...values, close]);
    assert.equal(bits(result.labels.at(-1).snapshots.at(-1).y), bits(close));
    assert.equal(consumer.result(), visible);
    assert.equal(session.confirmedResult(), confirmedBefore);
  }
  const close = Number.MIN_VALUE;
  const changes = session.applyConfirmed(JSON.stringify({ time: values.length, open: close, high: close, low: close, close, volume: 1 }));
  assert.equal(consumer.apply(changes), true);
  const complete = session.result();
  checkValues(complete, [...values, close]);
  assert.equal(consumer.result(), complete);
  assert.equal(session.confirmedResult(), complete);
  consumer.free();
  session.free();
  extremeProgram.free();
}

// i64 occurrences must be validated before conversion to wasm32 usize. Long
// series histories also exercise persistent valuewhen checkpoints in the ABI.
{
  const valuewhenSource = `//@version=6
indicator("valuewhen offset and checkpoints")
huge = 4294967296 + bar_index % 2
plot(ta.valuewhen(true, close, 4294967296))
plot(ta.valuewhen(true, close, 4294967303))
plot(ta.valuewhen(true, close, 9223372036854775807))
plot(ta.valuewhen(true, close, huge))
plot(ta.valuewhen(true, close, 0))
plot(ta.valuewhen(true, close, 7))
plot(ta.valuewhen(true, close, bar_index % 257))
`;
  const count = 260;
  const csv = ['time,open,high,low,close,volume', ...Array.from({length:count}, (_, index) =>
    `${index},${index + 1},${index + 1},${index + 1},${index + 1},1`), ''].join('\n');
  const expected = [
    ...Array.from({length:4}, () => Array(count).fill(null)),
    Array.from({length:count}, (_, index) => index + 1),
    Array.from({length:count}, (_, index) => index < 7 ? null : index - 6),
    Array.from({length:count}, (_, index) => index + 1 - index % 257),
  ];
  for (const version of [5,6]) {
    const result = JSON.parse(pine.runScriptCsv(valuewhenSource.replace('version=6', `version=${version}`), csv));
    assert.deepEqual(result.diagnostics, []);
    assert.deepEqual(result.plots.map(plot => plot.values), expected);
  }
  const valuewhenProgram = pine.compileScript(valuewhenSource);
  const session = valuewhenProgram.realtimeSession();
  assert.deepEqual(JSON.parse(session.seed(csv)).plots.map(plot => plot.values), expected);
  const confirmedBefore = session.confirmedResult();
  const consumer = session.replica();
  for (const close of [700001,700002]) {
    const update = JSON.stringify({time:count, open:close, high:close, low:close, close, volume:1});
    const delta = session.applyForming(update);
    assert.equal(consumer.apply(delta), true);
    assert.equal(consumer.apply(delta), false);
    const visible = session.result();
    const result = JSON.parse(visible);
    assert.deepEqual(result.plots.map(plot => plot.values.at(-1)), [null,null,null,null,close,254,258]);
    assert.deepEqual(result.plots.map(plot => plot.values.slice(0,count)), expected);
    assert.equal(consumer.result(), visible);
    assert.equal(session.confirmedResult(), confirmedBefore);
  }
  const delta = session.applyConfirmed(JSON.stringify({time:count, open:700002, high:700002,
    low:700002, close:700002, volume:1}));
  assert.equal(consumer.apply(delta), true);
  assert.equal(consumer.apply(delta), false);
  assert.equal(consumer.result(), session.result());
  assert.equal(session.confirmedResult(), session.result());
  assert.deepEqual(JSON.parse(session.result()).plots.map(plot => plot.values.at(-1)),
    [null,null,null,null,700002,254,258]);
  consumer.free();
  session.free();
  valuewhenProgram.free();
}

// Logical event limits are optional and preserve failed forming/replay state.
{
  const program = pine.compileScript(`//@version=6
indicator("logical event budget")
plot(ta.valuewhen(true, close, bar_index % 3))
plot(ta.valuewhen(true, close * 2, bar_index % 3))
`);
  const session = program.realtimeSession();
  const csv = n => ['time,open,high,low,close,volume', ...Array.from({length:n}, (_, i) =>
    `${i * 60000},${i + 1},${i + 1},${i + 1},${i + 1},1`), ''].join('\n');
  const bar = (i, close) => JSON.stringify({time:i * 60000, open:close, high:close, low:close, close, volume:1});
  assert.ok(session.valueWhenLimit() == null);
  session.seed(csv(2));
  session.setValueWhenLimit(6);
  session.applyForming(bar(2,3));
  session.applyForming(bar(2,4));
  assert.equal(session.valueWhenRetainedValues(), 6);
  assert.equal(session.confirmedValueWhenRetainedValues(), 4);
  const before = session.result(), cache = session.lastChanges(), revision = session.revision;
  for (const invalid of [true,false,-1,1.5,"6",NaN,Infinity,4294967296,9007199254740992]) {
    assert.throws(() => session.setValueWhenLimit(invalid), /maxValues/);
    assert.equal(session.valueWhenLimit(),6);
    assert.equal(session.result(),before);
  }
  assert.throws(() => session.setValueWhenLimit(5), /E_VALUEWHEN_BUDGET/);
  assert.equal(session.lastChanges(),cache);
  assert.equal(session.revision,revision);
  session.applyConfirmed(bar(2,4));
  const confirmed = session.result(), confirmedCache = session.lastChanges(), confirmedRevision = session.revision;
  assert.throws(() => session.applyForming(bar(3,5)), /E_VALUEWHEN_BUDGET/);
  assert.equal(session.result(),confirmed);
  assert.equal(session.lastChanges(),confirmedCache);
  assert.equal(session.revision,confirmedRevision);
  session.setValueWhenLimit(8);
  session.applyForming(bar(3,5));
  session.applyConfirmed(bar(3,5));
  assert.equal(session.confirmedValueWhenRetainedValues(),8);
  const replayBefore = session.result();
  assert.throws(() => session.replay(csv(5)), /E_VALUEWHEN_BUDGET/);
  assert.equal(session.result(),replayBefore);
  assert.equal(session.valueWhenLimit(),8);
  session.setValueWhenLimit(null);
  session.replay(csv(5));
  assert.equal(session.valueWhenRetainedValues(),10);
  assert.ok(session.valueWhenLimit() == null);
  session.free();
  program.free();
}

// Ref-backed conversion still returns independent JSON and all duplicate alerts.
{
  const count = 257, prefix = '告警🧠'.repeat(600);
  const program = pine.compileScript(`//@version=6
indicator("borrowed alert conversion")
if barstate.isrealtime
    for iteration = 0 to ${count-1}
        alert("${prefix}" + (close == 17 ? "A" : "B"), alert.freq_all)
plot(close)
`);
  const session = program.realtimeSession();
  session.seed('time,open,high,low,close,volume\n0,1,1,1,1,1\n');
  const replica = session.replica();
  const bar = close => JSON.stringify({time:60000,open:close,high:close,low:close,close,volume:1});
  const firstWire = session.applyForming(bar(17)), first = JSON.parse(firstWire);
  assert.equal(first.alerts.length,count);
  assert.ok(first.alerts.every(change => change.action === 'add' && change.event.message === prefix+'A'));
  assert.equal(replica.apply(firstWire),true);
  assert.equal(replica.apply(firstWire),false);
  first.alerts[0].event.message = 'independent decoded object';
  assert.equal(JSON.parse(session.lastChanges()).alerts[0].event.message,prefix+'A');
  const replacementWire = session.applyForming(bar(18)), replacement = JSON.parse(replacementWire);
  assert.equal(replacement.alerts.length,count*2);
  assert.deepEqual(replacement.alerts.map(change => change.action),[
    ...Array(count).fill('remove'),...Array(count).fill('add')]);
  assert.deepEqual(replacement.alerts.map(change => change.event.message),[
    ...Array(count).fill(prefix+'A'),...Array(count).fill(prefix+'B')]);
  assert.equal(replica.apply(replacementWire),true);
  assert.equal(replica.apply(replacementWire),false);
  assert.equal(replica.result(),session.result());
  const confirmation = session.applyConfirmed(bar(18));
  assert.equal(replica.apply(confirmation),true);
  assert.equal(replica.result(),session.confirmedResult());
  assert.equal(JSON.parse(session.confirmedResult()).alerts.length,count);
  replica.free();
  session.free();
  program.free();
}

// Implicit assignment returns and optional else survive both WASM entrypoints.
{
  const csv='time,open,high,low,close,volume\n0,10,10,10,10,1\n60000,11,11,11,11,1\n120000,12,12,12,12,1\n180000,13,13,13,13,1\n';
  for (const version of [4,5,6]) {
    const declaration=version===4?'study':'indicator';
    const boolProbe=version===6?'b==false?1:0':'na(b)?1:0';
    const source=`//@version=${version}\n${declaration}("block returns")\nf(n)=>\n    float s=0\n    if n>0\n        for i=1 to n\n            s:=s+i\n    else\n        s:=0\nx=0\ny=if bar_index%2==0\n    x+=7\nq=for i=0 to 2\n    x+=i\nb=if bar_index<0\n    true\nplot(f(3))\nplot(y)\nplot(q)\nplot(x)\nplot(${boolProbe})\n`;
    const prepared=pine.compileScript(source);
    for (const wire of [pine.runScriptCsv(source,csv),prepared.runCsvWithRequestBarsAndInputOverrides(csv,'{}','{}')]) {
      assert.deepEqual(JSON.parse(wire).plots.map(p=>p.values),[[6,6,6,6],[7,null,7,null],[10,3,10,3],[10,3,10,3],[1,1,1,1]]);
    }
    prepared.free();
    const zero=`//@version=${version}\nstrategy("zero default",default_qty_type=strategy.percent_of_equity,default_qty_value=0)\nif bar_index==0\n    strategy.entry("Zero",strategy.long)\nif bar_index==2\n    strategy.entry("Explicit",strategy.long,qty=1)\nplot(strategy.position_size)\n`;
    const zeroPrepared=pine.compileScript(zero);
    for (const wire of [pine.runScriptCsv(zero,csv),zeroPrepared.runCsvWithRequestBarsAndInputOverrides(csv,'{}','{}')]) {
      assert.deepEqual(JSON.parse(wire).plots[0].values,[0,0,0,1]);
    }
    zeroPrepared.free();
    assert.equal(JSON.parse(pine.analyzeScript(zero.replace('default_qty_value=0','default_qty_value=-1'))).executable,false);
  }
}

// Numeric for bounds and current strategy defaults share direct/prepared semantics.
{
  const csv='time,open,high,low,close,volume\n0,10,10,10,10,1\n60000,11,11,11,11,1\n120000,12,12,12,12,1\n';
  for (const version of [4,5,6]) {
    const source=`//@version=${version}\n${version===4?'study':'indicator'}("float loops")\na=0.0\nb=0\nfor i=0.8 to 3.8 by 0.5\n    a+=i\nfor j=-3 to -0.8\n    b+=1\nplot(a)\nplot(b)\n`;
    const prepared=pine.compileScript(source);
    for (const wire of [pine.runScriptCsv(source,csv),prepared.runCsvWithRequestBarsAndInputOverrides(csv,'{}','{}')]) {
      const plots=JSON.parse(wire).plots;
      assert.ok(plots[0].values.every(v=>Math.abs(v-16.1)<1e-12));
      assert.deepEqual(plots[1].values,[3,3,3]);
    }
    prepared.free();
    assert.equal(JSON.parse(pine.analyzeScript(source.replace('0.8 to 3.8 by 0.5','0 to 3.8 by 0.5'))).executable,false);
    assert.throws(()=>pine.runScriptCsv(source.replace('by 0.5','by -0.5'),csv),/step must be greater than zero/);
    const precision=`//@version=${version}\n${version===4?'study':'indicator'}("precision")\nn=0\nfor i=0.1 to 0.3 by 0.1\n    n+=1\nplot(n)\n`;
    assert.deepEqual(JSON.parse(pine.runScriptCsv(precision,csv)).plots[0].values,[3,3,3]);
  }
  for (const version of [5,6]) {
    const source=`//@version=${version}\nstrategy("state defaults",default_qty_value=2)\nf(a=strategy.position_avg_price,p=strategy.position_size)=>nz(a)+p\nif bar_index==0\n    strategy.entry("L",strategy.long)\nplot(f())\nplot(f(p=7))\n`;
    const prepared=pine.compileScript(source);
    for (const wire of [pine.runScriptCsv(source,csv),prepared.runCsvWithRequestBarsAndInputOverrides(csv,'{}','{}')]) {
      assert.deepEqual(JSON.parse(wire).plots.map(p=>p.values),[[0,13,13],[7,18,18]]);
    }
    prepared.free();
    assert.equal(JSON.parse(pine.analyzeScript(source.replace(`version=${version}`,'version=4'))).executable,false);
  }
}

// Method object writes and Pine extend values survive the public WASM routes.
{
  const csv='time,open,high,low,close,volume\n0,10,10,10,10,1\n60000,10,10,10,10,1\n120000,10,10,10,10,1\n';
  for (const version of [5,6]) {
    const source=`//@version=${version}
indicator("method drawing compatibility")
type Child40
    float value
type Parent40
    Child40 child
    chart.point point
method bump40(Parent40 self)=>
    self.child.value += 1
    self.child.value
var values=array.new<Parent40>()
if barstate.isfirst
    values.push(Parent40.new(Child40.new(0),chart.point.from_index(2,10)))
alias=values.first()
value=input.string("r")
item=line.new(0,1,2,2,extend=value)
line.set_extend(item,value)
boxItem=box.new(0,2,1,1,extend=value)
box.set_extend(boxItem,value)
plot(alias.bump40())
plot(values.first().child.value)
plot(values.first().point.price)
plot(extend.right=="r"?1:0)
`;
    const prepared=pine.compileScript(source);
    const historical=JSON.parse(pine.runScriptCsv(source,csv));
    assert.deepEqual(historical.plots.map(p=>p.values),[[1,2,3],[1,2,3],[10,10,10],[1,1,1]]);
    assert.deepEqual(JSON.parse(prepared.runCsvWithRequestBarsAndInputOverrides(csv,'{}','{}')),historical);
    assert.ok(historical.lines.every(line=>line.snapshots.every(snapshot=>snapshot.extend==='extend.right')));
    assert.ok(historical.boxes.every(box=>box.snapshots.every(snapshot=>snapshot.extend==='extend.right')));
    const session=prepared.realtimeSession();
    session.seed('time,open,high,low,close,volume\n0,10,10,10,10,1\n60000,10,10,10,10,1\n');
    const replica=session.replica();
    const bar=close=>JSON.stringify({time:120000,open:close,high:close,low:close,close,volume:1});
    for (const close of [40,35]) {
      const changes=session.applyForming(bar(close));
      assert.equal(replica.apply(changes),true);
      assert.equal(replica.result(),session.result());
      assert.deepEqual(JSON.parse(session.result()).plots.map(p=>p.values.at(-1)),[3,3,10,1]);
    }
    assert.equal(replica.apply(session.applyConfirmed(bar(10))),true);
    assert.equal(replica.result(),session.confirmedResult());
    assert.deepEqual(JSON.parse(session.confirmedResult()),historical);
    replica.free();session.free();prepared.free();
    const invalid=`//@version=${version}\nindicator("invalid extend")\nitem=line.new(0,1,2,2,extend="extend.right")\nplot(1)\n`;
    assert.equal(JSON.parse(pine.analyzeScript(invalid)).executable,true);
    assert.throws(()=>pine.runScriptCsv(invalid,csv),/invalid extend value/);
  }
}

// Contract metadata is a capability separate from the package version.
assert.equal(pine.contractMetadataSchemaVersion(), 1);
for (const pointValue of [0.01, 0.5, 5, 50]) {
  const pointSource = '//@version=6\nindicator("Contract metadata")\nplot(syminfo.pointvalue)\nplot(syminfo.mincontract)';
  const output = JSON.parse(pine.runScriptCsvWithRequestBars(pointSource, bars,
    JSON.stringify({$chart:{pointValue,quantityStep:'0.25',minContract:'2'}})));
  assert.equal(output.plots[0].values[0], pointValue);
  assert.equal(output.plots[1].values[0], 2);
}
// A changed inner feed also invalidates fallback values cached by an outer request.
{
  const compiled = pine.compileScript('//@version=6\nindicator("nested feed")\nplot(request.security("C","1",request.security("B","1",close)))');
  const bar = (index, value) => ({time:index*60000,open:value,high:value,low:value,close:value,volume:1});
  const session = compiled.realtimeSessionWithRequestBars(JSON.stringify({
    'B:1':[bar(0,100)], 'C:1':[bar(0,1),bar(1,1)]
  }));
  session.seed('time,open,high,low,close,volume\n0,1,1,1,1,1\n');
  const replica = session.replica();
  assert.equal(session.applyRequestConfirmed('B','1',JSON.stringify(bar(1,200))), 'null');
  replica.apply(session.applyConfirmed(JSON.stringify(bar(1,1))));
  assert.deepEqual(JSON.parse(session.result()).plots[0].values,[100,200]);
  assert.equal(replica.result(),session.result());
  const before=session.result(), revision=session.revision;
  assert.throws(()=>session.applyRequestConfirmed('B','1',JSON.stringify(bar(1,999))),/E_REQUEST_FEED_TIME/);
  assert.equal(session.result(),before);
  assert.equal(session.revision,revision);
  replica.free(); session.free(); compiled.free();
}
console.log('wasm Node smoke passed: instantiate, analyze, run, compile/run, combined hosts, JS exceptions');

// Minimal WASM/Node embedding path for the local 0.3.0-rc.1 candidate.
// Usage: node docs/examples/wasm_embed.mjs <generated-pine_wasm.js>
import assert from 'node:assert/strict';
import path from 'node:path';
import { createRequire } from 'node:module';

if (process.argv.length !== 3) {
  throw new Error('usage: node docs/examples/wasm_embed.mjs <generated-pine_wasm.js>');
}

const require = createRequire(import.meta.url);
const pine = require(path.resolve(process.argv[2]));
assert.equal(pine.packageVersion(), '0.3.0-rc.1');

const source = '//@version=6\nindicator("wasm embed")\nplot(close * 2)\nplot(timenow)\n';
const bars = [
  'time,open,high,low,close,volume',
  '0,10,10,10,10,1',
  '1,11,11,11,11,1',
  '',
].join('\n');

const program = pine.compileScript(source);
const requirements = JSON.parse(program.hostRequirements());
const historical = JSON.parse(program.runCsvWithRequestBars(
  bars,
  JSON.stringify({ $executionTimes: [1000, 2000] }),
));
program.free();

let compileError = null;
try {
  pine.compileScript('//@version=6\nindicator("bad")\nplot(unknown_name)\n');
} catch (error) {
  compileError = String(error);
}

process.stdout.write(`${JSON.stringify({
  version: pine.packageVersion(),
  clock: requirements.execution.clock,
  historicalScaled: historical.plots[0].values,
  compileError,
}, null, 2)}\n`);

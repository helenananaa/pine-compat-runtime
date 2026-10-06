// Batch acceptance only. A historical snapshot does not establish live tick parity.
const fs = require('node:fs');
const path = require('node:path');
const pine = require(path.resolve(process.argv[2]));
const p = JSON.parse(fs.readFileSync(process.argv[3], 'utf8'));
process.stdout.write(pine.runScriptCsvWithRequestBarsAndInputOverrides(
  p.source, p.barsCsv, JSON.stringify(p.request), JSON.stringify(p.overrides)
));

'use strict';
const assert = require('node:assert/strict');
module.exports = function(modulePath) {
  const pine = require(modulePath);
  const source = "//@version=5\nindicator(\"V5 transparency modern oracle 23\",precision=0)\nt=input.int(40)\nbgcolor(color.green,transp=t,force_overlay=true)\nplotshape(true,color=color.blue,transp=30,display=display.all,force_overlay=true,title=\"shape\")\nplotchar(true,color=color.red,transp=20,force_overlay=true,title=\"char\")\nplotarrow(1,colorup=color.blue,colordown=color.red,transp=10,force_overlay=true,title=\"arrow\")\np=plot(1,color=color.blue,transp=50,force_overlay=true,title=\"line\")\nq=plot(2,force_overlay=true,title=\"line2\")\nfill(p,q,color=color.green,transp=60,title=\"fill\")\n";
  const overlay = "//@version=6\nindicator(\"V6 output overlay oracle 23\",precision=0)\nbgcolor(color.green,force_overlay=true)\nplotchar(true,title=\"char\",color=color.red,force_overlay=true)\nplot(1,title=\"marker\")\n";
  const csv = 'time,open,high,low,close,volume\n0,1,1,1,1,1\n';
  const check = result => {
    assert.deepEqual(result.plots.map(p=>p.values),[[1],[2]]);
    for(const key of ['bgColors','plotShapes','plotChars','plotArrows','plots']) assert.equal(result[key][0].forceOverlay,true);
    assert.equal(result.plotChars[0].title,'char');
    assert.equal(result.fills.length,1);
  };
  check(JSON.parse(pine.runScriptCsv(source,csv)));
  check(JSON.parse(pine.compileScript(source).runCsvWithRequestBars(csv,'{}')));
  for(const version of [5,6]) {
    const script=overlay.replace('version=6',`version=${version}`);
    for(const result of [JSON.parse(pine.runScriptCsv(script,csv)),JSON.parse(pine.compileScript(script).runCsvWithRequestBars(csv,'{}'))]) {
      assert.equal(result.bgColors[0].forceOverlay,true);
      assert.equal(result.plotChars[0].forceOverlay,true);
    }
  }
  const repeated='//@version=5\nindicator("duplicate")\nplotshape(true,transp=0,transp=0)\nplot(1)\n';
  assert.deepEqual(JSON.parse(pine.runScriptCsv(repeated,csv)).plotShapes[0].values,[true]);
  assert.deepEqual(JSON.parse(pine.compileScript(repeated).runCsvWithRequestBars(csv,'{}')).plotShapes[0].values,[true]);
  assert.throws(()=>pine.compileScript(source.replace('version=5','version=6')),/transp/);
  assert.throws(()=>pine.compileScript('//@version=5\nindicator("negative")\nbgcolor(color.green,transp=bar_index%100)\nplot(1)\n'),/input integer/);
};

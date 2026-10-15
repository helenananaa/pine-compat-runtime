'use strict';
const assert = require('node:assert/strict');

module.exports = function(modulePath) {
  const pine = require(modulePath);
  const days = [8,9,12].map(d=>[
    Date.parse(`2026-10-${String(d).padStart(2,'0')}T13:30:00Z`),
    Date.parse(`2026-10-${String(d).padStart(2,'0')}T20:00:00Z`),
    Date.parse(`2026-10-${String(d).padStart(2,'0')}T00:00:00Z`)
  ]);
  const timeframes = ['D','1'].map(timeframe=>({timeframe, intervals:days.flatMap(([open,close,tradingDay])=> {
    if(timeframe==='D') return [{open,close,tradingDay}];
    return Array.from({length:390},(_,i)=>({open:open+i*60000,close:open+(i+1)*60000,tradingDay}));
  })}));
  const input = {schemaVersion:1,symbols:[{symbol:'NASDAQ:AAPL',timezone:'America/New_York',timeframes}]};
  const host = {$chart:{symbol:'NASDAQ:AAPL',timeframe:'D'},$barCalendars:input};
  const csv = `time,open,high,low,close,volume\n${days[1][0]},10,10,10,10,1\n`;
  const source='//@version=6\nindicator("calendar")\nplot(time_close-time)\nplot(time_close("1")-time)\nplot(time_close("1",timeframe_bars_back=1)-time)\nplot(time("1",timeframe_bars_back=1)-time)\nplot(hour)\nplot(hour(time))\nplot(timestamp(year,month,dayofmonth,9,30)-time)\nplot(str.format_time(time,"HH:mm")=="09:30"?1:0)\nplot(syminfo.timezone=="America/New_York"?1:0)\nplot(time("D",bars_back=-1)-time)\n';
  const expected=[23400000,23400000,-63000000,-86400000,9,9,0,1,1,259200000].map(v=>[v]);
  const run=(source,host)=>JSON.parse(pine.runScriptCsvWithRequestBars(source,csv,JSON.stringify(host)));
  assert.deepEqual(run(source,host).plots.map(p=>p.values),expected);
  const prepared=pine.compileScript(source);
  assert.deepEqual(JSON.parse(prepared.runCsvWithRequestBars(csv,JSON.stringify(host))).plots.map(p=>p.values),expected);
  assert.throws(()=>run(source,{...host,$barCalendars:{...input,schemaVersion:2}}),/schema version/);
  assert.throws(()=>run('//@version=6\nindicator("missing")\nplot(time_close("15"))\n',host),/missing bar calendar timeframe/);
  const virtual='//@version=6\nindicator("virtual")\nplot(time("1","0930-0945",timeframe_bars_back=1)-time)\nplot(time_close("1","0930-0945",timeframe_bars_back=1)-time)\nplot(time("1","1500-1600",timeframe_bars_back=1)-time)\nplot(time_close("1","1500-1600",timeframe_bars_back=1)-time)\nplot(time("1","0930-0945",timeframe_bars_back=-16)-time)\nplot(time("1","0930-0945:23456",timeframe_bars_back=-16)-time)\nplot(time("1","24x7")-time)\n';
  const virtualExpected=[-86400000,-85500000,-66600000,-63000000,86400000,259200000,-34200000].map(v=>[v]);
  assert.deepEqual(run(virtual,host).plots.map(p=>p.values),virtualExpected);
  assert.deepEqual(JSON.parse(pine.compileScript(virtual).runCsvWithRequestBars(csv,JSON.stringify(host))).plots.map(p=>p.values),virtualExpected);
  const lowerCalendarSource="//@version=6\nindicator(\"Lower calendar close oracle 21\",precision=0)\nplot(time(\"D\")-time,title=\"dailyOpen\")\nplot(time_close(\"D\")-time,title=\"dailyClose\")\nplot(time(\"W\")-time,title=\"weeklyOpen\")\nplot(time_close(\"W\")-time,title=\"weeklyClose\")\nplot(time_close(\"D\",timeframe_bars_back=1)-time,title=\"previousDailyClose\")\nplot(time_close(\"D\",timeframe_bars_back=-1)-time,title=\"nextDailyClose\")\nplot(time_close(\"W\",timeframe_bars_back=1)-time,title=\"previousWeeklyClose\")\nplot(time_close(\"W\",timeframe_bars_back=-1)-time,title=\"nextWeeklyClose\")\nplot(time_close(\"D\",\"0930-0945\",\"UTC\")-time,title=\"shortDailyClose\")\nplot(time_close(\"W\",\"0930-0945\",\"UTC\")-time,title=\"shortWeeklyClose\")\nplot(time_close(\"D\",bars_back=1)-time,title=\"chartPreviousDailyClose\")\n";
  const lowerCalendarCsv='time,open,high,low,close,volume\n1790812800000,1,1,1,1,1\n';
  const lowerCalendarHost=JSON.stringify({$chart:{symbol:'BINANCE:BTCUSDT',timeframe:'M'}});
  const lowerCalendarExpected=[0,86400000,-259200000,345600000,0,172800000,-259200000,950400000,null,294300000,-2505600000].map(v=>[v]);
  assert.deepEqual(JSON.parse(pine.runScriptCsvWithRequestBars(lowerCalendarSource,lowerCalendarCsv,lowerCalendarHost)).plots.map(p=>p.values),lowerCalendarExpected);
  assert.deepEqual(JSON.parse(pine.compileScript(lowerCalendarSource).runCsvWithRequestBars(lowerCalendarCsv,lowerCalendarHost)).plots.map(p=>p.values),lowerCalendarExpected);
  for(const version of [5,6]) {
    const compactSource="//@version=5\nindicator(\"Compact date oracle 22\",precision=0)\nplot(timestamp(\"15Aug 2022 14:00 +0000\"),title=\"start\")\nplot(timestamp(\"15Aug 2022 20:00 +0000\"),title=\"end\")\nplot(timestamp(\"15Aug 2022 14:00 +0530\"),title=\"offset\")\nplot(timestamp(\"15 Aug 2022 14:00 +0530\"),title=\"spacedOffset\")\nplot(timestamp(\"15Aug 2022 14:00 GMT+0530\"),title=\"gmtOffset\")\nplot(timestamp(\"2022-08-15T14:00:00+05:30\"),title=\"isoOffset\")\nplot(timestamp(\"1Jan 2024 05:30:01 +0530\"),title=\"secondsOffset\")\nplot(timestamp(\"15Aug 2022\"),title=\"dateOnly\")\nplot(timestamp(\"15Aug 2022 14:00:30 +0000\"),title=\"seconds\")\nplot(timestamp(\"15Aug 2022 14:00 -0400\"),title=\"negativeOffset\")\nplot(timestamp(\"15August 2022 14:00 +0000\"),title=\"fullMonth\")\nplot(timestamp(\"15aug 2022 14:00\"),title=\"lowercase\")\n".replace('version=5',`version=${version}`);
    const compactCsv='time,open,high,low,close,volume\n0,1,1,1,1,1\n';
    const compactExpected=[1660572000000, 1660593600000, 1660572000000, 1660552200000, 1660572000000, 1660552200000, 1704087001000, 1660521600000, 1660572030000, 1660572000000, 1660572000000, 1660572000000].map(v=>[v]);
    assert.deepEqual(JSON.parse(pine.runScriptCsv(compactSource,compactCsv)).plots.map(p=>p.values),compactExpected);
    assert.deepEqual(JSON.parse(pine.compileScript(compactSource).runCsvWithRequestBars(compactCsv,'{}')).plots.map(p=>p.values),compactExpected);
  }
};

import csv
import json
from pathlib import Path

import pine_compat
import pytest

EVIDENCE = Path(__file__).resolve().parents[2] / "docs/evidence/strategy-library-20261010"
RECORDS = json.loads((EVIDENCE / "fx-native-ledger-oracles-31.json").read_text(encoding="utf-8"))
RATES = json.loads((EVIDENCE / "fx-native-rates-input-31.json").read_text(encoding="utf-8"))
REALTIME_RECORDS = json.loads((EVIDENCE / "fx-native-realtime-oracles-31.json").read_text(encoding="utf-8"))
with (EVIDENCE / "fx-native-daily-input-31.csv").open(encoding="utf-8") as stream:
    BARS = [dict(time=int(r["time"]), **{k:float(r[k]) for k in ["open","high","low","close"]}, volume=0.) for r in csv.DictReader(stream)]
CHARTS = {}
for timeframe in ["D","W","M"]:
    with (EVIDENCE / f"fx-native-chart-{timeframe.lower()}-31.csv").open(encoding="utf-8") as stream:
        CHARTS[timeframe] = [dict(time=int(r["time"]),**{k:float(r[k]) for k in ["open","high","low","close","volume"]}) for r in csv.DictReader(stream)]


@pytest.mark.parametrize("record", RECORDS, ids=[r["stem"] for r in RECORDS])
def test_native_currency_conversion_quantity_and_ledger(record):
    source = (EVIDENCE / (record["stem"] + ".pine")).read_text(encoding="utf-8")
    program = pine_compat.compile_script(source)
    host = {"$chart":dict(minMove=1,priceScale=100,quantityPrecision=5,currency="USDT"), "$currencyRates":RATES}
    timeframe = record["chart_timeframe"]
    result = program.run(CHARTS[timeframe], request_bars=host, chart_symbol="BINANCE:BTCUSDT", chart_timeframe=timeframe)
    actual = [p["values"][-1] for p in result["plots"]]
    assert len(actual) == len(record["native_values"])
    for a,b in zip(actual,record["native_values"]):
        if b is None:
            assert a is None
        else:
            assert a == pytest.approx(b,rel=0,abs=5.1e-9)


def test_missing_pair_is_input_error_even_when_invalid_pair_is_ignored():
    program = pine_compat.compile_script('//@version=6\nindicator("missing")\nplot(request.currency_rate("USD","EUR",true))\n')
    with pytest.raises(ValueError,match="missing currency rate data"):
        program.run(BARS)
    unavailable = dict(schemaVersion=1,pairs=[dict(**{"from":"USD","to":"EUR"},unavailable=True)])
    result = program.run(BARS,request_bars={"$currencyRates":unavailable})
    assert result["plots"][0]["values"] == [None]*len(BARS)


@pytest.mark.parametrize("rates", [
    {"schemaVersion": 2, "pairs": []},
    {"schemaVersion": 1, "pairs": [{"from": "USD", "to": "EUR"}]},
    {"schemaVersion": 1, "pairs": [{"from": "USD", "to": "EUR", "rates": [{"effectiveTime": 0, "rate": 0}]}]},
])
def test_currency_input_is_validated_even_without_a_query(rates):
    program = pine_compat.compile_script('//@version=6\nindicator("input")\nplot(close)\n')
    with pytest.raises(ValueError):
        program.run(BARS, request_bars={"$currencyRates": rates})


def test_native_lowercase_currency_queries_use_canonical_host_keys():
    program = pine_compat.compile_script((EVIDENCE / "fx-name-controls-v6-oracle-31.pine").read_text(encoding="utf-8"))
    rates = {"schemaVersion":1,"pairs":[
        {"from":"USD","to":"EUR","rates":[{"effectiveTime":1792195200000,"rate":0.891}]},
        {"from":"USD","to":"ZZZ","unavailable":True},
    ]}
    bars = [dict(time=1792195200000,open=1.,high=1.,low=1.,close=1.,volume=0.)]
    result = program.run(bars,request_bars={"$currencyRates":rates})
    assert [p["values"][-1] for p in result["plots"]] == [1.,0.891,1.,None]


@pytest.mark.parametrize("record", REALTIME_RECORDS, ids=[r["stem"] for r in REALTIME_RECORDS])
def test_native_realtime_currency_queries_keep_the_chart_open_context(record):
    source = (EVIDENCE / (record["stem"] + ".pine")).read_text(encoding="utf-8")
    host = {"$chart":dict(minMove=1,priceScale=100,quantityPrecision=5,currency="USDT"),"$currencyRates":RATES}
    session = pine_compat.create_realtime_session(source,request_bars=host,chart_symbol="BINANCE:BTCUSDT",chart_timeframe=record["chart_timeframe"])
    session.seed([])
    expected = record["native_values"]
    bar = dict(time=int(expected[4]),open=100.,high=100.,low=100.,close=100.,volume=0.)
    for clock in [int(expected[5]),int(expected[5])+60000]:
        result = session.update_forming(bar,execution_time=clock)
        values = [p["values"][-1] for p in result["plots"]]
        assert values == pytest.approx(expected[:5]+[clock]+expected[6:],rel=0,abs=5.1e-9)

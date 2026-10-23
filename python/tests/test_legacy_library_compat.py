import math
import pytest
import pine_compat


@pytest.mark.parametrize("version", [3, 4])
def test_legacy_function_and_value_namespaces(version):
    source = f'''//@version={version}
study("namespaces")
macd=7.0
atr=11.0
sma=19.0
f(sma)=>
    ema=13.0
    sma(close,2)+ema(close,2)+sma+ema
[m,s,h]=macd(close,2,3,2)
plot(m+s+h)
plot(atr(2))
plot(f(17))
plot(macd+atr+sma)
'''
    canonical = source.replace(f"version={version}", "version=5").replace("study(", "indicator(")
    for name in ["sma", "ema", "macd", "atr"]:
        canonical = canonical.replace(f"{name}(", f"ta.{name}(")
    actual = pine_compat.compile_script(source).run(BARS)["plots"]
    expected = pine_compat.compile_script(canonical).run(BARS)["plots"]
    assert actual == expected
    assert actual[-1]["values"] == [37]*4


def test_v4_pvt_alias_and_earlier_version_rejection():
    source = '//@version=4\nstudy("pvt")\nplot(pvt)\nplot(pvt[1])\n'
    canonical = source.replace("version=4", "version=5").replace("study(", "indicator(").replace("pvt", "ta.pvt")
    assert pine_compat.compile_script(source).run(BARS)["plots"] == pine_compat.compile_script(canonical).run(BARS)["plots"]
    for version in [1, 2, 3, 5, 6]:
        declaration = "study" if version < 5 else "indicator"
        assert not pine_compat.analyze_script(f'//@version={version}\n{declaration}("pvt")\nplot(pvt)\n')["executable"]


@pytest.mark.parametrize("timeframe", ["D", "W", "M"])
def test_utc_lower_timeframe_closes_and_independent_offsets(timeframe):
    source = '''//@version=6
indicator("lower close")
plot(time_close("1S")-time)
plot(time_close("30S","0000-0001","UTC")-time)
plot(time_close("30S",timeframe_bars_back=-1)-time)
plot(time_close("30S",timeframe_bars_back=1)-time)
plot(time_close("30S",bars_back=1)-time("",bars_back=1))
plot(time_close("30S","0001-0002","UTC"))
'''
    bars = [{**BARS[0], "time":1704067200000}]
    result = pine_compat.compile_script(source).run(bars, chart_symbol="UTC:TEST", chart_timeframe=timeframe)
    assert [p["values"] for p in result["plots"]] == [[86400000], [60000], [86400000], [0], [86400000], [None]]


def test_utc_lower_intraday_close_keeps_requested_granularity():
    source = '//@version=6\nindicator("intraday")\nplot(time_close("1S")-time)\nplot(time_close("30S")-time)\nplot(time_close("1")-time)\n'
    result = pine_compat.compile_script(source).run([{**BARS[0], "time":1704067200000}], chart_timeframe="5")
    assert [p["values"] for p in result["plots"]] == [[1000], [30000], [60000]]

BARS = [{"time": i * 60000, "open": i+10, "high": i+10,
         "low": i+10, "close": i+10, "volume": 1} for i in range(4)]

@pytest.mark.parametrize("version", [2, 3, 4])
def test_legacy_order_directions_and_default_quantity(version):
    source = f'''//@version={version}
strategy("legacy order",default_qty_value=2)
if bar_index==0
    strategy.order("L",long=true)
if bar_index==2
    strategy.order("S",0,qty=3)
plot(strategy.position_size)
'''
    assert pine_compat.compile_script(source).run(BARS)["plots"][0]["values"] == [0, 2, 2, -1]
    for modern in [5, 6]:
        assert not pine_compat.analyze_script(source.replace(f"version={version}", f"version={modern}"))["executable"]


@pytest.mark.parametrize("version", [4, 5, 6])
def test_table_clear_omitted_end_coordinates(version):
    declaration = "study" if version == 4 else "indicator"
    source = f'''//@version={version}
{declaration}("clear defaults")
var t=table.new(position.top_right,3,3)
if bar_index==0
    for c=0 to 2
        for r=0 to 2
            table.cell(t,c,r,"keep")
    table.clear(t,0,0)
    table.clear(t,1,1,2)
    table.clear(t,0,1,end_row=2)
plot(close)
'''
    result = pine_compat.compile_script(source).run(BARS)
    cells = result["tables"][0]["snapshots"][-1]["cells"]
    assert [(c["column"], c["row"]) for c in cells] == [(1, 0), (2, 0), (1, 2), (2, 2)]

@pytest.mark.parametrize("version", [1, 2, 3, 4])
def test_duplicate_legacy_input_priority_and_cos(version):
    source = f'''//@version={version}
study("duplicate input")
a=input(2,defval=1,title="A")
b=input(2,defval=3,title="B")
c=input(1,title="First title",title="Second title")
plot(a)
plot(b)
plot(c)
plot(cos(0.5))
'''
    report = pine_compat.analyze_script(source)
    assert report["diagnostics"] == []
    assert [item["title"] for item in report["inputs"]] == ["A", "B", "Second title"]
    result = pine_compat.compile_script(source).run(BARS)
    assert [p["values"] for p in result["plots"]] == [[2]*4, [2]*4, [1]*4, [math.cos(0.5)]*4]

@pytest.mark.parametrize("version", [4, 5, 6])
def test_nested_inline_input_keeps_one_shared_metadata_callsite(version):
    declaration = "study" if version == 4 else "indicator"
    source = f'''//@version={version}
{declaration}("nested input")
f(x)=>x+1
g()=>f(input(2,"Nested input")+1)
plot(g())
plot(g()+g())
'''
    report = pine_compat.analyze_script(source)
    assert report["diagnostics"] == []
    assert len(report["inputs"]) == 1
    result = pine_compat.compile_script(source).run(BARS)
    assert [p["values"] for p in result["plots"]] == [[4]*4, [8]*4]

@pytest.mark.parametrize("version", [4, 5, 6])
def test_close_source_positions_preserve_quantity_and_when(version):
    prefix = "true," if version <= 5 else ""
    tail = ",false,false" if version >= 5 else ""
    source = f'''//@version={version}
strategy("close source positions")
if bar_index==0
    strategy.entry("E",strategy.long,qty=2)
if bar_index==2
    strategy.close("E",{prefix}"half",1,na,"half alert"{tail})
plot(strategy.position_size)
'''
    result = pine_compat.compile_script(source).run(BARS)
    assert result["plots"][0]["values"] == [0, 2, 2, 1]
    assert len(result["strategy"]["trades"]) == 1


def test_v4_numeric_direction_and_modern_rejection():
    source = '''//@version=4
strategy("numeric direction")
if bar_index==0
    strategy.entry("L",1,qty=2)
if bar_index==2
    strategy.entry("S",0,qty=1)
plot(strategy.position_size)
'''
    assert pine_compat.compile_script(source).run(BARS)["plots"][0]["values"] == [0, 2, 2, -1]
    for version in [5, 6]:
        assert not pine_compat.analyze_script(source.replace("version=4", f"version={version}"))["executable"]


def test_legacy_percentile_aliases_preserve_rank_and_interpolation():
    source = '''//@version=3
study("percentile alias")
plot(percentile_nearest_rank(close,3,25))
plot(percentile_linear_interpolation(close,3,25))
'''
    result = pine_compat.compile_script(source).run(BARS)
    assert result["plots"][0]["values"] == [None, None, 10, 11]
    assert result["plots"][1]["values"] == [None, None, 10.5, 11.5]


@pytest.mark.parametrize("version", [4, 5, 6])
def test_ema_integer_arithmetic_alias_preserves_fractional_value(version):
    declaration = "study" if version == 4 else "indicator"
    ema = "ema" if version == 4 else "ta.ema"
    source = f'''//@version={version}
{declaration}("EMA aliases")
n=input(5)
a=n/2
b=a*2
plot(a)
plot({ema}(close,a))
plot({ema}(close,int(a)))
plot({ema}(close,b))
plot({ema}(close,5))
'''
    bars = BARS + [{"time": i*60000, "open": 10+i, "high": 10+i,
                    "low": 10+i, "close": 10+i, "volume": 1} for i in range(4, 12)]
    result = pine_compat.compile_script(source).run(bars)
    values = [p["values"] for p in result["plots"]]
    assert values[0] == [2.5]*12
    assert values[1] == values[2]
    assert values[3] == values[4]
    assert values[3][-1] is not None
    for body in ["a=input(2.8)", "n=input(5)\na=n/2.0"]:
        invalid = f'//@version={version}\n{declaration}("bad")\n{body}\nplot({ema}(close,a))\n'
        assert not pine_compat.analyze_script(invalid)["executable"]


def test_v4_resolution_named_options_are_exported():
    source = '''//@version=4
study("resolution")
tf=input("60",title="Timeframe",type=input.resolution,options=["1","60","D"])
plot(tf=="60"?1:0)
'''
    report = pine_compat.analyze_script(source)
    assert report["diagnostics"] == []
    assert report["inputs"][0]["options"] == ["1", "60", "D"]
    assert pine_compat.compile_script(source).run(BARS)["plots"][0]["values"] == [1]*4


@pytest.mark.parametrize("version", [4, 5, 6])
def test_implicit_assignment_returns_and_missing_else(version):
    declaration = "study" if version == 4 else "indicator"
    bool_probe = "b==false?1:0" if version == 6 else "na(b)?1:0"
    source = f'''//@version={version}
{declaration}("block returns")
f(n)=>
    float s=0
    if n>0
        for i=1 to n
            s:=s+i
    else
        s:=0
x=0
y=if bar_index%2==0
    x+=7
q=for i=0 to 2
    x+=i
b=if bar_index<0
    true
plot(f(3))
plot(y)
plot(q)
plot(x)
plot({bool_probe})
'''
    result = pine_compat.compile_script(source).run(BARS)
    assert [p["values"] for p in result["plots"]] == [
        [6]*4, [7, None, 7, None], [10, 3, 10, 3], [10, 3, 10, 3], [1]*4]


@pytest.mark.parametrize("version", [4, 5, 6])
def test_zero_default_quantity_keeps_explicit_entry(version):
    source = f'''//@version={version}
strategy("zero default",default_qty_type=strategy.percent_of_equity,default_qty_value=0)
if bar_index==0
    strategy.entry("Zero",strategy.long)
if bar_index==2
    strategy.entry("Explicit",strategy.long,qty=1)
plot(strategy.position_size)
'''
    assert pine_compat.compile_script(source).run(BARS)["plots"][0]["values"] == [0, 0, 0, 1]
    assert not pine_compat.analyze_script(source.replace("default_qty_value=0", "default_qty_value=-1"))["executable"]


@pytest.mark.parametrize("version", [4,5,6])
def test_fractional_for_bounds_preserve_float_counter_and_int_end_comparison(version):
    declaration="study" if version==4 else "indicator"
    source=f'''//@version={version}
{declaration}("numeric loops")
a=0.0
b=0.0
c=0
d=0
for i=0.8 to 3.8 by 0.5
    a+=i
for j=-0.8 to -3.8
    b+=j
for k=-3 to -0.8
    c+=1
for m=3 to 0.8
    d+=1
plot(a)
plot(b)
plot(c)
plot(d)
'''
    plots=pine_compat.compile_script(source).run(BARS)["plots"]
    for plot,expected in zip(plots,[16.1,-9.2,3,3]):
        assert plot["values"]==pytest.approx([expected]*4)
    bad=source.replace("0.8 to 3.8 by 0.5","0 to 3.8 by 0.5")
    assert not pine_compat.analyze_script(bad)["executable"]


@pytest.mark.parametrize("version", [5,6])
def test_strategy_defaults_evaluate_broker_state_at_each_call(version):
    source=f'''//@version={version}
strategy("state defaults",default_qty_value=2)
f(a=strategy.position_avg_price,p=strategy.position_size)=>nz(a)+p
if bar_index==0
    strategy.entry("L",strategy.long)
plot(f())
plot(f(p=7))
plot(f(100))
plot(f(100,7))
'''
    result=pine_compat.compile_script(source).run(BARS)
    assert [p["values"] for p in result["plots"]]==[[0,13,13,13],[7,18,18,18],[100,102,102,102],[107]*4]
    assert not pine_compat.analyze_script(source.replace(f"version={version}","version=4"))["executable"]


@pytest.mark.parametrize("version", [4,5,6])
def test_negative_numeric_for_steps_fail_at_runtime(version):
    declaration="study" if version==4 else "indicator"
    for header in ["0 to 3 by -1", "0.8 to 3.8 by -0.5"]:
        source=f'//@version={version}\n{declaration}("negative step")\nfor i={header}\n    value=i\nplot(close)\n'
        program=pine_compat.compile_script(source)
        with pytest.raises(Exception,match="step must be greater than zero"):
            program.run(BARS)


@pytest.mark.parametrize("version", [4,5,6])
def test_float_loop_comparison_boundary_keeps_last_decimal_increment(version):
    declaration="study" if version==4 else "indicator"
    source=f'//@version={version}\n{declaration}("loop precision")\na=0\nb=0\nfor i=0.1 to 0.3 by 0.1\n    a+=1\nfor j=0.0 to 0.3 by 0.1\n    b+=1\nplot(a)\nplot(b)\n'
    result=pine_compat.compile_script(source).run(BARS)
    assert [p["values"] for p in result["plots"]]==[[3]*4,[4]*4]

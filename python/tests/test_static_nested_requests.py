import pine_compat
import pytest


def bar(time, close):
    return dict(time=time, open=close, high=close, low=close, close=close, volume=1)


@pytest.mark.parametrize("version", [4, 5, 6])
def test_requested_local_state_and_loops_keep_provider_history(version):
    if version == 4:
        body = ('study("iff history")\nf()=>\n    pos=0.0\n'
                '    pos:=iff(close>0,nz(pos[1])+1,0)\n    pos\na=f()\n'
                'plot(security("B","2",a[1]))\n')
        expected = [[None, None, None, 1, 1, 2, 2, 3]]
    else:
        body = ('indicator("requested loops")\nf(src)=>\n    total=0.0\n'
                '    for i=0 to 2\n        total+=src[i]\n    total\n'
                'g(src)=>\n    total=0.0\n    i=0\n    while i<3\n'
                '        total+=src[i]\n        i+=1\n    total\n'
                'plot(request.security("B","2",f(close)))\n'
                'plot(request.security("B","2",g(close)))\n')
        expected = [[None]*5 + [60, 60, 90]]*2
    program = pine_compat.compile_script(f'//@version={version}\n' + body)
    result = program.run([bar(i*60_000, 1+i) for i in range(8)],
                         chart_symbol="A", chart_timeframe="1",
                         request_bars={"B:2": [bar(i*120_000, 10*(i+1)) for i in range(4)]})
    assert [plot["values"] for plot in result["plots"]] == expected


@pytest.mark.parametrize("version", [5, 6])
@pytest.mark.parametrize("callee", ["input", "input.source"])
def test_source_inputs_use_requested_ohlc_and_preserve_selection_override(version, callee):
    source = (
        f'//@version={version}\nindicator("requested source")\n'
        f'src={callee}(title="Source",defval=hl2)\n'
        'plot(src)\nplot(request.security("B","2",src))\n'
    )
    program = pine_compat.compile_script(source)
    source_id = pine_compat.analyze_script(source)["inputs"][0]["callSiteId"]
    chart = [dict(time=i*60_000, open=4, high=5, low=3, close=4, volume=1) for i in range(4)]
    supplied = {"B:2": [dict(time=0, open=30, high=40, low=20, close=30, volume=1),
                         dict(time=120_000, open=70, high=80, low=60, close=70, volume=1)]}
    for overrides, expected in [({}, [[4]*4, [None, 30, 30, 70]]),
                                ({source_id: "high"}, [[5]*4, [None, 40, 40, 80]])]:
        result = program.run(chart, chart_symbol="A", chart_timeframe="1",
                             request_bars=supplied, input_overrides=overrides)
        assert [plot["values"] for plot in result["plots"]] == expected


@pytest.mark.parametrize("version", [4, 5, 6])
def test_function_request_dependencies_keep_argument_specialization(version):
    if version == 4:
        body = ('study("tuple")\nf(src)=>\n    alias=src+timeframe.multiplier\n'
                '    [alias,time]\n[a,b]=security("B","2",f(close))\nplot(a)\nplot(b)\n')
        expected = [[None, 12, 12, 22], [None, 0, 0, 120_000]]
    else:
        body = ('indicator("parameters")\nf(src)=>\n    alias=src+timeframe.multiplier\n'
                '    request.security("B","2",alias)\ng(src)=>f(src)\n'
                'plot(g(close))\nplot(f(src=10))\n')
        expected = [[None, 12, 12, 22], [None, 12, 12, 12]]
    program = pine_compat.compile_script(f'//@version={version}\n' + body)
    result = program.run([bar(i*60_000, 1+i) for i in range(4)],
                         chart_symbol="A", chart_timeframe="1",
                         request_bars={"B:2": [bar(0, 10), bar(120_000, 20)]})
    assert [plot["values"] for plot in result["plots"]] == expected


@pytest.mark.parametrize("version", [5, 6])
def test_request_mode_and_chart_series_survive_prepared_python_execution(version):
    bars = [bar(i * 60_000, 10 + i) for i in range(8)]
    supplied = {"B:4": [bar(0, 100), bar(240_000, 200)]}
    for dynamic, expected in [(False, [13, 13, 13, 13, 17]), (True, [100, 100, 100, 100, 200])]:
        source = (
            f'//@version={version}\nindicator("nested",dynamic_requests={str(dynamic).lower()})\n'
            'inner=request.security("","",close)\n'
            'plot(request.security("B","4",inner))\n'
        )
        program = pine_compat.compile_script(source)
        result = program.run(bars, chart_symbol="A", chart_timeframe="1", request_bars=supplied)
        assert result["plots"][0]["values"] == [None, None, None, *expected]
        assert program.run(bars, chart_symbol="A", chart_timeframe="1", request_bars=supplied) == result


@pytest.mark.parametrize("version", [5, 6])
@pytest.mark.parametrize("dynamic", [False, True])
def test_named_nested_requests_keep_version_context_and_merge_policies(version, dynamic):
    source = (
        f'//@version={version}\nindicator("named nested",dynamic_requests={str(dynamic).lower()})\n'
        'inner=request.security(expression=timeframe.multiplier,timeframe="",symbol=syminfo.tickerid)\n'
        'merged=request.security(gaps=barmerge.gaps_on,symbol=syminfo.tickerid,'
        'timeframe="",expression=timeframe.multiplier,lookahead=barmerge.lookahead_off)\n'
        'plot(inner)\nplot(request.security(expression=inner,timeframe="4",symbol="B"))\n'
        'plot(merged)\nplot(request.security(expression=merged,timeframe="4",symbol="B"))\n'
    )
    program = pine_compat.compile_script(source)
    result = program.run([bar(i*60_000, 100) for i in range(4)],
                         chart_symbol="A", chart_timeframe="1",
                         request_bars={"B:4": [bar(0, 10)]})
    outer = [None, None, None, 4 if dynamic else 1]
    assert [plot["values"] for plot in result["plots"]] == [[1]*4, outer, [1]*4, outer]


@pytest.mark.parametrize("version", [5, 6])
def test_uneven_higher_timeframe_gaps_and_static_dependencies(version):
    source = (
        f'//@version={version}\nindicator("uneven",dynamic_requests=false)\n'
        'inner=request.security("","",time)\n'
        'plot(request.security("A","6",time))\n'
        'plot(request.security("A","6",time,gaps=barmerge.gaps_on))\n'
        'plot(request.security("A","6",time,gaps=barmerge.gaps_on,lookahead=barmerge.lookahead_on))\n'
        'plot(request.security("A","6",inner))\n'
    )
    program = pine_compat.compile_script(source)
    result = program.run(
        [bar(i * 300_000, 100) for i in range(4)],
        chart_symbol="A", chart_timeframe="5",
        request_bars={"A:6": [bar(i * 360_000, 100) for i in range(4)]},
    )
    assert [plot["values"] for plot in result["plots"]] == [
        [None, 0, 360_000, 720_000],
        [None, 0, 360_000, 720_000],
        [0, None, 360_000, 720_000],
        [None, 0, 300_000, 600_000],
    ]


@pytest.mark.parametrize("version", [4, 5, 6])
def test_scalar_input_fixnan_and_vwap_reconstruct_requested_state(version):
    declaration = "study" if version == 4 else "indicator"
    request = "security" if version == 4 else "request.security"
    sma = "sma" if version == 4 else "ta.sma"
    vwap = "vwap" if version == 4 else "ta.vwap"
    input_call = "input" if version == 4 else "input.int"
    source = (f'//@version={version}\n{declaration}("scalar inputs")\n'
              'f(src)=>fixnan(src>15 ? na : src)\n'
              f'plot({request}("B","2",{sma}(close,{input_call}(title="Length",defval=2))))\n'
              f'plot({request}("B","2",f(close)))\n'
              f'plot({request}("B","2",{vwap}))\n')
    program = pine_compat.compile_script(source)
    input_id = pine_compat.analyze_script(source)["inputs"][0]["callSiteId"]
    requested = {"B:2": [bar(i*120_000,10*(i+1)) for i in range(3)]}
    chart = [bar(i*60_000,i+1) for i in range(6)]
    for overrides, sma_values in [({}, [None,None,None,15,15,25]),
                                   ({input_id:1}, [None,10,10,20,20,30])]:
        result = program.run(chart, chart_symbol="A", chart_timeframe="1",
                             request_bars=requested, input_overrides=overrides)
        assert [plot["values"] for plot in result["plots"]] == [
            sma_values, [None,10,10,10,10,10], [None,10,10,15,15,20]]


@pytest.mark.parametrize("version", [4,5,6])
def test_requested_global_alias_does_not_capture_same_named_udf_local(version):
    declaration = "study" if version == 4 else "indicator"
    request = "security" if version == 4 else "request.security"
    source = (f'//@version={version}\n{declaration}("shadow graph")\n'
              'f(src)=>\n    value=0.0\n    value:=nz(value[1])+src\n    value\n'
              f'value=f(close)\nplot(value)\nplot({request}("B","2",value))\n')
    program = pine_compat.compile_script(source)
    result = program.run([bar(i*60_000,i+1) for i in range(6)],
                         chart_symbol="A",chart_timeframe="1",
                         request_bars={"B:2":[bar(i*120_000,10*(i+1)) for i in range(3)]})
    assert [plot["values"] for plot in result["plots"]] == [
        [1,3,6,10,15,21],[None,10,10,30,30,60]]


@pytest.mark.parametrize("version", [5,6])
def test_requested_constants_color_calendar_and_reused_static_wrapper(version):
    source = (f'//@version={version}\nindicator("value graph",dynamic_requests=false)\n'
              'f(src)=>request.security("B","2",src)\na=f(close)\nb=f(a)\nplot(a)\nplot(b)\n'
              '[x,y,z]=request.security("B","2",[math.pi,color.r(color.new(color.rgb(12,34,56),25)),minute(time)+second(time_close)])\n'
              'plot(x)\nplot(y)\nplot(z)\n')
    program = pine_compat.compile_script(source)
    result = program.run([bar(i*60_000,i+1) for i in range(6)],
                         chart_symbol="A",chart_timeframe="1",
                         request_bars={"B:2":[bar(i*120_000,10*(i+1)) for i in range(3)]})
    values = [plot["values"] for plot in result["plots"]]
    assert values[:2] == [[None,10,10,20,20,30]]*2
    assert values[2][0] is None
    assert values[2][1:] == pytest.approx([3.141592653589793]*5)
    assert values[3:] == [[None,12,12,12,12,12],[None,0,0,2,2,4]]

import pytest
import pine_compat

SOURCE = '//@version=5\nindicator("V5 transparency modern oracle 23",precision=0)\nt=input.int(40)\nbgcolor(color.green,transp=t,force_overlay=true)\nplotshape(true,color=color.blue,transp=30,display=display.all,force_overlay=true,title="shape")\nplotchar(true,color=color.red,transp=20,force_overlay=true,title="char")\nplotarrow(1,colorup=color.blue,colordown=color.red,transp=10,force_overlay=true,title="arrow")\np=plot(1,color=color.blue,transp=50,force_overlay=true,title="line")\nq=plot(2,force_overlay=true,title="line2")\nfill(p,q,color=color.green,transp=60,title="fill")\n'
OVERLAY = '//@version=6\nindicator("V6 output overlay oracle 23",precision=0)\nbgcolor(color.green,force_overlay=true)\nplotchar(true,title="char",color=color.red,force_overlay=true)\nplot(1,title="marker")\n'
BARS = [{"time": 0, "open": 1.0, "high": 1.0, "low": 1.0, "close": 1.0, "volume": 1.0}]


def test_v5_transparency_keeps_native_values_and_modern_metadata():
    result = pine_compat.compile_script(SOURCE).run(BARS)
    assert [result[k][0]["values"] for k in ["plotShapes", "plotChars", "plotArrows"]] == [[True], [True], [1]]
    assert [p["values"] for p in result["plots"]] == [[1], [2]]
    for key in ["bgColors", "plotShapes", "plotChars", "plotArrows", "plots"]:
        assert result[key][0]["forceOverlay"] is True
    assert result["plotChars"][0]["title"] == "char"
    assert len(result["fills"]) == 1
    repeated = pine_compat.compile_script('//@version=5\nindicator("duplicate")\nplotshape(true,transp=0,transp=0)\nplot(1)\n').run(BARS)
    assert repeated["plotShapes"][0]["values"] == [True]


@pytest.mark.parametrize("version", [5, 6])
def test_bgcolor_plotchar_overlay_without_transparency(version):
    result = pine_compat.compile_script(OVERLAY.replace("version=6", f"version={version}")).run(BARS)
    assert result["bgColors"][0]["forceOverlay"] is True
    assert result["plotChars"][0]["forceOverlay"] is True
    assert result["plots"][0]["values"] == [1]


def test_transparency_rejections_remain_versioned_and_typed():
    for version, body, message in [
        (6, "bgcolor(color.green,transp=30)", "no argument named"),
        (5, "bgcolor(color.green,transp=bar_index%100)", "input integer"),
        (5, "plotshape(true,transp=30,transp=40)", "more than once"),
        (5, "barcolor(color.green,transp=30)", "no argument named"),
    ]:
        with pytest.raises(ValueError, match=message):
            pine_compat.compile_script(f'//@version={version}\nindicator("negative")\n{body}\nplot(1)\n')

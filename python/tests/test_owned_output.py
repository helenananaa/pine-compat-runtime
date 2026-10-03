import copy
import math

import pine_compat


def typed_tree(value):
    if isinstance(value, dict):
        return [(key, typed_tree(item)) for key, item in value.items()]
    if isinstance(value, list):
        return [typed_tree(item) for item in value]
    return type(value), value


def test_owned_snapshot_preserves_borrowed_types_and_independence():
    source = '''//@version=6
strategy("owned output")
plot(close)
plot(1)
plotchar(true)
plot(float(na))
plotshape(close > open)
bgcolor(color.red)
label.new(bar_index, close, "你好\\n\\\"")
strategy.entry("buy", strategy.long)
'''
    def bar(index, close):
        return dict(time=index*60000, open=1., high=close, low=1., close=close, volume=10.)

    session = pine_compat.compile_script(source).realtime_session()
    seeded = session.seed([bar(0, 2.), bar(1, 3.)])
    before = copy.deepcopy(seeded)
    replica = session.replica()
    assert typed_tree(seeded) == typed_tree(replica.result())
    assert typed_tree(session.result()) == typed_tree(replica.result())
    change = session.apply_confirmed(bar(2, 4.))
    replica.apply(change)
    assert typed_tree(session.result()) == typed_tree(replica.result())
    assert seeded == before
    assert session.result()["plots"][0]["values"][-1] == 4.


def test_repeated_scalar_histories_preserve_types_bits_and_list_independence():
    source = '''//@version=6
indicator("scalar sharing")
plot(1000)
plot(bar_index % 2 == 0 ? -0.0 : 0.0)
bgcolor(color.red)
'''
    bars = [dict(time=i*60000, open=1., high=1., low=1., close=1., volume=1.)
            for i in range(128)]
    session = pine_compat.compile_script(source).realtime_session()
    seeded = session.seed(bars)
    replica = session.replica()
    for result in [seeded, session.result(), replica.result()]:
        assert result["plots"][0]["values"] == [1000] * 128
        assert all(type(value) is int for value in result["plots"][0]["values"])
        zeros = result["plots"][1]["values"]
        assert all(type(value) is float for value in zeros)
        assert [math.copysign(1., value) for value in zeros] == [-1., 1.] * 64
        assert len(set(result["bgColors"][0]["values"])) == 1
    seeded["plots"][0]["values"][0] = 42
    seeded["bgColors"][0]["values"][0] = None
    assert session.result()["plots"][0]["values"][0] == 1000
    assert replica.result()["plots"][0]["values"][0] == 1000
    assert session.result()["bgColors"][0]["values"][0] is not None

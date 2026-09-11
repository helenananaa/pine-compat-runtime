import pine_compat as pine


def bar(i, bump=0):
    close = 100.0 + i + bump
    return dict(time=i * 60000, open=close, high=close, low=close, close=close, volume=1.)


SOURCE = '''//@version=6
indicator("retention boundaries")
var l = line.new(0, close, 0, close)
var t = table.new(position.top_right, 1, 1)
line.set_xy2(l, bar_index, close)
table.cell(t, 0, 0, str.tostring(close))
alert("event", alert.freq_all)
a = plot(close)
b = plot(ta.sma(close, 3))
fill(a, b, color.red)
plotshape(close > 0)
plotcandle(open, high, low, close)
bgcolor(color.red)
'''


def test_physical_retention_boundary_preserves_values_drawings_and_alerts():
    limited = pine.create_realtime_session(SOURCE)
    full = pine.create_realtime_session(SOURCE)
    limited.set_output_retention(16)
    bars = [bar(i) for i in range(400)]
    limited.seed(bars)
    full.seed(bars)
    replica = limited.replica()
    for i in range(400, 430):
        for bump in (0., 1.):
            change = limited.apply_forming(bar(i, bump))
            full.apply_forming(bar(i, bump))
            replica.apply(change)
            assert replica.result() == limited.result()
            assert len(limited.result()['plots'][0]['values']) == 17
        replica.apply(limited.apply_confirmed(bar(i)))
        full.apply_confirmed(bar(i))
        actual, reference = limited.result(), full.result()
        assert actual == replica.result()
        for small, large in zip(actual['plots'], reference['plots']):
            assert small['values'] == large['values'][-16:]
        assert actual['alerts'] == [a for a in reference['alerts'] if a['barIndex'] >= limited.display_origin]
        assert len(actual['plotShapes'][0]['values']) == 16
        assert len(actual['plotCandles'][0]['closes']) == 16
        assert len(actual['fills'][0]['colors']) == 16


def test_complete_result_updates_also_apply_retention_after_compaction():
    session = pine.create_realtime_session('//@version=6\nindicator("full")\nplot(close)')
    session.set_output_retention(8)
    session.seed([bar(i) for i in range(300)])
    for i in range(300, 320):
        assert len(session.update_forming(bar(i))['plots'][0]['values']) == 9
        assert len(session.update_confirmed(bar(i))['plots'][0]['values']) == 8


def test_expanding_retention_does_not_relabel_already_discarded_history():
    session = pine.create_realtime_session('//@version=6\nindicator("expand")\nplot(close)')
    session.set_output_retention(3)
    session.seed([bar(i) for i in range(300)])
    replica = session.replica()
    session.set_output_retention(None)
    assert session.display_origin == 297
    replica.apply(session.apply_confirmed(bar(300)))
    assert session.display_origin == 297
    assert replica.result() == session.result()
    assert session.result()['plots'][0]['values'] == [397., 398., 399., 400.]


def test_dense_broker_realized_profit_matches_ordered_closed_trade_sum():
    source = '//@version=6\nstrategy("ordered profit", process_orders_on_close=true)\nif strategy.position_size == 0\n    strategy.entry("L", strategy.long, qty=1)\nelse\n    strategy.close("L")\nplot(strategy.netprofit)'
    session = pine.create_realtime_session(source)
    session.set_output_retention(4)
    session.seed([])
    for i in range(160):
        session.apply_confirmed(bar(i, (i % 7) * 0.1))
        result = session.result()
        expected = 0.0
        # The script plot precedes this bar's process-on-close fill. Equity's
        # cash-derived netProfit has a different rounding path, so use the
        # script-visible realized sum at the same execution point.
        for trade in result['strategy']['trades']:
            if trade['exitBarIndex'] < i:
                expected += trade['profit']
        assert result['plots'][0]['values'][-1] == expected

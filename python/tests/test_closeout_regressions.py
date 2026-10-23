import pine_compat


def test_matrix_sum_keeps_float_element_type_for_missing_scalar():
    source = '''//@version=6
indicator("typed sum")
a=matrix.new<int>(1,1,2)
b=matrix.sum(a,float(na))
plot(b.get(0,0))
b.set(0,0,1.5)
plot(b.get(0,0))
plot(a.get(0,0))
'''
    bars = [dict(time=0, open=1.0, high=1.0, low=1.0, close=1.0, volume=1.0)]
    result = pine_compat.run_script(source, bars)
    assert [plot["values"] for plot in result["plots"]] == [[None], [1.5], [2]]


def test_numeric_timestamp_evaluates_each_argument_once():
    source = '''//@version=6
indicator("timestamp arguments")
var calls=array.new_int()
part(value)=>
    array.push(calls,value)
    value
plot(timestamp(part(2024),part(1),part(1),part(0),part(0),part(0)))
plot(array.size(calls))
'''
    bars = [dict(time=0, open=1.0, high=1.0, low=1.0, close=1.0, volume=1.0)]
    result = pine_compat.run_script(source, bars)
    assert [plot["values"] for plot in result["plots"]] == [[1704067200000], [6]]

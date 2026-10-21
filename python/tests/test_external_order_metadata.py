"""Order annotations are evaluated once; the Host owns fill selection/delivery."""
import pine_compat
import pytest

BARS = [dict(time=i*60000,open=10,high=11,low=9,close=10,volume=100) for i in range(3)]
ACCOUNTS = [dict(time=bar["time"],equity=1000,initial_capital=1000,netprofit=0,openprofit=0,
                 position_size=0,position_avg_price=None) for bar in BARS]
COMMON = 'comment="备注",alert_message="fill {{strategy.order.id}}",disable_alert=true'


def compile(body, version=6):
    return pine_compat.compile_script(f'//@version={version}\nstrategy("Annotations")\n'+body+'\n')


@pytest.mark.parametrize("version", [5,6])
@pytest.mark.parametrize("call", [
    f'strategy.entry("L",strategy.long,qty=1,{COMMON})',
    f'strategy.order("L",strategy.long,qty=1,{COMMON})',
    f'strategy.close("L",{COMMON})',
    f'strategy.close_all({COMMON})',
    f'strategy.exit("X","L",stop=9,{COMMON})',
])
def test_metadata_is_literal_utf8_and_survives_batch_and_incremental(call, version):
    program = compile(call, version)
    batch = program.run_external(BARS, ACCOUNTS, order_metadata=True)
    assert pine_compat.EXTERNAL_ORDER_METADATA_SCHEMA_VERSION == 1
    assert len(batch["intents"]) == 3
    for intent in batch["intents"]:
        assert intent["metadata"] == {"comment":"备注","alert_message":"fill {{strategy.order.id}}","disable_alert":True}
    assert "strategy" not in batch["output"]
    session = program.external_session(order_metadata=True)
    for i in range(3):
        assert session.append(BARS[i:i+1],ACCOUNTS[i:i+1]) == program.run_external(BARS[:i+1],ACCOUNTS[:i+1],order_metadata=True)


def test_exit_preserves_each_leg_without_selecting_a_fill_in_the_core():
    keys = ["comment","comment_profit","comment_loss","comment_trailing", "alert_message","alert_profit","alert_loss","alert_trailing"]
    expected = {key:key+" ✓" for key in keys}
    source = 'strategy.exit("X","L",limit=11,stop=9,trail_price=12,trail_offset=1,'
    source += ','.join(f'{key}="{value}"' for key,value in expected.items())+')'
    result = compile(source).run_external(BARS[:1],ACCOUNTS[:1],order_metadata=True)
    assert result["intents"][0]["metadata"] == {**expected,"disable_alert":False}


@pytest.mark.parametrize("argument,expected", [
    ('',None),('comment=string(na)',None),('disable_alert=false',None),
    ('comment=""',{"comment":"","disable_alert":False}),
    ('alert_message=""',{"alert_message":"","disable_alert":False}),
    ('disable_alert=true',{"disable_alert":True}),
])
def test_absent_and_empty_metadata_have_distinct_wire_semantics(argument, expected):
    suffix = ','+argument if argument else ''
    intent = compile(f'strategy.entry("L",strategy.long,qty=1{suffix})').run_external(BARS[:1],ACCOUNTS[:1],order_metadata=True)["intents"][0]
    assert intent.get("metadata") == expected
    assert ("metadata" in intent) == (expected is not None)


SIDE_EFFECT = '''var array<int> seen=array.new_int()
tag()=>
    array.push(seen,bar_index)
    "tag"+str.tostring(array.size(seen))
'''


def test_each_expression_is_evaluated_once_and_prefix_state_is_preserved():
    program = compile(SIDE_EFFECT+'''strategy.entry("L",strategy.long,qty=1,comment=tag(),alert_message=tag())
plot(array.size(seen))''')
    batch = program.run_external(BARS,ACCOUNTS,order_metadata=True)
    assert batch["output"]["plots"][0]["values"] == [2,4,6]
    assert [row["metadata"]["comment"] for row in batch["intents"]] == ["tag1","tag3","tag5"]
    assert [row["metadata"]["alert_message"] for row in batch["intents"]] == ["tag2","tag4","tag6"]
    session = program.external_session(order_metadata=True)
    for i in range(3):
        assert session.append(BARS[i:i+1],ACCOUNTS[i:i+1]) == program.run_external(BARS[:i+1],ACCOUNTS[:i+1],order_metadata=True)


def test_legacy_false_when_does_not_evaluate_annotations():
    program = compile(SIDE_EFFECT+'''strategy.entry("L",strategy.long,qty=1,when=false,comment=tag())
plot(array.size(seen))''',5)
    result = program.run_external(BARS,ACCOUNTS,order_metadata=True)
    assert result["intents"] == []
    assert result["output"]["plots"][0]["values"] == [0,0,0]


def test_zero_quantity_still_evaluates_metadata_like_native_without_emitting_intent():
    program = compile(SIDE_EFFECT+'''strategy.entry("L",strategy.long,qty=0,comment=tag())
plot(array.size(seen))''')
    result = program.run_external(BARS,ACCOUNTS,order_metadata=True)
    assert result["intents"] == []
    assert result["output"]["plots"] == program.run(BARS)["plots"]


@pytest.mark.parametrize("call", ['strategy.close("L",immediately=true)', 'strategy.close_all(immediately=true)', 'strategy.exit("X","L",stop=9,oca_name="group")'])
def test_annotations_do_not_silently_enable_immediate_matching(call):
    with pytest.raises(ValueError,match="E_EXTERNAL_UNSUPPORTED"):
        compile(call).run_external(BARS[:1],ACCOUNTS[:1],order_metadata=True)


def test_metadata_requires_explicit_host_opt_in_and_failures_poison_sessions():
    program = compile('strategy.entry("L",strategy.long,qty=1,comment="annotation")')
    with pytest.raises(ValueError,match="order metadata requires explicit Host opt-in"):
        program.run_external(BARS,ACCOUNTS)
    session = program.external_session()
    with pytest.raises(ValueError,match="order metadata requires explicit Host opt-in"):
        session.append(BARS[:1],ACCOUNTS[:1])
    with pytest.raises(ValueError,match="E_RUNTIME_POISONED"):
        session.append(BARS[:1],ACCOUNTS[:1])


def test_metadata_observes_the_host_account_instead_of_a_native_account():
    accounts = [{**frame,"equity":equity,"netprofit":equity-1000} for frame,equity in zip(ACCOUNTS,[1000,800,1600])]
    program = compile('strategy.entry("L",strategy.long,qty=1,comment=str.tostring(strategy.equity))')
    result = program.run_external(BARS,accounts,order_metadata=True)
    assert [row["metadata"]["comment"] for row in result["intents"]] == ["1000","800","1600"]

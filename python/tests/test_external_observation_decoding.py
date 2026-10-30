"""Direct tick conversion preserves the public JSON contract and atomic errors."""
from copy import deepcopy
import math
import random
import struct

import pine_compat
import pytest

from test_external_stream import SOURCE, data


class JsonDictionary(dict):
    """Built-in JSON accepts this, while the direct decoder uses exact dicts."""


def exercise(groups, bars, state, *, legacy):
    engine = pine_compat.compile_script(SOURCE).external_session(scheduled=True)
    replies = []
    for bar, group in zip(bars, groups, strict=True):
        engine.begin_bar(bar, bar_state=state)
        for observation in group:
            replies.append(engine.observe(JsonDictionary(observation) if legacy else observation, snapshot=False))
        replies.append(engine.commit_bar(snapshot=False))
    return replies, engine.snapshot()


@pytest.mark.parametrize("state", ["historical", "realtime"])
@pytest.mark.parametrize("requests", ["omitted", "null", "empty_data", "empty_updates"])
def test_observation_results_match_json_for_all_tick_and_empty_request_shapes(state, requests):
    bars, _, groups = data(state)
    for group in groups:
        for observation in group:
            if requests == "null":
                observation["evaluation"].update(request_data=None, request_updates=None)
            elif requests == "empty_data":
                observation["evaluation"]["request_data"] = []
            elif requests == "empty_updates":
                observation["evaluation"]["request_updates"] = []
    assert exercise(groups, bars, state, legacy=False) == exercise(groups, bars, state, legacy=True)


def edit(path, value):
    def mutate(observation):
        target = observation
        for key in path[:-1]:
            target = target[key]
        target[path[-1]] = value
    return mutate


MUTATIONS = [
    edit(("extra",), 1),
    edit(("evaluation", "extra"), 1),
    edit(("context", "extra"), 1),
    edit(("evaluation", "account", "extra"), 1),
    edit(("evaluation", "bar", "extra"), 1),  # Bar serde ignores unknown fields.
    edit(("evaluation", "bar", "time"), True),
    edit(("evaluation", "event_time_ms"), 1.5),
    edit(("evaluation", "event_time_ms"), 2**70),
    edit(("evaluation", "confirmed"), 1),
    edit(("context", "is_new"), 1),
    edit(("context", "trigger"), "unknown"),
    edit(("context", "bar_state"), "realtime "),
    edit(("evaluation", "account", "position_size"), True),
    edit(("evaluation", "account", "position_size"), "1"),
    edit(("evaluation", "account", "position_size"), 2**53 + 1),
    edit(("evaluation", "account", "equity"), float("nan")),
    edit(("evaluation", "account", "openprofit"), float("inf")),
    edit(("evaluation", "account", "netprofit"), -float("inf")),
    edit(("evaluation", "request_data"), {}),
    edit(("evaluation", "request_updates"), [dict(symbol="B")]),
    lambda item: item["evaluation"].update(request_data=[], request_updates=[]),
    edit(("evaluation", "bar", "volume"), []),
    edit(("evaluation", "bar", "close"), None),
    lambda item: item["evaluation"]["account"].pop("position_avg_price"),
    lambda item: item["evaluation"]["bar"].pop("open"),
    lambda item: item["context"].pop("is_new"),
    lambda item: item.pop("context"),
    lambda item: item.update(evaluation=JsonDictionary(item["evaluation"])),
]


@pytest.mark.parametrize("mutation", MUTATIONS)
def test_unusual_and_invalid_inputs_keep_the_same_result_error_and_retry_state(mutation):
    bars, _, groups = data("realtime")
    original = groups[0][0]
    changed = deepcopy(original)
    mutation(changed)
    outcomes = []
    for legacy in (False, True):
        engine = pine_compat.compile_script(SOURCE).external_session(scheduled=True)
        engine.begin_bar(bars[0], bar_state="realtime")
        before = engine.snapshot()
        try:
            reply = engine.observe(JsonDictionary(changed) if legacy else changed, snapshot=False)
            outcome = ("accepted", reply, engine.snapshot())
        except (ValueError, TypeError, OverflowError) as error:
            assert engine.snapshot() == before
            outcome = (type(error).__name__, str(error), engine.observe(original, snapshot=False), engine.snapshot())
        outcomes.append(outcome)
    assert outcomes[0] == outcomes[1]


def test_finite_float_bits_and_integer_boundaries_match_json_conversion():
    rng = random.Random(20261024)
    values = [0.0, -0.0, math.ulp(0.0), -math.ulp(0.0), 2**53, -(2**53), 2**53 + 1, -(2**53 + 1)]
    while len(values) < 1008:
        value = struct.unpack(">d", rng.getrandbits(64).to_bytes(8, "big"))[0]
        if math.isfinite(value):
            values.append(value)
    bars, _, groups = data("realtime")
    source = '//@version=6\nstrategy("conversion",calc_on_every_tick=true)\nplot(strategy.netprofit)\nplot(strategy.openprofit)'
    for value in values:
        observation = deepcopy(groups[0][0])
        observation["evaluation"]["account"].update(netprofit=value, openprofit=-0.0)
        outputs = []
        for legacy in (False, True):
            engine = pine_compat.compile_script(source).external_session(scheduled=True)
            engine.begin_bar(bars[0], bar_state="realtime")
            try:
                output = engine.observe(JsonDictionary(observation) if legacy else observation)
                outputs.append(repr(output))  # Preserves signed zero, unlike float equality.
            except ValueError as error:
                outputs.append((type(error).__name__, str(error)))
        assert outputs[0] == outputs[1], value


def test_empty_acknowledgements_and_window_status_are_independent_containers():
    bars, _, groups = data("historical")
    engine = pine_compat.compile_script('//@version=6\nstrategy("empty",calc_on_every_history_tick=true)\nplot(close)').external_session(scheduled=True)
    assert engine.window_status() is None
    engine.initialize_history(bars, observed_feedback=True, requested_history=[])
    status = engine.window_status()
    status["input_skip"] = 999
    assert engine.window_status()["input_skip"] == 0
    engine.begin_bar(bars[0], bar_state="historical")
    first = engine.observe(groups[0][0], snapshot=False)
    first["intents"].append({"unexpected": True})
    assert engine.observe(groups[0][1], snapshot=False)["intents"] == []
    assert engine.snapshot()["intents"] == []

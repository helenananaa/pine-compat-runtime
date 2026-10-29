//! Differential replay of requested checkpoints. Every fixture runs through the
//! same feeds twice: once on the incremental path and once with the complete
//! evaluator forced. Each step's error, delta and public result must be
//! byte-identical, and the forced runtime must never save a checkpoint.

use super::{COMPLETE_EVALUATOR, IncrementalCheck};
use crate::{
    Bar, BarUpdate, ChartContext, ExecutionLimits, HistoricalRuntime, InMemoryRequestDataProvider,
    RealtimeRuntime, RequestEnvironment, RequestKey, RequestTimeframe, RuntimeError,
    public_runtime_changes_json, public_runtime_result_json,
};
use pine_ir::HirProgram;
use pine_sema::analyze_source;
use pine_syntax::{Severity, SourceFile};
use std::{collections::HashMap, sync::Arc};

const MINUTE: i64 = 60_000;
const PERIOD: i64 = 5 * MINUTE;
const SPIKE_PERIOD: i64 = 15;
/// Requested close at the spike; budget fixtures exceed their limits there.
const SPIKE: f64 = 5_000.0;

struct Fixture {
    name: &'static str,
    definitions: &'static str,
    expression: &'static str,
    limits: ExecutionLimits,
    incremental: bool,
}

impl Fixture {
    fn incremental(
        name: &'static str,
        definitions: &'static str,
        expression: &'static str,
    ) -> Self {
        Self {
            name,
            definitions,
            expression,
            limits: ExecutionLimits::default(),
            incremental: true,
        }
    }

    fn fallback(name: &'static str, definitions: &'static str, expression: &'static str) -> Self {
        Self {
            incremental: false,
            ..Self::incremental(name, definitions, expression)
        }
    }

    fn limited(mut self, limits: ExecutionLimits) -> Self {
        self.limits = limits;
        self
    }

    fn program(&self, timeframe: &str, merge: &str) -> HirProgram {
        let source = format!(
            "//@version=6\nindicator(\"differential\")\n{}plot(request.security(\"B\", \"{timeframe}\", {}{merge}))\n",
            self.definitions, self.expression
        );
        let analysis = analyze_source(&SourceFile::new(format!("{}.pine", self.name), source));
        assert!(
            !analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.severity == Severity::Error),
            "{}: {:?}",
            self.name,
            analysis.diagnostics
        );
        analysis.hir.expect("fixture HIR")
    }
}

fn loop_fixtures() -> Vec<Fixture> {
    vec![
        Fixture::incremental(
            "single_iteration",
            "f() =>\n    float x = 0.0\n    for i = 0 to 0\n        x += close\n    x\n",
            "f()",
        ),
        Fixture::incremental(
            "break_continue_dynamic_history",
            "f() =>\n    float acc = 0.0\n    for i = 0 to math.min(bar_index, 8)\n        if i % 3 == 1\n            continue\n        if nz(close[i]) > close + 6\n            break\n        acc += nz(close[i]) * (i + 1)\n    acc\n",
            "f()",
        ),
        Fixture::incremental(
            "while_loop_local_var",
            "f() =>\n    int i = 0\n    float acc = 0.0\n    while i < 4\n        i += 1\n        var float seen = 0.0\n        seen += close % 3\n        if i == 2\n            continue\n        acc += seen + nz(high[i])\n        if acc > 1000000\n            break\n    acc\n",
            "f()",
        ),
        Fixture::incremental(
            "varip_and_var_in_loop",
            "f() =>\n    varip float ticks = 0.0\n    var float total = 0.0\n    for i = 1 to 2\n        ticks += 1\n        total += close / i\n    ticks * 1000 + total\n",
            "f()",
        ),
        Fixture::incremental(
            "nested_udfs",
            "inner(float src, int n) =>\n    float s = 0.0\n    for j = 0 to n\n        s += nz(src[j])\n    s\nouter() =>\n    float total = 0.0\n    for k = 1 to 3\n        total += inner(close, k)\n        if total > 100000\n            break\n    total\n",
            "outer()",
        ),
        Fixture::incremental(
            "loop_expressions",
            "f() =>\n    float last = for i = 4 to 0 by 2\n        nz(close[i]) - i\n    int n = 0\n    float grown = while n < 3\n        n += 1\n        close * n\n    last + grown\n",
            "f()",
        ),
        Fixture::incremental(
            "global_loop_initializer",
            "float window = for i = 0 to 2\n    nz(close[i])\n",
            "window * 2",
        ),
        Fixture::incremental(
            "ta_call_in_loop",
            "f() =>\n    float acc = 0.0\n    for i = 0 to 1\n        acc += nz(ta.change(close)) + i\n    acc\n",
            "f()",
        ),
    ]
}

/// Bar-time builtins, named constants, `switch`, and dependency graphs longer
/// or wider than the old depth-tracked walk admitted.
fn admission_fixtures() -> Vec<Fixture> {
    let chain = (1..=48)
        .map(|level| format!("x{level} = x{} + 1\n", level - 1))
        .collect::<String>();
    let udf_graph = (1..=10)
        .map(|level| format!("y{level} = f(y{}, y{}) / 2\n", level - 1, level - 1))
        .collect::<String>();
    vec![
        Fixture::incremental(
            "calendar_fields",
            "",
            "close + minute * 0.01 + hour + second + dayofmonth + month + year % 100 + weekofyear + (dayofweek == dayofweek.thursday ? 0.5 : 0.0)",
        ),
        Fixture::incremental("time_close", "", "close + (time_close - time) / 60000"),
        Fixture::incremental(
            "named_constants",
            "",
            "close * math.pi + color.r(color.red) + color.b(color.blue)",
        ),
        Fixture::incremental(
            "switch_expression",
            "f() =>\n    switch\n        close > open + 1 => close\n        close < open => open\n        => hl2\n",
            "f()",
        ),
        Fixture::incremental(
            "switch_selector_and_statement",
            "f() =>\n    float out = 0.0\n    switch minute % 3\n        0 =>\n            float bump = close * 2\n            out := bump\n        1 =>\n            out := nz(ta.change(close))\n        =>\n            out := hl2\n    out\n",
            "f()",
        ),
        Fixture::incremental("long_chain", format!("x0 = close\n{chain}").leak(), "x48"),
        Fixture::incremental(
            "udf_graph",
            format!("f(float a, float b) =>\n    a + nz(b[1])\ny0 = close\n{udf_graph}").leak(),
            "y10",
        ),
    ]
}

fn budget_fixtures() -> Vec<Fixture> {
    let iterations = ExecutionLimits {
        max_loop_iterations_per_bar: 400,
        ..ExecutionLimits::default()
    };
    let steps = ExecutionLimits {
        max_steps_per_bar: 20_000,
        ..ExecutionLimits::default()
    };
    vec![
        Fixture::incremental(
            "loop_iteration_budget",
            "f() =>\n    int n = close > 1000 ? 600 : 2\n    float acc = 0.0\n    for i = 1 to n\n        acc += i\n    acc\n",
            "f()",
        )
        .limited(iterations),
        Fixture::incremental(
            "runaway_while_budget",
            "f() =>\n    int i = 0\n    while close > 1000 or i < 2\n        i += 1\n    i\n",
            "f()",
        )
        .limited(iterations),
        Fixture::incremental(
            "step_budget",
            "f() =>\n    int n = close > 1000 ? 20000 : 2\n    float acc = 0.0\n    for i = 1 to n\n        acc += i\n    acc\n",
            "f()",
        )
        .limited(steps),
    ]
}

fn fallback_fixtures() -> Vec<Fixture> {
    vec![
        Fixture::fallback(
            "endpoint_read_in_loop",
            "f() =>\n    float acc = 0.0\n    for i = 0 to 1\n        if barstate.islast\n            acc += 1\n        acc += close\n    acc\n",
            "f()",
        ),
        Fixture::fallback(
            "nested_request_in_loop",
            "f() =>\n    float acc = 0.0\n    for i = 0 to 1\n        acc += request.security(\"C\", \"5\", close) + i\n    acc\n",
            "f()",
        ),
        Fixture::fallback(
            "confirmation_in_switch",
            "f() =>\n    switch\n        barstate.isconfirmed => close\n        => open\n",
            "f()",
        ),
        Fixture::fallback(
            "last_bar_time",
            "",
            "close + (last_bar_time - time) / 60000",
        ),
    ]
}

/// Forces the complete evaluator on this thread until dropped.
struct ForcedComplete;

impl ForcedComplete {
    fn new() -> Self {
        COMPLETE_EVALUATOR.with(|forced| forced.set(true));
        Self
    }
}

impl Drop for ForcedComplete {
    fn drop(&mut self) {
        COMPLETE_EVALUATOR.with(|forced| forced.set(false));
    }
}

trait Session {
    fn output(&self) -> String;
    /// Requested bars each chart-level checkpoint replayed when last evaluated.
    /// Nested requests checkpoint inside requested forks and are not listed.
    fn replays(&self) -> Vec<usize>;
}

impl Session for HistoricalRuntime<'_> {
    fn output(&self) -> String {
        public_runtime_result_json(&self.result())
    }

    fn replays(&self) -> Vec<usize> {
        self.request_evaluations
            .values()
            .map(|saved| saved.replayed_bars)
            .collect()
    }
}

impl Session for RealtimeRuntime<'_> {
    fn output(&self) -> String {
        public_runtime_result_json(&self.result())
    }

    fn replays(&self) -> Vec<usize> {
        self.live_runtime().replays()
    }
}

struct Pair<'f, R> {
    fixture: &'f Fixture,
    fast: R,
    full: R,
    checkpointed: bool,
}

impl<R: Session> Pair<'_, R> {
    /// Applies one host action to both runtimes. A bounded step executes at
    /// most one chart bar, so a checkpoint replays at most two requested bars.
    fn step(
        &mut self,
        label: &str,
        bounded: bool,
        mut action: impl FnMut(&mut R) -> Result<String, RuntimeError>,
    ) -> bool {
        let name = self.fixture.name;
        let fast = action(&mut self.fast).map_err(|error| error.message);
        let full = {
            let _complete = ForcedComplete::new();
            action(&mut self.full).map_err(|error| error.message)
        };
        assert_eq!(fast, full, "{name}/{label}");
        assert_eq!(self.fast.output(), self.full.output(), "{name}/{label}");
        assert!(
            self.full.replays().is_empty(),
            "{name}/{label}: the forced complete evaluator saved a checkpoint"
        );
        let replays = self.fast.replays();
        assert!(
            !bounded || replays.iter().all(|replayed| *replayed <= 2),
            "{name}/{label}: checkpoint replays {replays:?}"
        );
        self.checkpointed |= !replays.is_empty();
        fast.is_ok()
    }
}

fn price(index: i64) -> f64 {
    100.0 + ((index * 7) % 13) as f64
}

fn requested(time: i64, close: f64) -> Bar {
    Bar {
        time,
        open: close - 1.0,
        high: close + 2.0,
        low: close - 2.0,
        close,
        volume: 10.0,
    }
}

fn chart(minute: i64, close: f64) -> Bar {
    Bar {
        time: minute * MINUTE,
        open: close,
        high: close + 0.5,
        low: close - 0.5,
        close,
        volume: 1.0,
    }
}

fn delta(changes: Option<crate::RuntimeChanges>) -> String {
    changes.map_or_else(String::new, |changes| public_runtime_changes_json(&changes))
}

/// Live five-minute period: forming requested and chart ticks, then confirms.
fn live_period(
    pair: &mut Pair<'_, RealtimeRuntime<'_>>,
    key: &RequestKey,
    period: i64,
    spike: bool,
) {
    let close = if spike && period == SPIKE_PERIOD {
        SPIKE
    } else {
        price(period)
    };
    for offset in 0..5 {
        let minute = period * 5 + offset;
        for tick in 0..3 {
            let bar = requested(period * PERIOD, close + (offset * 3 + tick) as f64 * 0.25);
            pair.step(
                &format!("request forming {minute}/{tick}"),
                true,
                |runtime| {
                    runtime
                        .apply_request_update(key.clone(), BarUpdate::forming(bar))
                        .map(delta)
                },
            );
            let bar = chart(minute, price(minute) + tick as f64);
            pair.step(&format!("chart forming {minute}/{tick}"), true, |runtime| {
                runtime
                    .apply_update(BarUpdate::forming(bar))
                    .map(|changes| public_runtime_changes_json(&changes))
            });
        }
        if offset == 4 {
            let bar = requested(period * PERIOD, close + 4.0);
            pair.step(&format!("request confirmed {period}"), true, |runtime| {
                runtime
                    .apply_request_update(key.clone(), BarUpdate::confirmed(bar))
                    .map(delta)
            });
        }
        let bar = chart(minute, price(minute) + 0.5);
        pair.step(&format!("chart confirmed {minute}"), true, |runtime| {
            runtime
                .apply_update(BarUpdate::confirmed(bar))
                .map(|changes| public_runtime_changes_json(&changes))
        });
    }
}

/// Provider history, historical feed growth, realtime forming updates, a
/// requested spike, then history corrections that trim and rebuild the feed.
fn realtime_session(fixture: &Fixture, merge: &str) -> bool {
    let program = fixture.program("5", merge);
    let key = RequestKey::new("B", RequestTimeframe::parse("5").unwrap());
    let nested = RequestKey::new("C", RequestTimeframe::parse("5").unwrap());
    let provider = InMemoryRequestDataProvider::from_streams([
        (
            key.clone(),
            (0..8)
                .map(|period| requested(period * PERIOD, price(period)))
                .collect(),
        ),
        (
            nested,
            (0..40)
                .map(|period| requested(period * PERIOD, price(period + 3)))
                .collect(),
        ),
    ])
    .unwrap();
    let environment = RequestEnvironment::new(
        ChartContext::new("A", RequestTimeframe::parse("1").unwrap()),
        Arc::new(provider),
    );
    let runtime = || {
        RealtimeRuntime::with_request_environment(&program, environment.clone())
            .with_execution_limits(fixture.limits)
    };
    let mut pair = Pair {
        fixture,
        fast: runtime(),
        full: runtime(),
        checkpointed: false,
    };
    let seed = (0..40)
        .map(|minute| chart(minute, price(minute)))
        .collect::<Vec<_>>();
    pair.step("seed", false, |runtime| {
        runtime.seed_historical(&seed).map(|_| String::new())
    });
    for period in 8..12 {
        let bar = requested(period * PERIOD, price(period));
        pair.step(&format!("request history {period}"), true, |runtime| {
            runtime
                .apply_request_update(key.clone(), BarUpdate::historical(bar))
                .map(delta)
        });
        for minute in period * 5..period * 5 + 5 {
            let bar = chart(minute, price(minute));
            pair.step(&format!("chart history {minute}"), true, |runtime| {
                runtime
                    .apply_update(BarUpdate::historical(bar))
                    .map(|changes| public_runtime_changes_json(&changes))
            });
        }
    }
    for period in 12..18 {
        live_period(&mut pair, &key, period, true);
    }
    // Retain chart history through period 14 so the spike's requested bar is
    // trimmed, then replace the last two periods with corrected prices.
    let corrected = (65..75)
        .map(|minute| chart(minute, price(minute) - 3.0))
        .collect::<Vec<_>>();
    pair.step("correct periods 13-14", false, |runtime| {
        runtime
            .correct_historical(13 * PERIOD, &corrected)
            .map(|result| public_runtime_result_json(&result))
    });
    for period in 15..19 {
        live_period(&mut pair, &key, period, false);
    }
    // Roll back only the latest confirmed chart bar.
    let last = [chart(94, price(94) + 2.0)];
    pair.step("correct last bar", false, |runtime| {
        runtime
            .correct_historical(94 * MINUTE, &last)
            .map(|result| public_runtime_result_json(&result))
    });
    live_period(&mut pair, &key, 19, false);
    pair.checkpointed
}

/// The audit's feed mode: an empty provider grows one requested bar per chart bar.
fn historical_feed(fixture: &Fixture) -> bool {
    let program = fixture.program("1", "");
    let key = RequestKey::new("B", RequestTimeframe::parse("1").unwrap());
    let provider = InMemoryRequestDataProvider::from_streams([(
        RequestKey::new("C", RequestTimeframe::parse("5").unwrap()),
        (0..16)
            .map(|period| requested(period * PERIOD, price(period + 3)))
            .collect(),
    )])
    .unwrap();
    let environment = RequestEnvironment::new(
        ChartContext::new("A", RequestTimeframe::parse("1").unwrap()),
        Arc::new(provider),
    );
    let runtime = || {
        HistoricalRuntime::with_request_environment(&program, environment.clone())
            .with_execution_limits(fixture.limits)
    };
    let mut pair = Pair {
        fixture,
        fast: runtime(),
        full: runtime(),
        checkpointed: false,
    };
    for minute in 0..64 {
        let close = if minute == 48 { SPIKE } else { price(minute) };
        let bar = requested(minute * MINUTE, close);
        pair.step(&format!("feed request {minute}"), true, |runtime| {
            runtime
                .apply_request_update(key.clone(), BarUpdate::historical(bar))
                .map(|()| String::new())
        });
        let bar = chart(minute, price(minute + 1));
        // A failed historical append leaves no transactional checkpoint; stop
        // after both evaluators reported the same error.
        if !pair.step(&format!("feed chart {minute}"), true, |runtime| {
            runtime.append_bar(bar).map(|()| String::new())
        }) {
            break;
        }
    }
    pair.checkpointed
}

fn assert_differential(fixtures: &[Fixture]) {
    for fixture in fixtures {
        let checkpointed = [
            realtime_session(fixture, ""),
            realtime_session(fixture, ", lookahead=barmerge.lookahead_on"),
            historical_feed(fixture),
        ];
        assert_eq!(
            checkpointed, [fixture.incremental; 3],
            "{}: checkpoints in realtime, lookahead and feed sessions",
            fixture.name
        );
    }
}

#[test]
fn loop_requests_match_complete_evaluator() {
    assert_differential(&loop_fixtures());
}

#[test]
fn admitted_requests_match_complete_evaluator() {
    assert_differential(&admission_fixtures());
}

#[test]
fn loop_request_budgets_match_complete_evaluator() {
    assert_differential(&budget_fixtures());
}

#[test]
fn fallback_requests_match_complete_evaluator() {
    assert_differential(&fallback_fixtures());
}

/// Analysis rejects for...in loops and external mutation inside requested
/// expressions, so no request fixture reaches them. The runtime gate keeps
/// them off checkpoints on its own; check it on the same loops in chart scope.
#[test]
fn loop_admission_rejects_for_in_external_mutation_and_endpoint_reads() {
    for (definition, admitted) in [
        ("float total = for i = 0 to 2\n    nz(close[i])\n", true),
        (
            "int n = 0\nfloat total = while n < 2\n    n += 1\n    close\n",
            false,
        ),
        (
            "float total = for v in array.from(close, open)\n    v\n",
            false,
        ),
        (
            "var float hits = 0.0\nfloat total = for i = 0 to 2\n    hits += 1\n    close + hits\n",
            false,
        ),
        (
            "float total = for i = 0 to 2\n    if barstate.islast\n        break\n    close\n",
            false,
        ),
    ] {
        let source = format!("//@version=6\nindicator(\"admission\")\n{definition}plot(total)\n");
        let analysis = analyze_source(&SourceFile::new("admission.pine", source));
        assert!(
            !analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.severity == Severity::Error),
            "{definition}: {:?}",
            analysis.diagnostics
        );
        let hir = analysis.hir.unwrap();
        let runtime = HistoricalRuntime::new(&hir);
        let program = runtime.program.clone();
        let dependencies = runtime.metadata.request_dependencies(&program);
        let total = hir
            .symbols
            .iter()
            .find(|symbol| symbol.name == "total")
            .unwrap()
            .id;
        let captures = HashMap::new();
        assert_eq!(
            IncrementalCheck::new(dependencies, &captures)
                .admits(dependencies.initializer(total).unwrap()),
            admitted,
            "{definition}"
        );
    }
}

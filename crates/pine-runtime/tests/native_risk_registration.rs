use pine_runtime::{Bar, BarUpdate, HistoricalRuntime, PineValue, RealtimeRuntime, RuntimeResult};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

fn program(rules: &str, body: &str) -> pine_ir::HirProgram {
    let source = format!(
        "//@version=6\nstrategy(\"Declarative risk\",initial_capital=1000,margin_long=0,margin_short=0)\n{rules}\n{body}\nplot(strategy.position_size)"
    );
    let analysis = analyze_source(&SourceFile::new("native-risk", source));
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    analysis.hir.unwrap()
}

fn bars(prices: &[f64]) -> Vec<Bar> {
    prices
        .iter()
        .enumerate()
        .map(|(i, &price)| Bar {
            time: i as i64 * 60_000,
            open: price,
            high: price,
            low: price,
            close: price,
            volume: 100.0,
        })
        .collect()
}

const ENTRY: &str = "if bar_index==0\n    strategy.entry(\"L\",strategy.long,qty=10)";

fn positions(result: &RuntimeResult) -> Vec<PineValue> {
    result.plots.last().unwrap().values.clone()
}

#[test]
fn hidden_and_late_declarations_apply_before_the_first_order() {
    for rules in [
        "if false\n    strategy.risk.max_position_size(2)",
        "if bar_index>100\n    strategy.risk.max_position_size(2)",
    ] {
        let hir = program(rules, ENTRY);
        let result = pine_runtime::run_historical(&hir, &bars(&[100.; 3])).unwrap();
        assert_eq!(positions(&result), [0., 2., 2.].map(PineValue::Float));
    }
    let hir = program(ENTRY, "strategy.risk.max_position_size(2)");
    let result = pine_runtime::run_historical(&hir, &bars(&[100.; 3])).unwrap();
    assert_eq!(positions(&result), [0., 2., 2.].map(PineValue::Float));
}

#[test]
fn duplicate_position_caps_use_the_strictest_value_in_either_source_order() {
    for rules in [
        "strategy.risk.max_position_size(2)\nstrategy.risk.max_position_size(5)",
        "strategy.risk.max_position_size(5)\nstrategy.risk.max_position_size(2)",
    ] {
        let hir = program(rules, ENTRY);
        let result = pine_runtime::run_historical(&hir, &bars(&[100.; 3])).unwrap();
        assert_eq!(positions(&result), [0., 2., 2.].map(PineValue::Float));
    }
}

#[test]
fn monetary_rules_keep_cash_and_percent_limits_independent() {
    for function in ["max_drawdown", "max_intraday_loss"] {
        for (strict, loose) in [
            ("2,strategy.percent_of_equity", "10000,strategy.cash"),
            ("20,strategy.cash", "99,strategy.percent_of_equity"),
        ] {
            for declarations in [
                format!("strategy.risk.{function}({strict})\nstrategy.risk.{function}({loose})"),
                format!("strategy.risk.{function}({loose})\nstrategy.risk.{function}({strict})"),
            ] {
                let hir = program(&declarations, ENTRY);
                let result =
                    pine_runtime::run_historical(&hir, &bars(&[100., 100., 96., 96.])).unwrap();
                assert_eq!(
                    positions(&result),
                    [0., 10., 0., 0.].map(PineValue::Float),
                    "{declarations}"
                );
            }
        }
    }
}

#[test]
fn count_rules_cannot_be_loosened_by_a_later_declaration() {
    let body = "if strategy.position_size==0\n    strategy.entry(\"L\",strategy.long,qty=1)\nelse\n    strategy.close(\"L\")";
    let input = bars(&[100.; 8]);
    let reference = program("strategy.risk.max_intraday_filled_orders(2)", body);
    let expected = pine_runtime::run_historical(&reference, &input).unwrap();
    let candidate = program(
        "if false\n    strategy.risk.max_intraday_filled_orders(2)\nstrategy.risk.max_intraday_filled_orders(5)",
        body,
    );
    let actual = pine_runtime::run_historical(&candidate, &input).unwrap();
    assert_eq!(positions(&actual), positions(&expected));
    assert_eq!(actual.strategy, expected.strategy);
}

#[test]
fn consecutive_loss_day_rules_preserve_the_strictest_count() {
    let body = "if strategy.position_size==0\n    strategy.entry(\"L\",strategy.long,qty=1)\nelse\n    strategy.close(\"L\")";
    let mut input = bars(&[100., 100., 90., 90., 80., 80., 70., 70.]);
    for (index, bar) in input.iter_mut().enumerate() {
        bar.time = index as i64 * 86_400_000;
    }
    let reference = program("strategy.risk.max_cons_loss_days(1)", body);
    let expected = pine_runtime::run_historical(&reference, &input).unwrap();
    assert!(expected.strategy.as_ref().unwrap().orders.len() >= 2);
    for rules in [
        "if false\n    strategy.risk.max_cons_loss_days(1)\nstrategy.risk.max_cons_loss_days(3)",
        "strategy.risk.max_cons_loss_days(3)\nstrategy.risk.max_cons_loss_days(1)",
    ] {
        let hir = program(rules, body);
        let actual = pine_runtime::run_historical(&hir, &input).unwrap();
        assert_eq!(actual.strategy, expected.strategy);
    }
}

#[test]
fn monetary_halts_select_the_triggered_rule_alert_instead_of_the_last_declaration() {
    for function in ["max_drawdown", "max_intraday_loss"] {
        let rules = format!(
            "strategy.risk.{function}(2,strategy.percent_of_equity,\"strict\")\nstrategy.risk.{function}(10000,strategy.cash,\"loose\")"
        );
        let hir = program(&rules, ENTRY);
        let actual = pine_runtime::run_historical(&hir, &bars(&[100., 100., 96., 96.])).unwrap();
        assert_eq!(
            actual.strategy.unwrap().alerts.last().unwrap().message,
            "strict"
        );
    }
}

#[test]
fn direction_declarations_apply_together_without_limiting_generic_orders() {
    let rules = "if false\n    strategy.risk.allow_entry_in(strategy.direction.long)\nstrategy.risk.allow_entry_in(strategy.direction.all)";
    let body = "if bar_index==0\n    strategy.entry(\"S\",strategy.short,qty=10)\nif bar_index==1\n    strategy.order(\"O\",strategy.short,qty=1)";
    let hir = program(rules, body);
    let actual = pine_runtime::run_historical(&hir, &bars(&[100.; 4])).unwrap();
    assert_eq!(positions(&actual), [0., 0., -1., -1.].map(PineValue::Float));
}

#[test]
fn registration_survives_incremental_clone_and_realtime_rollback() {
    let hir = program("if false\n    strategy.risk.max_position_size(2)", ENTRY);
    let input = bars(&[100.; 4]);
    let expected = pine_runtime::run_historical(&hir, &input).unwrap();
    let mut prefix = HistoricalRuntime::new(&hir);
    prefix.append_bar(input[0]).unwrap();
    let mut restored = prefix.clone();
    restored.append_bars(&input[1..]).unwrap();
    assert_eq!(positions(&restored.result()), positions(&expected));
    let mut realtime = RealtimeRuntime::new(&hir);
    realtime.seed_historical(&input[..2]).unwrap();
    for _ in 0..3 {
        realtime.update(BarUpdate::forming(input[2])).unwrap();
    }
    let confirmed = realtime.update(BarUpdate::confirmed(input[2])).unwrap();
    assert_eq!(positions(&confirmed), [0., 2., 2.].map(PineValue::Float));
}

#[test]
fn unresolved_hidden_directives_fail_closed_without_executing_side_effects() {
    let hir = program(
        "f()=>\n    runtime.error(\"must not execute\")\n    2.0\nif false\n    strategy.risk.max_position_size(f())",
        ENTRY,
    );
    let error = pine_runtime::run_historical(&hir, &bars(&[100.; 3])).unwrap_err();
    assert!(
        error.message.contains("E_NATIVE_RISK_UNSUPPORTED"),
        "{error:?}"
    );
    assert!(!error.message.contains("must not execute"));
}

#[test]
fn registration_uses_actual_symbol_and_timeframe_instead_of_synthetic_defaults() {
    for (expression, expected) in [
        ("str.length(syminfo.ticker)", 10.),
        ("str.length(syminfo.prefix)", 4.),
        ("timeframe.multiplier", 15.),
        ("timeframe.in_seconds()/60", 15.),
        ("timestamp(2020,1,1,0,0)>0 ? 2 : 3", 2.),
        ("str.length(str.match(syminfo.ticker,\"LONG\"))", 4.),
    ] {
        let hir = program(
            &format!("if false\n    strategy.risk.max_position_size({expression})"),
            "if bar_index==0\n    strategy.entry(\"L\",strategy.long,qty=100)",
        );
        let chart = pine_runtime::ChartContext::new(
            "TEST:LONGTICKER",
            pine_runtime::RequestTimeframe::parse("15").unwrap(),
        );
        let environment = pine_runtime::RequestEnvironment::new(
            chart,
            std::sync::Arc::new(pine_runtime::NoRequestDataProvider),
        );
        let mut runtime = HistoricalRuntime::with_request_environment(&hir, environment);
        runtime.append_bars(&bars(&[100.; 3])).unwrap();
        assert_eq!(
            positions(&runtime.result()),
            [0., expected, expected].map(PineValue::Float),
            "{expression}"
        );
    }
}

#[test]
fn local_scalar_loop_arguments_keep_existing_execution_semantics() {
    for flow in [
        "for i=1 to n\n        total+=1",
        "while total<n\n        total+=1",
    ] {
        let rules = format!(
            "f(simple int n)=>\n    total=0\n    {flow}\n    total\nif false\n    strategy.risk.max_position_size(f(2))"
        );
        let hir = program(&rules, ENTRY);
        let result = pine_runtime::run_historical(&hir, &bars(&[100.; 3])).unwrap();
        assert_eq!(positions(&result), [0., 2., 2.].map(PineValue::Float));
    }
}

#[test]
fn native_scalar_registration_respects_caller_budgets_instead_of_metadata_probe_caps() {
    let hir = program(
        "f(simple int n)=>\n    total=0\n    for i=1 to n\n        total+=1\n    total/10000\nif false\n    strategy.risk.max_position_size(f(20000))",
        ENTRY,
    );
    let expected = pine_runtime::run_historical(&hir, &bars(&[100.; 3])).unwrap();
    assert_eq!(positions(&expected), [0., 2., 2.].map(PineValue::Float));
    let mut limited =
        HistoricalRuntime::new(&hir).with_execution_limits(pine_runtime::ExecutionLimits {
            max_steps_per_bar: 10000,
            max_loop_iterations_per_bar: 100,
        });
    assert!(limited.append_bar(bars(&[100.])[0]).is_err());
    assert!(limited.result().strategy.unwrap().orders.is_empty());
}

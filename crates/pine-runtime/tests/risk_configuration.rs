use pine_ir::HirProgram;
use pine_runtime::{
    ChartContext, ExternalExecutionOptions, HistoricalRuntime, InputOverrides, PineValue,
    RequestTimeframe, strategy_risk_configuration,
};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

fn program(source: &str) -> HirProgram {
    let analysis = analyze_source(&SourceFile::new("risk", source));
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    analysis.hir.unwrap()
}
fn chart() -> ChartContext {
    ChartContext::new("EXCHANGE:TEST", RequestTimeframe::parse("1").unwrap())
}

#[test]
fn inventory_resolves_all_six_rules_and_retains_stricter_duplicate_candidates() {
    let hir = program(
        "//@version=6\nstrategy(\"Risk\")\nn=input.int(3)\nif false\n    strategy.risk.max_position_size(n+1)\nstrategy.risk.allow_entry_in(strategy.direction.long)\nstrategy.risk.max_drawdown(10,strategy.percent_of_equity,\"DD\")\nstrategy.risk.max_intraday_loss(20,strategy.cash)\nstrategy.risk.max_intraday_filled_orders(2)\nstrategy.risk.max_cons_loss_days(3)\nstrategy.risk.max_position_size(8)",
    );
    let defaults = strategy_risk_configuration(&hir, &InputOverrides::new(), &chart());
    assert_eq!(defaults.schema_version, 1);
    assert_eq!(defaults.rules.len(), 7);
    assert!(defaults.rules.iter().all(|rule| rule.resolved));
    assert!(defaults.unresolved_call_site_ids.is_empty());
    assert_eq!(defaults.rules[0].value, Some(4.0));
    assert_eq!(defaults.rules[6].value, Some(8.0));
    assert_eq!(
        defaults.rules[1].direction.as_deref(),
        Some("strategy.direction.long")
    );
    assert_eq!(defaults.rules[2].alert_message.as_deref(), Some("DD"));
    let input = pine_runtime::input_calls(&hir)[0].call_site_id;
    let changed = strategy_risk_configuration(
        &hir,
        &InputOverrides::new().with_value(input, PineValue::Int(6)),
        &chart(),
    );
    assert_eq!(changed.rules[0].value, Some(7.0));
}

#[test]
fn context_and_inlined_pure_functions_resolve_without_evaluating_control_flow() {
    let hir = program(
        "//@version=6\nstrategy(\"Risk\")\nf(simple float n)=>n*2\nif false\n    strategy.risk.max_position_size(f(syminfo.mintick*8))\nwhile true\n    runtime.error(\"not executed by discovery\")",
    );
    let context = chart().with_price_grid(1, 4).unwrap();
    let result = strategy_risk_configuration(&hir, &InputOverrides::new(), &context);
    assert!(result.rules[0].resolved, "{result:?}");
    assert_eq!(result.rules[0].value, Some(4.0));
}

#[test]
fn side_effecting_argument_is_reported_unresolved_instead_of_executed() {
    let hir = program(
        "//@version=6\nstrategy(\"Risk\")\nf()=>\n    runtime.error(\"no metadata side effect\")\n    2.0\nstrategy.risk.max_position_size(f())",
    );
    let result = strategy_risk_configuration(&hir, &InputOverrides::new(), &chart());
    assert!(!result.rules[0].resolved);
    assert_eq!(
        result.unresolved_call_site_ids,
        vec![result.rules[0].call_site_id]
    );
    assert!(
        HistoricalRuntime::new(&hir)
            .with_external_accounts_and_options(
                vec![],
                None,
                ExternalExecutionOptions {
                    risk_rules: true,
                    ..Default::default()
                }
            )
            .is_err()
    );
}

#[test]
fn external_risk_contract_is_explicit_even_for_never_executed_calls() {
    let hir = program(
        "//@version=6\nstrategy(\"Risk\")\nif false\n    strategy.risk.max_position_size(2)",
    );
    assert!(
        HistoricalRuntime::new(&hir)
            .with_external_accounts(vec![])
            .is_err()
    );
    let runtime = HistoricalRuntime::new(&hir)
        .with_external_accounts_and_options(
            vec![],
            None,
            ExternalExecutionOptions {
                risk_rules: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        runtime.external_risk_configuration().unwrap().rules[0].value,
        Some(2.0)
    );
}

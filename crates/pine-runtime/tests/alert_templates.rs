use pine_runtime::{
    Bar, BarUpdate, HistoricalRuntime, RealtimeRuntime, host_requirements, run_historical,
    run_historical_with_execution_times,
};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

fn program(source: &str) -> pine_ir::HirProgram {
    let analysis = analyze_source(&SourceFile::new("template.pine", source));
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    analysis.hir.unwrap()
}

fn bar(time: i64, close: f64) -> Bar {
    Bar {
        time,
        open: close,
        high: close,
        low: close,
        close,
        volume: 1.0,
    }
}

#[test]
fn saved_native_templates_bind_forward_plots_and_explicit_execution_time() {
    for source in [
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/alert-template-v4-oracle-26.pine"
        ),
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/alert-template-v5-oracle-26.pine"
        ),
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/alert-template-v6-oracle-26.pine"
        ),
    ] {
        let hir = program(source);
        assert_eq!(
            host_requirements(&hir).execution.clock,
            "explicitTimestampWhenEvaluated"
        );
        assert!(
            run_historical(&hir, &[bar(0, 1.0)])
                .unwrap_err()
                .message
                .contains("explicit execution timestamp")
        );
        let result = run_historical_with_execution_times(
            &hir,
            &[bar(0, 1.0), bar(60000, 2.0)],
            &[1700000000123, 1700000001987],
        )
        .unwrap();
        assert!(result.diagnostics.is_empty());
        assert_eq!(
            result
                .alerts
                .iter()
                .map(|a| a.message.as_str())
                .collect::<Vec<_>>(),
            vec![
                "P0=1|P19=20|NOW=2023-11-14T22:13:20Z",
                "P0=1|P19=20|NOW=2023-11-14T22:13:21Z"
            ]
        );
    }
}

#[test]
fn named_and_numbered_templates_wait_for_current_values_and_preserve_event_order() {
    let hir = program(
        "//@version=6\nindicator(\"order\")\nalert(\"literal {{timenow}}\")\nalertcondition(true,\"forward\",'{{plot_0}}/{{plot(\"Later\")}}')\nalert(\"last\")\nplot(close,title=\"Later\")\n",
    );
    assert_eq!(host_requirements(&hir).execution.clock, "notUsed");
    let result = run_historical(&hir, &[bar(0, 1.25), bar(60000, 2.5)]).unwrap();
    assert!(result.diagnostics.is_empty());
    assert_eq!(
        result
            .alerts
            .iter()
            .map(|a| a.message.as_str())
            .collect::<Vec<_>>(),
        vec![
            "literal {{timenow}}",
            "1.25/1.25",
            "last",
            "literal {{timenow}}",
            "2.5/2.5",
            "last"
        ]
    );
}

#[test]
fn failed_forming_clock_does_not_leave_pending_events_or_previous_plot_values() {
    let hir = program(
        "//@version=6\nindicator(\"rollback\")\na=\"{{time\"+\"now}}/{{plot_0}}\"\nalertcondition(close>0,\"clock\",a)\nplot(close)\n",
    );
    assert_eq!(
        host_requirements(&hir).execution.clock,
        "explicitTimestampWhenEvaluated"
    );
    let mut runtime = RealtimeRuntime::new(&hir);
    let mut control = RealtimeRuntime::new(&hir);
    for r in [&mut runtime, &mut control] {
        r.update(BarUpdate::historical(bar(0, -1.0))).unwrap();
    }
    assert!(runtime.update(BarUpdate::forming(bar(60000, 2.0))).is_err());
    let actual = runtime
        .update_with_execution_time(BarUpdate::forming(bar(60000, 2.0)), 1700000000123)
        .unwrap();
    let expected = control
        .update_with_execution_time(BarUpdate::forming(bar(60000, 2.0)), 1700000000123)
        .unwrap();
    assert_eq!(actual, expected);
    assert_eq!(actual.alerts[0].message, "2023-11-14T22:13:20Z/2");
    let actual = runtime
        .update_with_execution_time(BarUpdate::confirmed(bar(60000, 3.0)), 1700000002123)
        .unwrap();
    assert_eq!(actual.alerts.len(), 1);
    assert_eq!(actual.alerts[0].message, "2023-11-14T22:13:22Z/3");
    let mut incremental = HistoricalRuntime::new(&hir);
    incremental.append_bar(bar(0, -1.0)).unwrap();
    incremental
        .append_bar_with_execution_time(bar(60000, 3.0), 1700000002123)
        .unwrap();
    assert_eq!(incremental.result(), actual);
}

#[test]
fn unqualified_mixed_plot_layouts_and_missing_slots_fail_explicitly() {
    for body in [
        "plotchar(true)\nplot(close)\nalertcondition(true,\"mixed\",\"{{plot_0}}\")",
        "alertcondition(true,\"missing\",\"{{plot_0}}\")",
    ] {
        let result = run_historical(
            &program(&format!("//@version=6\nindicator(\"guard\")\n{body}\n")),
            &[bar(0, 1.0)],
        )
        .unwrap();
        assert!(result.alerts.is_empty());
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.code == "E_UNSUPPORTED_ALERT_PLACEHOLDER")
        );
    }
}

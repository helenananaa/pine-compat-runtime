use pine_runtime::{Bar, BarUpdate, RealtimeRuntime, run_historical};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

fn compile(version: u8, body: &str) -> pine_ir::HirProgram {
    let declaration = if version == 4 { "study" } else { "indicator" };
    let source = format!("//@version={version}\n{declaration}(\"function alerts\")\n{body}");
    let analysis = analyze_source(&SourceFile::new("alerts.pine", source));
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
        open: 10.0,
        high: 20.0,
        low: 5.0,
        close,
        volume: 1.0,
    }
}

#[test]
fn nested_functions_keep_messages_frequency_and_distinct_call_sites() {
    for version in [4, 5, 6] {
        for (frequency, per_call) in [("once_per_bar", 1), ("all", 3), ("once_per_bar_close", 1)] {
            let program = compile(
                version,
                &format!(
                    "emit(msg) =>\n    alert(msg, alert.freq_{frequency})\nwrapped(msg) =>\n    for i = 0 to 2\n        emit(msg)\nif bar_index == 1\n    wrapped(\"first\")\n    wrapped(\"second\")\nplot(close)"
                ),
            );
            let result = run_historical(&program, &[bar(0, 10.0), bar(60000, 12.0)]).unwrap();
            assert_eq!(result.alerts.len(), per_call * 2);
            assert!(
                result.alerts[..per_call]
                    .iter()
                    .all(|a| a.message == "first" && a.bar_index == 1)
            );
            assert!(
                result.alerts[per_call..]
                    .iter()
                    .all(|a| a.message == "second" && a.bar_index == 1)
            );
            assert_ne!(result.alerts[0].id, result.alerts[per_call].id);
        }
    }
}

#[test]
fn forming_snapshots_roll_back_and_close_frequency_waits_for_confirmation() {
    let program = compile(
        6,
        r#"
emit(msg, freq) =>
    alert(msg, freq)
    close
if close > 10
    emit(str.tostring(close), alert.freq_all)
    emit("closed", alert.freq_once_per_bar_close)
plot(close)
"#,
    );
    let mut runtime = RealtimeRuntime::new(&program);
    runtime.seed_historical(&[bar(0, 10.0)]).unwrap();
    let forming = runtime
        .update(BarUpdate::forming(bar(60000, 12.0)))
        .unwrap();
    assert_eq!(forming.alerts.len(), 1);
    assert_eq!(forming.alerts[0].message, "12");
    let revised = runtime
        .update(BarUpdate::forming(bar(60000, 13.0)))
        .unwrap();
    assert_eq!(revised.alerts.len(), 1);
    assert_eq!(revised.alerts[0].message, "13");
    let withdrawn = runtime.update(BarUpdate::forming(bar(60000, 9.0))).unwrap();
    assert!(withdrawn.alerts.is_empty());
    let confirmed = runtime
        .update(BarUpdate::confirmed(bar(60000, 14.0)))
        .unwrap();
    assert_eq!(
        confirmed
            .alerts
            .iter()
            .map(|a| a.message.as_str())
            .collect::<Vec<_>>(),
        ["14", "closed"]
    );
}

#[test]
fn alertcondition_and_plot_remain_forbidden_in_functions() {
    for call in ["alertcondition(true, \"title\", \"msg\")", "plot(close)"] {
        let analysis = analyze_source(&SourceFile::new(
            "forbidden.pine",
            format!("//@version=6\nindicator(\"forbidden\")\nf() =>\n    {call}\n    close\nf()"),
        ));
        assert!(
            analysis
                .compatibility
                .unsupported
                .iter()
                .any(|u| u.feature == "function_side_effect")
        );
        assert!(analysis.hir.is_none());
    }
}

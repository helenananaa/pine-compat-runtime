use pine_runtime::{Bar, run_historical};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

fn execute(
    version: u16,
    body: &str,
) -> Result<pine_runtime::RuntimeResult, pine_runtime::RuntimeError> {
    let declaration = if version == 4 { "study" } else { "indicator" };
    let analysis = analyze_source(&SourceFile::new(
        "frequency.pine",
        format!("//@version={version}\n{declaration}(\"frequency\")\n{body}\n"),
    ));
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    run_historical(
        &analysis.hir.unwrap(),
        &[Bar {
            time: 0,
            open: 10.,
            high: 10.,
            low: 10.,
            close: 10.,
            volume: 1.,
        }],
    )
}

#[test]
fn input_frequency_uses_native_constant_values_and_per_callsite_filtering() {
    for version in [4, 5, 6] {
        for (constant, value, count) in [
            ("alert.freq_all", "all", 2),
            ("alert.freq_once_per_bar", "once_per_bar", 1),
            ("alert.freq_once_per_bar_close", "once_per_bar_close", 1),
        ] {
            let input = if version == 4 {
                format!("input({constant})")
            } else {
                format!("input.string({constant})")
            };
            let markers = if version == 4 {
                "plot(1)".to_owned()
            } else {
                format!("plot(str.length({constant}))\nplot({constant}==\"{value}\"?1:0)")
            };
            let result = execute(
                version,
                &format!(
                    "frequency={input}\nfor i=0 to 1\n    alert(\"probe\",frequency)\n{markers}"
                ),
            );
            let result = result.unwrap();
            assert_eq!(result.alerts.len(), count);
            if version >= 5 {
                assert_eq!(result.plots[0].values[0].as_f64(), Some(value.len() as f64));
                assert_eq!(result.plots[1].values[0].as_f64(), Some(1.));
            }
        }
    }
}

#[test]
fn invalid_frequency_is_rejected_only_when_the_alert_executes() {
    for version in [5, 6] {
        for expression in [
            "\"invalid\"",
            "\"alert.freq_all\"",
            "input.string(\"invalid\")",
        ] {
            let error =
                execute(version, &format!("alert(\"probe\",{expression})\nplot(1)")).unwrap_err();
            assert!(
                error.message.contains("unsupported alert frequency"),
                "{error:?}"
            );
            assert!(
                execute(
                    version,
                    &format!("if false\n    alert(\"probe\",{expression})\nplot(1)")
                )
                .is_ok()
            );
        }
    }
}

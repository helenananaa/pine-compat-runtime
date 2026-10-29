use pine_runtime::{Bar, PineValue, run_historical};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

fn source(version: u8, expr: &str) -> String {
    let declaration = if version <= 4 { "study" } else { "indicator" };
    format!("//@version={version}\n{declaration}(\"nz names\")\nplot({expr})")
}

#[test]
fn versioned_names_and_reordered_arguments_preserve_source_and_replacement() {
    let bars: Vec<_> = (0..3)
        .map(|i| Bar {
            time: i * 60000,
            open: 10.0 + i as f64,
            high: 10.0 + i as f64,
            low: 10.0 + i as f64,
            close: 10.0 + i as f64,
            volume: 1.0,
        })
        .collect();
    for version in [3, 4, 5, 6] {
        let (first, second) = if version <= 4 {
            ("x", "y")
        } else {
            ("source", "replacement")
        };
        for expr in [
            "nz(close[1], 7.0)".to_owned(),
            format!("nz({first}=close[1], {second}=7.0)"),
            format!("nz({second}=7.0, {first}=close[1])"),
            format!("nz(close[1], {second}=7.0)"),
        ] {
            let analysis = analyze_source(&SourceFile::new("nz.pine", source(version, &expr)));
            assert!(
                analysis.diagnostics.is_empty(),
                "{expr}: {:?}",
                analysis.diagnostics
            );
            let output = run_historical(&analysis.hir.unwrap(), &bars).unwrap();
            assert_eq!(
                output.plots[0].values,
                [7.0, 10.0, 11.0].map(PineValue::Float)
            );
        }
        let analysis = analyze_source(&SourceFile::new(
            "nz.pine",
            source(version, &format!("nz({first}=close[1])")),
        ));
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        let output = run_historical(&analysis.hir.unwrap(), &bars).unwrap();
        assert_eq!(output.plots[0].values[0], PineValue::Int(0));
    }
}

#[test]
fn wrong_version_names_duplicates_and_argument_order_are_rejected() {
    for version in [4, 5, 6] {
        let (first, second, wrong) = if version == 4 {
            ("x", "y", "source")
        } else {
            ("source", "replacement", "x")
        };
        for (expr, code) in [
            (format!("nz({wrong}=close)"), "E_CALL_ARG_NAME"),
            (format!("nz(close, {first}=close)"), "E_CALL_ARG_DUPLICATE"),
            (
                format!("nz({first}=close, {second}=1, {second}=2)"),
                "E_CALL_ARG_DUPLICATE",
            ),
            (format!("nz({first}=close, 0)"), "E_CALL_ARG_ORDER"),
        ] {
            let analysis = analyze_source(&SourceFile::new("nz.pine", source(version, &expr)));
            assert!(
                analysis.diagnostics.iter().any(|d| d.code == code),
                "{expr}: {:?}",
                analysis.diagnostics
            );
            assert!(analysis.hir.is_none());
        }
    }
}

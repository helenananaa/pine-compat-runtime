use pine_runtime::{Bar, run_historical};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

#[test]
fn native_object_methods_and_drawing_extension_controls() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let cases: serde_json::Value = serde_json::from_str(include_str!(
        "../../../docs/evidence/strategy-library-20261010/object-drawing-oracles-40.json"
    ))
    .unwrap();
    let bars = (0..3)
        .map(|i| Bar {
            time: i * 60000,
            open: 10.,
            high: 10.,
            low: 10.,
            close: 10.,
            volume: 1.,
        })
        .collect::<Vec<_>>();
    for case in cases["records"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let text = std::fs::read_to_string(root.join(case["source"].as_str().unwrap())).unwrap();
        let report = analyze_source(&SourceFile::new(name, text));
        if case["native_status"] == "analysis_rejected" {
            assert!(report.hir.is_none(), "{name}");
            continue;
        }
        assert!(
            report.diagnostics.is_empty(),
            "{name}: {:?}",
            report.diagnostics
        );
        let result = run_historical(&report.hir.unwrap(), &bars);
        if case["native_status"] == "runtime_error" {
            assert!(
                result.unwrap_err().message.contains("invalid extend value"),
                "{name}"
            );
            continue;
        }
        let values = result
            .unwrap()
            .plots
            .iter()
            .map(|p| p.values.last().unwrap().as_f64())
            .collect::<Vec<_>>();
        let expected = case["native_values"]
            .as_array()
            .unwrap()
            .iter()
            .map(serde_json::Value::as_f64)
            .collect::<Vec<_>>();
        assert_eq!(values, expected, "{name}");
    }
}

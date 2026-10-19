use pine_sema::analyze_source;
use pine_syntax::SourceFile;
use std::path::Path;

#[test]
fn native_dynamic_request_and_simple_argument_qualifiers_are_preserved() {
    let dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/evidence/strategy-library-20261010");
    let records: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(dir.join("fx-native-request-controls-31.json")).unwrap(),
    )
    .unwrap();
    for record in records.as_array().unwrap() {
        let name = record["stem"].as_str().unwrap();
        let source = std::fs::read_to_string(dir.join(format!("{name}.pine"))).unwrap();
        let analysis = analyze_source(&SourceFile::new(name, source));
        let accepted = record["native_status"] == "compiled_and_executed";
        assert_eq!(
            analysis.diagnostics.is_empty(),
            accepted,
            "{name}: {:?}",
            analysis.diagnostics
        );
    }
}

#[test]
fn native_currency_names_are_case_insensitive_and_unknown_identity_is_one() {
    let dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/evidence/strategy-library-20261010");
    let source = std::fs::read_to_string(dir.join("fx-name-controls-v6-oracle-31.pine")).unwrap();
    let analysis = analyze_source(&SourceFile::new("names.pine", source));
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let provider=pine_runtime::currency_rates_from_json(r#"{"schemaVersion":1,"pairs":[{"from":"USD","to":"EUR","rates":[{"effectiveTime":1792195200000,"rate":0.891}]},{"from":"USD","to":"ZZZ","unavailable":true}]}"#).unwrap();
    let environment = pine_runtime::RequestEnvironment::default()
        .with_currency_rates(std::sync::Arc::new(provider));
    let bars = [pine_runtime::Bar {
        time: 1792195200000,
        open: 1.,
        high: 1.,
        low: 1.,
        close: 1.,
        volume: 0.,
    }];
    let result = pine_runtime::run_historical_with_request_environment(
        &analysis.hir.unwrap(),
        &bars,
        environment,
    )
    .unwrap();
    assert_eq!(
        result
            .plots
            .iter()
            .map(|p| p.values[0].as_f64())
            .collect::<Vec<_>>(),
        [Some(1.), Some(0.891), Some(1.), None]
    );
}

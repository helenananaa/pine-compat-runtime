use pine_runtime::{Bar, HistoricalRuntime, PineValue};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

#[test]
fn native_contextual_simple_type_names_remain_ordinary_typed_declarations() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join(
            "docs/evidence/strategy-library-20261010/simple-declaration-name-oracles-44.json",
        ))
        .unwrap(),
    )
    .unwrap();
    let bars: Vec<_> = (0..3)
        .map(|i| Bar {
            time: i * 60000,
            open: 10.,
            high: 10.,
            low: 10.,
            close: 10.,
            volume: 1.,
        })
        .collect();
    for case in manifest["records"].as_array().unwrap() {
        let source = std::fs::read_to_string(root.join(case["source"].as_str().unwrap())).unwrap();
        let analysis = analyze_source(&SourceFile::new("simple-type.pine", source));
        assert!(
            analysis.diagnostics.is_empty(),
            "{}: {:?}",
            case["name"],
            analysis.diagnostics
        );
        let program = analysis.hir.unwrap();
        let mut runtime = HistoricalRuntime::new(&program);
        runtime.append_bars(&bars).unwrap();
        assert_eq!(
            runtime.result().plots[0]
                .values
                .iter()
                .map(PineValue::as_f64)
                .collect::<Vec<_>>(),
            vec![Some(case["native_values"][0].as_f64().unwrap()); 3]
        );
    }
}

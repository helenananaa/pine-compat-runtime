use pine_runtime::{Bar, run_historical};
use pine_sema::{AnalysisInput, analyze_input, analyze_source};
use pine_syntax::SourceFile;

#[test]
fn native_const_parameter_positive_sources_execute_with_their_recorded_values() {
    let oracle: serde_json::Value = serde_json::from_str(include_str!(
        "../../../docs/evidence/strategy-library-20261010/constant-param-oracles-35.json"
    ))
    .unwrap();
    let bars = (0..3)
        .map(|index| Bar {
            time: index * 60_000,
            open: 10.,
            high: 10.,
            low: 10.,
            close: 10.,
            volume: 1.,
        })
        .collect::<Vec<_>>();
    let mut failures = Vec::new();
    for case in oracle["records"].as_array().unwrap() {
        if case["qualification_scope"] != "candidate_bounded_control"
            || case["native_status"] != "compiled_and_executed"
        {
            continue;
        }
        let source = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join(case["source"].as_str().unwrap()),
        )
        .unwrap();
        let analysis = analyze_source(&SourceFile::new("native-parameter.pine", source));
        let Some(hir) = analysis.hir else {
            failures.push(format!("{}: {:?}", case["name"], analysis.diagnostics));
            continue;
        };
        let result = run_historical(&hir, &bars).unwrap();
        let actual = result
            .plots
            .iter()
            .map(|plot| plot.values.last().unwrap().as_f64())
            .collect::<Vec<_>>();
        let expected = case["native_values"]
            .as_array()
            .unwrap()
            .iter()
            .map(serde_json::Value::as_f64)
            .collect::<Vec<_>>();
        if actual != expected {
            failures.push(format!("{}: {actual:?} != {expected:?}", case["name"]));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn supplied_library_private_const_helpers_keep_their_qualifiers_and_defaults() {
    let root = SourceFile::new(
        "root.pine",
        "//@version=6\nimport test/parameters/1 as lib\nindicator(\"private helper\")\nplot(lib.value(true))",
    );
    let library = SourceFile::new(
        "lib.pine",
        "//@version=6\nlibrary(\"private helper\")\nhidden(const int x=2)=>x+1\nexport value(simple bool enabled)=>enabled?hidden():0",
    );
    let input =
        AnalysisInput::with_library_sources(root, vec![("test/parameters/1".into(), library)])
            .unwrap();
    let analysis = analyze_input(&input);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let bars = [Bar {
        time: 0,
        open: 10.,
        high: 10.,
        low: 10.,
        close: 10.,
        volume: 1.,
    }];
    let result = run_historical(&analysis.hir.unwrap(), &bars).unwrap();
    assert_eq!(result.plots[0].values[0].as_f64(), Some(3.));
}

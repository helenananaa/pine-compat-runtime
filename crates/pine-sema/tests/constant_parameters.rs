use pine_sema::{AnalysisInput, analyze_input, analyze_source};
use pine_syntax::SourceFile;

#[test]
fn native_parameter_rejections_remain_errors_after_const_syntax_is_admitted() {
    macro_rules! source {
        ($name:literal) => {
            include_str!(concat!(
                "../../../docs/evidence/strategy-library-20261010/constant-param-35/",
                $name,
                ".pine"
            ))
        };
    }
    let sources = [
        source!("parameter-const-series-v6-35"),
        source!("parameter-const-input-v6-35"),
        source!("parameter-input-literal-v6-35"),
        source!("parameter-const-version-v3-35"),
        source!("parameter-const-version-v4-35"),
        source!("parameter-const-missing-bool-v6-35"),
        source!("parameter-const-double-qualifier-v6-35"),
        source!("parameter-const-reassign-v6-35"),
        source!("parameter-plain-reassign-v5-35"),
        source!("parameter-plain-reassign-v6-35"),
        source!("parameter-const-dead-reassign-v6-35"),
        source!("parameter-const-reference-udt-v6-35"),
        source!("parameter-const-reference-udt-v5-35"),
        source!("parameter-const-reference-qualifier-v6-35"),
        source!("parameter-plain-missing-bool-v6-35"),
        source!("parameter-const-default-series-v6-35"),
        source!("parameter-plain-dead-reassign-v6-35"),
    ];
    let mut failures = Vec::new();
    for source in sources {
        let analysis = analyze_source(&SourceFile::new("native-parameter.pine", source));
        if analysis.hir.is_some() {
            failures.push(source);
        }
    }
    assert!(failures.is_empty(), "accepted native-invalid: {failures:?}");
}

#[test]
fn supplied_library_sources_validate_const_export_parameters_even_when_unused() {
    let root = SourceFile::new(
        "root.pine",
        "//@version=6\nimport test/parameters/1 as lib\nindicator(\"export guard\")\nplot(close)",
    );
    let library = SourceFile::new(
        "lib.pine",
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/constant-param-35/parameter-exported-const-v6-35.pine"
        ),
    );
    let input =
        AnalysisInput::with_library_sources(root, vec![("test/parameters/1".into(), library)])
            .unwrap();
    let analysis = analyze_input(&input);
    assert!(analysis.hir.is_none());
    assert!(analysis.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "E_FUNCTION_PARAM_TYPE"
            && diagnostic.message.contains("exported")
            && diagnostic.message.contains("const")
    }));
}

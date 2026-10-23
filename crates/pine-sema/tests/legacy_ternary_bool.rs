use pine_sema::analyze_source;
use pine_syntax::SourceFile;

#[test]
fn native_v4_v5_numeric_bool_ternaries_compile_but_v6_stays_strict() {
    for source in [
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/bool-numeric-ternary-expanded-v4-oracle-24.pine"
        ),
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/bool-numeric-ternary-expanded-v5-oracle-24.pine"
        ),
    ] {
        let result = analyze_source(&SourceFile::new("native.pine", source));
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        assert!(result.hir.is_some());
    }
    let result = analyze_source(&SourceFile::new(
        "v6.pine",
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/bool-int-ternary-v6-negative-24.pine"
        ),
    ));
    assert!(result.hir.is_none());
    assert!(result.diagnostics.iter().any(|d| d.code == "E_BRANCH_TYPE"));
}

#[test]
fn legacy_mixed_ternary_result_is_bool_without_relaxing_unrelated_types() {
    for source in [
        include_str!("../../../docs/evidence/strategy-library-20261010/bool-numeric-ternary-type-negative-v5-24.pine").to_owned(),
        "//@version=5\nindicator(\"negative\")\nx=time>0?true:\"text\"\nplot(x?1:0)\n".to_owned(),
        "//@version=5\nindicator(\"qualifier\")\nx=time>0?true:1\ny=input.bool(x)\nplot(y?1:0)\n".to_owned(),
    ] {
        let result = analyze_source(&SourceFile::new("negative.pine", &source));
        assert!(result.hir.is_none(), "{source}");
        assert!(!result.diagnostics.is_empty());
    }
}

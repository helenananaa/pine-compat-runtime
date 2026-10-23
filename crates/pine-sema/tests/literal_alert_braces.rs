use pine_sema::analyze_source;
use pine_syntax::SourceFile;

#[test]
fn alert_messages_and_alertcondition_titles_accept_native_literal_braces() {
    for source in [
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/literal-alert-braces-v4-oracle-25.pine"
        ),
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/literal-alert-braces-v5-oracle-25.pine"
        ),
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/literal-alert-braces-v6-oracle-25.pine"
        ),
    ] {
        let result = analyze_source(&SourceFile::new("native.pine", source));
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        assert!(result.hir.is_some());
    }
}

#[test]
fn plain_braces_do_not_relax_message_templates_types_or_scope() {
    for body in [
        "alertcondition(true,\"Title\",\"{{plot_20}}\")",
        "alertcondition(true,\"Title\",\"{{unknown}}\")",
        "alertcondition(true,str.tostring(close),\"message\")",
        "alert(1)",
        "f()=>alert(\"{{ticker}}\")\nf()",
    ] {
        let result = analyze_source(&SourceFile::new(
            "negative.pine",
            format!("//@version=5\nindicator(\"negative\")\n{body}\nplot(1)\n"),
        ));
        assert!(result.hir.is_none(), "{body}");
        assert!(!result.diagnostics.is_empty());
    }
}

#[test]
fn literal_braces_keep_frequency_value_validation_at_execution() {
    let report = analyze_source(&SourceFile::new(
        "frequency.pine",
        "//@version=5\nindicator(\"frequency\")\nalert(\"{{ticker}}\",\"invalid_frequency\")\nplot(1)\n",
    ));
    assert!(report.hir.is_some(), "{:?}", report.diagnostics);
}

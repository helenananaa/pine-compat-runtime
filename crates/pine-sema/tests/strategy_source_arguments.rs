use pine_sema::analyze_source;
use pine_syntax::SourceFile;

#[test]
fn strategy_source_binding_rejects_positional_arguments_after_named_arguments() {
    for version in [4, 5, 6] {
        let comment_or_when = if version >= 6 { "\"comment\"" } else { "true" };
        for call in [
            "strategy.exit(id=\"exit\", \"entry\", limit=10)".to_owned(),
            "strategy.exit(\"exit\", from_entry=\"entry\", 1, limit=10)".to_owned(),
            format!("strategy.close(id=\"entry\", {comment_or_when})"),
        ] {
            let source = format!("//@version={version}\nstrategy(\"argument order\")\n{call}\n");
            let analysis = analyze_source(&SourceFile::new("strategy-order.pine", source));
            assert!(analysis.hir.is_none(), "v{version} admitted {call}");
            assert!(
                analysis
                    .diagnostics
                    .iter()
                    .any(|diagnostic| { diagnostic.code == "E_CALL_ARG_ORDER" }),
                "v{version} {call}: {:?}",
                analysis.diagnostics
            );
        }
    }
}

#[test]
fn strategy_source_binding_preserves_positional_prefix_and_named_suffix() {
    for version in [4, 5, 6] {
        let source = format!(
            "//@version={version}\nstrategy(\"argument order\")\nstrategy.exit(\"exit\", from_entry=\"entry\", limit=10)\nstrategy.close(\"entry\", qty=1)\n"
        );
        let analysis = analyze_source(&SourceFile::new("strategy-order.pine", source));
        assert!(
            analysis.hir.is_some(),
            "v{version}: {:?}",
            analysis.diagnostics
        );
    }
}

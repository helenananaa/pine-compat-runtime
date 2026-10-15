use pine_runtime::{Bar, run_historical};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

#[test]
fn plain_alert_text_and_title_remain_literal_while_message_placeholders_render() {
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
        let output = run_historical(
            &result.hir.unwrap(),
            &[Bar {
                time: 0,
                open: 1.0,
                high: 1.0,
                low: 1.0,
                close: 1.0,
                volume: 1.0,
            }],
        )
        .unwrap();
        assert_eq!(output.alerts.len(), 3);
        assert_eq!(
            output.alerts[0].message,
            "{{ticker}}|{{close}}|{{unknown}}|{{"
        );
        assert_eq!(
            output.alerts[1].message,
            "{{ticker}}|{{close}}|{{unknown}}|{{x"
        );
        assert_eq!(output.alerts[2].source, "{{ticker}}|{{unknown}}|{{");
        assert_eq!(output.alerts[2].message, "C=1");
    }
}

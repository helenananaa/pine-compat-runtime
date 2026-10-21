use pine_runtime::{Bar, ExternalAccountFrame, ExternalExecutionOptions, HistoricalRuntime};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

#[test]
fn all_order_commands_preserve_evaluated_annotations_without_native_fills() {
    let common = r#"comment="备注",alert_message="{{strategy.order.id}}",disable_alert=true"#;
    for call in [
        format!(r#"strategy.entry("L",strategy.long,qty=1,{common})"#),
        format!(r#"strategy.order("L",strategy.long,qty=1,{common})"#),
        format!(r#"strategy.close("L",{common})"#),
        format!(r#"strategy.close_all({common})"#),
        format!(r#"strategy.exit("X","L",stop=9,{common},comment_loss="止损",alert_loss="loss")"#),
    ] {
        let source = format!("//@version=6\nstrategy(\"Metadata\")\n{call}\n");
        let analysis = analyze_source(&SourceFile::new("external-metadata.pine", &source));
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        let hir = analysis.hir.unwrap();
        let mut runtime = HistoricalRuntime::new(&hir)
            .with_external_accounts_and_options(
                vec![ExternalAccountFrame {
                    time: 0,
                    position_size: 0.0,
                    position_avg_price: None,
                    equity: 1000.0,
                    initial_capital: 1000.0,
                    netprofit: 0.0,
                    openprofit: 0.0,
                }],
                None,
                ExternalExecutionOptions {
                    order_metadata: true,
                    ..Default::default()
                },
            )
            .unwrap();
        runtime
            .append_bars(&[Bar {
                time: 0,
                open: 10.0,
                high: 11.0,
                low: 9.0,
                close: 10.0,
                volume: 100.0,
            }])
            .unwrap();
        let intents = runtime.external_intents().unwrap();
        assert_eq!(intents.len(), 1);
        let metadata = intents[0].metadata.as_ref().unwrap();
        assert_eq!(metadata.comment.as_deref(), Some("备注"));
        assert_eq!(
            metadata.alert_message.as_deref(),
            Some("{{strategy.order.id}}")
        );
        assert!(metadata.disable_alert);
        if intents[0].action == "exit" {
            assert_eq!(metadata.comment_loss.as_deref(), Some("止损"));
            assert_eq!(metadata.alert_loss.as_deref(), Some("loss"));
        }
        assert_eq!(pine_runtime::EXTERNAL_ORDER_METADATA_SCHEMA_VERSION, 1);
    }
}

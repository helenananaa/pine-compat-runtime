use pine_sema::analyze_source;
use pine_syntax::SourceFile;

#[test]
fn native_typed_v5_boolean_payload_does_not_enable_constant_type_pruning() {
    let analysis = analyze_source(&SourceFile::new(
        "typed-bool-typeflow.pine",
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/constant-bool-34/bool-declaration-typeflow-v5-35.pine"
        ),
    ));
    assert!(analysis.hir.is_none());
    assert!(analysis.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "E_CALL_ARG_TYPE"
            && diagnostic.message.contains("simple int")
            && diagnostic.message.contains("series int")
    }));
}

macro_rules! native_source {
    ($name:literal) => {
        include_str!(concat!(
            "../../../docs/evidence/strategy-library-20261010/constant-bool-34/",
            $name,
            ".pine"
        ))
    };
}

#[test]
fn native_constant_if_identities_do_not_prune_input_qualifiers() {
    let sources = [
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/constant-bool-34/constant-bool-typeflow-literal-then-v5-34.pine"
        ),
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/constant-bool-34/constant-bool-typeflow-input-then-v5-34.pine"
        ),
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/constant-bool-34/constant-bool-typeflow-literal-then-v6-34.pine"
        ),
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/constant-bool-34/constant-bool-typeflow-input-then-v6-34.pine"
        ),
    ];
    for source in sources {
        let analysis = analyze_source(&SourceFile::new("native-typeflow.pine", source));
        assert!(analysis.hir.is_none(), "{source}");
        assert!(analysis.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "E_CALL_ARG_TYPE"
                && diagnostic.message.contains("const bool")
                && diagnostic.message.contains("input bool")
        }));
    }
}

#[test]
fn native_dynamic_request_annotation_is_resolved_before_boolean_value_evaluation() {
    let cases = [
        (
            native_source!("constant-bool-dynamic-config-literal-false-v5-34"),
            false,
        ),
        (
            native_source!("constant-bool-dynamic-config-box-true-v5-34"),
            true,
        ),
        (
            native_source!("constant-bool-dynamic-config-box-false-v5-34"),
            true,
        ),
        (
            native_source!("constant-bool-dynamic-config-not-box-v5-34"),
            true,
        ),
        (
            native_source!("constant-bool-dynamic-config-missing-v6-34"),
            true,
        ),
        (
            native_source!("constant-bool-config-control-alias-false-v5-34"),
            true,
        ),
        (
            native_source!("constant-bool-config-control-not-true-v5-34"),
            true,
        ),
        (
            native_source!("constant-bool-config-control-group-false-v5-34"),
            false,
        ),
        (
            native_source!("constant-bool-config-control-not-true-v6-34"),
            true,
        ),
        (native_source!("dynamic-requests-numeric-zero-v5-34"), false),
        (native_source!("dynamic-requests-na-v5-34"), true),
        (native_source!("constant-bool-not-binding-v5-34"), true),
        (native_source!("constant-bool-config-true-v5-34"), true),
    ];
    for (source, expected) in cases {
        let analysis = analyze_source(&SourceFile::new("native-declaration.pine", source));
        assert_eq!(
            analysis.hir.is_some(),
            expected,
            "{source}\n{:?}",
            analysis.diagnostics
        );
        if let Some(program) = analysis.hir {
            assert!(program.dynamic_requests);
        } else {
            assert!(
                analysis
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.code == "E_CALL_ARG_TYPE")
            );
        }
    }
}

#[test]
fn native_broker_settings_consume_bool_payloads_independently_of_annotation_flags() {
    let source = native_source!("constant-bool-strategy-config-v5-34");
    for expected in [false, true] {
        let source = if expected {
            source.replace("    false", "    true")
        } else {
            source.to_owned()
        };
        let analysis = analyze_source(&SourceFile::new("native-broker-options.pine", source));
        let settings = analysis
            .hir
            .expect("native config should compile")
            .strategy_settings;
        assert_eq!(
            [
                settings.process_orders_on_close,
                settings.calc_on_order_fills,
                settings.calc_on_every_tick,
                settings.use_bar_magnifier
            ],
            [expected; 4]
        );
    }
}

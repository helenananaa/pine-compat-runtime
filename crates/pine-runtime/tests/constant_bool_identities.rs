use pine_runtime::{Bar, run_historical};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

fn check_native_even_bar(source: &str, expected: &[i64]) {
    let analysis = analyze_source(&SourceFile::new("native-identity.pine", source));
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
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
    let result = run_historical(&analysis.hir.unwrap(), &bars).unwrap();
    let actual = result
        .plots
        .iter()
        .map(|plot| plot.values.last().unwrap().as_f64())
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        expected
            .iter()
            .map(|value| Some(*value as f64))
            .collect::<Vec<_>>()
    );
}

#[test]
fn native_v5_typed_boolean_declarations_materialize_without_changing_v6_or_untyped_var() {
    let oracle: serde_json::Value = serde_json::from_str(include_str!(
        "../../../docs/evidence/strategy-library-20261010/constant-bool-declaration-context-oracles-34.json"
    ))
    .unwrap();
    for case in oracle["records"].as_array().unwrap() {
        if case["native_status"] != "compiled_and_executed" {
            continue;
        }
        let source = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join(case["source"].as_str().unwrap()),
        )
        .unwrap();
        let expected = case["native_values"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_i64().unwrap())
            .collect::<Vec<_>>();
        check_native_even_bar(&source, &expected);
    }
}

#[test]
fn native_v5_constant_boolean_values_keep_language_identity_until_materialized() {
    check_native_even_bar(
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/constant-bool-34/constant-bool-identities-v5-34.pine"
        ),
        &[0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 1, 1, 1, 0, 1, 1, 0, 1, 0],
    );
}

#[test]
fn native_v6_missing_constant_boolean_identity_is_distinct_from_false() {
    check_native_even_bar(
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/constant-bool-34/constant-bool-identities-v6-34.pine"
        ),
        &[1, 0, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 0],
    );
}

#[test]
fn native_v5_dynamic_and_function_boundaries_materialize_constant_booleans() {
    check_native_even_bar(
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/constant-bool-34/constant-bool-transport-identities-v5-34.pine"
        ),
        &[1, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0],
    );
}

#[test]
fn native_v6_parameter_dependent_if_preserves_its_missing_identity() {
    check_native_even_bar(
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/constant-bool-34/constant-bool-transport-identities-v6-34.pine"
        ),
        &[0, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0],
    );
}

#[test]
fn native_v5_mixed_qualifiers_materialize_payload_and_preserve_legacy_na() {
    check_native_even_bar(
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/constant-bool-34/constant-bool-legacy-promotion-v5-34.pine"
        ),
        &[1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0],
    );
}

#[test]
fn native_v6_input_simple_and_series_operands_materialize_missing_bool_payload() {
    check_native_even_bar(
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/constant-bool-34/constant-bool-promotion-v6-34.pine"
        ),
        &[1, 1, 1, 0, 0, 0, 1],
    );
}

#[test]
fn native_v6_persistence_and_nested_function_returns_keep_distinct_boundaries() {
    check_native_even_bar(
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/constant-bool-34/constant-bool-lifetimes-valid-v6-34.pine"
        ),
        &[0, 1, 0, 0, 1, 0, 1, 1, 1, 1],
    );
}

#[test]
fn native_v6_udt_fields_expose_false_payload_while_parameter_return_keeps_identity() {
    check_native_even_bar(
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/constant-bool-34/constant-bool-field-boundaries-valid-v6-34.pine"
        ),
        &[0, 0, 0],
    );
}

#[test]
fn native_v6_input_metadata_matches_the_titles_and_defaults_shown_to_users() {
    let source = include_str!(
        "../../../docs/evidence/strategy-library-20261010/constant-bool-34/constant-bool-input-metadata-v6-34.pine"
    );
    let analysis = analyze_source(&SourceFile::new("native-input-metadata.pine", source));
    let inputs = pine_runtime::input_calls(&analysis.hir.unwrap());
    assert_eq!(
        inputs
            .iter()
            .map(|input| input.title.as_deref())
            .collect::<Vec<_>>(),
        vec![
            Some("truthy title"),
            Some("cast default"),
            Some("ternary default"),
            Some("or default")
        ]
    );
    assert_eq!(
        inputs
            .iter()
            .map(|input| input.default_value.clone())
            .collect::<Vec<_>>(),
        [false, true, true, false]
            .into_iter()
            .map(|value| Some(pine_runtime::PineValue::Bool(value)))
            .collect::<Vec<_>>()
    );
    check_native_even_bar(source, &[0, 1, 1, 0]);
}

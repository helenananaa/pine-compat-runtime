use pine_runtime::{Bar, run_historical};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

fn check(cases: &[(&str, &str, Option<&[f64]>)]) {
    for (name, source, expected) in cases {
        let report = analyze_source(&SourceFile::new(*name, *source));
        let Some(expected) = expected else {
            assert!(report.hir.is_none(), "{name}");
            continue;
        };
        assert!(
            report.diagnostics.is_empty(),
            "{name}: {:?}",
            report.diagnostics
        );
        let bars = (0..3)
            .map(|i| Bar {
                time: i * 60000,
                open: 10.,
                high: 10.,
                low: 10.,
                close: 10.,
                volume: 1.,
            })
            .collect::<Vec<_>>();
        let result = run_historical(&report.hir.unwrap(), &bars).unwrap();
        let values = result
            .plots
            .iter()
            .map(|p| p.values.last().unwrap().as_f64())
            .collect::<Vec<_>>();
        assert_eq!(
            values,
            expected.iter().copied().map(Some).collect::<Vec<_>>(),
            "{name}"
        );
    }
}

#[test]
fn native_method_scalars_and_exact_receiver_types() {
    check(&[
        (
            "method-scalars-v5-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-scalars-v5-38.pine"
            ),
            Some(&[12.0, 2.0, 0.0, 3.0, 25.0]),
        ),
        (
            "method-numeric-coercion-v5-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-numeric-coercion-v5-38.pine"
            ),
            None,
        ),
        (
            "method-scalars-v6-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-scalars-v6-38.pine"
            ),
            Some(&[12.0, 2.0, 0.0, 3.0, 25.0]),
        ),
        (
            "method-numeric-coercion-v6-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-numeric-coercion-v6-38.pine"
            ),
            None,
        ),
    ]);
}

#[test]
fn native_method_declared_qualifiers() {
    check(&[
        (
            "method-const-receiver-v5-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-const-receiver-v5-38.pine"
            ),
            Some(&[2.0]),
        ),
        (
            "method-simple-receiver-v5-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-simple-receiver-v5-38.pine"
            ),
            Some(&[2.0]),
        ),
        (
            "method-const-param-v5-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-const-param-v5-38.pine"
            ),
            Some(&[12.0]),
        ),
        (
            "method-simple-param-v5-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-simple-param-v5-38.pine"
            ),
            Some(&[12.0]),
        ),
        (
            "method-const-receiver-invalid-v5-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-const-receiver-invalid-v5-38.pine"
            ),
            None,
        ),
        (
            "method-simple-receiver-invalid-v5-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-simple-receiver-invalid-v5-38.pine"
            ),
            None,
        ),
        (
            "method-const-param-invalid-v5-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-const-param-invalid-v5-38.pine"
            ),
            None,
        ),
        (
            "method-simple-param-invalid-v5-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-simple-param-invalid-v5-38.pine"
            ),
            None,
        ),
        (
            "method-series-body-invalid-v5-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-series-body-invalid-v5-38.pine"
            ),
            None,
        ),
        (
            "method-const-receiver-v6-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-const-receiver-v6-38.pine"
            ),
            Some(&[2.0]),
        ),
        (
            "method-simple-receiver-v6-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-simple-receiver-v6-38.pine"
            ),
            Some(&[2.0]),
        ),
        (
            "method-const-param-v6-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-const-param-v6-38.pine"
            ),
            Some(&[12.0]),
        ),
        (
            "method-simple-param-v6-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-simple-param-v6-38.pine"
            ),
            Some(&[12.0]),
        ),
        (
            "method-const-receiver-invalid-v6-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-const-receiver-invalid-v6-38.pine"
            ),
            None,
        ),
        (
            "method-simple-receiver-invalid-v6-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-simple-receiver-invalid-v6-38.pine"
            ),
            None,
        ),
        (
            "method-const-param-invalid-v6-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-const-param-invalid-v6-38.pine"
            ),
            None,
        ),
        (
            "method-simple-param-invalid-v6-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-simple-param-invalid-v6-38.pine"
            ),
            None,
        ),
        (
            "method-series-body-invalid-v6-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-series-body-invalid-v6-38.pine"
            ),
            None,
        ),
        (
            "method-udt-simple-param-v5-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-udt-simple-param-v5-38.pine"
            ),
            Some(&[7.0]),
        ),
        (
            "method-udt-simple-param-invalid-v5-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-udt-simple-param-invalid-v5-38.pine"
            ),
            None,
        ),
        (
            "method-udt-simple-param-v6-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-udt-simple-param-v6-38.pine"
            ),
            Some(&[7.0]),
        ),
        (
            "method-udt-simple-param-invalid-v6-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-udt-simple-param-invalid-v6-38.pine"
            ),
            None,
        ),
    ]);
}

#[test]
fn native_method_expression_receivers() {
    check(&[
        (
            "method-chain-v5-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-chain-v5-38.pine"
            ),
            Some(&[15.0]),
        ),
        (
            "method-grouped-v5-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-grouped-v5-38.pine"
            ),
            Some(&[13.0]),
        ),
        (
            "method-function-syntax-v5-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-function-syntax-v5-38.pine"
            ),
            Some(&[12.0]),
        ),
        (
            "method-chain-v6-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-chain-v6-38.pine"
            ),
            Some(&[15.0]),
        ),
        (
            "method-grouped-v6-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-grouped-v6-38.pine"
            ),
            Some(&[13.0]),
        ),
        (
            "method-function-syntax-v6-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-function-syntax-v6-38.pine"
            ),
            Some(&[12.0]),
        ),
        (
            "method-builtin-result-v5-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-builtin-result-v5-38.pine"
            ),
            Some(&[4.0]),
        ),
        (
            "method-literal-v5-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-literal-v5-38.pine"
            ),
            Some(&[2.0]),
        ),
        (
            "method-builtin-result-v6-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-builtin-result-v6-38.pine"
            ),
            Some(&[4.0]),
        ),
        (
            "method-literal-v6-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-literal-v6-38.pine"
            ),
            Some(&[2.0]),
        ),
    ]);
}

#[test]
fn native_method_independent_history() {
    check(&[
        (
            "method-state-v5-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-state-v5-38.pine"
            ),
            Some(&[2.0, 1.0]),
        ),
        (
            "method-state-v6-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-state-v6-38.pine"
            ),
            Some(&[2.0, 1.0]),
        ),
    ]);
}

#[test]
fn native_method_collection_dispatch() {
    check(&[
        (
            "method-array-v5-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-array-v5-38.pine"
            ),
            Some(&[7.0]),
        ),
        (
            "method-point-array-v5-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-point-array-v5-38.pine"
            ),
            Some(&[2.0]),
        ),
        (
            "method-array-v6-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-array-v6-38.pine"
            ),
            Some(&[7.0]),
        ),
        (
            "method-point-array-v6-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-point-array-v6-38.pine"
            ),
            Some(&[2.0]),
        ),
        (
            "method-array-shadow-used-v5-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-array-shadow-used-v5-38.pine"
            ),
            Some(&[42.0, 2.0]),
        ),
        (
            "method-array-shadow-used-v6-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-array-shadow-used-v6-38.pine"
            ),
            Some(&[42.0, 2.0]),
        ),
    ]);
}

#[test]
fn native_method_tuple_returns() {
    check(&[
        (
            "method-tuple-v5-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-tuple-v5-38.pine"
            ),
            Some(&[2.0, 3.0]),
        ),
        (
            "method-tuple-v6-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-tuple-v6-38.pine"
            ),
            Some(&[2.0, 3.0]),
        ),
    ]);
}

#[test]
fn native_method_previous_method_research() {
    check(&[
        (
            "parameter-method-const-receiver-v6-35",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/constant-param-35/parameter-method-const-receiver-v6-35.pine"
            ),
            Some(&[2.0]),
        ),
        (
            "parameter-method-const-arg-v6-35",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/constant-param-35/parameter-method-const-arg-v6-35.pine"
            ),
            Some(&[12.0]),
        ),
        (
            "parameter-method-const-series-arg-v6-35",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/constant-param-35/parameter-method-const-series-arg-v6-35.pine"
            ),
            None,
        ),
    ]);
}

#[test]
fn native_method_version_boundary_rejects_v4_and_earlier() {
    let source = include_str!(
        "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-v4-guard-38.pine"
    );
    for version in 1..=4 {
        let source = source.replace("version=4", &format!("version={version}"));
        let report = analyze_source(&SourceFile::new("legacy-method", source));
        assert!(report.hir.is_none());
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.code == "E_METHOD_VERSION"),
            "{:?}",
            report.diagnostics
        );
    }
}

#[test]
fn native_object_and_udt_array_receiver_identity() {
    check(&[
        (
            "method-udt-array-v5-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-udt-array-v5-38.pine"
            ),
            Some(&[7.0]),
        ),
        (
            "method-udt-array-invalid-v5-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-udt-array-invalid-v5-38.pine"
            ),
            None,
        ),
        (
            "method-objects-v5-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-objects-v5-38.pine"
            ),
            Some(&[2.0, 10.0, 10.0, 1.0, 7.0]),
        ),
        (
            "method-udt-array-v6-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-udt-array-v6-38.pine"
            ),
            Some(&[7.0]),
        ),
        (
            "method-udt-array-invalid-v6-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-udt-array-invalid-v6-38.pine"
            ),
            None,
        ),
        (
            "method-objects-v6-38",
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/method-receivers-38/method-objects-v6-38.pine"
            ),
            Some(&[2.0, 10.0, 10.0, 1.0, 7.0]),
        ),
    ]);
}

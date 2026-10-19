use pine_runtime::{Bar, PineValue, RuntimeResult, run_historical};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

fn execute(source: &str) -> RuntimeResult {
    let report = analyze_source(&SourceFile::new("native-missing.pine", source));
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
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
    run_historical(&report.hir.unwrap(), &bars).unwrap()
}

fn last_values(result: RuntimeResult) -> Vec<PineValue> {
    result
        .plots
        .into_iter()
        .map(|p| p.values.last().unwrap().clone())
        .collect()
}

#[test]
fn native_missing_string_values_preserve_na_and_use_empty_string_comparison() {
    for source in [
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/missing-string-37/missing-string-values-v5-37.pine"
        ),
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/missing-string-37/missing-string-values-v6-37.pine"
        ),
    ] {
        assert_eq!(
            last_values(execute(source)),
            [1, 0, 1, 1, 0, 1, 1, 0, 1].map(PineValue::Int)
        );
    }
}

#[test]
fn native_missing_timeframe_inputs_literals_and_typed_strings_inherit_chart() {
    for source in [
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/missing-string-37/request-na-input-v5-37.pine"
        ),
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/missing-string-37/request-na-typed-v5-37.pine"
        ),
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/missing-string-37/request-na-literal-v5-37.pine"
        ),
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/missing-string-37/request-na-input-v6-37.pine"
        ),
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/missing-string-37/request-na-typed-v6-37.pine"
        ),
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/missing-string-37/request-na-literal-v6-37.pine"
        ),
    ] {
        assert_eq!(
            last_values(execute(source)),
            [PineValue::Int(1), PineValue::Int(0)]
        );
    }
}

#[test]
fn native_string_na_is_simple_but_rejected_in_const_declarations() {
    for source in [
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/missing-string-37/request-na-simple-param-v5-37.pine"
        ),
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/missing-string-37/request-na-simple-param-v6-37.pine"
        ),
    ] {
        assert_eq!(last_values(execute(source)), [PineValue::Int(0)]);
    }
    for source in [
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/missing-string-37/missing-string-const-declaration-v5-37.pine"
        ),
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/missing-string-37/missing-string-const-declaration-v6-37.pine"
        ),
    ] {
        let report = analyze_source(&SourceFile::new("native-const-missing.pine", source));
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.code == "E_CONST_DECL_VALUE"),
            "{:?}",
            report.diagnostics
        );
    }
}

#[test]
fn native_missing_string_constant_reassignment_keeps_simple_floor() {
    let source = include_str!(
        "../../../docs/evidence/strategy-library-20261010/missing-string-37/missing-string-constant-reassignment-v5-37.pine"
    );
    let report = analyze_source(&SourceFile::new("native-string-reassignment.pine", source));
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| d.code == "E_FUNCTION_ARG_TYPE" && d.message.contains("simple string")),
        "{:?}",
        report.diagnostics
    );
}

#[test]
fn native_missing_string_switch_matches_empty_selector_and_case() {
    let source = include_str!(
        "../../../docs/evidence/strategy-library-20261010/missing-string-37/missing-string-switch-v5-37.pine"
    );
    assert_eq!(
        last_values(execute(source)),
        [PineValue::Int(1), PineValue::Int(1)]
    );
}

#[test]
fn native_v6_rejects_untyped_na_comparison_while_v5_string_context_remains_valid() {
    let source = include_str!(
        "../../../docs/evidence/strategy-library-20261010/missing-string-37/missing-string-literal-comparison-v6-37.pine"
    );
    let report = analyze_source(&SourceFile::new("native-literal-na-v6.pine", source));
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| d.code == "E_OPERATOR_TYPE"),
        "{:?}",
        report.diagnostics
    );
    let legacy = include_str!(
        "../../../docs/evidence/strategy-library-20261010/missing-string-37/missing-string-literal-comparison-v5-37.pine"
    );
    assert_eq!(
        last_values(execute(legacy)),
        [1, 1, 1, 0, 1, 0].map(PineValue::Int)
    );
}

#[test]
fn native_v6_numeric_relational_na_operands_remain_legal_and_false() {
    for source in [
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/missing-string-37/v6-literal-na-guard-gt-37.pine"
        ),
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/missing-string-37/v6-literal-na-guard-ge-37.pine"
        ),
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/missing-string-37/v6-literal-na-guard-lt-37.pine"
        ),
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/missing-string-37/v6-literal-na-guard-le-37.pine"
        ),
    ] {
        assert_eq!(last_values(execute(source)), [PineValue::Int(0)]);
    }
}

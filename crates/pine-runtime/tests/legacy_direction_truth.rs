use pine_runtime::{Bar, PineValue, run_historical};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

fn run(source: &str) -> pine_runtime::RuntimeResult {
    let analysis = analyze_source(&SourceFile::new("direction.pine", source));
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let bars: Vec<_> = (0..6)
        .map(|i| Bar {
            time: 1_700_000_000_000 + i * 60_000,
            open: 10.,
            high: 10.,
            low: 10.,
            close: 10.,
            volume: 1.,
        })
        .collect();
    run_historical(&analysis.hir.unwrap(), &bars).unwrap()
}

#[test]
fn native_legacy_truth_identity_and_boolean_comparison_matrix() {
    for source in [
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/legacy-direction-truth-v1-oracle-28.pine"
        ),
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/legacy-direction-truth-v2-oracle-28.pine"
        ),
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/legacy-direction-truth-v3-oracle-28.pine"
        ),
    ] {
        let result = run(source);
        let expected = [1, 1, 0, 0, 0, 0, 1, 1, 0, 1, 0, 0, 1, 1, 1, 1, 1, 1];
        assert_eq!(result.plots.len(), expected.len());
        for (plot, value) in result.plots.iter().zip(expected) {
            assert_eq!(plot.values, vec![PineValue::Int(value); 6]);
        }
    }
}

#[test]
fn aliases_functions_and_history_preserve_direction_identity_and_condition_truth() {
    let result = run(
        "//@version=3\nstudy(\"history\")\nf(x)=>x\nn=cum(1)-1\nd=n%2==0?strategy.short:strategy.long\na=f(d)\nplot(a?1:0)\nplot(a[1]?1:0)\nplot(a==strategy.short?1:0)\nplot(a==true?1:0)\nplot(not a?1:0)\n",
    );
    for (plot, values) in result.plots.iter().zip([
        [1, 1, 1, 1, 1, 1],
        [0, 1, 1, 1, 1, 1],
        [1, 0, 1, 0, 1, 0],
        [0; 6],
        [0; 6],
    ]) {
        assert_eq!(plot.values, values.map(PineValue::Int));
    }
}

#[test]
fn regular_strings_are_not_legacy_directions() {
    for version in [1, 2, 3] {
        let source =
            format!("//@version={version}\nstudy(\"string guard\")\nplot(\"strategy.long\"?1:0)\n");
        assert!(
            !analyze_source(&SourceFile::new("guard.pine", source))
                .diagnostics
                .is_empty()
        );
    }
}

#[test]
fn order_when_interprets_direction_identity_without_changing_generic_truth() {
    let source = include_str!(
        "../../../docs/evidence/strategy-library-20261010/legacy-direction-orders-v3-oracle-28.pine"
    );
    let result = run(source);
    assert_eq!(
        result.plots[0].values,
        [0., 1., 1., 1., -1., 0.].map(PineValue::Float)
    );
    let strategy = result.strategy.as_ref().unwrap();
    assert_eq!(strategy.orders.len(), 3);
    assert_eq!(strategy.trades.len(), 2);
}

#[test]
fn entry_exit_and_close_all_when_follow_native_direction_gate() {
    let result = run(include_str!(
        "../../../docs/evidence/strategy-library-20261010/legacy-direction-when-v3-oracle-28.pine"
    ));
    assert_eq!(
        result.plots[0].values,
        [0., 0., 1., 1., 1., 0.].map(PineValue::Float)
    );
    let strategy = result.strategy.as_ref().unwrap();
    assert_eq!(strategy.orders.len(), 2);
    assert_eq!(strategy.trades.len(), 1);
}

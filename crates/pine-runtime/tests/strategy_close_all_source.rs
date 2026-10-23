use pine_runtime::{Bar, PineValue, run_historical};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

const EVIDENCE: &str = "../../../docs/evidence/strategy-library-20261010/";

fn run(source: &str) -> pine_runtime::RuntimeResult {
    let analyzed = analyze_source(&SourceFile::new("close-all.pine", source));
    assert!(
        analyzed.diagnostics.is_empty(),
        "{:?}",
        analyzed.diagnostics
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
    run_historical(&analyzed.hir.unwrap(), &bars).unwrap()
}

#[test]
fn legacy_when_positions_keep_false_gate_and_later_close() {
    for source in [
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/close-all-bool-v3-oracle-29.pine"
        ),
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/close-all-bool-v4-oracle-29.pine"
        ),
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/close-all-bool-v5-oracle-29.pine"
        ),
    ] {
        let result = run(source);
        assert_eq!(
            result.plots[0].values,
            [0., 1., 1., 1., 0., 0.].map(PineValue::Float)
        );
        let strategy = result.strategy.unwrap();
        // close_all fills are exposed in alerts/trades; the stable orders list
        // contains the entry event, as in the existing close_all goldens.
        assert_eq!(strategy.orders.len(), 1);
        assert_eq!(strategy.alerts.len(), 2);
        assert_eq!(strategy.trades.len(), 1);
        assert_eq!(strategy.trades[0].exit_bar_index, 4);
    }
}

#[test]
fn modern_positions_keep_immediately_before_disable_alert() {
    for (source, exit_bar, alerts) in [
        (
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/close-all-comment-v5-oracle-29.pine"
            ),
            4,
            1,
        ),
        (
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/close-all-comment-v6-oracle-29.pine"
            ),
            3,
            2,
        ),
    ] {
        let result = run(source);
        let strategy = result.strategy.unwrap();
        assert_eq!(strategy.trades[0].exit_bar_index, exit_bar);
        assert_eq!(strategy.alerts.len(), alerts);
        if alerts == 2 {
            assert_eq!(strategy.alerts[1].message, "exit");
        }
    }
}

#[test]
fn false_when_evaluates_arguments_once_in_the_selected_overload_parameter_order() {
    for (source, expected) in [
        (
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/close-all-evaluation-v4-oracle-29.pine"
            ),
            123,
        ),
        (
            include_str!(
                "../../../docs/evidence/strategy-library-20261010/close-all-evaluation-named-v4-oracle-29.pine"
            ),
            321,
        ),
    ] {
        let result = run(source);
        assert_eq!(result.plots[0].values, vec![PineValue::Int(expected); 6]);
        assert!(result.strategy.unwrap().orders.is_empty());
    }
}

#[test]
fn saved_native_type_version_and_overload_rejections_remain_errors() {
    for name in [
        "close-all-bool-flags-v5-negative-29",
        "close-all-bool-four-v5-negative-29",
        "close-all-string-v4-negative-29",
        "close-all-comment-v3-negative-29",
        "close-all-mixed-names-v5-negative-29",
        "close-all-comment-when-v5-negative-29",
        "close-all-bool-v6-negative-29",
        "close-all-direction-v5-negative-29",
        "close-all-flags-v4-negative-29",
        "close-all-when-v6-negative-29",
    ] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join(EVIDENCE)
            .join(format!("{name}.pine"));
        let source = std::fs::read_to_string(path).unwrap();
        let result = analyze_source(&SourceFile::new(name, source));
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.severity == pine_syntax::Severity::Error),
            "{name}"
        );
    }
}

#[test]
fn string_aliases_and_udf_returns_select_the_modern_overload() {
    for version in [5, 6] {
        let result = run(&format!(
            "//@version={version}\nstrategy(\"aliases\")\nf(x)=>x\nc=bar_index%2==0?\"first\":\"second\"\nif bar_index==0\n    strategy.entry(\"L\",strategy.long)\nif bar_index==2\n    strategy.close_all(f(c),\"exit\",true,false)\nplot(strategy.position_size)\n"
        ));
        let strategy = result.strategy.unwrap();
        assert_eq!(strategy.trades[0].exit_bar_index, 2);
        assert_eq!(strategy.alerts[1].message, "exit");
    }
}

#[test]
fn normalization_preserves_source_order_and_duplicate_argument_errors() {
    for version in [3, 4, 5, 6] {
        let source = format!(
            "//@version={version}\nstrategy(\"order\")\nstrategy.close_all(comment=\"x\",false)\n"
        );
        let result = analyze_source(&SourceFile::new("order.pine", source));
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.code == "E_CALL_ARG_ORDER")
        );
    }
    let result = analyze_source(&SourceFile::new(
        "duplicate.pine",
        "//@version=4\nstrategy(\"duplicate\")\nstrategy.close_all(true,when=false)\n",
    ));
    assert!(!result.diagnostics.is_empty());
}

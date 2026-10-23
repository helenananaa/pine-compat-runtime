use pine_runtime::{Bar, PineValue, run_historical};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

fn run(source: &str) -> pine_runtime::RuntimeResult {
    let result = analyze_source(&SourceFile::new("directions.pine", source));
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let bars: Vec<_> = (0..6)
        .map(|i| Bar {
            time: 1_700_000_000_000 + i * 60_000,
            open: 10.0,
            high: 10.0,
            low: 10.0,
            close: 10.0,
            volume: 1.0,
        })
        .collect();
    run_historical(&result.hir.unwrap(), &bars).unwrap()
}

#[test]
fn legacy_direction_constants_are_boolean_in_aliases_functions_and_order_conditions() {
    let source = include_str!(
        "../../../docs/evidence/strategy-library-20261010/legacy-direction-v4-oracle-27.pine"
    );
    {
        let result = run(source);
        assert_eq!(result.plots.len(), 6);
        for (plot, expected) in result.plots.iter().zip([1, 0, 1, 1, 1]) {
            assert_eq!(plot.values, vec![PineValue::Int(expected); 6]);
        }
        assert_eq!(
            result.plots[5].values,
            [0., 1., 1., 1., 0., 0.].map(PineValue::Float)
        );
    }
}

#[test]
fn v5_close_accepts_both_comment_and_boolean_position_overloads() {
    let comment = include_str!(
        "../../../docs/evidence/strategy-library-20261010/close-comment-v5-oracle-27.pine"
    );
    let boolean = include_str!(
        "../../../docs/evidence/strategy-library-20261010/close-bool-v5-oracle-27.pine"
    );
    for (source, expected) in [(comment, 0.), (boolean, 1.)] {
        let result = run(&format!("{source}\nplot(strategy.position_size)\n"));
        assert_eq!(
            result.plots[1].values.last(),
            Some(&PineValue::Float(expected))
        );
    }
}

#[test]
fn comment_overload_keeps_quantity_metadata_and_immediate_positions() {
    for version in [5, 6] {
        let result = run(&format!(
            r#"//@version={version}
strategy("comments")
f(x)=>x
c=bar_index%2==0?"half":"other"
if bar_index==0
    strategy.entry("L",strategy.long,qty=4)
if bar_index==2
    strategy.close("L",f(c),1,25,"exit",true,true)
plot(strategy.position_size)
"#
        ));
        // The script observes the position before immediate post-pass fills.
        assert_eq!(
            result.plots[0].values,
            [0., 4., 4., 3., 3., 3.].map(PineValue::Float)
        );
    }
}

#[test]
fn modern_direction_constants_do_not_become_boolean_conditions() {
    for version in [5, 6] {
        let source =
            format!("//@version={version}\nstrategy(\"guard\")\nplot(strategy.long?1:0)\n");
        let result = analyze_source(&SourceFile::new("guard.pine", source));
        assert!(!result.diagnostics.is_empty());
    }
}

#[test]
fn modern_direction_type_survives_aliases_ternaries_and_functions() {
    for version in [5, 6] {
        for argument in [
            "strategy.long",
            "alias",
            "pick(strategy.short)",
            "bar_index%2==0?strategy.long:strategy.short",
        ] {
            let source = format!(
                "//@version={version}\nstrategy(\"enum guard\")\nalias=strategy.long\npick(x)=>x\nstrategy.close(\"L\",{argument})\n"
            );
            let result = analyze_source(&SourceFile::new("enum.pine", source));
            assert!(!result.diagnostics.is_empty(), "{version}: {argument}");
        }
        let result = run(&format!(
            "//@version={version}\nstrategy(\"enum route\")\npick(x)=>x\nd=bar_index%2==0?strategy.long:strategy.short\nif bar_index==0\n    strategy.entry(\"L\",pick(d),qty=1)\nif bar_index==3\n    strategy.close(\"L\",comment=\"strategy.long\")\nplot(strategy.position_size)\n"
        ));
        assert_eq!(
            result.plots[0].values,
            [0., 1., 1., 1., 0., 0.].map(PineValue::Float)
        );
    }
}

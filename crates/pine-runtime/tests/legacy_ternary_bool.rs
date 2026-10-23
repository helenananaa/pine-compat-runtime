use pine_runtime::{Bar, PineValue, run_historical};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

fn run(source: &str) -> pine_runtime::RuntimeResult {
    let result = analyze_source(&SourceFile::new("ternary.pine", source));
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let bars: Vec<_> = (0..4)
        .map(|i| Bar {
            time: 1_700_000_000_000 + i * 60_000,
            open: 1.0,
            high: 1.0,
            low: 1.0,
            close: 1.0,
            volume: 1.0,
        })
        .collect();
    run_historical(&result.hir.unwrap(), &bars).unwrap()
}

#[test]
fn legacy_ternary_both_numeric_branch_orders_match_native_values() {
    for source in [
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/bool-numeric-ternary-expanded-v4-oracle-24.pine"
        ),
        include_str!(
            "../../../docs/evidence/strategy-library-20261010/bool-numeric-ternary-expanded-v5-oracle-24.pine"
        ),
    ] {
        let result = run(source);
        for (plot, expected) in result
            .plots
            .iter()
            .zip([1, 1, 1, 0, 0, 1, 1, 1, 0, 0, 0, 0])
        {
            assert_eq!(plot.values, vec![PineValue::Int(expected); 4]);
        }
        assert_eq!(result.plots.len(), 12);
    }
}

#[test]
fn mixed_ternary_alias_history_reassignment_and_udf_calls_keep_bool_values() {
    for version in [4, 5] {
        let declaration = if version == 4 { "study" } else { "indicator" };
        let result = run(&format!(
            r#"//@version={version}
{declaration}("history")
pick(a,b,c)=>c?a:b
c=bar_index%2==0
x=c?false:-2
alias=x
var bool state=false
state:=c?0:true
plot(alias?1:0)
plot(alias[1]?1:0)
plot(state?1:0)
plot(pick(true,0,c)?1:0)
plot(pick(0,false,c)?1:0)
plot(pick(1,2,c))
"#
        ));
        for (plot, expected) in result.plots.iter().zip([
            [0, 1, 0, 1],
            [0, 0, 1, 0],
            [0, 1, 0, 1],
            [1, 0, 1, 0],
            [0, 0, 0, 0],
            [1, 2, 1, 2],
        ]) {
            assert_eq!(plot.values, expected.map(PineValue::Int).to_vec());
        }
        assert_eq!(result.plots.len(), 6);
    }
}

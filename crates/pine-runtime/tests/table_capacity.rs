use pine_runtime::{Bar, run_historical};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

fn execute(source: &str) -> Result<pine_runtime::RuntimeResult, pine_runtime::RuntimeError> {
    let analysis = analyze_source(&SourceFile::new("native-table.pine", source));
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    run_historical(
        &analysis.hir.unwrap(),
        &[Bar {
            time: 0,
            open: 10.,
            high: 10.,
            low: 10.,
            close: 10.,
            volume: 1.,
        }],
    )
}

#[test]
fn native_table_capacity_boundary_preserves_sparse_layout_and_rejects_overflow() {
    let result = execute(include_str!("../../../docs/evidence/strategy-library-20261010/table-capacity-36/table-edge-10000-v6-36.pine")).unwrap();
    assert_eq!(result.tables[0].columns, 10000);
    assert_eq!(result.tables[0].snapshots.last().unwrap().cells.len(), 1);
    let error=execute(include_str!("../../../docs/evidence/strategy-library-20261010/table-capacity-36/table-edge-10001-v6-36.pine")).unwrap_err();
    assert!(error.message.contains("10000"), "{error:?}");
}

#[test]
fn native_fully_populated_1400_cell_table_executes_and_retains_complete_state() {
    let result=execute(include_str!("../../../docs/evidence/strategy-library-20261010/table-capacity-36/table-1400-populated-v5-36.pine")).unwrap();
    assert_eq!(result.plots[0].values[0].as_f64(), Some(1400.));
    let last = result.tables[0].snapshots.last().unwrap();
    assert_eq!(last.cells.len(), 1400);
    assert_eq!(
        (
            last.cells.last().unwrap().column,
            last.cells.last().unwrap().row
        ),
        (6, 199)
    );
    assert_eq!(result.tables[0].snapshots.len(), 1401);
    assert_eq!(
        result.tables[0]
            .snapshots
            .iter()
            .map(|s| s.cells.len())
            .sum::<usize>(),
        980700
    );
}

#[test]
fn expanded_table_capacity_keeps_dimension_product_and_coordinate_guards() {
    for (operation, message) in [
        (
            "table.new(position.top_right,0,100)",
            "table dimensions must be positive",
        ),
        (
            "table.new(position.top_right,4294967296,4294967296)",
            "table cell count overflow",
        ),
        (
            "t=table.new(position.top_right,100,100)\ntable.cell(t,100,0,\"outside\")",
            "out of bounds",
        ),
    ] {
        let source = format!("//@version=6\nindicator(\"guards\")\n{operation}\nplot(1)\n");
        let error = execute(&source).unwrap_err();
        assert!(error.message.contains(message), "{error:?}");
    }
}

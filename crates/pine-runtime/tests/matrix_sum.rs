use pine_runtime::{Bar, PineValue, run_historical};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

#[test]
fn matrix_sum_preserves_float_type_when_scalar_is_na() {
    for rhs in ["float(na)", "bar_index == 0 ? float(na) : 0.5"] {
        let source = format!(
            "//@version=6\nindicator(\"typed sum\")\na=matrix.new<int>(1,1,2)\nb=matrix.sum(a,{rhs})\nplot(b.get(0,0))\nb.set(0,0,1.5)\nplot(b.get(0,0))\nplot(a.get(0,0))\n"
        );
        let analysis = analyze_source(&SourceFile::new("sum.pine", source));
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        let bars = [0, 60000].map(|time| Bar {
            time,
            open: 1.0,
            high: 1.0,
            low: 1.0,
            close: 1.0,
            volume: 1.0,
        });
        let result = run_historical(&analysis.hir.unwrap(), &bars).unwrap();
        assert_eq!(result.plots[0].values[0], PineValue::Na);
        assert_eq!(result.plots[1].values, vec![PineValue::Float(1.5); 2]);
        assert_eq!(result.plots[2].values, vec![PineValue::Int(2); 2]);
    }
}

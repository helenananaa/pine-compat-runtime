use pine_syntax::SourceFile;

use super::*;

#[test]
fn udf_mutates_global_udt_fields_without_reassigning_reference() {
    let source = SourceFile::new(
        "udf_global_udt.pine",
        r#"//@version=5
indicator("global UDT field")
type Counter
    int value
var Counter counter = Counter.new(0)
increment() =>
    counter.value := counter.value + 1
    counter.value
plot(increment())
plot(counter.value)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let result = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0), bar(2.0), bar(3.0)])
        .expect("runtime result");
    assert_values_close(&result.plots[0].values, &[1.0, 2.0, 3.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 2.0, 3.0]);
}

#[test]
fn udf_final_udt_field_assignment_returns_assigned_value() {
    let source = SourceFile::new(
        "udf_field_return.pine",
        r#"//@version=5
indicator("field return")
type Counter
    int value
var Counter counter = Counter.new(0)
increment() =>
    counter.value := counter.value + 1
plot(increment())
plot(counter.value)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let result = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0), bar(2.0), bar(3.0)])
        .expect("runtime result");
    assert_values_close(&result.plots[0].values, &[1.0, 2.0, 3.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 2.0, 3.0]);
}

#[test]
fn udf_if_without_else_can_return_udt_or_na() {
    let source = SourceFile::new(
        "udf_udt_optional_return.pine",
        r#"//@version=5
indicator("optional UDT return")
type Point
    int x
var points = array.from(Point.new(7))
take(bool condition) =>
    if condition
        array.remove(points, 0)
point = take(bar_index == 0)
plot(na(point) ? na : point.x)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let result =
        run_historical(&analysis.hir.expect("HIR"), &[bar(1.0), bar(2.0)]).expect("runtime result");
    assert_eq!(
        result.plots[0].values,
        vec![PineValue::Int(7), PineValue::Na]
    );
}

#[test]
fn runs_local_user_type_constructors_and_field_reads() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("udt")
type Point
    float x
    float y
p = Point.new(close, open)
plot(p.x + p.y)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar_ohlc(1.0, 1.0, 1.0, 2.0), bar_ohlc(3.0, 3.0, 3.0, 4.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[3.0, 7.0]);
}

#[test]
fn runs_branch_local_user_type_values() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("udt branch")
type Point
    float x
    float y
if close > open
    p = Point.new(close, open)
    plot(p.x - p.y)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar_ohlc(1.0, 1.0, 1.0, 3.0), bar_ohlc(4.0, 4.0, 4.0, 2.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_eq!(
        result.plots[0].values,
        vec![PineValue::Float(2.0), PineValue::Na]
    );
}

#[test]
fn var_user_type_values_persist_historically() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("udt var")
type Point
    float x
var p = Point.new(close)
plot(p.x)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[1.0, 1.0, 1.0]);
}

#[test]
fn user_type_value_history_reads_previous_scalar_fields() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("udt history")
type Point
    float x
p = Point.new(close)
prior = p[1]
plot(na(prior) ? na : prior.x)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_eq!(
        result.plots[0].values,
        vec![PineValue::Na, PineValue::Float(1.0), PineValue::Float(2.0)]
    );
}

#[test]
fn var_user_type_value_history_reads_persisted_previous_value() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("var udt history")
type Point
    float x
var Point p = Point.new(close)
prior = p[1]
plot(na(prior) ? na : prior.x)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_eq!(
        result.plots[0].values,
        vec![PineValue::Na, PineValue::Float(1.0), PineValue::Float(1.0)]
    );
}

#[test]
fn nested_user_type_value_history_reads_previous_scalar_fields() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("nested udt history")
type Point
    float x
type Wrapper
    Point point
point = Point.new(close)
wrapper = Wrapper.new(point)
prior = wrapper[1]
plot(na(prior) ? na : prior.point.x)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_eq!(
        result.plots[0].values,
        vec![PineValue::Na, PineValue::Float(1.0), PineValue::Float(2.0)]
    );
}

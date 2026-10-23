use pine_runtime::{Bar, BarUpdate, PineValue, RealtimeRuntime, run_historical};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

fn bar(index: i64) -> Bar {
    Bar {
        time: index * 60000,
        open: 10.,
        high: 10.,
        low: 10.,
        close: 10.,
        volume: 1.,
    }
}

#[test]
fn undefined_points_fail_coordinate_reads_but_field_assignment_is_a_noop() {
    let operations = [
        "plot(p.price)",
        "q=p.copy()",
        "id=line.new(p,p)",
        "id=box.new(p,p)",
        "id=label.new(p,\"x\")",
        "line id=na\nline.set_first_point(id,p)",
        "line id=na\nline.set_second_point(id,p)",
        "box id=na\nbox.set_top_left_point(id,p)",
        "box id=na\nbox.set_bottom_right_point(id,p)",
        "label id=na\nlabel.set_point(id,p)",
        "id=line.new(p,p,extend=\"invalid\")",
        "id=box.new(p,p,extend=\"invalid\")",
    ];
    for version in [5, 6] {
        for operation in operations {
            let text = format!(
                "//@version={version}\nindicator(\"undefined points\")\nchart.point p=na\n{operation}\n"
            );
            let report = analyze_source(&SourceFile::new("undefined.pine", text));
            assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
            let error = run_historical(&report.hir.unwrap(), &[bar(0)]).unwrap_err();
            assert!(
                error.message.contains("E_CHART_POINT_NA_FIELD"),
                "{operation}: {error:?}"
            );
        }
        let text = format!(
            "//@version={version}\nindicator(\"undefined assignment\")\nchart.point p=na\np.price:=20\nplot(na(p)?1:0)\nq=chart.point.new(na,na,na)\ncopy=q.copy()\nplot(na(copy.price)?1:0)\n"
        );
        let report = analyze_source(&SourceFile::new("assignment.pine", text));
        assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
        let result = run_historical(&report.hir.unwrap(), &[bar(0)]).unwrap();
        assert!(
            result
                .plots
                .iter()
                .all(|plot| plot.values == [PineValue::Int(1)])
        );
    }
}

#[test]
fn point_equality_operators_are_rejected_instead_of_exposing_internal_handles() {
    for version in [5, 6] {
        for operator in ["==", "!="] {
            let text = format!(
                "//@version={version}\nindicator(\"point comparison\")\np=chart.point.from_index(1,10)\na=p\nplot(p{operator}a?1:0)\n"
            );
            let report = analyze_source(&SourceFile::new("point-comparison.pine", text));
            assert!(
                report
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.code == "E_OPERATOR_TYPE"),
                "{:?}",
                report.diagnostics
            );
        }
    }
}

#[test]
fn point_identity_copy_and_history_match_native_relations() {
    for version in [5, 6] {
        let text = format!(
            r#"//@version={version}
indicator("point identities")
var p=chart.point.from_index(1,0)
p.price:=bar_index
alias=p
copy=p.copy()
alias.price+=10
copy.price+=20
items=array.from(p)
other=items.copy()
q=other.first()
q.index:=9
previous=p[1]
plot(p.price)
plot(copy.price)
plot(p.index)
plot(na(previous)?na:previous.price)
"#
        );
        let report = analyze_source(&SourceFile::new("points.pine", text));
        assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
        let result = run_historical(&report.hir.unwrap(), &[bar(0), bar(1), bar(2)]).unwrap();
        assert_eq!(
            result.plots[0]
                .values
                .iter()
                .map(PineValue::as_f64)
                .collect::<Vec<_>>(),
            [Some(10.), Some(11.), Some(12.)]
        );
        assert_eq!(
            result.plots[1]
                .values
                .iter()
                .map(PineValue::as_f64)
                .collect::<Vec<_>>(),
            [Some(20.), Some(21.), Some(22.)]
        );
        assert_eq!(result.plots[2].values, vec![PineValue::Int(9); 3]);
        assert_eq!(
            result.plots[3].values,
            vec![PineValue::Na, PineValue::Float(11.), PineValue::Float(12.)]
        );
    }
}

#[test]
fn point_intrabar_retention_distinguishes_binding_fields_and_collections() {
    for version in [5, 6] {
        let text = format!(
            r#"//@version={version}
indicator("point intrabar")
type Wrap
    chart.point point
type Persist
    varip chart.point point
var normal=chart.point.from_index(1,0)
varip direct=chart.point.from_index(1,0)
varip points=array.from(chart.point.from_index(1,0))
varip object=Wrap.new(chart.point.from_index(1,0))
varip objects=array.from(Wrap.new(chart.point.from_index(1,0)))
var special=Persist.new(chart.point.from_index(1,0))
if barstate.isrealtime
    normal.price+=1
    direct.price+=1
    p=points.first()
    p.price+=1
    object.point.price+=1
    w=objects.first()
    w.point.price+=1
    special.point.price+=1
plot(normal.price)
plot(direct.price)
plot(points.first().price)
plot(object.point.price)
plot(objects.first().point.price)
plot(special.point.price)
"#
        );
        let report = analyze_source(&SourceFile::new("intrabar.pine", text));
        assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
        let hir = report.hir.unwrap();
        let mut session = RealtimeRuntime::new(&hir);
        session.update(BarUpdate::historical(bar(0))).unwrap();
        for tick in 1..=3 {
            let result = session.update(BarUpdate::forming(bar(1))).unwrap();
            assert_eq!(
                result
                    .plots
                    .iter()
                    .map(|p| p.values[1].as_f64())
                    .collect::<Vec<_>>(),
                [
                    Some(1.),
                    Some(tick as f64),
                    Some(tick as f64),
                    Some(1.),
                    Some(tick as f64),
                    Some(tick as f64)
                ]
            );
        }
        session.update(BarUpdate::confirmed(bar(1))).unwrap();
        let result = session.update(BarUpdate::forming(bar(2))).unwrap();
        assert_eq!(
            result
                .plots
                .iter()
                .map(|p| p.values[2].as_f64())
                .collect::<Vec<_>>(),
            [Some(2.), Some(5.), Some(5.), Some(2.), Some(5.), Some(5.)]
        );
    }
}

#[test]
fn drawing_point_snapshots_do_not_retain_mutable_point_handles() {
    let text = r#"//@version=6
indicator("point snapshot")
p=chart.point.from_index(0,10)
points=array.from(p)
polyline.new(points)
p.price:=20
line.new(p,p)
plot(p.price)
"#;
    let report = analyze_source(&SourceFile::new("snapshots.pine", text));
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    let result = run_historical(&report.hir.unwrap(), &[bar(0)]).unwrap();
    let PineValue::ChartPoint(point) = &result.polylines[0].snapshots[0].points[0] else {
        panic!("point snapshot must be owned")
    };
    assert_eq!(point.price.as_f64(), Some(10.));
    assert_eq!(result.lines[0].snapshots[0].y1.as_f64(), Some(20.));
    let wire = pine_runtime::public_runtime_result_json(&result);
    let decoded = pine_runtime::runtime_result_from_json(&wire).unwrap();
    assert_eq!(pine_runtime::public_runtime_result_json(&decoded), wire);
}

#[test]
fn ordinary_bindings_restore_shared_object_data_over_varip_collection_paths() {
    for version in [5, 6] {
        let source = format!(
            r#"//@version={version}
indicator("ordinary object anchors")
type Wrap
    float count
    chart.point point
var p=chart.point.from_index(1,0)
varip a=array.from(Wrap.new(0,p))
varip direct=chart.point.from_index(1,0)
var ordinary=array.from(direct)
varip wrapper=Wrap.new(0,chart.point.from_index(1,0))
varip b=array.from(wrapper)
varip fresh=array.from(Wrap.new(0,chart.point.from_index(1,0)))
if barstate.isrealtime
    p.price+=1
    direct.price+=1
    wrapper.count+=1
    wrapper.point.price+=1
    child=fresh.first()
    child.count+=1
    child.point.price+=1
plot(p.price)
plot(a.first().point.price)
plot(direct.price)
plot(ordinary.first().price)
plot(wrapper.count)
plot(b.first().point.price)
plot(fresh.first().count)
plot(fresh.first().point.price)
"#
        );
        let report = analyze_source(&SourceFile::new("anchors.pine", source));
        assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
        let hir = report.hir.unwrap();
        let mut session = RealtimeRuntime::new(&hir);
        session.update(BarUpdate::historical(bar(0))).unwrap();
        for tick in 1..=3 {
            let result = session.update(BarUpdate::forming(bar(1))).unwrap();
            assert_eq!(
                result
                    .plots
                    .iter()
                    .map(|p| p.values[1].as_f64())
                    .collect::<Vec<_>>(),
                [
                    Some(1.),
                    Some(1.),
                    Some(1.),
                    Some(1.),
                    Some(1.),
                    Some(1.),
                    Some(tick as f64),
                    Some(tick as f64)
                ]
            );
        }
    }
}

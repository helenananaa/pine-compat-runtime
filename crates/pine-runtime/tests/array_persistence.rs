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
fn persistent_arrays_snapshot_between_executions_and_share_within_one_execution() {
    for version in [5, 6] {
        let text = format!(
            r#"//@version={version}
indicator("array execution snapshots")
var a=array.new_int()
var b=a
a.push(1)
b.push(2)
int x=0
int y=0
for i=0 to 1
    var c=array.new_int()
    var d=c
    c.push(1)
    d.push(2)
    x:=c.size()
    y:=d.size()
plot(a.size())
plot(b.size())
plot(x)
plot(y)
"#
        );
        let report = analyze_source(&SourceFile::new("snapshots.pine", text));
        assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
        let result = run_historical(&report.hir.unwrap(), &[bar(0), bar(1), bar(2)]).unwrap();
        for plot in &result.plots[..2] {
            assert_eq!(
                plot.values,
                vec![PineValue::Int(2), PineValue::Int(3), PineValue::Int(4)]
            );
        }
        for plot in &result.plots[2..] {
            assert_eq!(
                plot.values,
                vec![PineValue::Int(4), PineValue::Int(6), PineValue::Int(8)]
            );
        }
    }
}

#[test]
fn array_fields_and_variables_keep_separate_buffers_but_point_ids_stay_shared() {
    for version in [5, 6] {
        let text = format!(
            r#"//@version={version}
indicator("array fields")
type Wrap
    array<int> first
    array<int> second
var a=array.new_int()
var w=Wrap.new(a,a)
w.first.push(1)
w.second.push(2)
var p=chart.point.from_index(1,0)
var q=p
p.price:=bar_index
plot(a.size())
plot(w.first.size())
plot(w.second.size())
plot(q.price)
"#
        );
        let report = analyze_source(&SourceFile::new("fields.pine", text));
        assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
        let result = run_historical(&report.hir.unwrap(), &[bar(0), bar(1), bar(2)]).unwrap();
        assert_eq!(result.plots[0].values, vec![PineValue::Int(2); 3]);
        for plot in &result.plots[1..3] {
            assert_eq!(
                plot.values,
                vec![PineValue::Int(2), PineValue::Int(3), PineValue::Int(4)]
            );
        }
        assert_eq!(
            result.plots[3].values,
            vec![
                PineValue::Float(0.),
                PineValue::Float(1.),
                PineValue::Float(2.)
            ]
        );
    }
}

#[test]
fn varip_array_does_not_change_an_ordinary_persistent_alias_on_forming_updates() {
    for version in [5, 6] {
        let text = format!(
            r#"//@version={version}
indicator("array realtime")
varip a=array.new_int()
var b=a
if barstate.isrealtime
    a.push(1)
plot(a.size())
plot(b.size())
"#
        );
        let report = analyze_source(&SourceFile::new("forming.pine", text));
        assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
        let hir = report.hir.unwrap();
        for history_bars in [1, 2] {
            let mut runtime = RealtimeRuntime::new(&hir);
            let history: Vec<_> = (0..history_bars).map(bar).collect();
            runtime.seed_historical(&history).unwrap();
            for tick in 1..=3 {
                let result = runtime
                    .update(BarUpdate::forming(bar(history_bars)))
                    .unwrap();
                assert_eq!(result.plots[0].values.last(), Some(&PineValue::Int(tick)));
                assert_eq!(result.plots[1].values.last(), Some(&PineValue::Int(0)));
            }
            let result = runtime
                .update(BarUpdate::confirmed(bar(history_bars)))
                .unwrap();
            assert_eq!(result.plots[0].values.last(), Some(&PineValue::Int(4)));
            assert_eq!(result.plots[1].values.last(), Some(&PineValue::Int(0)));
        }
    }
}

#[test]
fn persistent_slice_aliases_keep_the_parent_view_instead_of_copying_its_elements() {
    let text = r#"//@version=6
indicator("array views")
var parent=array.from(0)
var view=parent.slice(0,1)
var view_alias=view
var alias=parent
parent.set(0,bar_index)
plot(parent.first())
plot(view.first())
plot(view_alias.first())
plot(alias.first())
"#;
    let report = analyze_source(&SourceFile::new("views.pine", text));
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    let result = run_historical(&report.hir.unwrap(), &[bar(0), bar(1), bar(2)]).unwrap();
    for plot in &result.plots[..3] {
        assert_eq!(
            plot.values,
            vec![PineValue::Int(0), PineValue::Int(1), PineValue::Int(2)]
        );
    }
    assert_eq!(result.plots[3].values, vec![PineValue::Int(0); 3]);
}

#[test]
fn native_rejects_array_identity_comparisons() {
    for version in [5, 6] {
        for op in ["==", "!="] {
            let text = format!(
                "//@version={version}\nindicator(\"array comparison\")\nvar a=array.new_int()\nvar b=a\nplot(a{op}b?1:0)\n"
            );
            let report = analyze_source(&SourceFile::new("comparison.pine", text));
            assert!(
                report
                    .diagnostics
                    .iter()
                    .any(|d| d.code == "E_OPERATOR_TYPE")
            );
            assert!(report.hir.is_none());
        }
    }
}

#[test]
fn realtime_array_reassignment_snapshots_after_the_whole_execution() {
    for version in [5, 6] {
        let text = format!(
            r#"//@version={version}
indicator("forming reassignment")
var a=array.new_int()
varip b=array.new_int()
varip int ticks=0
if barstate.isrealtime
    ticks+=1
    if ticks==1
        b:=a
    a.push(1)
    b.push(2)
plot(ticks)
plot(a.size())
plot(b.size())
"#
        );
        let report = analyze_source(&SourceFile::new("reassign.pine", text));
        assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
        let hir = report.hir.unwrap();
        let mut runtime = RealtimeRuntime::new(&hir);
        runtime.seed_historical(&[bar(0)]).unwrap();
        for tick in 1..=3 {
            let result = runtime.update(BarUpdate::forming(bar(1))).unwrap();
            let expected_a = if tick == 1 { 2 } else { 1 };
            assert_eq!(
                result.plots[1].values.last(),
                Some(&PineValue::Int(expected_a))
            );
            assert_eq!(
                result.plots[2].values.last(),
                Some(&PineValue::Int(tick + 1))
            );
        }
    }
}

#[test]
fn transient_array_bindings_and_function_returns_keep_their_own_slice_parent() {
    for version in [5, 6] {
        for producer in [
            "var a=array.from(0)\nc=a\na.set(0,bar_index)",
            "f()=>\n    var a=array.from(0)\n    a.set(0,bar_index)\n    a\nc=f()",
        ] {
            let text = format!(
                "//@version={version}\nindicator(\"transient view\")\n{producer}\nvar b=c\nvar view=c.slice(0,1)\nplot(c.first())\nplot(b.first())\nplot(view.first())\n"
            );
            let report = analyze_source(&SourceFile::new("transient.pine", text));
            assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
            let result = run_historical(&report.hir.unwrap(), &[bar(0), bar(1), bar(2)]).unwrap();
            assert_eq!(
                result.plots[0].values,
                vec![PineValue::Int(0), PineValue::Int(1), PineValue::Int(2)]
            );
            assert_eq!(result.plots[1].values, vec![PineValue::Int(0); 3]);
            assert_eq!(result.plots[2].values, vec![PineValue::Int(0); 3]);
        }
    }
}

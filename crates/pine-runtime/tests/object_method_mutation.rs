use pine_runtime::{Bar, BarUpdate, RealtimeRuntime, run_historical};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

fn bar(index: i64, close: f64) -> Bar {
    Bar {
        time: index * 60000,
        open: close,
        high: close,
        low: close,
        close,
        volume: 1.,
    }
}

#[test]
fn nested_method_compound_writes_share_aliases_and_roll_back_forming_ticks() {
    for version in [5, 6] {
        let source = format!(
            r#"//@version={version}
indicator("method reference rollback")
type Child
    float value
type Parent
    Child child
method bump(Parent self) =>
    self.child.value += 4
    self.child.value -= 1
    self.child.value *= 2
    self.child.value /= 2
    self.child.value %= 100
    self.child.value
var item=Parent.new(Child.new(0))
other=item
plot(other.bump())
plot(item.child.value)
"#
        );
        let analysis = analyze_source(&SourceFile::new("method.pine", source));
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        let hir = analysis.hir.unwrap();
        let bars = [bar(0, 10.), bar(1, 20.), bar(2, 30.)];
        let expected = run_historical(&hir, &bars).unwrap();
        for plot in &expected.plots {
            assert_eq!(
                plot.values.iter().map(|v| v.as_f64()).collect::<Vec<_>>(),
                [Some(3.), Some(6.), Some(9.)]
            );
        }
        let mut runtime = RealtimeRuntime::new(&hir);
        runtime.update(BarUpdate::historical(bars[0])).unwrap();
        runtime.update(BarUpdate::historical(bars[1])).unwrap();
        runtime.update(BarUpdate::forming(bar(2, 40.))).unwrap();
        runtime.update(BarUpdate::forming(bar(2, 35.))).unwrap();
        runtime.update(BarUpdate::confirmed(bars[2])).unwrap();
        assert_eq!(runtime.result(), expected);
    }
}

#[test]
fn method_field_support_preserves_binding_and_field_type_rejections() {
    for version in [5, 6] {
        for body in [
            "method change(Counter self)=>\n    global:=Counter.new(1)\nglobal.change()",
            "method change(Counter self)=>\n    self:=Counter.new(1)\nglobal.change()",
            "method change(Counter self)=>\n    self.value:=\"wrong\"\nglobal.change()",
        ] {
            let source = format!(
                "//@version={version}\nindicator(\"guard\")\ntype Counter\n    int value\nvar global=Counter.new(0)\n{body}\nplot(global.value)\n"
            );
            let analysis = analyze_source(&SourceFile::new("guard.pine", source));
            assert!(
                analysis.hir.is_none(),
                "{version}: {body}: {:?}",
                analysis.diagnostics
            );
        }
    }
}

#[test]
fn imported_udt_point_fields_can_be_stored_in_arrays() {
    for version in [5, 6] {
        let root = SourceFile::new(
            "root.pine",
            format!(
                "//@version={version}\nimport example/pair/1 as lib\nindicator(\"points\")\nitems=array.new<lib.Pair>()\nitems.push(lib.Pair.new(chart.point.from_index(2,10)))\nplot(items.first().point.price)\n"
            ),
        );
        let library = SourceFile::new(
            "pair.pine",
            format!(
                "//@version={version}\nlibrary(\"pair\")\nexport type Pair\n    chart.point point\n"
            ),
        );
        let input = pine_sema::AnalysisInput::with_library_sources(
            root,
            vec![("example/pair/1".to_owned(), library)],
        )
        .unwrap();
        let analysis = pine_sema::analyze_input(&input);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        let result = run_historical(&analysis.hir.unwrap(), &[bar(0, 10.)]).unwrap();
        assert_eq!(result.plots[0].values[0].as_f64(), Some(10.));
    }
}

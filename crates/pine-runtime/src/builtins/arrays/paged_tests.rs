use super::*;
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

fn program(source: &str) -> pine_ir::HirProgram {
    let analysis = analyze_source(&SourceFile::new("paged_arrays.pine", source));
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    analysis.hir.expect("HIR")
}

fn bar(close: f64) -> Bar {
    Bar {
        time: 0,
        open: close,
        high: close,
        low: close,
        close,
        volume: 1.0,
    }
}

#[test]
fn paged_arrays_preserve_slice_aliases_copy_history_and_checkpoint_isolation() {
    let program = program("//@version=6\nindicator(\"arrays\")\nplot(close)\n");
    let mut runtime = HistoricalRuntime::new(&program);
    let PineValue::Array(id) = runtime.new_array_from_values(
        ArrayElementKind::Int,
        (0..512).map(PineValue::Int).collect(),
    ) else {
        panic!()
    };
    let PineValue::Array(slice) = runtime.new_array_slice(id, 127, 257) else {
        panic!()
    };
    let PineValue::Array(invalidated) = runtime.new_array_slice(id, 400, 512) else {
        panic!()
    };
    let PineValue::Array(history) = runtime
        .clone_collection_history_value(PineValue::Array(id))
        .unwrap()
    else {
        panic!()
    };
    let checkpoint = runtime.clone();
    runtime
        .array_set_value(slice, 1, PineValue::Int(900))
        .unwrap();
    assert_eq!(
        runtime.array_get_cloned(id, 128).unwrap(),
        Some(PineValue::Int(900))
    );
    assert_eq!(
        checkpoint.array_get_cloned(slice, 1).unwrap(),
        Some(PineValue::Int(128))
    );
    assert_eq!(
        runtime.array_get_cloned(history, 128).unwrap(),
        Some(PineValue::Int(128))
    );
    runtime.array_clear_values(slice).unwrap();
    assert_eq!(runtime.array_len(slice).unwrap(), Some(0));
    assert_eq!(runtime.array_len(id).unwrap(), Some(382));
    assert_eq!(
        runtime.array_get_cloned(id, 127).unwrap(),
        Some(PineValue::Int(257))
    );
    assert!(
        runtime
            .array_len(invalidated)
            .unwrap_err()
            .message
            .contains("out of bounds")
    );
    assert_eq!(checkpoint.array_len(invalidated).unwrap(), Some(112));
    assert_eq!(checkpoint.array_len(slice).unwrap(), Some(130));
    assert_eq!(runtime.array_len(history).unwrap(), Some(512));
    runtime.array_clear_values(id).unwrap();
    assert!(runtime.array_len(slice).is_err());
}

#[test]
fn paged_realtime_arrays_rollback_var_and_preserve_varip_through_slice_writes() {
    let program = program(
        r#"//@version=6
indicator("paged rollback")
var ordinary = array.new_float(257, 0)
varip sticky = array.new_float(257, 0)
var window = array.slice(ordinary, 127, 130)
varip stickyWindow = array.slice(sticky, 127, 130)
array.set(window, 1, array.get(window, 1) + close)
array.set(stickyWindow, 1, array.get(stickyWindow, 1) + close)
plot(array.get(ordinary, 128))
plot(array.get(sticky, 128))
plot(array.get(ordinary, 127))
plot(array.get(sticky, 129))
"#,
    );
    let mut runtime = RealtimeRuntime::new(&program);
    for (update, ordinary, sticky) in [
        (BarUpdate::historical(bar(1.0)), 1.0, 1.0),
        (BarUpdate::forming(bar(2.0)), 3.0, 3.0),
        (BarUpdate::forming(bar(3.0)), 4.0, 6.0),
        (BarUpdate::confirmed(bar(4.0)), 5.0, 10.0),
        (BarUpdate::forming(bar(5.0)), 10.0, 15.0),
    ] {
        let result = runtime.update(update).unwrap();
        assert_eq!(
            result.plots[0].values.last(),
            Some(&PineValue::Float(ordinary))
        );
        assert_eq!(
            result.plots[1].values.last(),
            Some(&PineValue::Float(sticky))
        );
        assert_eq!(result.plots[2].values.last(), Some(&PineValue::Float(0.0)));
        assert_eq!(result.plots[3].values.last(), Some(&PineValue::Float(0.0)));
    }
}

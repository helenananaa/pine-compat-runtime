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
fn concat_snapshots_overlapping_sources_and_preserves_other_slice_ranges() {
    let program = program(
        r#"//@version=6
indicator("slice concat")
parent = array.from(0, 1, 2, 3, 4)
target = array.slice(parent, 1, 3)
source = array.slice(parent, 2, 5)
array.concat(target, source)
plot(array.join(parent, "|") == "0|1|2|2|3|4|3|4" ? 1 : 0)
plot(array.join(target, "|") == "1|2|2|3|4" ? 1 : 0)
plot(array.join(source, "|") == "2|2|3" ? 1 : 0)
array.concat(target, target)
plot(array.size(target))
plot(array.join(target, "|") == "1|2|2|3|4|1|2|2|3|4" ? 1 : 0)
tail = array.from(7, 8)
array.concat(tail, tail)
plot(array.join(tail, "|") == "7|8|7|8" ? 1 : 0)
"#,
    );
    let result = run_historical(&program, &[bar(1.0)]).unwrap();
    for (plot, expected) in result.plots.iter().zip([1, 1, 1, 10, 1, 1]) {
        assert_eq!(plot.values, [PineValue::Int(expected)]);
    }
}

#[test]
fn batched_slice_insert_keeps_checkpoint_and_counts_one_parent_payload_copy() {
    let program = program("//@version=6\nindicator(\"batch insert\")\nplot(close)\n");
    let mut runtime = HistoricalRuntime::new(&program);
    let PineValue::Array(parent) = runtime.new_array_from_values(
        ArrayElementKind::String,
        vec![PineValue::String("old".repeat(100)); 257],
    ) else {
        panic!("parent");
    };
    let PineValue::Array(slice) = runtime.new_array_slice(parent, 127, 129) else {
        panic!("slice");
    };
    let checkpoint = runtime.clone();
    runtime.collection_gc_allocated_bytes = 0;
    runtime
        .array_insert_values(slice, 2, vec![PineValue::String("new".into()); 256])
        .unwrap();
    let cell_bytes = std::mem::size_of::<PineValue>();
    assert_eq!(
        runtime.collection_gc_allocated_bytes,
        257 * (cell_bytes + 300) + 256 * (cell_bytes + 3)
    );
    assert_eq!(runtime.array_len(parent).unwrap(), Some(513));
    assert_eq!(runtime.array_len(slice).unwrap(), Some(258));
    assert_eq!(
        runtime.array_get_cloned(parent, 129).unwrap(),
        Some(PineValue::String("new".into()))
    );
    assert_eq!(
        runtime.array_get_cloned(parent, 385).unwrap(),
        Some(PineValue::String("old".repeat(100)))
    );
    assert_eq!(checkpoint.array_len(parent).unwrap(), Some(257));
    assert_eq!(checkpoint.array_len(slice).unwrap(), Some(2));
    assert_eq!(
        checkpoint.array_get_cloned(parent, 129).unwrap(),
        Some(PineValue::String("old".repeat(100)))
    );
}

#[test]
fn slice_concat_forming_replacements_rebuild_from_the_confirmed_checkpoint() {
    let program = program(
        r#"//@version=6
indicator("forming concat")
var parent = array.new_float(257, 0)
var window = array.slice(parent, 0, 1)
source = array.from(close)
array.concat(window, source)
plot(array.size(parent))
plot(array.size(window))
plot(array.last(window))
plot(array.get(parent, array.size(window)))
"#,
    );
    let mut runtime = RealtimeRuntime::new(&program);
    for (update, parent_len, window_len, last) in [
        (BarUpdate::historical(bar(1.0)), 258, 2, 1.0),
        (
            BarUpdate::forming(Bar {
                time: 60000,
                ..bar(2.0)
            }),
            259,
            3,
            2.0,
        ),
        (
            BarUpdate::forming(Bar {
                time: 60000,
                ..bar(3.0)
            }),
            259,
            3,
            3.0,
        ),
        (
            BarUpdate::confirmed(Bar {
                time: 60000,
                ..bar(4.0)
            }),
            259,
            3,
            4.0,
        ),
    ] {
        let result = runtime.update(update).unwrap();
        for (plot, expected) in result.plots.iter().zip([
            PineValue::Int(parent_len),
            PineValue::Int(window_len),
            PineValue::Float(last),
            PineValue::Float(0.0),
        ]) {
            assert_eq!(plot.values.last(), Some(&expected));
        }
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

#[test]
fn string_page_writes_collect_orphans_and_preserve_slice_and_checkpoint_values() {
    let program = program("//@version=6\nindicator(\"string pages\")\nplot(close)\n");
    let mut runtime = HistoricalRuntime::new(&program);
    let PineValue::Array(id) = runtime.new_array_from_values(
        ArrayElementKind::String,
        vec![PineValue::String("x".repeat(32768)); 257],
    ) else {
        panic!("array");
    };
    let PineValue::Array(slice) = runtime.new_array_slice(id, 127, 130) else {
        panic!("slice");
    };
    runtime
        .call_state
        .insert(pine_ir::CallSiteId(0), PineValue::Array(slice));
    runtime.collect_temporary_collections();
    let checkpoint = runtime.clone();
    runtime.new_array_from_values(
        ArrayElementKind::String,
        vec![PineValue::String(String::new())],
    );
    runtime.collection_gc_allocated_bytes = 0;
    assert!(
        runtime
            .array_set_value(slice, 3, PineValue::String("invalid".into()))
            .is_err()
    );
    assert_eq!(runtime.collection_gc_allocated_bytes, 0);
    // The replacement is small; pressure comes from its shared old String page.
    runtime
        .array_set_value(slice, 1, PineValue::String("changed".into()))
        .unwrap();
    runtime.collect_temporary_collections();
    assert_eq!(runtime.array_store.len(), 1);
    assert_eq!(
        runtime.array_get_cloned(id, 128).unwrap(),
        Some(PineValue::String("changed".into()))
    );
    assert_eq!(
        checkpoint.array_get_cloned(slice, 1).unwrap(),
        Some(PineValue::String("x".repeat(32768)))
    );
    assert_eq!(runtime.array_len(slice).unwrap(), Some(3));
    // Clearing a slice rebuilds the remaining parent payload with owned values.
    runtime.new_array_from_values(
        ArrayElementKind::String,
        vec![PineValue::String(String::new())],
    );
    runtime.collection_gc_allocated_bytes = 0;
    runtime.array_clear_values(slice).unwrap();
    runtime.collect_temporary_collections();
    assert_eq!(runtime.array_store.len(), 1);
    assert_eq!(runtime.array_len(id).unwrap(), Some(254));
    assert_eq!(runtime.array_len(slice).unwrap(), Some(0));
    assert_eq!(checkpoint.array_len(id).unwrap(), Some(257));
    assert!(runtime.next_array_id < 1024);
}

#[test]
fn replacing_short_lived_array_with_strings_triggers_payload_collection() {
    let program = program("//@version=6\nindicator(\"replacement\")\nplot(close)\n");
    let mut runtime = HistoricalRuntime::new(&program);
    let kept = runtime.new_array_from_values(
        ArrayElementKind::String,
        vec![PineValue::String("kept".into())],
    );
    runtime
        .call_state
        .insert(pine_ir::CallSiteId(0), kept.clone());
    for _ in 0..4 {
        let PineValue::Array(id) =
            runtime.new_array_from_values(ArrayElementKind::String, vec![PineValue::Na; 128])
        else {
            panic!("array");
        };
        runtime
            .array_replace_values(id, vec![PineValue::String("x".repeat(32768)); 128])
            .unwrap();
        runtime.collect_temporary_collections();
        assert_eq!(runtime.array_store.len(), 1);
    }
    let PineValue::Array(id) = kept else {
        panic!("array");
    };
    assert_eq!(
        runtime.array_get_cloned(id, 0).unwrap(),
        Some(PineValue::String("kept".into()))
    );
    assert!(runtime.next_array_id < 1024);
}

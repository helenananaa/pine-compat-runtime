//! Allocation regression checks use per-thread counters, without timing gates.
use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
};

use pine_runtime::{Bar, BarUpdate, HistoricalRuntime, PineValue, RealtimeRuntime};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

struct TrackingAllocator;
thread_local! {
    static TRACK: Cell<bool> = const { Cell::new(false) };
    static MAX_ALLOCATION: Cell<usize> = const { Cell::new(0) };
}
fn record(size: usize) {
    if TRACK.try_with(Cell::get).unwrap_or(false) {
        let _ = MAX_ALLOCATION.try_with(|largest| largest.set(largest.get().max(size)));
    }
}
unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record(layout.size());
        unsafe { System.alloc(layout) }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        record(layout.size());
        unsafe { System.alloc_zeroed(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        record(size);
        unsafe { System.realloc(ptr, layout, size) }
    }
}
#[global_allocator]
static ALLOCATOR: TrackingAllocator = TrackingAllocator;

#[test]
fn binary_search_borrows_large_arrays_and_cross_page_slices_without_materializing_them() {
    let analysis = analyze_source(&SourceFile::new(
        "array_search.pine",
        r#"//@version=6
indicator("borrowed search")
var a = array.new_float(100000, 1)
var window = array.slice(a, 127, 99999)
plot(array.binary_search(a, 1))
plot(array.binary_search_rightmost(a, 1))
plot(array.binary_search_leftmost(window, 1))
plot(array.binary_search_rightmost(window, 1))
plot(array.includes(a, 1) ? 1 : 0)
"#,
    ));
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let program = analysis.hir.expect("HIR");
    let mut runtime = HistoricalRuntime::new(&program);
    let bar = Bar {
        time: 0,
        open: 1.0,
        high: 1.0,
        low: 1.0,
        close: 1.0,
        volume: 1.0,
    };
    runtime.append_bar(bar).unwrap();
    MAX_ALLOCATION.set(0);
    TRACK.set(true);
    let result = runtime.append_bar(bar);
    TRACK.set(false);
    result.unwrap();
    // A cloned 100,000-element PineValue buffer is several MB. The generous
    // bound permits unrelated runtime bookkeeping while rejecting that copy.
    assert!(
        MAX_ALLOCATION.get() < 64 * 1024,
        "largest allocation: {}",
        MAX_ALLOCATION.get()
    );
    let result = runtime.result();
    for (plot, expected) in result.plots.iter().zip([0, 99999, 0, 99871, 1]) {
        assert_eq!(plot.values.last(), Some(&PineValue::Int(expected)));
    }
}

#[test]
fn varip_sparse_updates_do_not_materialize_the_whole_array_during_restore() {
    let analysis = analyze_source(&SourceFile::new(
        "array_varip.pine",
        r#"//@version=6
indicator("paged varip")
varip a = array.new_float(100000, 0)
array.set(a, 50000, array.get(a, 50000) + close)
plot(array.get(a, 50000))
"#,
    ));
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let program = analysis.hir.expect("HIR");
    let mut runtime = RealtimeRuntime::new(&program);
    let bar = Bar {
        time: 0,
        open: 1.0,
        high: 1.0,
        low: 1.0,
        close: 1.0,
        volume: 1.0,
    };
    runtime.apply_update(BarUpdate::historical(bar)).unwrap();
    runtime.apply_update(BarUpdate::forming(bar)).unwrap();
    MAX_ALLOCATION.set(0);
    TRACK.set(true);
    let result = runtime.apply_update(BarUpdate::forming(bar));
    TRACK.set(false);
    result.unwrap();
    assert!(
        MAX_ALLOCATION.get() < 64 * 1024,
        "largest allocation: {}",
        MAX_ALLOCATION.get()
    );
    assert_eq!(
        runtime.result().plots[0].values.last(),
        Some(&PineValue::Float(3.0))
    );
}

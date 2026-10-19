//! Persistent heap roots for rollback between strategy script passes.
//! The sparse stores share payloads; capture never scans collection elements.
use super::*;

#[derive(Clone)]
pub(crate) struct CollectionCheckpoint {
    pub(crate) array_store: IdStore<ArrayValues>,
    pub(crate) array_kinds: IdStore<ArrayElementKind>,
    pub(crate) array_user_types: IdStore<String>,
    pub(crate) array_slices: IdStore<ArraySlice>,
    pub(crate) array_primary_owners: IdStore<super::super::persistence::PersistentArrayOwner>,
    pub(crate) object_store: IdStore<Vec<PineValue>>,
    pub(crate) object_varip_fields: IdStore<Vec<bool>>,
    pub(crate) object_varip_ids: IdStore<u32>,
    pub(crate) next_object_id: u64,
    pub(crate) matrix_store: IdStore<MatrixStorage>,
    pub(crate) map_store: IdStore<MapStorage>,
    labels: DrawingStore<RuntimeLabel>,
    active_labels: Arc<BTreeSet<u32>>,
    lines: DrawingStore<RuntimeLine>,
    active_lines: Arc<BTreeSet<u32>>,
    line_fills: DrawingStore<RuntimeLineFill>,
    polylines: DrawingStore<RuntimePolyline>,
    active_polylines: Arc<BTreeSet<u32>>,
    boxes: DrawingStore<RuntimeBox>,
    active_boxes: Arc<BTreeSet<u32>>,
    tables: DrawingStore<RuntimeTable>,
}

impl CollectionCheckpoint {
    pub(crate) fn capture(runtime: &HistoricalRuntime<'_>) -> Self {
        Self {
            array_store: runtime.array_store.clone(),
            array_kinds: runtime.array_kinds.clone(),
            array_user_types: runtime.array_user_types.clone(),
            array_slices: runtime.array_slices.clone(),
            array_primary_owners: runtime.array_primary_owners.clone(),
            object_store: runtime.object_store.clone(),
            object_varip_fields: runtime.object_varip_fields.clone(),
            object_varip_ids: runtime.object_varip_ids.clone(),
            next_object_id: runtime.next_object_id,
            matrix_store: runtime.matrix_store.clone(),
            map_store: runtime.map_store.clone(),
            labels: runtime.labels.clone(),
            active_labels: runtime.active_labels.clone(),
            lines: runtime.lines.clone(),
            active_lines: runtime.active_lines.clone(),
            line_fills: runtime.line_fills.clone(),
            polylines: runtime.polylines.clone(),
            active_polylines: runtime.active_polylines.clone(),
            boxes: runtime.boxes.clone(),
            active_boxes: runtime.active_boxes.clone(),
            tables: runtime.tables.clone(),
        }
    }

    pub(super) fn restore(&self, runtime: &mut HistoricalRuntime<'_>) {
        runtime.array_store.clone_from(&self.array_store);
        runtime.array_kinds.clone_from(&self.array_kinds);
        runtime.array_user_types.clone_from(&self.array_user_types);
        runtime.array_slices.clone_from(&self.array_slices);
        runtime
            .array_primary_owners
            .clone_from(&self.array_primary_owners);
        runtime.object_store.clone_from(&self.object_store);
        runtime
            .object_varip_fields
            .clone_from(&self.object_varip_fields);
        runtime.object_varip_ids.clone_from(&self.object_varip_ids);
        runtime.matrix_store.clone_from(&self.matrix_store);
        runtime.map_store.clone_from(&self.map_store);
        runtime.labels.clone_from(&self.labels);
        runtime.active_labels.clone_from(&self.active_labels);
        runtime.lines.clone_from(&self.lines);
        runtime.active_lines.clone_from(&self.active_lines);
        runtime.line_fills.clone_from(&self.line_fills);
        runtime.polylines.clone_from(&self.polylines);
        runtime.active_polylines.clone_from(&self.active_polylines);
        runtime.boxes.clone_from(&self.boxes);
        runtime.active_boxes.clone_from(&self.active_boxes);
        runtime.tables.clone_from(&self.tables);
        // Allocation frontiers are monotonic even when speculative objects die.
    }
}

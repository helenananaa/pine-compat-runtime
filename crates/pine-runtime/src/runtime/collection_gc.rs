//! Reclaim unreachable collection graphs at completed script-bar boundaries.
//! Checkpoints own persistent store roots, so sweeping one evaluator cannot
//! change another evaluator's reachable values or recycle handle identities.
use std::collections::HashSet;

use crate::{HistoricalRuntime, PineValue};

impl HistoricalRuntime<'_> {
    pub(crate) fn collect_temporary_collections(&mut self) {
        let allocated = u64::from(self.next_array_id)
            + u64::from(self.next_map_id)
            + u64::from(self.next_matrix_id)
            + self.next_object_id;
        if allocated < self.collection_gc_next_id {
            return;
        }
        // Fill callbacks may still restore their evaluator checkpoint. The
        // ordinary bar boundary clears it before calling this collector.
        let mut pending: Vec<&PineValue> = self
            .current_symbols
            .values()
            .chain(self.current_series.values())
            .chain(self.var_store.values())
            .chain(self.call_state.values())
            .chain(self.input_overrides.values())
            .collect();
        for values in self.series_store.buffers.values() {
            pending.extend(values);
        }
        for values in self.valuewhen_state.values() {
            pending.extend(values);
        }
        for cross in self.cross_state.values() {
            pending.extend([
                &cross.current_left,
                &cross.current_right,
                &cross.previous_left,
                &cross.previous_right,
            ]);
        }
        for evaluation in self.request_evaluations.values() {
            pending.extend(evaluation.capture_values());
        }
        // Child runtimes have independent ID spaces. Only their parent-side
        // captures are roots here; their stores are collected independently.
        for captures in self.bounded_same_context_captures.values() {
            pending.extend(captures.values());
        }
        let mut arrays = HashSet::new();
        let mut objects = HashSet::new();
        let mut matrices = HashSet::new();
        let mut maps = HashSet::new();
        if self.current_bar_update_kind == crate::BarUpdateKind::Forming {
            // A committed variable can be reassigned during this speculative
            // pass. Its old object's varip fields must still be available when
            // the next pass restores that variable from the committed runtime.
            for &id in self.object_varip_ids.values() {
                objects.insert(id);
                if let Some(fields) = self.object_store.get(&id) {
                    pending.extend(fields);
                }
            }
        }
        while let Some(value) = pending.pop() {
            match value {
                PineValue::Array(id) => {
                    let mut id = *id;
                    while arrays.insert(id) {
                        if let Some(values) = self.array_store.get(&id) {
                            pending.extend(values);
                        }
                        let Some(slice) = self.array_slices.get(&id) else {
                            break;
                        };
                        id = slice.parent_id;
                    }
                }
                PineValue::UserTypeRef(id) if objects.insert(*id) => {
                    if let Some(fields) = self.object_store.get(id) {
                        pending.extend(fields);
                    }
                }
                PineValue::Matrix(id) if matrices.insert(*id) => {
                    if let Some(matrix) = self.matrix_store.get(id) {
                        pending.extend(&matrix.values);
                    }
                }
                PineValue::Map(id) if maps.insert(*id) => {
                    if let Some(map) = self.map_store.get(id) {
                        for (key, value) in &map.entries {
                            pending.extend([key, value]);
                        }
                    }
                }
                PineValue::Tuple(values) | PineValue::UserType(values) => pending.extend(values),
                PineValue::ChartPoint(point) => pending.extend([
                    point.time.as_ref(),
                    point.index.as_ref(),
                    point.price.as_ref(),
                ]),
                _ => {}
            }
        }
        self.array_store.retain(|id, _| arrays.contains(&id));
        self.array_slices.retain(|id, _| arrays.contains(&id));
        let store = &self.array_store;
        let slices = &self.array_slices;
        self.array_kinds
            .retain(|id, _| store.contains_key(&id) || slices.contains_key(&id));
        self.array_user_types
            .retain(|id, _| store.contains_key(&id) || slices.contains_key(&id));
        self.object_store.retain(|id, _| objects.contains(&id));
        self.object_varip_fields
            .retain(|id, _| objects.contains(&id));
        self.object_varip_ids.retain(|id, _| objects.contains(&id));
        self.matrix_store.retain(|id, _| matrices.contains(&id));
        self.map_store.retain(|id, _| maps.contains(&id));
        // If the script actually retains many collections, defer the next complete
        // root walk in proportion to that live set instead of scanning it on
        // every bar. Handle IDs are monotonic and are never recycled.
        let live = self.array_kinds.len()
            + self.object_store.len()
            + self.matrix_store.len()
            + self.map_store.len();
        self.collection_gc_next_id = allocated.saturating_add((live as u64).max(1024));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ArrayElementKind, Bar, SeriesId};
    use pine_ir::CallSiteId;

    fn program(source: &str) -> pine_ir::HirProgram {
        pine_sema::analyze_source(&pine_syntax::SourceFile::new("gc.pine", source))
            .hir
            .unwrap()
    }

    #[test]
    fn scalar_temporaries_are_reclaimed_while_persistent_arrays_keep_identity() {
        let program = program(
            "//@version=6\nindicator(\"gc\")\nvar held = array.new_float(1, 7)\ntemp = array.new_float(1, close)\nplot(array.get(held, 0) + array.get(temp, 0))\n",
        );
        let mut runtime = HistoricalRuntime::new(&program);
        for index in 0..2050 {
            runtime
                .append_bar(Bar {
                    time: index * 60000,
                    open: 1.,
                    high: 1.,
                    low: 1.,
                    close: index as f64,
                    volume: 1.,
                })
                .unwrap();
        }
        assert!(runtime.profile().array_slots < 16);
        assert!(runtime.next_array_id > 2050);
        assert_eq!(
            runtime.result().plots[0].values.last(),
            Some(&PineValue::Float(2056.))
        );
    }

    #[test]
    fn scalar_history_and_nested_reference_roots_survive_collection_and_checkpoint() {
        let program = program("//@version=6\nindicator(\"gc\")\nplot(close)\n");
        let mut runtime = HistoricalRuntime::new(&program);
        let history =
            runtime.new_array_from_values(ArrayElementKind::Float, vec![PineValue::Float(3.)]);
        runtime
            .series_store
            .commit(SeriesId(0), history.clone(), None);
        let nested =
            runtime.new_array_from_values(ArrayElementKind::Float, vec![PineValue::Float(5.)]);
        runtime.call_state.insert(
            CallSiteId(0),
            PineValue::Tuple(vec![PineValue::UserType(vec![nested.clone()])]),
        );
        runtime.new_array_from_values(ArrayElementKind::Float, vec![PineValue::Float(99.)]);
        let checkpoint = runtime.clone();
        runtime.collection_gc_next_id = 0;
        runtime.collect_temporary_collections();
        assert_eq!(runtime.array_store.len(), 2);
        assert_eq!(checkpoint.array_store.len(), 3);
        let PineValue::Array(id) = nested else {
            panic!("array")
        };
        runtime
            .array_set_value(id, 0, PineValue::Float(8.))
            .unwrap();
        assert_eq!(
            checkpoint.array_get_cloned(id, 0).unwrap(),
            Some(PineValue::Float(5.))
        );
        assert_eq!(runtime.series_store.read(SeriesId(0), 1), history);
    }

    #[test]
    fn unbounded_dynamic_array_history_remains_readable_after_many_collections() {
        let program = program(
            "//@version=6\nindicator(\"gc\")\na = array.new_float(1, close)\nold = a[bar_index]\nplot(na(old) ? na : array.get(old, 0))\n",
        );
        let mut runtime = HistoricalRuntime::new(&program);
        for index in 0..2050 {
            runtime
                .append_bar(Bar {
                    time: index * 60000,
                    open: 1.,
                    high: 1.,
                    low: 1.,
                    close: index as f64,
                    volume: 1.,
                })
                .unwrap();
        }
        assert_eq!(
            runtime.result().plots[0].values.last(),
            Some(&PineValue::Float(0.))
        );
        assert!(runtime.array_store.len() >= 2050);
    }

    #[test]
    fn slices_and_arrays_nested_in_objects_matrices_and_maps_are_roots() {
        use crate::builtins::arrays::ArraySlice;
        use crate::builtins::maps::MapStorage;
        use crate::builtins::matrices::{MatrixElementKind, MatrixStorage};
        let program = program("//@version=6\nindicator(\"gc\")\nplot(close)\n");
        let mut runtime = HistoricalRuntime::new(&program);
        let mut ids = Vec::new();
        for i in 0..5 {
            let PineValue::Array(id) = runtime
                .new_array_from_values(ArrayElementKind::Float, vec![PineValue::Float(i as f64)])
            else {
                panic!("array")
            };
            ids.push(id);
        }
        let slice_id = runtime.next_array_id;
        runtime.next_array_id += 1;
        runtime.array_slices.insert(
            slice_id,
            ArraySlice {
                parent_id: ids[0],
                start: 0,
                len: 1,
            },
        );
        runtime
            .array_kinds
            .insert(slice_id, ArrayElementKind::Float);
        runtime
            .call_state
            .insert(CallSiteId(0), PineValue::Array(slice_id));
        runtime
            .object_store
            .insert(0, vec![PineValue::Array(ids[1])]);
        runtime.matrix_store.insert(
            0,
            MatrixStorage {
                kind: MatrixElementKind::UserType(0),
                rows: 1,
                columns: 1,
                values: vec![PineValue::UserType(vec![PineValue::Array(ids[2])])],
            },
        );
        runtime.map_store.insert(
            0,
            MapStorage {
                key_kind: ArrayElementKind::String,
                value_kind: ArrayElementKind::UserType,
                entries: vec![(
                    PineValue::String("root".into()),
                    PineValue::UserType(vec![PineValue::Array(ids[3])]),
                )],
            },
        );
        runtime.call_state.insert(
            CallSiteId(1),
            PineValue::Tuple(vec![
                PineValue::UserTypeRef(0),
                PineValue::Matrix(0),
                PineValue::Map(0),
            ]),
        );
        runtime.collection_gc_next_id = 0;
        runtime.collect_temporary_collections();
        assert_eq!(runtime.array_store.len(), 4);
        assert_eq!(
            runtime.array_get_cloned(slice_id, 0).unwrap(),
            Some(PineValue::Float(0.))
        );
        assert!(runtime.array_store.get(&ids[4]).is_none());
    }

    #[test]
    fn varip_and_replica_survive_collection_during_forming_replacement_and_confirmation() {
        use crate::{BarUpdate, RealtimeRuntime};
        let program = program(
            "//@version=6\nindicator(\"gc\")\nvarip a = array.new_float(1, 0)\narray.set(a, 0, array.get(a, 0) + 1)\ntemp = array.new_float(1, close)\nplot(array.get(a, 0))\n",
        );
        let mut runtime = RealtimeRuntime::new(&program);
        let bars: Vec<_> = (0..1020)
            .map(|index| Bar {
                time: index * 60000,
                open: 1.,
                high: 1.,
                low: 1.,
                close: 1.,
                volume: 1.,
            })
            .collect();
        runtime.seed_historical(&bars).unwrap();
        let mut replica = runtime.replica();
        for index in 1020..1060 {
            let bar = Bar {
                time: index * 60000,
                open: 1.,
                high: 1.,
                low: 1.,
                close: 1.,
                volume: 1.,
            };
            for update in [
                BarUpdate::forming(bar),
                BarUpdate::forming(bar),
                BarUpdate::confirmed(bar),
            ] {
                replica
                    .apply(&runtime.apply_update(update).unwrap())
                    .unwrap();
            }
        }
        assert_eq!(runtime.result(), *replica.result());
        assert_eq!(
            runtime.result().plots[0].values.last(),
            Some(&PineValue::Float(1140.))
        );
    }

    fn bar(index: i64) -> Bar {
        Bar {
            time: index * 60000,
            open: 1.,
            high: 1.,
            low: 1.,
            close: index as f64,
            volume: 1.,
        }
    }

    #[test]
    fn map_matrix_and_object_temporaries_are_bounded_without_any_array_allocations() {
        let program = program(
            "//@version=6\nindicator(\"gc\")\ntype Point\n    float value\nm = map.new<string,float>()\nmap.put(m, \"n\", close)\nx = matrix.new<float>(1, 1, close)\np = Point.new(close)\nplot(map.get(m, \"n\") + matrix.get(x, 0, 0) + p.value)\n",
        );
        let mut runtime = HistoricalRuntime::new(&program);
        for index in 0..2050 {
            runtime.append_bar(bar(index)).unwrap();
        }
        assert_eq!(runtime.next_array_id, 0);
        assert!(runtime.map_store.len() < 1024);
        assert!(runtime.matrix_store.len() < 1024);
        assert!(runtime.object_store.len() < 1024);
        assert_eq!(
            runtime.result().plots[0].values.last(),
            Some(&PineValue::Float(6147.))
        );
        runtime.collection_gc_next_id = 0;
        runtime.collect_temporary_collections();
        assert_eq!(
            (
                runtime.map_store.len(),
                runtime.matrix_store.len(),
                runtime.object_store.len()
            ),
            (1, 1, 1)
        );
        assert_eq!(runtime.next_object_id, 2050);
        assert_eq!(runtime.next_map_id, 2050);
        assert_eq!(runtime.next_matrix_id, 2050);
        runtime.append_bar(bar(2050)).unwrap();
        assert_eq!(runtime.next_object_id, 2051);
    }

    #[test]
    fn dynamic_history_keeps_all_three_collection_types_and_copies_readable() {
        let program = program(
            "//@version=6\nindicator(\"history\")\ntype Point\n    float value\nm = map.new<string,float>()\nmap.put(m, \"n\", close)\nx = matrix.new<float>(1, 1, close)\np = Point.new(close)\noldm = m[bar_index]\noldx = x[bar_index]\noldp = p[bar_index]\nplot(na(oldm) ? na : map.get(oldm, \"n\"))\nplot(na(oldx) ? na : matrix.get(oldx, 0, 0))\nplot(na(oldp) ? na : oldp.value)\n",
        );
        let mut runtime = HistoricalRuntime::new(&program);
        for index in 0..2050 {
            runtime.append_bar(bar(index)).unwrap();
        }
        runtime.collection_gc_next_id = 0;
        runtime.collect_temporary_collections();
        let checkpoint = runtime.clone();
        runtime.append_bar(bar(2050)).unwrap();
        for plot in &runtime.result().plots {
            assert_eq!(plot.values.last(), Some(&PineValue::Float(0.)));
        }
        assert!(checkpoint.object_store.len() >= 2050);
        assert!(checkpoint.map_store.len() >= 2050);
        assert!(checkpoint.matrix_store.len() >= 2050);
    }

    #[test]
    fn cyclic_objects_are_traced_once_and_reclaimed_without_reusing_identities() {
        let program =
            program("//@version=6\nindicator(\"cycle\")\ntype Node\n    int next\nplot(close)\n");
        let mut runtime = HistoricalRuntime::new(&program);
        let identity = &program.user_types[0].identity;
        // A synthetic cycle verifies tracing without claiming support for
        // recursive UDT declarations in the language front end.
        let first = runtime
            .allocate_object(vec![PineValue::Na], identity)
            .unwrap();
        let second = runtime
            .allocate_object(vec![first.clone()], identity)
            .unwrap();
        runtime.set_object_field(0, 0, second).unwrap();
        runtime.call_state.insert(CallSiteId(0), first);
        let checkpoint = runtime.clone();
        runtime.collection_gc_next_id = 0;
        runtime.collect_temporary_collections();
        assert_eq!(runtime.object_store.len(), 2);
        runtime.call_state.clear();
        runtime.collection_gc_next_id = 0;
        runtime.collect_temporary_collections();
        assert_eq!(runtime.object_store.len(), 0);
        assert_eq!(runtime.object_store.capacity(), 0);
        assert_eq!(
            checkpoint.object_field(0, 0).unwrap(),
            PineValue::UserTypeRef(1)
        );
        assert_eq!(
            runtime
                .allocate_object(vec![PineValue::Na], identity)
                .unwrap(),
            PineValue::UserTypeRef(2)
        );
    }

    #[test]
    fn same_context_children_do_not_root_unrelated_parent_handle_ids() {
        let program = program("//@version=6\nindicator(\"contexts\")\nplot(close)\n");
        let mut parent = HistoricalRuntime::new(&program);
        let mut child = HistoricalRuntime::new(&program);
        let unused =
            parent.new_array_from_values(ArrayElementKind::Float, vec![PineValue::Float(1.)]);
        let owned =
            child.new_array_from_values(ArrayElementKind::Float, vec![PineValue::Float(2.)]);
        assert_eq!(unused, owned); // Equal integers belong to different stores.
        child.call_state.insert(CallSiteId(0), owned);
        let captured =
            parent.new_array_from_values(ArrayElementKind::Float, vec![PineValue::Float(3.)]);
        let key = crate::request::RequestCacheKey::new(CallSiteId(0), "TEST", "1");
        parent
            .bounded_same_context_evaluations
            .insert(key.clone(), Box::new(child));
        parent
            .bounded_same_context_captures
            .insert(key.clone(), [(pine_ir::SymbolId(0), captured)].into());
        parent.collection_gc_next_id = 0;
        parent.collect_temporary_collections();
        assert!(parent.array_store.get(&0).is_none());
        assert_eq!(
            parent.array_get_cloned(1, 0).unwrap(),
            Some(PineValue::Float(3.))
        );
        assert_eq!(
            parent.bounded_same_context_evaluations[&key]
                .array_get_cloned(0, 0)
                .unwrap(),
            Some(PineValue::Float(2.))
        );
    }

    #[test]
    fn requested_temporary_array_is_frozen_before_child_collection() {
        let program = program(
            "//@version=6\nindicator(\"contexts\")\na = request.security(syminfo.tickerid, timeframe.period, array.from(close), calc_bars_count=2050)\nplot(array.get(a, 0))\n",
        );
        let mut runtime = HistoricalRuntime::new(&program);
        let bars: Vec<_> = (0..2050).map(bar).collect();
        runtime.append_bars(&bars).unwrap();
        assert_eq!(
            runtime.result().plots[0].values.last(),
            Some(&PineValue::Float(2049.))
        );
        let child = runtime
            .bounded_same_context_evaluations
            .values()
            .next()
            .unwrap();
        assert!(child.array_store.len() < 1024);
    }
}

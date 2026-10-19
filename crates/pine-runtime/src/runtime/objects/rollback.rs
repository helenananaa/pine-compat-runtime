//! Ordinary persistent bindings anchor the committed state of referenced
//! objects, even when another path reaches them through a varip collection.
use super::{IntrabarReference, enqueue_intrabar_references, scalar_array_kind};
use crate::*;
use std::collections::HashSet;

impl HistoricalRuntime<'_> {
    pub(super) fn ordinary_object_roots(&self) -> Result<HashSet<u32>, RuntimeError> {
        let mut pending = Vec::new();
        for kind in [PersistenceKind::Var, PersistenceKind::Varip] {
            for slot in self.metadata.persistent_slots(kind) {
                if let Some(value) = self.var_store.get(slot)
                    && (kind == PersistenceKind::Var || matches!(value, PineValue::UserTypeRef(_)))
                {
                    enqueue_intrabar_references(&mut pending, value, false);
                }
            }
        }
        let mut objects = HashSet::new();
        let mut seen = HashSet::new();
        while let Some((reference, _)) = pending.pop() {
            match reference {
                IntrabarReference::Point(id) if seen.insert((0, id)) => {
                    objects.insert(id);
                }
                IntrabarReference::Object(id) if seen.insert((0, id)) => {
                    objects.insert(id);
                    if let Some(fields) = self.object_store.get(&id) {
                        let flags = self.object_varip_fields.get(&id);
                        for (index, field) in fields.iter().enumerate() {
                            // A varip UDT field retains its binding; a nested
                            // UDT still has its own ordinary/varip field rules.
                            if flags.is_some_and(|v| v[index])
                                && !matches!(field, PineValue::UserTypeRef(_))
                            {
                                continue;
                            }
                            enqueue_intrabar_references(&mut pending, field, false);
                        }
                    }
                }
                IntrabarReference::Array(id) if seen.insert((1, id)) => {
                    if scalar_array_kind(self.array_kinds.get(&id)) {
                        continue;
                    }
                    if let Some(values) = self.array_values(id)? {
                        for field in values {
                            enqueue_intrabar_references(&mut pending, field, false);
                        }
                    }
                }
                IntrabarReference::Map(id) if seen.insert((2, id)) => {
                    if let Some(map) = self.map_store.get(&id)
                        && !scalar_array_kind(Some(&map.value_kind))
                    {
                        for (_, field) in &map.entries {
                            enqueue_intrabar_references(&mut pending, field, false);
                        }
                    }
                }
                IntrabarReference::Matrix(id) if seen.insert((3, id)) => {
                    if let Some(matrix) = self.matrix_store.get(&id)
                        && matches!(
                            matrix.kind,
                            crate::builtins::matrices::MatrixElementKind::UserType(_)
                        )
                    {
                        for field in &matrix.values {
                            enqueue_intrabar_references(&mut pending, field, false);
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(objects)
    }
}

//! Array bindings share writes within one script execution. Their buffers
//! become independent at the execution boundary; slice views keep the bound
//! parent's handle rather than a cloned element buffer.
use crate::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PersistentArrayOwner {
    Variable(pine_ir::VarSlotId),
    Field(u32, usize),
}

impl HistoricalRuntime<'_> {
    pub(crate) fn bind_array_value(
        &mut self,
        value: PineValue,
        owner: Option<PersistentArrayOwner>,
    ) -> Result<PineValue, RuntimeError> {
        let PineValue::Array(source) = value else {
            return Ok(value);
        };
        let prior = self.array_primary_owners.get(&source).copied();
        let id = if prior.is_some() && prior != owner {
            let Some(kind) = self.array_kinds.get(&source).copied() else {
                return Err(RuntimeError {
                    message: "invalid array binding handle".to_owned(),
                });
            };
            let id = self.next_array_id;
            self.next_array_id = id.checked_add(1).ok_or_else(|| RuntimeError {
                message: "array identity space exhausted".to_owned(),
            })?;
            let linked = if self.array_slices.contains_key(&source) {
                self.array_slices.link_entry(source, id)
            } else {
                self.array_store.link_entry(source, id)
            };
            if !linked {
                return Err(RuntimeError {
                    message: "invalid array binding payload".to_owned(),
                });
            }
            self.array_kinds.insert(id, kind);
            self.copy_array_user_type_metadata(source, id);
            id
        } else {
            source
        };
        if let Some(owner) = owner
            && self.array_primary_owners.get(&id) != Some(&owner)
        {
            self.array_primary_owners.insert(id, owner);
        }
        Ok(PineValue::Array(id))
    }

    pub(crate) fn snapshot_persistent_array_aliases(&mut self) -> Result<(), RuntimeError> {
        self.array_store.finish_linked_entries();
        self.array_slices.finish_linked_entries();
        Ok(())
    }
}

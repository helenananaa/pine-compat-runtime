use super::changes::{
    PUBLIC_RUNTIME_CHANGES_SCHEMA_VERSION, RuntimeChanges, apply_runtime_changes_in_place,
};
use crate::{RuntimeError, RuntimeResult};

/// A consumer bound to one host-selected stream. Snapshot and revision must be
/// captured together; the host owns routing and stream identity across sessions.
pub struct RuntimeReplica {
    result: RuntimeResult,
    revision: u64,
    last_changes: Option<RuntimeChanges>,
}

impl RuntimeReplica {
    #[must_use]
    pub fn new(result: RuntimeResult, revision: u64) -> Self {
        Self {
            result,
            revision,
            last_changes: None,
        }
    }

    #[must_use]
    pub fn revision(&self) -> u64 {
        self.revision
    }

    #[must_use]
    pub fn result(&self) -> &RuntimeResult {
        &self.result
    }

    /// Replace state with an authoritative snapshot after a gap or reconnect.
    pub fn reset(&mut self, result: RuntimeResult, revision: u64) {
        *self = Self::new(result, revision);
    }

    /// Returns false for an identical retransmission. Rejected updates cannot
    /// mutate result or cursor. Payloads must belong to this replica's stream.
    pub fn apply(&mut self, changes: &RuntimeChanges) -> Result<bool, RuntimeError> {
        if changes.schema_version != PUBLIC_RUNTIME_CHANGES_SCHEMA_VERSION {
            return Err(error("E_STREAM_SCHEMA: unsupported changes schema"));
        }
        if changes.base_revision.checked_add(1) != Some(changes.revision) {
            return Err(error(
                "E_STREAM_REVISION: revision must follow baseRevision",
            ));
        }
        if changes.revision == self.revision {
            return if self.last_changes.as_ref() == Some(changes) {
                Ok(false)
            } else {
                Err(error(
                    "E_STREAM_CONFLICT: revision already has a different or unknown payload",
                ))
            };
        }
        if changes.revision < self.revision {
            return Err(error(
                "E_STREAM_STALE: change precedes the current revision",
            ));
        }
        if changes.base_revision != self.revision {
            return Err(error(
                "E_STREAM_GAP: missing changes; restore an authoritative snapshot",
            ));
        }
        apply_runtime_changes_in_place(&mut self.result, changes);
        self.revision = changes.revision;
        self.last_changes = Some(changes.clone());
        Ok(true)
    }
}

fn error(message: &str) -> RuntimeError {
    RuntimeError {
        message: message.to_owned(),
    }
}

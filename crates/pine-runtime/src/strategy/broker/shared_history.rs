use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

/// Immutable history is shared by broker checkpoints. Any mutable access first
/// detaches it, so snapshots and previously returned results remain independent.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct SharedHistory<T>(Arc<Vec<T>>);

impl<T> Default for SharedHistory<T> {
    fn default() -> Self {
        Self(Arc::new(Vec::new()))
    }
}

impl<T> Deref for SharedHistory<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Clone> DerefMut for SharedHistory<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::make_mut(&mut self.0)
    }
}

impl<T: Clone> SharedHistory<T> {
    pub(super) fn push(&mut self, value: T) {
        if let Some(history) = Arc::get_mut(&mut self.0) {
            history.push(value);
            return;
        }
        // Detach with room for this append. Cloning a Vec at exactly its old
        // length and then pushing would immediately allocate/copy it again.
        let mut history = Vec::with_capacity(self.0.len().saturating_add(1));
        history.extend_from_slice(&self.0);
        history.push(value);
        self.0 = Arc::new(history);
    }
}

impl<'a, T> IntoIterator for &'a SharedHistory<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_checkpoints_detach_on_append_and_existing_record_mutation() {
        let mut history = SharedHistory::default();
        history.push(String::from("confirmed"));
        let checkpoint = history.clone();
        assert!(Arc::ptr_eq(&history.0, &checkpoint.0));
        history.push(String::from("tentative"));
        assert!(!Arc::ptr_eq(&history.0, &checkpoint.0));
        assert_eq!(checkpoint.as_slice(), ["confirmed"]);
        let mut replacement = checkpoint.clone();
        replacement[0].push_str(" changed");
        assert_eq!(checkpoint.as_slice(), ["confirmed"]);
        assert_eq!(replacement.as_slice(), ["confirmed changed"]);
    }
}

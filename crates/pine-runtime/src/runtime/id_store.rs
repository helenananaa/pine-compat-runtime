//! Sparse persistent storage for monotonically allocated runtime handles.
//! Checkpoints share entries; a write copies one bounded leaf and its path.
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

const LEAF_BITS: u32 = 7;
const LEAF_SIZE: usize = 1 << LEAF_BITS;

#[derive(Clone, Debug)]
enum Node<V> {
    Branch {
        left: Option<Arc<Node<V>>>,
        right: Option<Arc<Node<V>>>,
    },
    Leaf(Vec<Option<Arc<V>>>),
}

#[derive(Clone, Debug)]
pub(crate) struct IdStore<V> {
    root: Option<Arc<Node<V>>>,
    len: usize,
    frame_aliases: Option<Arc<BTreeMap<u32, u32>>>,
}

impl<V: Clone> IdStore<V> {
    pub(crate) fn new() -> Self {
        Self {
            root: None,
            len: 0,
            frame_aliases: None,
        }
    }
    pub(crate) fn len(&self) -> usize {
        self.len
    }
    pub(crate) fn capacity(&self) -> usize {
        fn slots<V>(node: &Node<V>) -> usize {
            match node {
                Node::Leaf(values) => values.capacity(),
                Node::Branch { left, right } => {
                    left.as_deref().map_or(0, slots) + right.as_deref().map_or(0, slots)
                }
            }
        }
        self.root.as_deref().map_or(0, slots)
    }
    pub(crate) fn contains_key(&self, key: &u32) -> bool {
        self.get(key).is_some()
    }

    pub(crate) fn get(&self, key: &u32) -> Option<&V> {
        self.get_arc(key).map(AsRef::as_ref)
    }

    fn get_arc(&self, key: &u32) -> Option<&Arc<V>> {
        let key = &self.resolve_key(*key);
        let mut node = self.root.as_deref()?;
        for bit in (LEAF_BITS..32).rev() {
            let Node::Branch { left, right } = node else {
                unreachable!("branch depth invariant")
            };
            node = if key & (1 << bit) == 0 { left } else { right }.as_deref()?;
        }
        let Node::Leaf(values) = node else {
            unreachable!("leaf depth invariant")
        };
        values[(*key as usize) & (LEAF_SIZE - 1)].as_ref()
    }

    fn resolve_key(&self, key: u32) -> u32 {
        self.frame_aliases
            .as_deref()
            .and_then(|aliases| aliases.get(&key))
            .copied()
            .unwrap_or(key)
    }

    /// Share writes for this execution. Finishing the execution freezes each
    /// handle's buffer independently, while slice parents keep their identity.
    pub(crate) fn link_entry(&mut self, source: u32, target: u32) -> bool {
        let source = self.resolve_key(source);
        if !self.alias_entry(source, target) {
            return false;
        }
        Arc::make_mut(
            self.frame_aliases
                .get_or_insert_with(|| Arc::new(BTreeMap::new())),
        )
        .insert(target, source);
        true
    }

    pub(crate) fn finish_linked_entries(&mut self) {
        let Some(aliases) = self.frame_aliases.take() else {
            return;
        };
        for (&target, &source) in aliases.iter() {
            self.alias_entry(source, target);
        }
    }

    /// Give a new internal identity the same immutable payload. A later write
    /// uses the usual budgeted copy-on-write path.
    pub(crate) fn alias_entry(&mut self, source: u32, target: u32) -> bool {
        let Some(value) = self.get_arc(&source).cloned() else {
            return false;
        };
        let slot = slot_mut(&mut self.root, 31, target);
        self.len += usize::from(slot.is_none());
        *slot = Some(value);
        true
    }

    pub(crate) fn insert(&mut self, key: u32, value: V) {
        let key = self.resolve_key(key);
        let slot = slot_mut(&mut self.root, 31, key);
        self.len += usize::from(slot.is_none());
        *slot = Some(Arc::new(value));
    }

    pub(crate) fn get_mut(&mut self, key: &u32) -> Option<&mut V> {
        self.get(key)?;
        let key = self.resolve_key(*key);
        slot_mut(&mut self.root, 31, key)
            .as_mut()
            .map(Arc::make_mut)
    }

    /// A shared path duplicates entry Arcs before making the value mutable,
    /// even when its current entry Arc still has only one direct owner.
    pub(crate) fn get_mut_clones_value(&self, key: &u32) -> bool {
        let key = &self.resolve_key(*key);
        let Some(mut node) = self.root.as_ref() else {
            return false;
        };
        let mut shared = Arc::strong_count(node) > 1;
        for bit in (LEAF_BITS..32).rev() {
            let Node::Branch { left, right } = node.as_ref() else {
                unreachable!("branch depth invariant")
            };
            let Some(next) = (if key & (1 << bit) == 0 { left } else { right }).as_ref() else {
                return false;
            };
            node = next;
            shared |= Arc::strong_count(node) > 1;
        }
        let Node::Leaf(values) = node.as_ref() else {
            unreachable!("leaf depth invariant")
        };
        values[(*key as usize) & (LEAF_SIZE - 1)]
            .as_ref()
            .is_some_and(|value| shared || Arc::strong_count(value) > 1)
    }

    pub(crate) fn remove(&mut self, key: u32) {
        if self.get(&key).is_none() {
            return;
        }
        fn remove<V: Clone>(root: &mut Option<Arc<Node<V>>>, bit: u32, key: u32) {
            let Some(node) = root else { return };
            let empty = match Arc::make_mut(node) {
                Node::Leaf(values) => {
                    values[key as usize & (LEAF_SIZE - 1)] = None;
                    values.iter().all(Option::is_none)
                }
                Node::Branch { left, right } => {
                    remove(
                        if key & (1 << bit) == 0 {
                            &mut *left
                        } else {
                            &mut *right
                        },
                        bit - 1,
                        key,
                    );
                    left.is_none() && right.is_none()
                }
            };
            if empty {
                *root = None;
            }
        }
        remove(&mut self.root, 31, key);
        self.len -= 1;
    }

    /// Visit changed identities in key order, skipping shared subtrees and
    /// shared entries. Identity, rather than PartialEq, detects replacements
    /// with equal lengths or values containing NaN. Returns visited node pairs.
    pub(crate) fn visit_differences(
        &self,
        previous: &Self,
        mut visit: impl FnMut(u32, Option<&V>, Option<&V>),
    ) -> usize {
        fn walk<V>(
            previous: Option<&Arc<Node<V>>>,
            current: Option<&Arc<Node<V>>>,
            bit: u32,
            prefix: u32,
            visit: &mut impl FnMut(u32, Option<&V>, Option<&V>),
        ) -> usize {
            if previous
                .zip(current)
                .is_some_and(|(a, b)| Arc::ptr_eq(a, b))
                || (previous.is_none() && current.is_none())
            {
                return 1;
            }
            if bit < LEAF_BITS {
                let previous = match previous.map(AsRef::as_ref) {
                    Some(Node::Leaf(values)) => Some(values),
                    None => None,
                    _ => unreachable!("leaf depth invariant"),
                };
                let current = match current.map(AsRef::as_ref) {
                    Some(Node::Leaf(values)) => Some(values),
                    None => None,
                    _ => unreachable!("leaf depth invariant"),
                };
                for index in 0..LEAF_SIZE {
                    let old = previous.and_then(|values| values[index].as_ref());
                    let new = current.and_then(|values| values[index].as_ref());
                    if old.zip(new).is_some_and(|(a, b)| Arc::ptr_eq(a, b))
                        || (old.is_none() && new.is_none())
                    {
                        continue;
                    }
                    visit(
                        prefix | index as u32,
                        old.map(AsRef::as_ref),
                        new.map(AsRef::as_ref),
                    );
                }
                return 1;
            }
            let (old_left, old_right) = match previous.map(AsRef::as_ref) {
                Some(Node::Branch { left, right }) => (left.as_ref(), right.as_ref()),
                None => (None, None),
                _ => unreachable!("branch depth invariant"),
            };
            let (new_left, new_right) = match current.map(AsRef::as_ref) {
                Some(Node::Branch { left, right }) => (left.as_ref(), right.as_ref()),
                None => (None, None),
                _ => unreachable!("branch depth invariant"),
            };
            1 + walk(old_left, new_left, bit - 1, prefix, visit)
                + walk(old_right, new_right, bit - 1, prefix | (1 << bit), visit)
        }
        walk(
            previous.root.as_ref(),
            self.root.as_ref(),
            31,
            0,
            &mut visit,
        )
    }

    /// Copy one retained identity between checkpoints without cloning payloads.
    pub(crate) fn copy_entry_from(&mut self, previous: &Self, key: u32) -> bool {
        let source_key = previous.resolve_key(key);
        let mut node = previous.root.as_deref();
        for bit in (LEAF_BITS..32).rev() {
            let Some(Node::Branch { left, right }) = node else {
                return false;
            };
            node = if source_key & (1 << bit) == 0 {
                left
            } else {
                right
            }
            .as_deref();
        }
        let Some(Node::Leaf(values)) = node else {
            return false;
        };
        let Some(value) = &values[(source_key as usize) & (LEAF_SIZE - 1)] else {
            return false;
        };
        let slot = slot_mut(&mut self.root, 31, key);
        self.len += usize::from(slot.is_none());
        *slot = Some(value.clone());
        true
    }

    pub(crate) fn values(&self) -> impl Iterator<Item = &V> {
        Values {
            stack: self
                .root
                .as_deref()
                .map(|node| (node, 31, 0))
                .into_iter()
                .collect(),
            leaf: None,
            store: self,
            prefix: 0,
        }
    }

    #[cfg(test)]
    pub(crate) fn entries(&self) -> impl Iterator<Item = (u32, &V)> {
        let mut entries = Vec::with_capacity(self.len);
        fn visit<'a, V>(node: &'a Node<V>, bit: u32, prefix: u32, out: &mut Vec<(u32, &'a V)>) {
            match node {
                Node::Leaf(values) => {
                    for (index, value) in values.iter().enumerate() {
                        if let Some(value) = value {
                            out.push((prefix | index as u32, value.as_ref()));
                        }
                    }
                }
                Node::Branch { left, right } => {
                    if let Some(left) = left {
                        visit(left, bit - 1, prefix, out);
                    }
                    if let Some(right) = right {
                        visit(right, bit - 1, prefix | (1 << bit), out);
                    }
                }
            }
        }
        if let Some(root) = &self.root {
            visit(root, 31, 0, &mut entries);
        }
        entries
            .into_iter()
            .map(|(id, _)| (id, self.get(&id).expect("live entry")))
    }

    pub(crate) fn retain(&mut self, mut keep: impl FnMut(u32, &V) -> bool) {
        fn visit<V: Clone>(
            root: &mut Option<Arc<Node<V>>>,
            bit: u32,
            prefix: u32,
            keep: &mut impl FnMut(u32, &V) -> bool,
        ) -> usize {
            let Some(node) = root else { return 0 };
            let count = match Arc::make_mut(node) {
                Node::Leaf(values) => {
                    let mut count = 0;
                    for (index, value) in values.iter_mut().enumerate() {
                        if value
                            .as_ref()
                            .is_some_and(|value| !keep(prefix | index as u32, value))
                        {
                            *value = None;
                        }
                        count += usize::from(value.is_some());
                    }
                    count
                }
                Node::Branch { left, right } => {
                    visit(left, bit - 1, prefix, keep)
                        + visit(right, bit - 1, prefix | (1 << bit), keep)
                }
            };
            if count == 0 {
                *root = None;
            }
            count
        }
        let aliases = self.frame_aliases.clone();
        let mut retained = BTreeMap::new();
        let mut roots = HashSet::new();
        if let Some(aliases) = aliases.as_deref() {
            for (&target, &source) in aliases {
                if self.get(&target).is_some_and(|value| keep(target, value)) {
                    retained.insert(target, source);
                    roots.insert(source);
                }
            }
        }
        self.len = visit(&mut self.root, 31, 0, &mut |id, value| {
            if aliases
                .as_deref()
                .is_some_and(|aliases| aliases.contains_key(&id))
            {
                retained.contains_key(&id)
            } else {
                roots.contains(&id) || keep(id, value)
            }
        });
        self.frame_aliases = if retained.is_empty() {
            None
        } else {
            Some(Arc::new(retained))
        };
    }
}

#[cfg(test)]
impl<V: Clone> std::ops::Index<&u32> for IdStore<V> {
    type Output = V;
    fn index(&self, key: &u32) -> &Self::Output {
        self.get(key).expect("valid runtime handle")
    }
}

fn slot_mut<V: Clone>(node: &mut Option<Arc<Node<V>>>, bit: u32, key: u32) -> &mut Option<Arc<V>> {
    let node = node.get_or_insert_with(|| {
        Arc::new(if bit < LEAF_BITS {
            Node::Leaf(vec![None; LEAF_SIZE])
        } else {
            Node::Branch {
                left: None,
                right: None,
            }
        })
    });
    match Arc::make_mut(node) {
        Node::Leaf(values) => &mut values[(key as usize) & (LEAF_SIZE - 1)],
        Node::Branch { left, right } => slot_mut(
            if key & (1 << bit) == 0 { left } else { right },
            bit - 1,
            key,
        ),
    }
}

struct Values<'a, V> {
    stack: Vec<(&'a Node<V>, u32, u32)>,
    leaf: Option<std::iter::Enumerate<std::slice::Iter<'a, Option<Arc<V>>>>>,
    store: &'a IdStore<V>,
    prefix: u32,
}
impl<'a, V: Clone> Iterator for Values<'a, V> {
    type Item = &'a V;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(leaf) = &mut self.leaf {
                for (index, value) in leaf.by_ref() {
                    if let Some(value) = value {
                        return Some(if self.store.frame_aliases.is_some() {
                            self.store
                                .get(&(self.prefix | index as u32))
                                .expect("live alias root")
                        } else {
                            value.as_ref()
                        });
                    }
                }
                self.leaf = None;
            }
            match self.stack.pop()? {
                (Node::Leaf(values), _, prefix) => {
                    self.leaf = Some(values.iter().enumerate());
                    self.prefix = prefix;
                }
                (Node::Branch { left, right }, bit, prefix) => {
                    if let Some(node) = right {
                        self.stack.push((node, bit - 1, prefix | (1 << bit)));
                    }
                    if let Some(node) = left {
                        self.stack.push((node, bit - 1, prefix));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn execution_links_share_writes_then_freeze_and_keep_live_gc_roots() {
        let mut store = IdStore::new();
        store.insert(1, vec![1]);
        assert!(store.link_entry(1, 2));
        assert!(store.link_entry(2, 3));
        store.get_mut(&2).unwrap().push(2);
        assert_eq!(store.get(&1), Some(&vec![1, 2]));
        assert_eq!(store.get(&3), Some(&vec![1, 2]));
        store.insert(3, vec![7]);
        assert!(store.values().all(|v| v == &vec![7]));
        let before = store.clone();
        store.retain(|id, _| id == 2);
        assert_eq!(store.get(&2), Some(&vec![7]));
        store.get_mut(&2).unwrap().push(8);
        assert_eq!(before.get(&2), Some(&vec![7]));
        store.finish_linked_entries();
        store.get_mut(&2).unwrap().push(9);
        assert_eq!(store.get(&1), Some(&vec![7, 8]));
        store.retain(|id, _| id == 2);
        assert_eq!(store.len(), 1);
        assert_eq!(store.get(&2), Some(&vec![7, 8, 9]));
    }

    #[test]
    fn differences_skip_shared_history_and_visit_rollback_removals() {
        let mut base = IdStore::new();
        for id in 0..32_768 {
            base.insert(id, id);
        }
        let cursor = base.clone();
        let mut seen = Vec::new();
        assert_eq!(base.visit_differences(&cursor, |id, _, _| seen.push(id)), 1);
        assert!(seen.is_empty());
        *base.get_mut(&16_384).unwrap() = 99;
        let visits = base.visit_differences(&cursor, |id, old, new| {
            seen.push(id);
            assert_eq!(old, Some(&16_384));
            assert_eq!(new, Some(&99));
        });
        assert_eq!(seen, [16_384]);
        assert!(
            visits <= 51,
            "one changed leaf must not scan all nodes: {visits}"
        );
        base.insert(u32::MAX, 55);
        base.remove(127);
        let mut changes = Vec::new();
        cursor.visit_differences(&base, |id, old, new| {
            changes.push((id, old.copied(), new.copied()))
        });
        assert_eq!(
            changes,
            [
                (127, None, Some(127)),
                (16_384, Some(99), Some(16_384)),
                (u32::MAX, Some(55), None)
            ]
        );
        assert_eq!(cursor.len(), 32_768);
    }

    #[test]
    fn removing_sparse_handles_preserves_checkpoints_and_reclaims_empty_paths() {
        let mut store = IdStore::new();
        for id in [0, 127, 128, 65_536, u32::MAX] {
            store.insert(id, id);
        }
        let checkpoint = store.clone();
        for id in [0, 127, 128, 65_536, u32::MAX] {
            store.remove(id);
            store.remove(id);
        }
        assert_eq!(store.len(), 0);
        assert_eq!(store.capacity(), 0);
        assert_eq!(checkpoint.len(), 5);
        assert_eq!(checkpoint.get(&u32::MAX), Some(&u32::MAX));
    }

    #[test]
    fn differences_use_version_identity_for_nan_payloads() {
        let mut store = IdStore::new();
        store.insert(5, f64::NAN);
        let previous = store.clone();
        assert_eq!(
            store.visit_differences(&previous, |_, _, _| panic!("shared NaN")),
            1
        );
        store.insert(5, f64::NAN);
        let mut visited = 0;
        store.visit_differences(&previous, |id, old, new| {
            visited += 1;
            assert_eq!(id, 5);
            assert!(old.unwrap().is_nan() && new.unwrap().is_nan());
        });
        assert_eq!(visited, 1);
    }
    #[test]
    fn retaining_sparse_handles_prunes_storage_without_mutating_checkpoints() {
        let mut store = IdStore::new();
        for id in [0, 127, 128, 65536, u32::MAX] {
            store.insert(id, vec![id]);
        }
        let original = store.clone();
        store.retain(|id, _| id == 128 || id == u32::MAX);
        assert_eq!(
            store.entries().map(|(id, _)| id).collect::<Vec<_>>(),
            [128, u32::MAX]
        );
        assert_eq!(store.len(), 2);
        assert_eq!(store.capacity(), 2 * LEAF_SIZE);
        store.get_mut(&128).unwrap().push(5);
        assert_eq!(original.get(&128), Some(&vec![128]));
        assert_eq!(original.len(), 5);
        store.retain(|_, _| false);
        assert_eq!(store.capacity(), 0);
    }
    #[test]
    fn sparse_ids_and_checkpoint_mutations_stay_independent() {
        let mut store = IdStore::new();
        for id in [0, 127, 128, 65536, u32::MAX] {
            store.insert(id, vec![id]);
        }
        let original = store.clone();
        store.get_mut(&128).unwrap().push(7);
        store.insert(u32::MAX, vec![5]);
        store.insert(500, vec![9]);
        assert_eq!(original.get(&128), Some(&vec![128]));
        assert_eq!(original.get(&u32::MAX), Some(&vec![u32::MAX]));
        assert!(original.get(&500).is_none());
        assert!(store.get_mut(&42).is_none());
        assert_eq!(store.len(), 6);
        assert_eq!(store.values().count(), 6);
        assert_eq!(original.values().count(), 5);
    }
    #[test]
    fn cloning_or_inserting_does_not_clone_untouched_payloads() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        struct Payload(Arc<AtomicUsize>);
        impl Clone for Payload {
            fn clone(&self) -> Self {
                self.0.fetch_add(1, Ordering::Relaxed);
                Self(self.0.clone())
            }
        }
        let clones = Arc::new(AtomicUsize::new(0));
        let mut store = IdStore::new();
        for id in 0..10000 {
            store.insert(id, Payload(clones.clone()));
        }
        let original = store.clone();
        store.insert(10000, Payload(clones.clone()));
        let mut imported = IdStore::new();
        assert!(imported.copy_entry_from(&original, 0));
        assert!(!imported.copy_entry_from(&original, 20000));
        assert_eq!(imported.len(), 1);
        assert_eq!(clones.load(Ordering::Relaxed), 0);
        store.get_mut(&0).unwrap();
        assert_eq!(clones.load(Ordering::Relaxed), 1);
        assert_eq!(original.len(), 10000);
    }

    #[test]
    fn aliases_share_payloads_until_an_independent_write() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        struct Payload {
            clones: Arc<AtomicUsize>,
            value: u32,
        }
        impl Clone for Payload {
            fn clone(&self) -> Self {
                self.clones.fetch_add(1, Ordering::Relaxed);
                Self {
                    clones: self.clones.clone(),
                    value: self.value,
                }
            }
        }
        let clones = Arc::new(AtomicUsize::new(0));
        let mut store = IdStore::new();
        store.insert(
            7,
            Payload {
                clones: clones.clone(),
                value: 10,
            },
        );
        assert!(store.alias_entry(7, 50000));
        assert_eq!(clones.load(Ordering::Relaxed), 0);
        assert!(store.get_mut_clones_value(&50000));
        store.get_mut(&50000).unwrap().value = 20;
        assert_eq!(clones.load(Ordering::Relaxed), 1);
        assert_eq!(store.get(&7).unwrap().value, 10);
        assert_eq!(store.get(&50000).unwrap().value, 20);
        assert!(!store.alias_entry(42, 50001));
    }
}

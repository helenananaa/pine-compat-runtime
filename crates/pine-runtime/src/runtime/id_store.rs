//! Sparse persistent storage for monotonically allocated runtime handles.
//! Checkpoints share entries; a write copies one bounded leaf and its path.
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
}

impl<V: Clone> IdStore<V> {
    pub(crate) fn new() -> Self {
        Self { root: None, len: 0 }
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
        values[(*key as usize) & (LEAF_SIZE - 1)].as_deref()
    }

    pub(crate) fn insert(&mut self, key: u32, value: V) {
        let slot = slot_mut(&mut self.root, 31, key);
        self.len += usize::from(slot.is_none());
        *slot = Some(Arc::new(value));
    }

    pub(crate) fn get_mut(&mut self, key: &u32) -> Option<&mut V> {
        self.get(key)?;
        slot_mut(&mut self.root, 31, *key)
            .as_mut()
            .map(Arc::make_mut)
    }

    /// Copy one retained identity between checkpoints without cloning payloads.
    pub(crate) fn copy_entry_from(&mut self, previous: &Self, key: u32) -> bool {
        let mut node = previous.root.as_deref();
        for bit in (LEAF_BITS..32).rev() {
            let Some(Node::Branch { left, right }) = node else {
                return false;
            };
            node = if key & (1 << bit) == 0 { left } else { right }.as_deref();
        }
        let Some(Node::Leaf(values)) = node else {
            return false;
        };
        let Some(value) = &values[(key as usize) & (LEAF_SIZE - 1)] else {
            return false;
        };
        let slot = slot_mut(&mut self.root, 31, key);
        self.len += usize::from(slot.is_none());
        *slot = Some(value.clone());
        true
    }

    pub(crate) fn values(&self) -> impl Iterator<Item = &V> {
        Values {
            stack: self.root.as_deref().into_iter().collect(),
            leaf: None,
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
        entries.into_iter()
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
        self.len = visit(&mut self.root, 31, 0, &mut keep);
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
    stack: Vec<&'a Node<V>>,
    leaf: Option<std::slice::Iter<'a, Option<Arc<V>>>>,
}
impl<'a, V> Iterator for Values<'a, V> {
    type Item = &'a V;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(leaf) = &mut self.leaf {
                if let Some(value) = leaf.flatten().next() {
                    return Some(value.as_ref());
                }
                self.leaf = None;
            }
            match self.stack.pop()? {
                Node::Leaf(values) => self.leaf = Some(values.iter()),
                Node::Branch { left, right } => {
                    if let Some(node) = right {
                        self.stack.push(node);
                    }
                    if let Some(node) = left {
                        self.stack.push(node);
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
}

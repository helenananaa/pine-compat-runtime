use std::{ops::Index, sync::Arc};

pub(crate) const APPEND_LEAF_SIZE: usize = 128;

/// Persistent append tree. Checkpoints share closed history; an append or tail
/// replacement copies at most one bounded leaf and a logarithmic branch path.
#[derive(Debug, Clone)]
pub(crate) struct AppendHistory<T> {
    root: Arc<Node<T>>,
    len: usize,
    start: usize,
    capacity: usize,
}

#[derive(Debug, Clone)]
enum Node<T> {
    Leaf(Vec<T>),
    Branch {
        left: Arc<Node<T>>,
        right: Option<Arc<Node<T>>>,
    },
}

impl<T> Default for AppendHistory<T> {
    fn default() -> Self {
        Self {
            root: Arc::new(Node::Leaf(Vec::new())),
            len: 0,
            start: 0,
            capacity: APPEND_LEAF_SIZE,
        }
    }
}

impl<T> AppendHistory<T> {
    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len == 0
    }

    // Existing profile field counts allocated value slots, not branch headers.
    pub(crate) fn capacity(&self) -> usize {
        fn slots<T>(node: &Node<T>) -> usize {
            match node {
                Node::Leaf(values) => values.capacity(),
                Node::Branch { left, right } => {
                    slots(left) + right.as_ref().map_or(0, |r| slots(r))
                }
            }
        }
        slots(&self.root)
    }

    pub(crate) fn get(&self, index: usize) -> Option<&T> {
        (index < self.len).then(|| element(&self.root, self.capacity, self.start + index))
    }

    pub(crate) fn last(&self) -> Option<&T> {
        self.len
            .checked_sub(1)
            .map(|index| element(&self.root, self.capacity, self.start + index))
    }

    pub(crate) fn iter(&self) -> Iter<'_, T> {
        Iter {
            root: &self.root,
            capacity: self.capacity,
            index: self.start,
            len: self.start + self.len,
        }
    }

    pub(crate) fn partition_point(&self, predicate: impl Fn(&T) -> bool) -> usize {
        let (mut lo, mut hi) = (0, self.len);
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            if predicate(element(&self.root, self.capacity, self.start + mid)) {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        lo
    }
}

impl<T: Clone> AppendHistory<T> {
    pub(crate) fn from_values(values: impl IntoIterator<Item = T>) -> Self {
        let mut history = Self::default();
        for value in values {
            history.push(value);
        }
        history
    }

    pub(crate) fn push(&mut self, value: T) {
        if self.start + self.len == self.capacity {
            self.root = Arc::new(Node::Branch {
                left: self.root.clone(),
                right: None,
            });
            self.capacity *= 2;
        }
        insert(&mut self.root, self.capacity, self.start + self.len, value);
        self.len += 1;
    }

    pub(crate) fn last_mut(&mut self) -> Option<&mut T> {
        let index = self.len.checked_sub(1)?;
        Some(element_mut(
            &mut self.root,
            self.capacity,
            self.start + index,
        ))
    }

    pub(crate) fn tail(&self, start: usize) -> Vec<T> {
        let mut values = Vec::with_capacity(self.len.saturating_sub(start));
        collect(
            &self.root,
            self.capacity,
            self.start.saturating_add(start),
            &mut values,
        );
        values
    }

    pub(crate) fn to_vec(&self) -> Vec<T> {
        self.tail(0)
    }

    pub(crate) fn drop_prefix(&mut self, count: usize) {
        if count == 0 {
            return;
        }
        if count >= self.len {
            *self = Self::default();
            return;
        }
        self.start += count;
        self.len -= count;
        // Re-root at a surviving right subtree before pruning. The address
        // space remains bounded by the live window rather than session age.
        while self.capacity > APPEND_LEAF_SIZE && self.start >= self.capacity / 2 {
            let Node::Branch { right, .. } = self.root.as_ref() else {
                unreachable!()
            };
            self.root = right.as_ref().expect("surviving suffix").clone();
            self.capacity /= 2;
            self.start -= self.capacity;
        }
        prune_prefix(&mut self.root, self.capacity, self.start);
    }
}

impl<T> Index<usize> for AppendHistory<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index)
            .unwrap_or_else(|| panic!("index {index} out of bounds"))
    }
}

impl<T: PartialEq> PartialEq for AppendHistory<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.root, &other.root) && self.start == other.start && self.len == other.len
            || self.len == other.len && self.iter().eq(other.iter())
    }
}

impl<'a, T> IntoIterator for &'a AppendHistory<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub(crate) struct Iter<'a, T> {
    root: &'a Node<T>,
    capacity: usize,
    index: usize,
    len: usize,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            return None;
        }
        let item = element(self.root, self.capacity, self.index);
        self.index += 1;
        Some(item)
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.index = self.index.saturating_add(n).min(self.len);
        self.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.len.saturating_sub(self.index);
        (remaining, Some(remaining))
    }
}

impl<T> ExactSizeIterator for Iter<'_, T> {}

fn prune_prefix<T: Clone>(node: &mut Arc<Node<T>>, span: usize, count: usize) {
    if count == 0 {
        return;
    }
    if count >= span {
        *node = empty(span);
        return;
    }
    // A partial boundary leaf retains at most 127 expired values. No surviving
    // values are cloned when trimming; checkpoints keep their original tree.
    if matches!(node.as_ref(), Node::Leaf(_)) {
        return;
    }
    if let Node::Branch { left, right } = Arc::make_mut(node) {
        let half = span / 2;
        prune_prefix(left, half, count.min(half));
        if count > half
            && let Some(right) = right
        {
            prune_prefix(right, half, count - half);
        }
    }
}

fn empty<T>(span: usize) -> Arc<Node<T>> {
    if span == APPEND_LEAF_SIZE {
        Arc::new(Node::Leaf(Vec::new()))
    } else {
        Arc::new(Node::Branch {
            left: empty(span / 2),
            right: None,
        })
    }
}

fn insert<T: Clone>(node: &mut Arc<Node<T>>, span: usize, index: usize, value: T) {
    match Arc::make_mut(node) {
        Node::Leaf(values) => {
            debug_assert_eq!(values.len(), index);
            values.push(value);
        }
        Node::Branch { left, right } => {
            let half = span / 2;
            if index < half {
                insert(left, half, index, value);
            } else {
                insert(
                    right.get_or_insert_with(|| empty(half)),
                    half,
                    index - half,
                    value,
                );
            }
        }
    }
}

fn element<T>(node: &Node<T>, span: usize, index: usize) -> &T {
    match node {
        Node::Leaf(values) => &values[index],
        Node::Branch { left, right } => {
            let half = span / 2;
            if index < half {
                element(left, half, index)
            } else {
                element(
                    right.as_ref().expect("occupied append path"),
                    half,
                    index - half,
                )
            }
        }
    }
}

fn element_mut<T: Clone>(node: &mut Arc<Node<T>>, span: usize, index: usize) -> &mut T {
    match Arc::make_mut(node) {
        Node::Leaf(values) => &mut values[index],
        Node::Branch { left, right } => {
            let half = span / 2;
            if index < half {
                element_mut(left, half, index)
            } else {
                element_mut(
                    right.as_mut().expect("occupied append path"),
                    half,
                    index - half,
                )
            }
        }
    }
}

fn collect<T: Clone>(node: &Node<T>, span: usize, start: usize, out: &mut Vec<T>) {
    if start >= span {
        return;
    }
    match node {
        Node::Leaf(values) => out.extend(values.get(start..).unwrap_or(&[]).iter().cloned()),
        Node::Branch { left, right } => {
            let half = span / 2;
            if start < half {
                collect(left, half, start, out);
            }
            if let Some(right) = right {
                collect(right, half, start.saturating_sub(half), out);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn branches_and_tail_replacement_preserve_checkpoints() {
        let mut history = AppendHistory::default();
        for i in 0..100_003 {
            history.push(i);
        }
        let checkpoint = history.clone();
        *history.last_mut().unwrap() = -1;
        history.push(-2);
        assert_eq!(checkpoint.tail(99_999), [99_999, 100_000, 100_001, 100_002]);
        assert_eq!(history.tail(100_001), [100_001, -1, -2]);
        assert!(history.tail(200_000).is_empty());
        assert_eq!(checkpoint.to_vec(), (0..100_003).collect::<Vec<_>>());
    }

    #[test]
    fn checkpoint_append_copies_only_a_bounded_leaf() {
        #[derive(Debug)]
        struct Counted(Arc<AtomicUsize>);
        impl Clone for Counted {
            fn clone(&self) -> Self {
                self.0.fetch_add(1, Ordering::Relaxed);
                Self(self.0.clone())
            }
        }
        let count = Arc::new(AtomicUsize::new(0));
        let mut history = AppendHistory::default();
        for _ in 0..100_001 {
            history.push(Counted(count.clone()));
        }
        let checkpoint = history.clone();
        count.store(0, Ordering::Relaxed);
        history.push(Counted(count.clone()));
        assert!(count.load(Ordering::Relaxed) <= APPEND_LEAF_SIZE);
        assert_eq!(checkpoint.len(), 100_001);
        assert_eq!(history.len(), 100_002);
    }
    #[test]
    fn repeated_trim_and_append_preserves_checkpoints_and_bounded_storage() {
        let mut history = AppendHistory::from_values(0..100_000);
        let original = history.clone();
        history.drop_prefix(99_984);
        assert_eq!(history.to_vec(), (99_984..100_000).collect::<Vec<_>>());
        for value in 100_000..110_000 {
            history.push(value);
            history.drop_prefix(1);
            assert_eq!(history.len(), 16);
            assert_eq!(history[0], value - 15);
            assert_eq!(*history.last().unwrap(), value);
            assert!(history.capacity() <= APPEND_LEAF_SIZE * 3);
        }
        assert_eq!(original.to_vec(), (0..100_000).collect::<Vec<_>>());
    }

    #[test]
    fn trimming_does_not_clone_surviving_values() {
        #[derive(Debug)]
        struct Counted(Arc<AtomicUsize>);
        impl Clone for Counted {
            fn clone(&self) -> Self {
                self.0.fetch_add(1, Ordering::Relaxed);
                Self(self.0.clone())
            }
        }
        let count = Arc::new(AtomicUsize::new(0));
        let mut history = AppendHistory::from_values((0..100_000).map(|_| Counted(count.clone())));
        let checkpoint = history.clone();
        count.store(0, Ordering::Relaxed);
        history.drop_prefix(10_001);
        assert_eq!(count.load(Ordering::Relaxed), 0);
        assert_eq!(history.len(), 89_999);
        assert_eq!(checkpoint.len(), 100_000);
    }
    #[test]
    fn iterator_skip_uses_logical_indexes_after_pruning() {
        let mut history = AppendHistory::from_values(0..1000);
        history.drop_prefix(750);
        let mut iter = history.iter();
        assert_eq!(iter.nth(100), Some(&850));
        assert_eq!(iter.len(), 149);
        assert_eq!(iter.nth(usize::MAX), None);
        assert_eq!(iter.len(), 0);
    }
}

use std::sync::Arc;

const LEAF_SIZE: usize = 128;

/// Persistent append tree. Checkpoints share closed history; an append or tail
/// replacement copies at most one bounded leaf and a logarithmic branch path.
#[derive(Debug, Clone)]
pub(crate) struct AppendHistory<T> {
    root: Arc<Node<T>>,
    len: usize,
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
            capacity: LEAF_SIZE,
        }
    }
}

impl<T: Clone> AppendHistory<T> {
    pub(crate) fn len(&self) -> usize {
        self.len
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

    pub(crate) fn partition_point(&self, predicate: impl Fn(&T) -> bool) -> usize {
        let (mut lo, mut hi) = (0, self.len);
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            if predicate(element(&self.root, self.capacity, mid)) {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        lo
    }

    pub(crate) fn push(&mut self, value: T) {
        if self.len == self.capacity {
            self.root = Arc::new(Node::Branch {
                left: self.root.clone(),
                right: None,
            });
            self.capacity *= 2;
        }
        insert(&mut self.root, self.capacity, self.len, value);
        self.len += 1;
    }

    pub(crate) fn last_mut(&mut self) -> Option<&mut T> {
        let index = self.len.checked_sub(1)?;
        Some(element_mut(&mut self.root, self.capacity, index))
    }

    pub(crate) fn tail(&self, start: usize) -> Vec<T> {
        let mut values = Vec::with_capacity(self.len.saturating_sub(start));
        collect(&self.root, self.capacity, start, &mut values);
        values
    }

    pub(crate) fn to_vec(&self) -> Vec<T> {
        self.tail(0)
    }
}

fn empty<T>(span: usize) -> Arc<Node<T>> {
    if span == LEAF_SIZE {
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
        assert!(count.load(Ordering::Relaxed) <= LEAF_SIZE);
        assert_eq!(checkpoint.len(), 100_001);
        assert_eq!(history.len(), 100_002);
    }
}

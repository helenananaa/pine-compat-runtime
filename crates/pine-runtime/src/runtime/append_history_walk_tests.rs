use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
struct Counted {
    value: usize,
    clones: Arc<AtomicUsize>,
}

impl Clone for Counted {
    fn clone(&self) -> Self {
        self.clones.fetch_add(1, Ordering::Relaxed);
        Self {
            value: self.value,
            clones: self.clones.clone(),
        }
    }
}

#[test]
fn sequential_iteration_visits_each_subtree_once_without_cloning_payloads() {
    let clones = Arc::new(AtomicUsize::new(0));
    let mut history = AppendHistory::from_values((0..100_003).map(|value| Counted {
        value,
        clones: clones.clone(),
    }));
    history.drop_prefix(10_001);
    let mut iter = history.iter();
    for expected in 10_001..100_003 {
        assert_eq!(iter.next().unwrap().value, expected);
        assert_eq!(iter.len(), 100_002 - expected);
    }
    assert!(iter.next().is_none());
    // Includes the two boundary leaves and the initial path to the suffix.
    assert!(iter.node_visits <= 2 * history.len().div_ceil(APPEND_LEAF_SIZE) + 64);
    assert_eq!(clones.load(Ordering::Relaxed), 0);
}

#[test]
#[allow(clippy::iter_nth_zero)] // Exercise the custom nth zero-offset branch explicitly.
fn iterator_nth_seeks_without_walking_skipped_leaves_and_can_resume() {
    let mut history = AppendHistory::from_values(0..100_003);
    history.drop_prefix(255);
    let mut iter = history.iter();
    assert_eq!(iter.next(), Some(&255));
    let before_seek = iter.node_visits;
    assert_eq!(iter.nth(90_000), Some(&90_256));
    assert!(iter.node_visits - before_seek <= usize::BITS as usize);
    assert_eq!(iter.len(), 9_746);
    assert_eq!(iter.nth(0), Some(&90_257));
    assert_eq!(iter.nth(126), Some(&90_384));
    assert_eq!(
        iter.by_ref().take(130).copied().collect::<Vec<_>>(),
        (90_385..90_515).collect::<Vec<_>>()
    );
    assert_eq!(iter.nth(usize::MAX), None);
    assert_eq!(iter.size_hint(), (0, Some(0)));
    assert_eq!(iter.nth(0), None);
    assert_eq!(iter.next(), None);
}

#[test]
fn trimming_inside_the_same_leaf_does_not_detach_shared_branches() {
    let clones = Arc::new(AtomicUsize::new(0));
    let mut history = AppendHistory::from_values((0..8192).map(|value| Counted {
        value,
        clones: clones.clone(),
    }));
    let checkpoint = history.clone();
    for count in 1..APPEND_LEAF_SIZE {
        history.drop_prefix(1);
        assert!(Arc::ptr_eq(&history.root, &checkpoint.root));
        assert_eq!(history[0].value, count);
    }
    history.drop_prefix(1);
    assert!(!Arc::ptr_eq(&history.root, &checkpoint.root));
    assert_eq!(history[0].value, APPEND_LEAF_SIZE);
    assert_eq!(checkpoint[0].value, 0);
    assert_eq!(clones.load(Ordering::Relaxed), 0);
}

fn assert_no_empty_branch_chains<T>(node: &Node<T>) -> bool {
    match node {
        Node::Empty => false,
        Node::Leaf(values) => !values.is_empty(),
        Node::Repeat { len, .. } => *len != 0,
        Node::Branch { left, right } => {
            let left_has_values = assert_no_empty_branch_chains(left);
            let right_has_values = right.as_deref().is_some_and(assert_no_empty_branch_chains);
            assert!(
                left_has_values || right_has_values,
                "pruned subtrees use one empty marker"
            );
            true
        }
    }
}

#[test]
fn large_pruned_subtrees_use_markers_and_survive_rerooting_and_append() {
    let mut history = AppendHistory::from_values(0..8192);
    let checkpoint = history.clone();
    history.drop_prefix(4095);
    assert_no_empty_branch_chains(&history.root);
    assert_eq!(
        history.iter().copied().collect::<Vec<_>>(),
        (4095..8192).collect::<Vec<_>>()
    );
    for value in 8192..12_500 {
        history.push(value);
        history.drop_prefix(1);
        if value % 127 == 0 {
            assert_no_empty_branch_chains(&history.root);
            assert_eq!(
                history.iter().copied().collect::<Vec<_>>(),
                ((value - 4096)..=value).collect::<Vec<_>>()
            );
            assert_eq!(
                history.to_vec(),
                history.iter().copied().collect::<Vec<_>>()
            );
        }
    }
    assert_eq!(
        checkpoint.iter().copied().collect::<Vec<_>>(),
        (0..8192).collect::<Vec<_>>()
    );
    history.drop_prefix(usize::MAX);
    assert!(history.iter().next().is_none());
    assert_eq!(history.iter().len(), 0);
    history.push(-1);
    assert_eq!(history.iter().copied().collect::<Vec<_>>(), [-1]);
}

#[test]
fn repeat_and_mixed_leaves_preserve_float_bits_and_checkpoint_branches() {
    use crate::PineValue;
    let bits = [
        0.0_f64.to_bits(),
        (-0.0_f64).to_bits(),
        0x7ff8000000000001,
        0x7ff8000000000002,
    ];
    let mut expected: Vec<_> = (0..1600)
        .map(|index| bits[(index / 131) % bits.len()])
        .collect();
    let mut history = AppendHistory::from_compact_values(
        expected
            .iter()
            .map(|bits| PineValue::Float(f64::from_bits(*bits))),
    );
    let original = history.clone();
    for count in [1, 126, 1, 129, 255, 1, 512] {
        history.drop_prefix(count);
        expected.drain(..count);
        let before_bits = expected.clone();
        let checkpoint = history.clone();
        let last_bits = ((expected.len() % 2) as f64).to_bits();
        *history.last_mut().unwrap() = PineValue::Float(f64::from_bits(last_bits));
        *expected.last_mut().unwrap() = last_bits;
        history.push_compact(PineValue::Float(f64::from_bits(bits[3])));
        expected.push(bits[3]);
        let observed: Vec<_> = history
            .iter()
            .map(|value| value.as_f64().unwrap().to_bits())
            .collect();
        assert_eq!(observed, expected);
        assert_eq!(checkpoint.len() + 1, history.len());
        assert_eq!(
            checkpoint
                .iter()
                .map(|value| value.as_f64().unwrap().to_bits())
                .collect::<Vec<_>>(),
            before_bits
        );
        let mut iter = checkpoint.iter();
        let offset = checkpoint.len() / 2;
        assert_eq!(
            iter.nth(offset)
                .map(|value| value.as_f64().unwrap().to_bits()),
            checkpoint
                .get(offset)
                .map(|value| value.as_f64().unwrap().to_bits())
        );
        assert_no_empty_branch_chains(&history.root);
    }
    let original_bits: Vec<_> = original
        .iter()
        .map(|value| value.as_f64().unwrap().to_bits())
        .collect();
    assert_eq!(
        original_bits,
        (0..1600)
            .map(|index| bits[(index / 131) % bits.len()])
            .collect::<Vec<_>>()
    );
}

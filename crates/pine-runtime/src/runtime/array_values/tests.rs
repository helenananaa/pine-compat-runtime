use std::{cell::Cell, rc::Rc};

use super::*;
use crate::{ChartPointValue, PineValue};

#[derive(Debug)]
struct Counted {
    value: usize,
    clones: Rc<Cell<usize>>,
}

#[test]
fn batched_middle_insertion_clones_each_old_cell_once_and_preserves_shared_checkpoints() {
    let clones = Rc::new(Cell::new(0));
    let original: ArrayValues<_> = (0..4096)
        .map(|value| Counted {
            value,
            clones: clones.clone(),
        })
        .collect::<Vec<_>>()
        .into();
    let checkpoint = original.clone();
    let incoming = (0..256)
        .map(|index| Counted {
            value: 10000 + index,
            clones: clones.clone(),
        })
        .collect();
    let inserted = original.with_inserted(1, incoming);
    assert_eq!(clones.get(), 4096);
    assert_eq!(inserted.len(), 4352);
    assert_eq!(inserted[0].value, 0);
    assert_eq!(inserted[1].value, 10000);
    assert_eq!(inserted[256].value, 10255);
    assert_eq!(inserted[257].value, 1);
    assert_eq!(checkpoint.len(), 4096);
    assert!(
        checkpoint
            .iter()
            .enumerate()
            .all(|(index, value)| value.value == index)
    );
}
impl Clone for Counted {
    fn clone(&self) -> Self {
        self.clones.set(self.clones.get() + 1);
        Self {
            value: self.value,
            clones: self.clones.clone(),
        }
    }
}

#[test]
fn sparse_checkpoint_writes_clone_only_touched_pages() {
    let clones = Rc::new(Cell::new(0));
    let original: ArrayValues<_> = (0..100_000)
        .map(|value| Counted {
            value,
            clones: clones.clone(),
        })
        .collect::<Vec<_>>()
        .into();
    let mut checkpoint = original.clone();
    assert_eq!(clones.get(), 0);
    checkpoint.get_mut(50_000).unwrap().value = 7;
    assert_eq!(clones.get(), PAGE_SIZE);
    checkpoint.get_mut(50_001).unwrap().value = 8;
    assert_eq!(clones.get(), PAGE_SIZE);
    checkpoint.get_mut(0).unwrap().value = 9;
    assert_eq!(clones.get(), PAGE_SIZE * 2);
    assert_eq!(original.get(50_000).unwrap().value, 50_000);
    assert_eq!(original.get(0).unwrap().value, 0);
    assert_eq!(checkpoint.get(50_000).unwrap().value, 7);
    assert_eq!(checkpoint.remove(99_999).value, 99_999);
    assert_eq!(clones.get(), PAGE_SIZE * 2 + 32);
    assert_eq!(original.len(), 100_000);
    assert_eq!(checkpoint.len(), 99_999);
}

#[test]
fn page_boundary_growth_shrink_middle_mutation_and_views_match_values() {
    for size in [0, 1, 127, 128, 129, 255, 256, 257, 1025] {
        let mut expected = (0..size).collect::<Vec<_>>();
        let mut values: ArrayValues<_> = expected.clone().into();
        let checkpoint = values.clone();
        for value in 0..260 {
            values.insert(values.len(), value);
            expected.push(value);
        }
        for _ in 0..130 {
            assert_eq!(values.remove(values.len() - 1), expected.pop().unwrap());
        }
        for index in [0, 1, 127, 128] {
            values.insert(index, 99_999);
            expected.insert(index, 99_999);
            assert_eq!(values.remove(index + 1), expected.remove(index + 1));
        }
        assert_eq!(values.to_vec(), expected);
        assert_eq!(checkpoint.to_vec(), (0..size).collect::<Vec<_>>());
        let view = values.view(127, 3);
        assert_eq!(view.iter().copied().collect::<Vec<_>>(), expected[127..130]);
        assert_eq!(
            view.iter().rev().copied().collect::<Vec<_>>(),
            expected[127..130].iter().rev().copied().collect::<Vec<_>>()
        );
        while !values.is_empty() {
            assert_eq!(values.remove(values.len() - 1), expected.pop().unwrap());
        }
        values.insert(0, 42);
        assert_eq!(values.to_vec(), [42]);
    }
}

#[test]
fn page_copy_preserves_nan_bits_and_owned_nested_value_independence() {
    let nan = f64::from_bits(0x7ff8_0000_0000_0042);
    let mut source = vec![PineValue::Float(nan); 257];
    source[127] = PineValue::String("source".into());
    source[128] = PineValue::ChartPoint(ChartPointValue::new(
        PineValue::Na,
        PineValue::Int(1),
        PineValue::Float(3.0),
    ));
    source[129] = PineValue::UserType(vec![PineValue::String("field".into())]);
    source[130] = PineValue::UserTypeRef(5);
    let original: ArrayValues = source.into();
    let mut copied = original.clone();
    let PineValue::String(value) = copied.get_mut(127).unwrap() else {
        panic!()
    };
    value.push_str(" changed");
    let PineValue::ChartPoint(value) = copied.get_mut(128).unwrap() else {
        panic!()
    };
    value.set_field(2, PineValue::Float(7.0));
    let PineValue::UserType(fields) = copied.get_mut(129).unwrap() else {
        panic!()
    };
    fields[0] = PineValue::String("changed".into());
    assert_eq!(original.get(127), Some(&PineValue::String("source".into())));
    let PineValue::ChartPoint(point) = original.get(128).unwrap() else {
        panic!()
    };
    assert_eq!(point.field(2), PineValue::Float(3.0));
    assert_eq!(
        original.get(129),
        Some(&PineValue::UserType(vec![PineValue::String(
            "field".into()
        )]))
    );
    assert_eq!(copied.get(130), Some(&PineValue::UserTypeRef(5)));
    for values in [&original, &copied] {
        let PineValue::Float(value) = values.get(126).unwrap() else {
            panic!()
        };
        assert_eq!(value.to_bits(), nan.to_bits());
    }
}

#[test]
fn paged_bulk_operations_preserve_checkpoints_and_cross_page_values() {
    let original: ArrayValues<_> = (0..513).collect::<Vec<_>>().into();
    let mut values = original.clone();
    let mut expected = (0..513).collect::<Vec<_>>();
    values.replace_range(127, 130, [900, 901]);
    expected.splice(127..130, [900, 901]);
    values.swap(127, 256);
    expected.swap(127, 256);
    values.swap(385, 129);
    expected.swap(385, 129);
    values.swap(128, 129);
    expected.swap(128, 129);
    values.reverse();
    expected.reverse();
    values.extend([800, 801]);
    expected.extend([800, 801]);
    assert_eq!(values.to_vec(), expected);
    values.fill(42);
    assert!(values.iter().all(|value| *value == 42));
    assert_eq!(original.to_vec(), (0..513).collect::<Vec<_>>());
    // Equality compares logical contents even when shrinking leaves pages.
    let mut paged = original.clone();
    while paged.len() > 128 {
        paged.remove(paged.len() - 1);
    }
    let flat: ArrayValues<_> = (0..128).collect::<Vec<_>>().into();
    assert_eq!(paged, flat);
}

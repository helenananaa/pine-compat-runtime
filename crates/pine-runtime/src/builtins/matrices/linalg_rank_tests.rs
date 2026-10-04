use std::cell::Cell;

use super::{super::scale, rank};

#[test]
fn rank_retries_only_non_finite_elimination_and_recovers_large_mirrored_rows() {
    for magnitude in [1e308, -1e308] {
        let original = [
            magnitude, magnitude, magnitude, -magnitude, magnitude, magnitude, 0.0, magnitude,
            -magnitude,
        ];
        let mut working = original;
        let retries = Cell::new(0);
        let actual = rank(&mut working, 3, 3, || {
            retries.set(retries.get() + 1);
            original.to_vec()
        });
        // Dividing each row by magnitude gives determinant -4 and full rank.
        assert_eq!(actual, 3);
        assert_eq!(retries.get(), 1);
    }
}

#[test]
fn rank_ordinary_mixed_diagonal_and_tiny_pivots_keep_the_original_path() {
    for (original, expected) in [
        ([1e308, 0.0, 0.0, 1e-308], 2),
        ([1e-308, 1e308, 0.0, 1.0], 2),
        ([f64::from_bits(1), 0.0, 0.0, f64::from_bits(2)], 2),
        ([-0.0, 0.0, 0.0, -0.0], 0),
        ([1.0, 2.0, 2.0, 4.0], 1),
    ] {
        let mut values = original;
        assert_eq!(
            rank(&mut values, 2, 2, || panic!(
                "ordinary rank requested backup"
            )),
            expected
        );
    }
    assert_eq!(
        rank(&mut [], 0, 0, || panic!("empty rank requested backup")),
        0
    );
}

#[test]
fn rank_row_scaling_preserves_every_nonzero_and_rejects_irreversible_rows() {
    let original = [1e308, -0.0, 0.0, 1e-308];
    let mut reversible = original;
    assert!(scale::normalize_rows_reversibly(&mut reversible, 2, 2));
    assert!(reversible[0] > 0.0 && reversible[3] > 0.0);
    assert_eq!(reversible[1].to_bits(), original[1].to_bits());
    let mut irreversible = [1e308, 1e-308];
    assert!(!scale::normalize_rows_reversibly(&mut irreversible, 1, 2));

    let original = [
        1e308, 1e-308, 1e308, -1e308, 1e308, 1e308, 0.0, 1e308, -1e308,
    ];
    let mut legacy = original;
    let expected = super::eliminate(&mut legacy, 3, 3);
    assert!(legacy.iter().any(|value| !value.is_finite()));
    let mut values = original;
    assert_eq!(rank(&mut values, 3, 3, || original.to_vec()), expected);
}

use super::{eigenvalues, eigenvectors, scale};

#[test]
fn exact_binary_scale_handles_subnormals_and_guards_irreversible_mixed_inputs() {
    for magnitude in [
        f64::from_bits(1),
        f64::from_bits(2),
        f64::from_bits((1 << 52) - 1),
        f64::MIN_POSITIVE,
        1e-100,
        1e200,
        f64::MAX,
    ] {
        let factor = scale::binary_scale(magnitude);
        assert!(factor.is_finite() && factor > 0.0 && factor <= magnitude);
        let ratio = magnitude / factor;
        assert!((1.0..2.0).contains(&ratio));
        assert_eq!((ratio * factor).to_bits(), magnitude.to_bits());
    }
    assert!(scale::extreme_eigen_normalization(&[1e308, 1e-308]).is_none());
    assert!(scale::extreme_eigen_normalization(&[1.0, 2.0, 3.0]).is_none());
    assert!(scale::extreme_eigen_normalization(&[-0.0, 0.0]).is_none());
}

#[test]
fn extreme_rank_one_eigenvalues_and_vectors_satisfy_scale_invariant_equations() {
    for magnitude in [f64::from_bits(1), 1e-200, 1e-100, 1e200] {
        let values = [magnitude; 4];
        let eigenvalues = eigenvalues(&values, 2).unwrap();
        assert!((eigenvalues[0] / magnitude).abs() <= 1e-14);
        assert!((eigenvalues[1] / magnitude - 2.0).abs() <= 1e-14);
        let vectors = eigenvectors(&values, 2).unwrap();
        for column in 0..2 {
            let norm_squared: f64 = (0..2).map(|row| vectors[row * 2 + column].powi(2)).sum();
            assert!((norm_squared - 1.0).abs() <= 1e-14);
            for row in 0..2 {
                let projected: f64 = (0..2)
                    .map(|inner| {
                        (values[row * 2 + inner] / magnitude) * vectors[inner * 2 + column]
                    })
                    .sum();
                let expected = (eigenvalues[column] / magnitude) * vectors[row * 2 + column];
                assert!((projected - expected).abs() <= 1e-14);
            }
        }
    }
}

#[test]
fn extreme_non_symmetric_triangular_eigenvalues_preserve_the_analytic_diagonal() {
    let magnitude = 1e200;
    let values = [2.0 * magnitude, magnitude, 0.0, magnitude];
    let eigenvalues = eigenvalues(&values, 2).unwrap();
    assert!((eigenvalues[0] / magnitude - 2.0).abs() <= 1e-14);
    assert!((eigenvalues[1] / magnitude - 1.0).abs() <= 1e-14);
    let vectors = eigenvectors(&values, 2).unwrap();
    for column in 0..2 {
        for row in 0..2 {
            let projected: f64 = (0..2)
                .map(|inner| (values[row * 2 + inner] / magnitude) * vectors[inner * 2 + column])
                .sum();
            let expected = (eigenvalues[column] / magnitude) * vectors[row * 2 + column];
            assert!((projected - expected).abs() <= 1e-14);
        }
    }
}

#[test]
fn eigen_normalization_keeps_mixed_diagonals_singletons_and_signed_zeros() {
    for magnitude in [1e-100, 1e200, f64::MAX] {
        let original = [magnitude];
        assert_eq!(
            eigenvalues(&original, 1).unwrap()[0].to_bits(),
            magnitude.to_bits()
        );
        assert_eq!(eigenvectors(&original, 1).unwrap(), [1.0]);
    }
    let mixed = [1e308, 0.0, 0.0, 1e-308];
    let result = eigenvalues(&mixed, 2).unwrap();
    assert_eq!(result[0].to_bits(), mixed[0].to_bits());
    assert_eq!(result[1].to_bits(), mixed[3].to_bits());
    assert_eq!(eigenvectors(&mixed, 2).unwrap(), [1.0, 0.0, 0.0, 1.0]);
    let zero = [-0.0, 0.0, 0.0, -0.0];
    assert!(
        eigenvalues(&zero, 2)
            .unwrap()
            .iter()
            .all(|value| value.to_bits() == (-0.0_f64).to_bits())
    );
    assert_eq!(eigenvectors(&zero, 2).unwrap(), [1.0, 0.0, 0.0, 1.0]);
    assert!(eigenvalues(&[], 0).unwrap().is_empty());
}

use super::{EIGEN_TOLERANCE, eigenvalues, eigenvectors, normalize_vector_columns, qr_decompose};

fn multiply(left: &[f64], right: &[f64], size: usize) -> Vec<f64> {
    (0..size)
        .flat_map(|row| {
            (0..size).map(move |column| {
                (0..size)
                    .map(|inner| left[row * size + inner] * right[inner * size + column])
                    .sum()
            })
        })
        .collect()
}

fn assert_factorization(values: &[f64], size: usize, tolerance: f64) -> Vec<f64> {
    let (q, r) = qr_decompose(values, size).unwrap();
    let reconstructed = multiply(&q, &r, size);
    for (actual, expected) in reconstructed.iter().zip(values) {
        assert!(
            (actual - expected).abs() <= tolerance,
            "QR reconstruction: {actual} != {expected}"
        );
    }
    for left in 0..size {
        for right in 0..size {
            let product: f64 = (0..size)
                .map(|row| q[row * size + left] * q[row * size + right])
                .sum();
            let expected = if left == right { 1.0 } else { 0.0 };
            assert!(
                (product - expected).abs() <= 1e-14,
                "Q transpose Q ({left}, {right}) = {product}"
            );
        }
    }
    r
}

#[test]
fn qr_fallback_completes_zero_columns_without_changing_the_factorization() {
    for values in [
        [1.0, 2.0, 3.0, 0.0, 0.0, 4.0, 0.0, 0.0, 5.0],
        [0.0; 9],
        [0.0, 2.0, 3.0, 0.0, 0.0, 4.0, 0.0, 0.0, 5.0],
    ] {
        let r = assert_factorization(&values, 3, 0.0);
        assert_eq!(r[4], 0.0);
        if values[0] == 0.0 {
            assert_eq!(r[0], 0.0);
        }
    }
}

#[test]
fn qr_dependent_columns_keep_reconstruction_and_an_orthogonal_completed_basis() {
    let values = [1.0, 2.0, 0.0, 2.0, 4.0, 1.0, 3.0, 6.0, 2.0];
    let r = assert_factorization(&values, 3, 1e-14);
    assert_eq!(r[4], 0.0);
}

#[test]
fn qr_existing_near_zero_tolerance_discards_only_the_original_small_residual() {
    for residual in [0.0, -0.0, 0.5 * EIGEN_TOLERANCE, -0.5 * EIGEN_TOLERANCE] {
        let values = [1.0, 2.0, 3.0, 0.0, residual, 4.0, 0.0, 0.0, 5.0];
        let r = assert_factorization(&values, 3, residual.abs());
        assert_eq!(r[4], 0.0);
        assert_eq!(eigenvalues(&values, 3).unwrap(), [1.0, 0.0, 5.0]);
    }
    for residual in [2.0 * EIGEN_TOLERANCE, -2.0 * EIGEN_TOLERANCE] {
        let values = [1.0, 2.0, 3.0, 0.0, residual, 4.0, 0.0, 0.0, 5.0];
        let r = assert_factorization(&values, 3, 0.0);
        assert_eq!(r[4], residual.abs());
        assert_eq!(eigenvalues(&values, 3).unwrap(), [1.0, residual, 5.0]);
    }
}

#[test]
fn singular_upper_triangular_eigenvalues_match_its_analytic_diagonal() {
    let values = [1.0, 2.0, 3.0, 0.0, 0.0, 4.0, 0.0, 0.0, 5.0];
    let eigenvalues = eigenvalues(&values, 3).unwrap();
    // Upper triangular characteristic polynomial is (1-lambda)*(-lambda)*(5-lambda).
    assert_eq!(eigenvalues, [1.0, 0.0, 5.0]);
    assert_eq!(eigenvalues.iter().sum::<f64>(), 6.0);
    assert_eq!(eigenvalues.iter().product::<f64>(), 0.0);
    let vectors = eigenvectors(&values, 3).unwrap();
    for column in 0..3 {
        for row in 0..3 {
            let actual: f64 = (0..3)
                .map(|inner| values[row * 3 + inner] * vectors[inner * 3 + column])
                .sum();
            let expected = eigenvalues[column] * vectors[row * 3 + column];
            assert!(
                (actual - expected).abs() <= 1e-14,
                "A v = lambda v ({row}, {column}): {actual} != {expected}"
            );
        }
    }
}

#[test]
fn column_normalization_workspace_keeps_signs_and_preserves_ineligible_columns() {
    let columns = vec![3.0, 0.0, 1e-10, 4.0, -2.0, 0.0, 0.0, 0.0, 0.0];
    let normalized = normalize_vector_columns(columns, 3);
    let expected: [f64; 9] = [0.6, -0.0, 1e-10, 0.8, 1.0, 0.0, 0.0, -0.0, 0.0];
    assert_eq!(
        normalized
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        expected
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
    );
    assert!(normalize_vector_columns(Vec::new(), 0).is_empty());
}

#[test]
fn overflowing_qr_column_norm_recovers_repeated_triangular_eigenvalues() {
    let magnitude = 1e154;
    let values = [
        magnitude, 0.0, 0.0, magnitude, magnitude, 0.0, magnitude, 0.0, magnitude,
    ];
    assert!(super::scale::extreme_eigen_normalization(&values).is_none());
    assert!(qr_decompose(&values, 3).is_none());
    let roots = eigenvalues(&values, 3).unwrap();
    for root in roots {
        assert_eq!(root.to_bits(), magnitude.to_bits());
    }
}

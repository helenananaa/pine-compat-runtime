use std::cmp::Ordering;

/// Reusable workspace, with no Pine-visible state. Runtime checkpoints start
/// with an empty workspace rather than copying a potentially large selection.
#[derive(Debug, Default)]
pub(crate) struct SelectionScratch {
    values: Vec<(f64, usize)>,
}

impl Clone for SelectionScratch {
    fn clone(&self) -> Self {
        Self::default()
    }
}

fn compare_samples(left: &(f64, usize), right: &(f64, usize)) -> Ordering {
    // Samples are finite. Preserve the former stable sort's tie order, including
    // signed zeros, even though selection itself is unstable.
    left.0
        .partial_cmp(&right.0)
        .unwrap_or(Ordering::Equal)
        .then_with(|| left.1.cmp(&right.1))
}

impl SelectionScratch {
    pub(crate) fn capacity(&self) -> usize {
        self.values.capacity()
    }

    pub(crate) fn select_pair(
        &mut self,
        samples: impl Iterator<Item = f64>,
        lower: usize,
        upper: usize,
    ) -> (f64, f64) {
        self.values.clear();
        self.values
            .extend(samples.enumerate().map(|(index, value)| (value, index)));
        debug_assert!(lower <= upper && upper < self.values.len());
        // Median and interpolated percentile need one rank or adjacent ranks.
        debug_assert!(upper - lower <= 1);
        let (left, selected, _) = self.values.select_nth_unstable_by(upper, compare_samples);
        let high = selected.0;
        let low = if lower == upper {
            high
        } else {
            left.iter().max_by(|a, b| compare_samples(a, b)).unwrap().0
        };
        self.values.clear();
        (low, high)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_matches_stable_sort_bits_for_every_adjacent_rank() {
        let mut scratch = SelectionScratch::default();
        for length in [1_usize, 2, 7, 16, 129, 1024] {
            let values: Vec<_> = (0..length)
                .map(|index| match index % 10 {
                    0 => 0.0,
                    1 => -0.0,
                    2 => 1e300,
                    3 => -1e300,
                    4 => f64::MAX,
                    5 => f64::MIN,
                    6 => f64::from_bits(1),
                    _ => (index * 137 % 31) as f64,
                })
                .collect();
            let mut sorted = values.clone();
            sorted.sort_by(|left, right| left.partial_cmp(right).unwrap());
            for upper in 0..length {
                for lower in [upper.saturating_sub(1), upper] {
                    let (low, high) = scratch.select_pair(values.iter().copied(), lower, upper);
                    assert_eq!(low.to_bits(), sorted[lower].to_bits());
                    assert_eq!(high.to_bits(), sorted[upper].to_bits());
                }
            }
        }
        assert!(scratch.capacity() >= 1024);
        assert_eq!(scratch.clone().capacity(), 0);
        let capacity = scratch.capacity();
        assert_eq!(
            scratch.select_pair([3.0, 1.0, 2.0].into_iter(), 1, 1),
            (2.0, 2.0)
        );
        assert_eq!(scratch.capacity(), capacity);
    }
}

use std::collections::VecDeque;

use pine_ir::CallSiteId;

#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct RollingWindowState {
    pub(crate) values: VecDeque<Option<f64>>,
    pub(crate) sum: f64,
    pub(crate) sum_squares: f64,
    pub(crate) na_count: usize,
    /// Bar that owns the uncommitted tail sample, if any.
    open_bar: Option<usize>,
    /// Exact aggregates from before the open append. Restored by assignment on
    /// same-bar undo so subtract/add cannot accumulate roundoff.
    prev_sum: f64,
    prev_sum_squares: f64,
    prev_na_count: usize,
    /// Items evicted by the open append only. Kept at this level (not inside
    /// `Option`) so the allocation is reused across bars.
    evicted: Vec<Option<f64>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum RollingWindowKey {
    Single(CallSiteId),
    VwmaWeighted(CallSiteId),
    VwmaVolume(CallSiteId),
    MfiPositive(CallSiteId),
    MfiNegative(CallSiteId),
    CmoPositive(CallSiteId),
    CmoNegative(CallSiteId),
    AoFast(CallSiteId),
    AoSlow(CallSiteId),
    CorrelationLeft(CallSiteId),
    CorrelationRight(CallSiteId),
    CorrelationProduct(CallSiteId),
    CovarianceLeft(CallSiteId),
    CovarianceRight(CallSiteId),
    CovarianceProduct(CallSiteId),
    StochHigh(CallSiteId),
    StochLow(CallSiteId),
    WprHigh(CallSiteId),
    WprLow(CallSiteId),
    HmaHalf(CallSiteId),
    HmaFull(CallSiteId),
    HmaSmooth(CallSiteId),
    Rma { call_site: CallSiteId, channel: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WindowExtreme {
    Highest,
    Lowest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RisingFallingMode {
    Rising,
    Falling,
}

impl RollingWindowState {
    pub(crate) fn retained_values(&self) -> usize {
        self.values.len() + self.evicted.len()
    }

    pub(crate) fn retained_capacity(&self) -> usize {
        self.values.capacity() + self.evicted.capacity()
    }

    pub(crate) fn push(&mut self, value: Option<f64>, length: usize) {
        while self.values.len() >= length {
            self.pop_front();
        }
        self.append(value);
    }

    /// Append `value` as the sample for `bar`.
    ///
    /// A second call with the same `bar` undoes the open append (restoring the
    /// exact pre-append aggregates and only the items that append evicted) and
    /// then applies `value`. A different `bar` commits the previous append.
    /// `None` is an ordinary NA sample, not a discard.
    pub(crate) fn push_for_bar(&mut self, value: Option<f64>, length: usize, bar: usize) {
        if self.open_bar == Some(bar) {
            self.undo_open_append();
        }
        self.begin_open_append(bar);
        while self.values.len() >= length {
            let Some(evicted) = self.values.front().copied() else {
                break;
            };
            self.pop_front();
            self.evicted.push(evicted);
        }
        self.append(value);
    }

    /// Undo the open append when it belongs to `bar`, leaving no sample for
    /// that bar. A later `push_for_bar` on the same bar starts a new append.
    pub(crate) fn discard_for_bar(&mut self, bar: usize) {
        if self.open_bar == Some(bar) {
            self.undo_open_append();
            self.open_bar = None;
        }
    }

    pub(crate) fn pop_front(&mut self) {
        if let Some(value) = self.values.pop_front() {
            if let Some(value) = value {
                self.sum -= value;
                self.sum_squares -= value * value;
            } else {
                self.na_count = self.na_count.saturating_sub(1);
            }
        }
    }

    pub(crate) fn is_ready(&self, length: usize) -> bool {
        self.values.len() == length && self.na_count == 0
    }

    pub(crate) fn mean(&self, length: usize) -> f64 {
        self.sum / length as f64
    }

    pub(crate) fn variance(&self, length: usize, biased: bool) -> f64 {
        if !biased && length < 2 {
            return f64::NAN;
        }
        let mean = self.mean(length);
        let squared_diff_sum = self
            .values
            .iter()
            .flatten()
            .map(|value| {
                let diff = *value - mean;
                diff * diff
            })
            .sum::<f64>();
        let denominator = if biased { length } else { length - 1 };
        (squared_diff_sum / denominator as f64).max(0.0)
    }

    pub(crate) fn extreme(&self, mode: WindowExtreme) -> Option<f64> {
        self.values
            .iter()
            .flatten()
            .copied()
            .reduce(|current, value| match mode {
                WindowExtreme::Highest => current.max(value),
                WindowExtreme::Lowest => current.min(value),
            })
    }

    pub(crate) fn range(&self) -> Option<f64> {
        let highest = self.extreme(WindowExtreme::Highest)?;
        let lowest = self.extreme(WindowExtreme::Lowest)?;
        Some(highest - lowest)
    }

    pub(crate) fn mean_absolute_deviation(&self, length: usize) -> f64 {
        let mean = self.mean(length);
        self.values
            .iter()
            .flatten()
            .map(|value| (*value - mean).abs())
            .sum::<f64>()
            / length as f64
    }

    pub(crate) fn center_of_gravity(&self, length: usize) -> f64 {
        let numerator = self
            .values
            .iter()
            .flatten()
            .enumerate()
            .map(|(index, value)| *value * (length - index) as f64)
            .sum::<f64>();
        -numerator / self.sum
    }

    pub(crate) fn weighted_mean(&self, length: usize) -> f64 {
        let weighted_sum = self
            .values
            .iter()
            .flatten()
            .enumerate()
            .map(|(index, value)| *value * (index + 1) as f64)
            .sum::<f64>();
        let denominator = length * (length + 1) / 2;
        weighted_sum / denominator as f64
    }

    fn append(&mut self, value: Option<f64>) {
        if let Some(value) = value {
            self.sum += value;
            self.sum_squares += value * value;
            self.values.push_back(Some(value));
        } else {
            self.na_count += 1;
            self.values.push_back(None);
        }
    }

    fn begin_open_append(&mut self, bar: usize) {
        self.open_bar = Some(bar);
        self.prev_sum = self.sum;
        self.prev_sum_squares = self.sum_squares;
        self.prev_na_count = self.na_count;
        self.evicted.clear();
    }

    fn undo_open_append(&mut self) {
        self.values.pop_back();
        self.sum = self.prev_sum;
        self.sum_squares = self.prev_sum_squares;
        self.na_count = self.prev_na_count;
        while let Some(value) = self.evicted.pop() {
            self.values.push_front(value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bits(value: f64) -> u64 {
        value.to_bits()
    }

    #[test]
    fn push_still_evicts_and_updates_aggregates() {
        let mut window = RollingWindowState::default();
        window.push(Some(1.0), 2);
        window.push(Some(2.0), 2);
        window.push(Some(3.0), 2);
        assert_eq!(window.values, VecDeque::from(vec![Some(2.0), Some(3.0)]));
        assert_eq!(window.sum, 5.0);
        assert_eq!(window.na_count, 0);
        window.push(None, 2);
        assert_eq!(window.values, VecDeque::from(vec![Some(3.0), None]));
        assert_eq!(window.na_count, 1);
        window.pop_front();
        assert_eq!(window.values, VecDeque::from(vec![None]));
        assert_eq!(window.na_count, 1);
    }

    #[test]
    fn same_bar_replacement_keeps_only_latest_sample() {
        let mut window = RollingWindowState::default();
        window.push_for_bar(Some(1.0), 3, 0);
        window.push_for_bar(Some(2.0), 3, 0);
        window.push_for_bar(Some(3.0), 3, 0);
        assert_eq!(window.values, VecDeque::from(vec![Some(3.0)]));
        assert_eq!(window.sum, 3.0);
        assert_eq!(window.na_count, 0);
    }

    #[test]
    fn new_bar_commits_previous_append() {
        let mut window = RollingWindowState::default();
        window.push_for_bar(Some(1.0), 3, 0);
        window.push_for_bar(Some(2.0), 3, 1);
        window.push_for_bar(Some(3.0), 3, 1);
        assert_eq!(window.values, VecDeque::from(vec![Some(1.0), Some(3.0)]));
        assert_eq!(window.sum, 4.0);
        window.discard_for_bar(0);
        assert_eq!(window.values, VecDeque::from(vec![Some(1.0), Some(3.0)]));
    }

    #[test]
    fn same_bar_replacement_restores_exact_sums_for_cancellation_prone_floats() {
        let mut window = RollingWindowState::default();
        window.push(Some(1.0), 2);
        let pre_sum = bits(window.sum);
        let pre_squares = bits(window.sum_squares);

        window.push_for_bar(Some(1e16), 2, 7);
        assert_eq!(window.sum, 1e16);
        assert_ne!(bits(window.sum), pre_sum);

        window.push_for_bar(Some(2.0), 2, 7);
        assert_eq!(window.values, VecDeque::from(vec![Some(1.0), Some(2.0)]));
        assert_eq!(bits(window.sum), bits(3.0));
        assert_eq!(bits(window.sum_squares), bits(5.0));

        let mut discarded = RollingWindowState::default();
        discarded.push(Some(1.0), 2);
        discarded.push_for_bar(Some(1e16), 2, 7);
        discarded.discard_for_bar(7);
        assert_eq!(discarded.values, VecDeque::from(vec![Some(1.0)]));
        assert_eq!(bits(discarded.sum), pre_sum);
        assert_eq!(bits(discarded.sum_squares), pre_squares);
        assert_ne!(bits(discarded.sum), bits(0.0));
    }

    #[test]
    fn same_bar_length_shrink_then_grow_restores_pre_bar_tail() {
        let mut window = RollingWindowState::default();
        for value in [1.0, 2.0, 3.0, 4.0, 5.0] {
            window.push(Some(value), 5);
        }
        let pre_bar = window.clone();

        window.push_for_bar(Some(99.0), 3, 4);
        assert_eq!(
            window.values,
            VecDeque::from(vec![Some(4.0), Some(5.0), Some(99.0)])
        );

        window.push_for_bar(Some(100.0), 5, 4);
        let mut expected = pre_bar.clone();
        expected.push_for_bar(Some(100.0), 5, 4);
        assert_eq!(window.values, expected.values);
        assert_eq!(
            window.values,
            VecDeque::from(vec![
                Some(2.0),
                Some(3.0),
                Some(4.0),
                Some(5.0),
                Some(100.0)
            ])
        );
        assert_eq!(bits(window.sum), bits(expected.sum));
        assert_eq!(bits(window.sum_squares), bits(expected.sum_squares));
        assert_eq!(window, expected);

        let mut shrunk = pre_bar.clone();
        shrunk.push_for_bar(Some(99.0), 3, 4);
        shrunk.discard_for_bar(4);
        assert_eq!(shrunk.values, pre_bar.values);
        assert_eq!(bits(shrunk.sum), bits(pre_bar.sum));
        assert_eq!(bits(shrunk.sum_squares), bits(pre_bar.sum_squares));
        assert_eq!(shrunk.na_count, pre_bar.na_count);
    }

    #[test]
    fn discard_then_repush_on_same_bar() {
        let mut window = RollingWindowState::default();
        window.push_for_bar(Some(1.0), 3, 0);
        window.push_for_bar(Some(2.0), 3, 1);
        window.discard_for_bar(1);
        assert_eq!(window.values, VecDeque::from(vec![Some(1.0)]));
        assert_eq!(window.sum, 1.0);
        assert_eq!(window.na_count, 0);

        window.push_for_bar(Some(3.0), 3, 1);
        assert_eq!(window.values, VecDeque::from(vec![Some(1.0), Some(3.0)]));
        assert_eq!(window.sum, 4.0);

        window.discard_for_bar(9);
        assert_eq!(window.values, VecDeque::from(vec![Some(1.0), Some(3.0)]));
    }

    #[test]
    fn cloned_checkpoint_restores_open_append_undo() {
        let mut live = RollingWindowState::default();
        live.push_for_bar(Some(1.0), 2, 0);
        live.push_for_bar(Some(2.0), 2, 1);
        let checkpoint = live.clone();
        assert_eq!(checkpoint, live);

        live.push_for_bar(Some(3.0), 2, 1);
        assert_eq!(live.values, VecDeque::from(vec![Some(1.0), Some(3.0)]));
        assert_ne!(live, checkpoint);

        let mut restored = checkpoint.clone();
        assert_eq!(restored, checkpoint);
        restored.push_for_bar(Some(4.0), 2, 1);
        assert_eq!(restored.values, VecDeque::from(vec![Some(1.0), Some(4.0)]));
        assert_eq!(bits(restored.sum), bits(5.0));
    }

    #[test]
    fn none_is_ordinary_push_not_discard() {
        let mut window = RollingWindowState::default();
        window.push_for_bar(Some(1.0), 3, 0);
        window.push_for_bar(None, 3, 1);
        assert_eq!(window.values, VecDeque::from(vec![Some(1.0), None]));
        assert_eq!(window.na_count, 1);
        window.push_for_bar(Some(2.0), 3, 1);
        assert_eq!(window.values, VecDeque::from(vec![Some(1.0), Some(2.0)]));
        assert_eq!(window.na_count, 0);
    }

    #[test]
    fn partial_eq_and_clone_include_undo_metadata() {
        let mut via_push = RollingWindowState::default();
        via_push.push(Some(1.0), 3);
        let mut via_bar = RollingWindowState::default();
        via_bar.push_for_bar(Some(1.0), 3, 0);
        assert_eq!(via_bar.values, via_push.values);
        assert_eq!(bits(via_bar.sum), bits(via_push.sum));
        assert_ne!(via_bar, via_push);
        assert_eq!(via_bar.clone(), via_bar);
        assert_eq!(RollingWindowState::default(), RollingWindowState::default());
    }

    #[test]
    fn every_bar_final_discard_leaves_no_committed_sample() {
        let mut discarded = RollingWindowState::default();
        for bar in 0..3 {
            discarded.push_for_bar(Some(1.0), 3, bar);
            discarded.push_for_bar(Some(2.0), 3, bar);
            discarded.discard_for_bar(bar);
        }
        assert!(discarded.values.is_empty());
        assert_eq!(discarded.sum, 0.0);
        assert_eq!(discarded.na_count, 0);

        let mut committed = RollingWindowState::default();
        committed.push_for_bar(Some(1.0), 3, 0);
        committed.push_for_bar(Some(2.0), 3, 0);
        committed.push_for_bar(Some(3.0), 3, 0);
        committed.push_for_bar(Some(4.0), 3, 1);
        assert_eq!(committed.values, VecDeque::from(vec![Some(3.0), Some(4.0)]));
    }
}

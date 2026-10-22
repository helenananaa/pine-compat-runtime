//! Known requested endpoints never supply future prices to an active evaluator.
use super::*;
use crate::external::external_error;
use crate::{ExternalRequestWindowHistory, RequestKey, RequestTimeframe};

impl HistoricalRuntime<'_> {
    /// Initialize fixed chart and requested window endpoints without executing
    /// Pine or populating requested providers. Only ordered bar times are used.
    pub fn initialize_external_stream_dataset_with_requested_history(
        &mut self,
        bars: &[Bar],
        requested: &[ExternalRequestWindowHistory],
    ) -> Result<usize, RuntimeError> {
        self.ensure_execution_ready()?;
        let histories = self.parse_requested_window_history(bars, requested)?;
        let mut candidate = self.clone();
        candidate.request_window_history = histories;
        let skip = candidate.initialize_external_stream_dataset(bars)?;
        *self = candidate;
        Ok(skip)
    }

    pub(super) fn parse_requested_window_history(
        &self,
        bars: &[Bar],
        requested: &[ExternalRequestWindowHistory],
    ) -> Result<KnownRequestedTimes, RuntimeError> {
        let mut histories = HashMap::new();
        let chart = self.request_environment.chart();
        for history in requested {
            if history.symbol.is_empty()
                || history.symbol.trim() != history.symbol
                || history.timeframe.is_empty()
                || history.timeframe.trim() != history.timeframe
                || history.times_ms.windows(2).any(|pair| pair[1] <= pair[0])
            {
                return Err(external_error("invalid requested window history"));
            }
            let timeframe = RequestTimeframe::parse(&history.timeframe)
                .map_err(|error| external_error(error.to_string()))?;
            let key = RequestKey::new(&history.symbol, timeframe);
            if key.symbol() == chart.symbol()
                && key.timeframe() == chart.timeframe()
                && !history
                    .times_ms
                    .iter()
                    .copied()
                    .eq(bars.iter().map(|bar| bar.time))
            {
                return Err(external_error(
                    "requested root window differs from chart history",
                ));
            }
            if histories
                .insert(key, Arc::<[i64]>::from(history.times_ms.clone()))
                .is_some()
            {
                return Err(external_error("duplicate requested window history"));
            }
        }
        Ok(Arc::new(histories))
    }
}

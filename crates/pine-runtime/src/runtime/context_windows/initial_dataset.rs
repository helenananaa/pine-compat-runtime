//! Shared dataset setup for public iterators and host-managed cursors.
use crate::{Bar, HistoricalRuntime, RuntimeError};
use pine_ir::ScriptMode;
impl HistoricalRuntime<'_> {
    /// Initialize a complete historical dataset without executing visible Pine
    /// bars. Returns the first retained index in the supplied dataset. Append
    /// bars starting at that index, in order, to execute a host-managed cursor.
    /// The optional execution clocks correspond to the complete input dataset.
    pub fn initialize_historical_dataset(
        &mut self,
        bars: &[Bar],
        execution_times: Option<&[i64]>,
    ) -> Result<usize, RuntimeError> {
        if self.bars != 0 || self.historical_end.is_some() || bars.is_empty() {
            return Err(RuntimeError { message: "historical dataset initialization requires an empty runtime and a nonempty dataset".to_owned() });
        }
        if execution_times.is_some_and(|times| times.len() != bars.len()) {
            return Err(RuntimeError {
                message: "execution timestamp count does not match historical dataset".to_owned(),
            });
        }
        self.prepare_historical_dataset(bars, execution_times)
            .map(|(skip, _)| skip)
    }

    pub(crate) fn prepare_historical_dataset(
        &mut self,
        bars: &[Bar],
        execution_times: Option<&[i64]>,
    ) -> Result<(usize, Option<usize>), RuntimeError> {
        self.ensure_execution_ready()?;
        // A batch supplies the complete initial dataset. Restrict its first
        // execution window before any series, bar indices or strategy state
        // are created. Later incremental appends extend that window normally.
        let skip = self.prepare_context_windows(bars, execution_times)?;
        self.finish_historical_dataset_plan(bars, skip)
    }

    pub(crate) fn finish_historical_dataset_plan(
        &mut self,
        bars: &[Bar],
        skip: usize,
    ) -> Result<(usize, Option<usize>), RuntimeError> {
        let bars = &bars[skip..];
        if self.program.script_mode == ScriptMode::Strategy {
            self.session_windows
                .validate_range(self.bars, self.bars + bars.len())
                .map_err(crate::SessionWindowInputError::runtime_error)?;
        }
        if self.bars == 0 && self.magnifier_chart_bar_count.is_none() {
            self.prepare_magnifier_chart_bar_count(bars.len())?;
        }
        self.prepare_external_window(skip)?;
        if self.static_requests.extend_dataset(self.bars, bars) {
            self.request_cache.clear();
            self.request_evaluations.clear();
        }
        let previous_historical_end = self.historical_end;
        self.historical_end = Some(self.bars + bars.len());
        if let Some(first) = bars.first() {
            self.chart_visible_left_time.get_or_insert(first.time);
        }
        if let Some((offset, last)) = bars
            .len()
            .checked_sub(1)
            .map(|offset| (offset, &bars[offset]))
        {
            self.last_bar_index = Some(self.bars + offset);
            self.last_bar_time = Some(last.time);
            self.chart_visible_right_time = Some(last.time);
        }
        Ok((skip, previous_historical_end))
    }
}

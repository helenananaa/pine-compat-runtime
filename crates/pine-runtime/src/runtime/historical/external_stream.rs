//! One external observation at a time, with one final history commit per bar.
use super::*;
use crate::external::{ExternalOpenBar, external_error};
use crate::{ExternalBarState, ExternalScheduledPass};

impl HistoricalRuntime<'_> {
    /// Plan a known historical dataset without inventing future account frames
    /// or executing Pine. The returned input skip applies before the first bar.
    pub fn initialize_external_stream_dataset(
        &mut self,
        bars: &[Bar],
    ) -> Result<usize, RuntimeError> {
        self.validate_external_stream_dataset(bars)?;
        let mut candidate = self.clone();
        let external = candidate.external_execution.take();
        let skip = candidate.prepare_context_windows_without_discovery(bars, None)?;
        candidate.finish_historical_dataset_plan(bars, skip)?;
        candidate.external_execution = external;
        *self = candidate;
        Ok(skip)
    }

    pub(super) fn validate_external_stream_dataset(
        &self,
        bars: &[Bar],
    ) -> Result<(), RuntimeError> {
        self.ensure_execution_ready()?;
        let state = self
            .external_execution
            .as_ref()
            .ok_or_else(|| external_error("external mode is not active"))?;
        if self.bars != 0
            || self.historical_end.is_some()
            || state.stream.is_some()
            || !state.frames.is_empty()
            || state
                .passes
                .as_ref()
                .is_none_or(|groups| !groups.is_empty())
            || state
                .contexts
                .as_ref()
                .is_none_or(|groups| !groups.is_empty())
            || bars.is_empty()
            || bars.iter().any(|bar| !valid_bar(*bar))
            || bars.windows(2).any(|pair| pair[1].time <= pair[0].time)
        {
            return Err(external_error(
                "external dataset planning requires a fresh scheduled runtime and ordered canonical bars",
            ));
        }
        Ok(())
    }

    /// Open one chart bar. Historical callers supply its canonical OHLCV;
    /// realtime callers supply the currently observed tuple. No Pine executes.
    pub fn begin_external_bar(
        &mut self,
        canonical: Bar,
        bar_state: ExternalBarState,
    ) -> Result<(), RuntimeError> {
        self.ensure_execution_ready()?;
        self.validate_external_window_bar(canonical, bar_state)?;
        let state = self
            .external_execution
            .as_ref()
            .ok_or_else(|| external_error("external mode is not active"))?;
        if state.stream.is_some()
            || state.active_pass.is_some()
            || state.frames.len() != self.bars
            || state
                .passes
                .as_ref()
                .is_none_or(|groups| groups.len() != self.bars)
            || state
                .contexts
                .as_ref()
                .is_none_or(|groups| groups.len() != self.bars)
        {
            return Err(external_error("a fresh explicit-schedule bar is required"));
        }
        if !valid_bar(canonical)
            || self
                .previous_bar_time
                .is_some_and(|time| canonical.time <= time)
            || (bar_state == ExternalBarState::Historical
                && state
                    .contexts
                    .as_ref()
                    .and_then(|groups| groups.last())
                    .and_then(|group| group.first())
                    .is_some_and(|context| context.bar_state == ExternalBarState::Realtime))
        {
            return Err(external_error(
                "invalid external stream bar or historical transition",
            ));
        }
        self.external_execution.as_mut().unwrap().stream = Some(ExternalOpenBar {
            canonical,
            bar_state,
            started: false,
        });
        Ok(())
    }

    /// Preflight one observation before mutation. Input rejections are
    /// retryable; failures while evaluating Pine poison the runtime.
    pub fn observe_external_bar(
        &mut self,
        pass: ExternalScheduledPass,
    ) -> Result<(), RuntimeError> {
        self.ensure_execution_ready()?;
        let state = self
            .external_execution
            .as_ref()
            .ok_or_else(|| external_error("external mode is not active"))?;
        let open = state
            .stream
            .ok_or_else(|| external_error("no external bar is open"))?;
        let options = state.options;
        let count = if open.started {
            state.passes.as_ref().unwrap()[self.bars].len()
        } else {
            0
        };
        let limit = if self.program.strategy_settings.calc_on_every_tick
            || self.program.strategy_settings.calc_on_every_history_tick
        {
            crate::MAX_STRATEGY_OBSERVATIONS_PER_BAR as usize
        } else {
            65
        };
        let bar = pass.evaluation.bar;
        if count >= limit
            || bar.time != open.canonical.time
            || pass.context.bar_state != open.bar_state
            || (open.bar_state == ExternalBarState::Historical
                && (bar.open != open.canonical.open
                    || bar.high > open.canonical.high
                    || bar.low < open.canonical.low
                    || bar.volume > open.canonical.volume
                    || (pass.evaluation.confirmed && bar != open.canonical)))
        {
            return Err(external_error(
                "invalid external stream observation or pass limit",
            ));
        }
        let mut pair = vec![];
        let mut contexts = vec![];
        if open.started {
            pair.push(
                state.passes.as_ref().unwrap()[self.bars]
                    .last()
                    .unwrap()
                    .clone(),
            );
            let mut previous = *state.contexts.as_ref().unwrap()[self.bars].last().unwrap();
            if previous.trigger == crate::ExternalEvaluationTrigger::ClosingFill {
                // The prior closing fill was already validated against its own
                // predecessor. In this bounded pair it is the confirmed anchor,
                // rather than a new first closing fill without a predecessor.
                previous.trigger = crate::ExternalEvaluationTrigger::BarClose;
            }
            contexts.push(previous);
        }
        pair.push(pass.evaluation.clone());
        contexts.push(pass.context);
        self.validate_external_feedback_shape(
            std::slice::from_ref(&pass.evaluation.account),
            Some(&[pair]),
            Some(&[contexts]),
            options,
        )?;
        // Only the new transaction is preflighted against the current feed.
        // Replaying the previous observation would duplicate confirmed rows.
        self.validate_external_request_transcript(
            Some(&[vec![pass.evaluation.clone()]]),
            Some(&[vec![pass.context]]),
        )?;
        if !open.started {
            self.session_windows
                .validate_range(self.bars, self.bars + 1)
                .map_err(crate::SessionWindowInputError::runtime_error)?;
        }
        let result = self.execute_external_observation(open, count, pass);
        if result.is_err() {
            self.execution_failed = true;
        }
        result
    }

    fn execute_external_observation(
        &mut self,
        open: ExternalOpenBar,
        count: usize,
        pass: ExternalScheduledPass,
    ) -> Result<(), RuntimeError> {
        if !open.started {
            let rate = self.account_currency_rate_for_bar(
                &open.canonical,
                BarUpdateKind::Historical,
                None,
            )?;
            self.reset_execution_budget();
            self.resource_budget.check()?;
            self.initialize_native_risk()?;
            self.strategy_broker
                .set_quote_to_account_rate(rate.unwrap_or(1.0))
                .map_err(|error| RuntimeError {
                    message: error.to_string(),
                })?;
            let state = self.external_execution.as_mut().unwrap();
            state.frames.push(pass.evaluation.account.clone());
            state.passes.as_mut().unwrap().push(vec![]);
            state.contexts.as_mut().unwrap().push(vec![]);
            self.prepare_bar_evaluation(open.canonical, BarUpdateKind::Historical, true, None)?;
            self.external_execution
                .as_mut()
                .unwrap()
                .stream
                .as_mut()
                .unwrap()
                .started = true;
        }
        let state = self.external_execution.as_mut().unwrap();
        state.frames[self.bars] = pass.evaluation.account.clone();
        state.passes.as_mut().unwrap()[self.bars].push(pass.evaluation.clone());
        state.contexts.as_mut().unwrap()[self.bars].push(pass.context);
        self.run_external_scheduled_pass(count, pass.evaluation, Some(pass.context))?;
        self.finalize_series_outputs();
        Ok(())
    }

    /// Close the input group and commit Pine series once, after any closing
    /// fill observations. Market confirmation alone does not commit history.
    pub fn commit_external_bar(&mut self) -> Result<(), RuntimeError> {
        self.ensure_execution_ready()?;
        let state = self
            .external_execution
            .as_ref()
            .ok_or_else(|| external_error("external mode is not active"))?;
        let open = state
            .stream
            .ok_or_else(|| external_error("no external bar is open"))?;
        let last = open
            .started
            .then(|| state.passes.as_ref().unwrap()[self.bars].last().unwrap());
        let last = last
            .filter(|pass| pass.confirmed)
            .ok_or_else(|| external_error("a confirmed observation is required before commit"))?;
        let final_bar = if open.bar_state == ExternalBarState::Historical {
            open.canonical
        } else {
            last.bar
        };
        let result = self.commit_bar_evaluation(final_bar);
        if result.is_err() {
            self.execution_failed = true;
        } else {
            self.external_execution.as_mut().unwrap().stream = None;
        }
        result
    }

    #[must_use]
    pub fn external_bar_open(&self) -> bool {
        self.external_execution
            .as_ref()
            .is_some_and(|state| state.stream.is_some())
    }
}

fn valid_bar(bar: Bar) -> bool {
    [bar.open, bar.high, bar.low, bar.close, bar.volume]
        .iter()
        .all(|value| value.is_finite())
        && bar.high >= bar.open.max(bar.close)
        && bar.low <= bar.open.min(bar.close)
        && bar.volume >= 0.0
}

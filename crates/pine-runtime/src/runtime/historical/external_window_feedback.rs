//! Reset tentative execution while retaining observed context reservations.
use super::*;
use crate::external::external_error;
use crate::{ExternalBarState, ExternalRequestWindowHistory, ExternalWindowFeedbackStatus};

#[derive(Clone)]
pub(crate) struct ExternalWindowFeedback<'a> {
    // The seed has no feedback state, so this shared snapshot cannot cycle.
    // In particular it holds the INITIAL feed, never a later observed prefix.
    seed: Arc<HistoricalRuntime<'a>>,
    dataset: Arc<[Bar]>,
    input_skip: usize,
    requires_discovery: bool,
}

impl HistoricalRuntime<'_> {
    /// Plan without executing Pine or fabricating any account observations.
    /// The host must run tentative executions using actual account feedback,
    /// discard them on rebase, and seal a complete stable pass before exposing
    /// durable results. Neither a status nor a reset performs host matching.
    pub fn initialize_external_stream_dataset_with_observed_feedback(
        &mut self,
        bars: &[Bar],
        requested: &[ExternalRequestWindowHistory],
    ) -> Result<usize, RuntimeError> {
        self.validate_external_stream_dataset(bars)?;
        let histories = self.parse_requested_window_history(bars, requested)?;
        let mut candidate = self.clone();
        candidate.request_window_history = histories;
        let seed = Arc::new(candidate.clone());
        let skip = candidate.prepare_context_windows_for_observed_feedback(bars)?;
        let requires_discovery = candidate.context_windows.discovering;
        let external = candidate.external_execution.take();
        candidate.finish_historical_dataset_plan(bars, skip)?;
        candidate.external_execution = external;
        candidate.external_window_feedback = Some(ExternalWindowFeedback {
            seed,
            dataset: Arc::from(bars),
            input_skip: skip,
            requires_discovery,
        });
        *self = candidate;
        Ok(skip)
    }

    /// Readable after a window-change interruption. Ordinary errors remain
    /// poisoned and never produce a recoverable rebase status.
    #[must_use]
    pub fn external_window_feedback_status(&self) -> Option<ExternalWindowFeedbackStatus> {
        let feedback = self.external_window_feedback.as_ref()?;
        Some(ExternalWindowFeedbackStatus {
            schema_version: crate::EXTERNAL_WINDOW_FEEDBACK_SCHEMA_VERSION,
            requires_discovery: feedback.requires_discovery
                || self.execution_failed && self.context_windows.restart,
            rebase_required: self.execution_failed && self.context_windows.restart,
            input_skip: self.context_windows.initial_skip(feedback.dataset.len()),
        })
    }

    /// Discard all tentative Pine, intents, rollback state and observed prices.
    /// Reservations survive. Sealing is permitted only after a complete pass
    /// without interruptions; the fresh final pass still guards window drift.
    pub fn rebase_external_stream_window(&mut self, seal: bool) -> Result<usize, RuntimeError> {
        let feedback = self
            .external_window_feedback
            .as_ref()
            .ok_or_else(|| external_error("observed window feedback is not active"))?;
        let required = self.execution_failed && self.context_windows.restart;
        let complete = !self.execution_failed
            && !self.external_bar_open()
            && self.bars == feedback.dataset.len() - feedback.input_skip;
        if (seal || !required) && !complete {
            return Err(external_error(
                "window rebase requires a window interruption or a complete stable pass; sealing requires the latter",
            ));
        }
        let mut next_feedback = feedback.clone();
        let mut candidate = (*feedback.seed).clone();
        candidate.context_windows = self.context_windows.clone();
        candidate.context_windows.restart = false;
        let skip = candidate
            .context_windows
            .initial_skip(feedback.dataset.len());
        let external = candidate.external_execution.take();
        candidate.finish_historical_dataset_plan(&feedback.dataset, skip)?;
        candidate.external_execution = external;
        next_feedback.input_skip = skip;
        next_feedback.requires_discovery = (next_feedback.requires_discovery || required) && !seal;
        candidate.external_window_feedback = Some(next_feedback);
        *self = candidate;
        Ok(skip)
    }

    pub(super) fn validate_external_window_bar(
        &self,
        canonical: Bar,
        state: ExternalBarState,
    ) -> Result<(), RuntimeError> {
        if let Some(feedback) = &self.external_window_feedback {
            if let Some(expected) = feedback.dataset.get(feedback.input_skip + self.bars) {
                if state != ExternalBarState::Historical || canonical != *expected {
                    return Err(external_error(
                        "observed window execution differs from planned history",
                    ));
                }
            } else if feedback.requires_discovery {
                return Err(external_error(
                    "seal the complete observed window before extending history",
                ));
            }
        }
        Ok(())
    }
}

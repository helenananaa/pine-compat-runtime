//! Explicit host-neutral evaluation schedules, independent of bar commit state.
use crate::external::{ExternalExecutionOptions, ExternalPass, external_error};
use crate::{ExternalAccountFrame, HistoricalRuntime, RuntimeError};
use serde::{Deserialize, Serialize};

pub const EXTERNAL_SCHEDULE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalBarState {
    Historical,
    Realtime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalEvaluationTrigger {
    BarClose,
    OrderFill,
    ClosingFill,
    RealtimeTick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalPassContext {
    pub bar_state: ExternalBarState,
    pub trigger: ExternalEvaluationTrigger,
    pub is_new: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalScheduledPass {
    /// `confirmed` closes the market observation, not the script's bar state.
    pub evaluation: ExternalPass,
    pub context: ExternalPassContext,
}

pub(crate) fn split_schedule(
    schedule: Vec<Vec<ExternalScheduledPass>>,
) -> (Vec<Vec<ExternalPass>>, Vec<Vec<ExternalPassContext>>) {
    schedule
        .into_iter()
        .map(|group| {
            group
                .into_iter()
                .map(|item| (item.evaluation, item.context))
                .unzip()
        })
        .unzip()
}

impl HistoricalRuntime<'_> {
    pub fn with_external_schedule(
        self,
        frames: Vec<ExternalAccountFrame>,
        schedule: Vec<Vec<ExternalScheduledPass>>,
        options: ExternalExecutionOptions,
    ) -> Result<Self, RuntimeError> {
        let (passes, contexts) = split_schedule(schedule);
        self.with_external_feedback(frames, Some(passes), Some(contexts), options)
    }

    pub fn append_external_schedule(
        &mut self,
        bars: &[crate::Bar],
        frames: Vec<ExternalAccountFrame>,
        schedule: Vec<Vec<ExternalScheduledPass>>,
        execution_times: Option<&[i64]>,
    ) -> Result<(), RuntimeError> {
        if execution_times.is_some_and(|times| times.len() != bars.len()) {
            return Err(external_error("execution timestamp count must match bars"));
        }
        let (passes, contexts) = split_schedule(schedule);
        self.append_external_bars_inner(bars, frames, Some(passes), Some(contexts), execution_times)
    }

    pub(crate) fn validate_external_schedule(
        &self,
        frames: &[ExternalAccountFrame],
        passes: Option<&[Vec<ExternalPass>]>,
        contexts: &[Vec<ExternalPassContext>],
        options: ExternalExecutionOptions,
    ) -> Result<(), RuntimeError> {
        let passes = passes.ok_or_else(|| external_error("schedule evaluations missing"))?;
        if contexts.len() != frames.len() || passes.len() != contexts.len() {
            return Err(external_error("schedule context count differs"));
        }
        let settings = self.program.strategy_settings;
        let mut realtime_seen = self
            .external_execution
            .as_ref()
            .and_then(|state| state.contexts.as_ref())
            .and_then(|groups| groups.last())
            .and_then(|group| group.first())
            .is_some_and(|context| context.bar_state == ExternalBarState::Realtime);
        for (group, contexts) in passes.iter().zip(contexts) {
            if contexts.len() != group.len() {
                return Err(external_error("schedule context count differs"));
            }
            if let Some(context) = contexts.first() {
                match context.bar_state {
                    ExternalBarState::Historical if realtime_seen => {
                        return Err(external_error("historical schedule after realtime; rebase"));
                    }
                    ExternalBarState::Realtime => realtime_seen = true,
                    _ => {}
                }
            }
            for (index, (pass, context)) in group.iter().zip(contexts).enumerate() {
                if contexts[0].bar_state != context.bar_state
                    || (context.bar_state == ExternalBarState::Historical && !context.is_new)
                    || (context.bar_state == ExternalBarState::Realtime
                        && index > 0
                        && context.is_new)
                    || match context.trigger {
                        ExternalEvaluationTrigger::OrderFill => !settings.calc_on_order_fills,
                        ExternalEvaluationTrigger::RealtimeTick => {
                            context.bar_state != ExternalBarState::Realtime
                                || !settings.calc_on_every_tick
                        }
                        ExternalEvaluationTrigger::BarClose => !pass.confirmed,
                        ExternalEvaluationTrigger::ClosingFill => {
                            !pass.confirmed
                                || index == 0
                                || !group[index - 1].confirmed
                                || !settings.calc_on_order_fills
                                || !settings.process_orders_on_close
                                || !options.closing_ticks
                        }
                    }
                    || (index > 0
                        && group[index - 1].confirmed
                        && context.trigger != ExternalEvaluationTrigger::ClosingFill)
                {
                    return Err(external_error("invalid scheduled execution context"));
                }
            }
        }
        Ok(())
    }
}

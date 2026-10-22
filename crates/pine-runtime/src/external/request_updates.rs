//! Causal, append-only requested-data observations for external evaluators.
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::external_error;
use crate::request::RequestFeed;
use crate::{
    Bar, BarUpdate, BarUpdateKind, HistoricalRuntime, RequestDataError, RequestKey,
    RequestTimeframe, RuntimeError,
};

pub const EXTERNAL_REQUEST_UPDATE_SCHEMA_VERSION: u32 = 1;

/// Availability is inclusive: a bar ending at T is confirmed no earlier than
/// T - 1 milliseconds. Supplying an observation never revises committed output.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedRequestUpdate {
    pub symbol: String,
    pub timeframe: String,
    pub bar: Bar,
    pub kind: BarUpdateKind,
    pub available_at_ms: i64,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ExternalRequestState {
    pub observed_at: Option<i64>,
    available_at: HashMap<RequestKey, i64>,
    active: bool,
    legacy: bool,
}

impl HistoricalRuntime<'_> {
    /// Apply one atomic feed transaction before the next evaluation. Callers
    /// append only observations whose clock is at least `observed_at_ms`.
    /// Invalid input leaves feed, caches, account decisions and output unchanged.
    pub fn apply_external_request_updates(
        &mut self,
        updates: &[ObservedRequestUpdate],
        observed_at_ms: i64,
    ) -> Result<(), RuntimeError> {
        self.ensure_execution_ready()?;
        if let Some(previous) = self.previous_bar_time {
            let earliest = self
                .request_environment
                .chart_close(previous)?
                .saturating_sub(1);
            if observed_at_ms < earliest {
                return Err(external_error(
                    "requested-data observation precedes a committed chart bar",
                ));
            }
        }
        let mut state = self
            .external_execution
            .as_ref()
            .ok_or_else(|| external_error("external mode is not active"))?
            .request_state
            .clone();
        let mut feed = self.request_feed.clone();
        self.prepare_external_request_updates(&mut feed, &mut state, updates, observed_at_ms)?;
        self.request_feed = feed;
        self.external_execution
            .as_mut()
            .expect("external mode checked")
            .request_state = state;
        if !updates.is_empty() {
            // A fallback cache may contain nested requests to the changed key.
            // Endpoint-independent direct-expression checkpoints remain reusable;
            // nested expressions are excluded from that incremental fast path.
            self.request_cache.clear();
        }
        Ok(())
    }

    fn prepare_external_request_updates(
        &self,
        feed: &mut RequestFeed,
        state: &mut ExternalRequestState,
        updates: &[ObservedRequestUpdate],
        observed_at: i64,
    ) -> Result<(), RuntimeError> {
        if state.legacy || state.observed_at.is_some_and(|time| observed_at < time) {
            return Err(external_error(
                "requested-data observation clock regressed or input mode changed",
            ));
        }
        for update in updates {
            if update.symbol.is_empty()
                || update.symbol.trim() != update.symbol
                || update.timeframe.is_empty()
                || update.timeframe.trim() != update.timeframe
            {
                return Err(external_error(
                    "requested-data keys must be exact nonempty selectors",
                ));
            }
            let timeframe = RequestTimeframe::parse(&update.timeframe)
                .map_err(|error| external_error(error.to_string()))?;
            let key = RequestKey::new(&update.symbol, timeframe);
            let bar = update.bar;
            if [bar.open, bar.high, bar.low, bar.close, bar.volume]
                .iter()
                .any(|v| !v.is_finite())
                || bar.high < bar.open.max(bar.close)
                || bar.low > bar.open.min(bar.close)
                || bar.high < bar.low
                || bar.volume < 0.0
            {
                return Err(external_error("invalid requested-data OHLCV observation"));
            }
            let earliest = if update.kind == BarUpdateKind::Forming {
                bar.time
            } else {
                self.request_environment
                    .nominal_close(key.symbol(), key.timeframe(), bar.time)?
                    .saturating_sub(1)
            };
            if update.available_at_ms < earliest
                || update.available_at_ms > observed_at
                || state
                    .available_at
                    .get(&key)
                    .is_some_and(|time| update.available_at_ms < *time)
            {
                return Err(external_error(
                    "requested-data availability is premature, future or regressed",
                ));
            }
            let provider_last = match self.request_environment.provider().bars(&key) {
                Ok(bars) => bars.last().map(|bar| bar.time),
                Err(RequestDataError::MissingData { .. }) => None,
                Err(error) => return Err(external_error(error.to_string())),
            };
            feed.apply(
                key.clone(),
                BarUpdate {
                    bar,
                    kind: update.kind,
                },
                provider_last,
            )
            .map_err(crate::request::RequestFeedError::runtime_error)?;
            state.available_at.insert(key, update.available_at_ms);
        }
        state.observed_at = Some(observed_at);
        state.active = true;
        Ok(())
    }

    pub(super) fn bind_external_request_updates(&mut self) -> Result<(), RuntimeError> {
        let Some(state) = &self.external_execution else {
            return Ok(());
        };
        let Some((index, pass)) = &state.active_pass else {
            if state.request_state.active {
                let observed_at = match self.current_execution_time {
                    Some(time) => time,
                    None => self
                        .request_environment
                        .chart_close(
                            self.current_bar
                                .ok_or_else(|| external_error("missing chart observation"))?
                                .time,
                        )?
                        .saturating_sub(1),
                };
                self.apply_external_request_updates(&[], observed_at)?;
            }
            return Ok(());
        };
        if pass.request_data.is_some() {
            if state.request_state.active {
                return Err(external_error("requested-data input mode changed"));
            }
            self.external_execution
                .as_mut()
                .unwrap()
                .request_state
                .legacy = true;
            return Ok(());
        }
        if pass.request_updates.is_some() || state.request_state.active {
            let updates = pass.request_updates.clone().unwrap_or_default();
            let context = state
                .contexts
                .as_ref()
                .and_then(|groups| groups.get(self.bars))
                .and_then(|group| group.get(*index));
            let observed_at = self.external_request_observation_time(pass, context)?;
            self.apply_external_request_updates(&updates, observed_at)?;
        }
        Ok(())
    }

    pub(crate) fn validate_external_request_transcript(
        &self,
        passes: Option<&[Vec<super::ExternalPass>]>,
        contexts: Option<&[Vec<crate::ExternalPassContext>]>,
    ) -> Result<(), RuntimeError> {
        let mut feed = self.request_feed.clone();
        let mut state = self
            .external_execution
            .as_ref()
            .map(|state| state.request_state.clone())
            .unwrap_or_default();
        self.prepare_external_request_groups(passes, contexts, &mut feed, &mut state)
    }

    pub(super) fn prepare_external_request_window(
        &mut self,
        skip: usize,
    ) -> Result<(), RuntimeError> {
        let Some(external) = &self.external_execution else {
            return Ok(());
        };
        let Some(groups) = external.passes.as_ref().map(|groups| &groups[..skip]) else {
            return Ok(());
        };
        let mut feed = self.request_feed.clone();
        let mut state = external.request_state.clone();
        let contexts = external.contexts.as_ref().map(|groups| &groups[..skip]);
        self.prepare_external_request_groups(Some(groups), contexts, &mut feed, &mut state)?;
        self.request_feed = feed;
        self.external_execution.as_mut().unwrap().request_state = state;
        Ok(())
    }

    fn prepare_external_request_groups(
        &self,
        passes: Option<&[Vec<super::ExternalPass>]>,
        contexts: Option<&[Vec<crate::ExternalPassContext>]>,
        feed: &mut RequestFeed,
        state: &mut ExternalRequestState,
    ) -> Result<(), RuntimeError> {
        for (bar_index, group) in passes.into_iter().flatten().enumerate() {
            for (pass_index, pass) in group.iter().enumerate() {
                if pass.request_data.is_some() {
                    if state.active || pass.request_updates.is_some() {
                        return Err(external_error(
                            "requested-data input modes are mutually exclusive",
                        ));
                    }
                    state.legacy = true;
                } else if pass.request_updates.is_some() || state.active {
                    let context = contexts
                        .and_then(|groups| groups.get(bar_index))
                        .and_then(|group| group.get(pass_index));
                    let observed_at = self.external_request_observation_time(pass, context)?;
                    self.prepare_external_request_updates(
                        feed,
                        state,
                        pass.request_updates.as_deref().unwrap_or_default(),
                        observed_at,
                    )?;
                }
            }
        }
        Ok(())
    }

    fn external_request_observation_time(
        &self,
        pass: &super::ExternalPass,
        context: Option<&crate::ExternalPassContext>,
    ) -> Result<i64, RuntimeError> {
        // Historical fill timestamps describe an interpolation path, while
        // Pine sees the completed canonical historical bar on every pass.
        // Realtime and legacy observed-print passes retain their event clock.
        if context.is_some_and(|context| {
            context.bar_state == crate::ExternalBarState::Historical
                && context.trigger != crate::ExternalEvaluationTrigger::HistoricalTick
        }) {
            self.request_environment
                .chart_close(pass.bar.time)
                .map(|close| close.saturating_sub(1))
        } else {
            Ok(pass.event_time_ms)
        }
    }
}

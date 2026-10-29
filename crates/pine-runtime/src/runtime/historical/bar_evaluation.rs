//! Shared setup and commit for complete bars and explicit streaming bars.
use super::*;

impl HistoricalRuntime<'_> {
    pub(super) fn prepare_bar_evaluation(
        &mut self,
        bar: Bar,
        update_kind: BarUpdateKind,
        is_new_bar: bool,
        execution_time: Option<i64>,
    ) -> Result<bool, RuntimeError> {
        let bar_index = self.bars;
        self.current_bar_update_kind = update_kind;
        self.current_bar_is_new = is_new_bar;
        self.current_bar = Some(bar);
        if self
            .static_requests
            .observe_bar(bar_index, bar, update_kind)
        {
            self.request_cache.clear();
            self.request_evaluations.clear();
        }
        self.current_execution_time = execution_time;
        self.first_bar_close.get_or_insert(bar.close);
        self.chart_visible_left_time.get_or_insert(bar.time);
        if self.historical_end.is_none()
            || matches!(
                self.current_bar_update_kind,
                BarUpdateKind::Forming | BarUpdateKind::Confirmed
            )
        {
            self.last_bar_index = Some(bar_index);
            self.last_bar_time = Some(bar.time);
            self.chart_visible_right_time = Some(bar.time);
        }
        self.series_store.set_current_bar(bar_index);
        self.current_symbols.clear();
        self.current_series.clear();
        self.alert_once_per_bar_calls.clear();
        if self.program.script_mode == ScriptMode::Strategy {
            self.strategy_scheduler.begin_bar(bar_index);
        }
        let history_ticks = self.program.script_mode == ScriptMode::Strategy
            && self.program.strategy_settings.calc_on_every_history_tick
            && update_kind == BarUpdateKind::Historical
            && self.external_execution.is_none();
        let price_flow = (history_ticks
            || self
                .external_execution
                .as_ref()
                .is_some_and(|state| state.passes.is_some()))
        .then(|| PriceFlowCheckpoint::capture(self));
        self.set_builtin_symbols(&bar, bar_index)?;
        // Fill recalculations and explicit external schedules roll back between
        // passes. Other single-pass strategies need no clone/restore cycle.
        if self.program.script_mode == ScriptMode::Strategy
            && (self.program.strategy_settings.calc_on_order_fills
                || history_ticks
                || self
                    .external_execution
                    .as_ref()
                    .is_some_and(|state| state.contexts.is_some()))
        {
            self.snapshot_strategy_eval_checkpoint();
            self.strategy_eval_checkpoint.as_mut().unwrap().price_flow = price_flow;
        }
        Ok(history_ticks)
    }
    pub(super) fn commit_bar_evaluation(&mut self, bar: Bar) -> Result<(), RuntimeError> {
        if self.program.script_mode == ScriptMode::Strategy {
            self.trace_strategy_phase(
                crate::runtime::strategy_scheduler::StrategyBarPhase::OutputCommit,
            );
        }
        self.strategy_eval_checkpoint = None;
        self.historical_tick_context_active = false;
        self.finalize_series_outputs();
        self.commit_current_series()?;
        if let Some(state) = &mut self.external_execution {
            state.active_pass = None;
        }
        self.previous_bar_time = Some(bar.time);
        self.bars += 1;
        self.collect_temporary_collections();
        self.current_bar_update_kind = BarUpdateKind::Historical;
        self.current_bar_is_new = true;
        self.current_bar = None;
        self.current_standard_bar = None;
        self.current_execution_time = None;
        Ok(())
    }
    pub(super) fn run_external_scheduled_pass(
        &mut self,
        index: usize,
        pass: crate::ExternalPass,
        context: Option<crate::ExternalPassContext>,
    ) -> Result<(), RuntimeError> {
        self.current_bar = Some(pass.bar);
        self.current_execution_time = Some(pass.event_time_ms);
        if let Some(context) = context {
            if context.bar_state == crate::ExternalBarState::Realtime {
                self.last_bar_index = Some(self.bars);
                self.last_bar_time = Some(pass.bar.time);
                self.chart_visible_right_time = Some(pass.bar.time);
            }
            self.current_bar_update_kind = match context.bar_state {
                crate::ExternalBarState::Historical => BarUpdateKind::Historical,
                crate::ExternalBarState::Realtime if pass.confirmed => BarUpdateKind::Confirmed,
                crate::ExternalBarState::Realtime => BarUpdateKind::Forming,
            };
            self.current_bar_is_new = context.is_new;
            self.historical_tick_context_active =
                self.program.strategy_settings.calc_on_every_history_tick
                    && context.bar_state == crate::ExternalBarState::Historical;
        }
        self.external_execution.as_mut().unwrap().active_pass = Some((index, pass));
        if self.static_requests.observe_execution_bar(
            self.bars,
            self.current_bar.unwrap(),
            self.current_bar_update_kind,
        ) {
            self.request_cache.clear();
        }
        self.run_strategy_script_pass()?;
        Ok(())
    }
}

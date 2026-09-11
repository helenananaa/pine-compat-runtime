use pine_ir::HirProgram;

use super::streaming::OutputCursor;
use crate::*;

pub struct RealtimeRuntime<'a> {
    confirmed: HistoricalRuntime<'a>,
    forming: Option<HistoricalRuntime<'a>>,
    revision: u64,
    cursor: OutputCursor,
    last_changes: Option<RuntimeChanges>,
}
impl<'a> RealtimeRuntime<'a> {
    #[must_use]
    pub fn new(program: &'a HirProgram) -> Self {
        Self::with_request_environment(program, RequestEnvironment::default())
    }

    #[must_use]
    pub fn with_request_environment(
        program: &'a HirProgram,
        request_environment: RequestEnvironment,
    ) -> Self {
        Self {
            confirmed: HistoricalRuntime::with_request_environment(program, request_environment),
            forming: None,
            revision: 0,
            cursor: OutputCursor::default(),
            last_changes: None,
        }
    }

    #[must_use]
    pub fn with_request_environment_and_input_overrides(
        program: &'a HirProgram,
        request_environment: RequestEnvironment,
        input_overrides: InputOverrides,
    ) -> Self {
        Self {
            confirmed: HistoricalRuntime::with_request_environment_and_input_overrides(
                program,
                request_environment,
                input_overrides,
            ),
            forming: None,
            revision: 0,
            cursor: OutputCursor::default(),
            last_changes: None,
        }
    }

    #[must_use]
    pub fn request_environment(&self) -> &RequestEnvironment {
        self.forming
            .as_ref()
            .unwrap_or(&self.confirmed)
            .request_environment()
    }

    #[must_use]
    pub fn with_magnifier_input(mut self, input: crate::MagnifierInput) -> Self {
        self.confirmed = self.confirmed.with_magnifier_input(input);
        self
    }

    #[must_use]
    pub fn magnifier_input(&self) -> &crate::MagnifierInput {
        self.confirmed.magnifier_input()
    }

    pub fn with_session_windows(
        mut self,
        input: crate::SessionWindowInput,
    ) -> Result<Self, RuntimeError> {
        let current = self.forming.as_ref().unwrap_or(&self.confirmed);
        current
            .session_windows
            .validate_replacement(&input, current.bars)
            .map_err(crate::SessionWindowInputError::runtime_error)?;
        if let Some(forming) = &mut self.forming {
            forming.session_windows = input.clone();
        }
        self.confirmed.session_windows = input;
        Ok(self)
    }

    pub fn extend_session_windows(
        &mut self,
        input: crate::SessionWindowInput,
    ) -> Result<(), RuntimeError> {
        let current = self.forming.as_ref().unwrap_or(&self.confirmed);
        current
            .session_windows
            .validate_extension(&input, current.bars)
            .map_err(crate::SessionWindowInputError::runtime_error)?;
        if let Some(forming) = &mut self.forming {
            forming.session_windows.extend_validated(input.clone());
        }
        self.confirmed.session_windows.extend_validated(input);
        Ok(())
    }

    /// Validate the complete historical range before streaming bar-zero input.
    pub fn prepare_magnifier_chart_bar_count(
        &mut self,
        chart_bar_count: usize,
    ) -> Result<(), RuntimeError> {
        self.confirmed
            .prepare_magnifier_chart_bar_count(chart_bar_count)
    }

    pub fn update(&mut self, update: BarUpdate) -> Result<RuntimeResult, RuntimeError> {
        self.update_and_snapshot(update, RealtimeUpdateContext::default())
    }

    pub fn update_with_execution_time(
        &mut self,
        update: BarUpdate,
        execution_time: i64,
    ) -> Result<RuntimeResult, RuntimeError> {
        self.update_and_snapshot(
            update,
            RealtimeUpdateContext {
                execution_time: Some(execution_time),
                opening_update: None,
            },
        )
    }

    pub fn update_with_context(
        &mut self,
        update: BarUpdate,
        context: RealtimeUpdateContext,
    ) -> Result<RuntimeResult, RuntimeError> {
        self.update_and_snapshot(update, context)
    }

    /// Capture a result and its cursor in one call. Hosts bind the replica to
    /// this stream; create a new replica when replacing the producer session.
    #[must_use]
    pub fn replica(&self) -> RuntimeReplica {
        RuntimeReplica::new(self.result(), self.revision)
    }

    pub fn apply_update(&mut self, update: BarUpdate) -> Result<RuntimeChanges, RuntimeError> {
        self.apply_update_with_context(update, RealtimeUpdateContext::default())
    }

    pub fn apply_update_with_execution_time(
        &mut self,
        update: BarUpdate,
        execution_time: i64,
    ) -> Result<RuntimeChanges, RuntimeError> {
        self.apply_update_with_context(
            update,
            RealtimeUpdateContext {
                execution_time: Some(execution_time),
                opening_update: None,
            },
        )
    }

    pub fn apply_update_with_context(
        &mut self,
        update: BarUpdate,
        context: RealtimeUpdateContext,
    ) -> Result<RuntimeChanges, RuntimeError> {
        let kind = update.kind;
        self.update_inner(update, context)?;
        self.revision += 1;
        let visibility = if kind == BarUpdateKind::Forming {
            StreamingVisibility::Preview
        } else {
            StreamingVisibility::Confirmed
        };
        let changes = self.cursor.diff(self.live(), self.revision, visibility);
        self.sync_cursor();
        self.last_changes = Some(changes.clone());
        Ok(changes)
    }

    fn update_and_snapshot(
        &mut self,
        update: BarUpdate,
        context: RealtimeUpdateContext,
    ) -> Result<RuntimeResult, RuntimeError> {
        self.update_inner(update, context)?;
        self.revision += 1;
        self.sync_cursor();
        self.last_changes = None;
        Ok(self.result())
    }

    fn live(&self) -> &HistoricalRuntime<'a> {
        self.forming.as_ref().unwrap_or(&self.confirmed)
    }

    fn sync_cursor(&mut self) {
        self.cursor = OutputCursor::capture(self.live());
    }

    fn update_inner(
        &mut self,
        update: BarUpdate,
        context: RealtimeUpdateContext,
    ) -> Result<(), RuntimeError> {
        if update.kind == BarUpdateKind::Historical && context.opening_update == Some(false) {
            return Err(RuntimeError {
                message: "historical bars always have an opening update".to_owned(),
            });
        }
        if update.kind != BarUpdateKind::Historical
            && self.forming.is_some()
            && context.opening_update == Some(true)
        {
            return Err(RuntimeError {
                message: "opening_update cannot repeat for an already forming bar".to_owned(),
            });
        }
        let execution_time = context.execution_time;
        if matches!(
            update.kind,
            BarUpdateKind::Forming | BarUpdateKind::Confirmed
        ) && self
            .confirmed
            .magnifier_input()
            .bars_for_chart_bar(self.confirmed.bars)
            .is_some()
        {
            return Err(MagnifierInputError::FormingBar {
                chart_bar_index: self.confirmed.bars,
            }
            .runtime_error());
        }
        match update.kind {
            BarUpdateKind::Historical => {
                let mut runtime = self.confirmed.clone();
                runtime.append_bar_with_context(update.bar, update.kind, true, execution_time)?;
                self.confirmed = runtime;
                self.forming = None;
                Ok(())
            }
            BarUpdateKind::Confirmed => {
                let runtime = self.replay_from_confirmed(update, context)?;
                self.confirmed = runtime;
                self.forming = None;
                Ok(())
            }
            BarUpdateKind::Forming => {
                let runtime = self.replay_from_confirmed(update, context)?;
                self.forming = Some(runtime);
                Ok(())
            }
        }
    }

    fn replay_from_confirmed(
        &mut self,
        update: BarUpdate,
        context: RealtimeUpdateContext,
    ) -> Result<HistoricalRuntime<'a>, RuntimeError> {
        if update.kind == BarUpdateKind::Forming
            && self.confirmed.program.script_mode == pine_ir::ScriptMode::Strategy
            && !self.confirmed.program.strategy_settings.calc_on_every_tick
            && !self.confirmed.program.strategy_settings.calc_on_order_fills
        {
            // No script can execute on this observation. Clone its last visible
            // state once, advance the broker transactionally, and retain all
            // user state instead of restoring then cloning it again.
            let mut runtime = self.forming.as_ref().unwrap_or(&self.confirmed).clone();
            runtime.advance_broker_only_forming(update.bar)?;
            return Ok(runtime);
        }
        let is_new_bar = context.opening_update.unwrap_or(self.forming.is_none());
        // User state rolls back, except varip. Orders and fills belong to the
        // live broker and survive successful updates of the same open bar.
        let mut runtime = self.confirmed.clone();
        if let Some(previous_forming) = &self.forming {
            runtime.seed_intrabar_persistence_from(previous_forming);
            runtime
                .strategy_broker
                .clone_from(&previous_forming.strategy_broker);
            runtime
                .strategy_scheduler
                .clone_from(&previous_forming.strategy_scheduler);
        }
        let script_passes = runtime.strategy_scheduler.script_passes();
        runtime.append_bar_with_context(
            update.bar,
            update.kind,
            is_new_bar,
            context.execution_time,
        )?;
        if update.kind == BarUpdateKind::Forming
            && runtime.program.script_mode == pine_ir::ScriptMode::Strategy
            && runtime.strategy_scheduler.script_passes() == script_passes
        {
            // A price observation without script execution cannot replace the
            // last visible plots/drawings or change user state. Only the broker
            // and its scheduling state advanced in the speculative runtime.
            let mut retained = self.forming.as_ref().unwrap_or(&self.confirmed).clone();
            retained.strategy_broker = runtime.strategy_broker;
            retained.strategy_scheduler = runtime.strategy_scheduler;
            return Ok(retained);
        }
        Ok(runtime)
    }

    pub fn seed_historical(&mut self, bars: &[Bar]) -> Result<RuntimeResult, RuntimeError> {
        self.seed_historical_inner(bars, None)
    }

    pub fn seed_historical_with_execution_times(
        &mut self,
        bars: &[Bar],
        execution_times: &[i64],
    ) -> Result<RuntimeResult, RuntimeError> {
        self.seed_historical_inner(bars, Some(execution_times))
    }

    fn seed_historical_inner(
        &mut self,
        bars: &[Bar],
        execution_times: Option<&[i64]>,
    ) -> Result<RuntimeResult, RuntimeError> {
        let mut runtime = self.confirmed.clone();
        match execution_times {
            Some(times) => runtime.append_bars_with_execution_times(bars, times)?,
            None => runtime.append_bars(bars)?,
        }
        self.confirmed = runtime;
        self.forming = None;
        self.revision += 1;
        self.sync_cursor();
        self.last_changes = None;
        Ok(self.confirmed.result())
    }

    #[must_use]
    pub fn result(&self) -> RuntimeResult {
        self.live().result()
    }

    #[must_use]
    pub fn confirmed_result(&self) -> RuntimeResult {
        self.confirmed.result()
    }

    #[must_use]
    pub fn revision(&self) -> u64 {
        self.revision
    }

    #[must_use]
    pub fn last_changes(&self) -> Option<&RuntimeChanges> {
        self.last_changes.as_ref()
    }

    #[must_use]
    pub fn profile(&self) -> RuntimeProfile {
        self.forming.as_ref().unwrap_or(&self.confirmed).profile()
    }

    #[must_use]
    pub fn confirmed_profile(&self) -> RuntimeProfile {
        self.confirmed.profile()
    }
}

impl RealtimeRuntime<'static> {
    #[must_use]
    pub fn from_program(program: HirProgram) -> Self {
        Self::from_program_with_request_environment_and_input_overrides(
            program,
            RequestEnvironment::default(),
            InputOverrides::new(),
        )
    }

    #[must_use]
    pub fn from_program_with_request_environment_and_input_overrides(
        program: HirProgram,
        request_environment: RequestEnvironment,
        input_overrides: InputOverrides,
    ) -> Self {
        Self {
            confirmed:
                HistoricalRuntime::with_owned_program_and_request_environment_and_input_overrides(
                    program,
                    request_environment,
                    input_overrides,
                ),
            forming: None,
            revision: 0,
            cursor: OutputCursor::default(),
            last_changes: None,
        }
    }
}

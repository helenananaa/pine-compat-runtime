use pine_ir::HirProgram;

use crate::*;

pub struct RealtimeRuntime<'a> {
    confirmed: HistoricalRuntime<'a>,
    forming: Option<HistoricalRuntime<'a>>,
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
        self.update_inner(update, None)
    }

    pub fn update_with_execution_time(
        &mut self,
        update: BarUpdate,
        execution_time: i64,
    ) -> Result<RuntimeResult, RuntimeError> {
        self.update_inner(update, Some(execution_time))
    }

    fn update_inner(
        &mut self,
        update: BarUpdate,
        execution_time: Option<i64>,
    ) -> Result<RuntimeResult, RuntimeError> {
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
                Ok(self.confirmed.result())
            }
            BarUpdateKind::Confirmed => {
                let runtime = self.replay_from_confirmed(update, execution_time)?;
                self.confirmed = runtime;
                self.forming = None;
                Ok(self.confirmed.result())
            }
            BarUpdateKind::Forming => {
                if !self.executes_strategy_on_forming() {
                    return Ok(self.confirmed.result());
                }
                let runtime = self.replay_from_confirmed(update, execution_time)?;
                let result = runtime.result();
                self.forming = Some(runtime);
                Ok(result)
            }
        }
    }

    fn executes_strategy_on_forming(&self) -> bool {
        self.confirmed.program.script_mode != pine_ir::ScriptMode::Strategy
            || self.confirmed.program.strategy_settings.calc_on_every_tick
    }

    fn replay_from_confirmed(
        &mut self,
        update: BarUpdate,
        execution_time: Option<i64>,
    ) -> Result<HistoricalRuntime<'a>, RuntimeError> {
        let is_new_bar = self.forming.is_none();
        let mut runtime = self.confirmed.clone();
        if let Some(previous_forming) = &self.forming {
            runtime.seed_intrabar_persistence_from(previous_forming);
        }
        runtime.restore_strategy_checkpoint(&self.confirmed);
        runtime.append_bar_with_context(update.bar, update.kind, is_new_bar, execution_time)?;
        Ok(runtime)
    }

    pub fn seed_historical(&mut self, bars: &[Bar]) -> Result<RuntimeResult, RuntimeError> {
        let mut runtime = self.confirmed.clone();
        runtime.append_bars(bars)?;
        self.confirmed = runtime;
        self.forming = None;
        Ok(self.confirmed.result())
    }

    #[must_use]
    pub fn result(&self) -> RuntimeResult {
        self.forming.as_ref().unwrap_or(&self.confirmed).result()
    }

    #[must_use]
    pub fn confirmed_result(&self) -> RuntimeResult {
        self.confirmed.result()
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
        }
    }
}

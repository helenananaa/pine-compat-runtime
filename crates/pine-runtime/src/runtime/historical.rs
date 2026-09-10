use std::{
    collections::{HashMap, HashSet, VecDeque},
    ops::Deref,
    sync::Arc,
};

use pine_ir::{HirProgram, ScriptMode};

use crate::*;

#[derive(Clone)]
pub(crate) enum RuntimeProgram<'a> {
    Borrowed(&'a HirProgram),
    Owned(Arc<HirProgram>),
}

impl Deref for RuntimeProgram<'_> {
    type Target = HirProgram;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(program) => program,
            Self::Owned(program) => program,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct InputOverrides {
    values: HashMap<u32, PineValue>,
}

impl InputOverrides {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, call_site_id: u32, value: PineValue) -> Option<PineValue> {
        self.values.insert(call_site_id, value)
    }

    #[must_use]
    pub fn with_value(mut self, call_site_id: u32, value: PineValue) -> Self {
        self.insert(call_site_id, value);
        self
    }

    #[must_use]
    pub fn get(&self, call_site_id: CallSiteId) -> Option<&PineValue> {
        self.values.get(&call_site_id.0)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[derive(Clone)]
struct StrategyEvalCheckpoint {
    rolling_windows: HashMap<RollingWindowKey, RollingWindowState>,
    rsi_state: HashMap<CallSiteId, RsiState>,
    macd_state: HashMap<CallSiteId, MacdState>,
    call_state: HashMap<CallSiteId, PineValue>,
    valuewhen_state: HashMap<CallSiteId, VecDeque<PineValue>>,
    vwap_call_state: HashMap<CallSiteId, VwapState>,
    pivot_point_state: HashMap<CallSiteId, PivotPointState>,
    random_state: HashMap<CallSiteId, u64>,
    current_symbols: HashMap<SymbolId, PineValue>,
    current_series: HashMap<SeriesId, PineValue>,
    active_series: HashSet<SeriesId>,
}

#[derive(Clone)]
pub struct HistoricalRuntime<'a> {
    pub(crate) program: RuntimeProgram<'a>,
    pub(crate) input_overrides: InputOverrides,
    pub(crate) magnifier_input: MagnifierInput,
    pub(crate) magnifier_chart_bar_count: Option<usize>,
    pub(crate) session_windows: crate::SessionWindowInput,
    pub(crate) bars: usize,
    pub(crate) historical_end: Option<usize>,
    pub(crate) current_bar_update_kind: BarUpdateKind,
    pub(crate) current_bar_is_new: bool,
    pub(crate) current_bar: Option<Bar>,
    pub(crate) current_execution_time: Option<i64>,
    pub(crate) last_bar_index: Option<usize>,
    pub(crate) last_bar_time: Option<i64>,
    pub(crate) chart_visible_left_time: Option<i64>,
    pub(crate) chart_visible_right_time: Option<i64>,
    pub(crate) first_bar_close: Option<f64>,
    pub(crate) request_environment: RequestEnvironment,
    pub(crate) request_cache: HashMap<RequestCacheKey, Vec<(i64, PineValue)>>,
    pub(crate) legacy_security_repaint_warnings: HashMap<CallSiteId, (i64, i64)>,
    pub(crate) eval_expr_depth: u32,
    pub(crate) series_store: SeriesStore,
    pub(crate) series_retention: SeriesRetention,
    pub(crate) history_dynamic_retention_misses: usize,
    pub(crate) history_dynamic_retention_max_bars_back: Option<usize>,
    pub(crate) history_dynamic_retention_max_missed_offset: Option<usize>,
    pub(crate) current_symbols: HashMap<SymbolId, PineValue>,
    pub(crate) current_series: HashMap<SeriesId, PineValue>,
    pub(crate) active_series: HashSet<SeriesId>,
    pub(crate) var_store: HashMap<VarSlotId, PineValue>,
    pub(crate) array_store: HashMap<u32, Vec<PineValue>>,
    pub(crate) array_kinds: HashMap<u32, ArrayElementKind>,
    pub(crate) array_user_types: HashMap<u32, String>,
    pub(crate) array_slices: HashMap<u32, ArraySlice>,
    pub(crate) next_array_id: u32,
    #[allow(dead_code)]
    pub(crate) matrix_store: HashMap<u32, MatrixStorage>,
    #[allow(dead_code)]
    pub(crate) next_matrix_id: u32,
    pub(crate) map_store: HashMap<u32, MapStorage>,
    pub(crate) next_map_id: u32,
    pub(crate) call_state: HashMap<CallSiteId, PineValue>,
    pub(crate) valuewhen_state: HashMap<CallSiteId, VecDeque<PineValue>>,
    pub(crate) rolling_windows: HashMap<RollingWindowKey, RollingWindowState>,
    pub(crate) rsi_state: HashMap<CallSiteId, RsiState>,
    pub(crate) macd_state: HashMap<CallSiteId, MacdState>,
    pub(crate) vwap_call_state: HashMap<CallSiteId, VwapState>,
    pub(crate) pivot_point_state: HashMap<CallSiteId, PivotPointState>,
    pub(crate) random_state: HashMap<CallSiteId, u64>,
    pub(crate) previous_bar_time: Option<i64>,
    pub(crate) price_flow_previous_close: Option<f64>,
    pub(crate) price_flow_previous_volume: Option<f64>,
    pub(crate) accdist_state: PineValue,
    pub(crate) accdist_current: PineValue,
    pub(crate) iii_current: PineValue,
    pub(crate) nvi_state: PineValue,
    pub(crate) nvi_current: PineValue,
    pub(crate) obv_state: PineValue,
    pub(crate) obv_current: PineValue,
    pub(crate) pvi_state: PineValue,
    pub(crate) pvi_current: PineValue,
    pub(crate) pvt_state: PineValue,
    pub(crate) pvt_current: PineValue,
    pub(crate) vwap_weighted_sum: f64,
    pub(crate) vwap_volume_sum: f64,
    pub(crate) vwap_current: PineValue,
    pub(crate) wad_state: PineValue,
    pub(crate) wad_current: PineValue,
    pub(crate) wvad_current: PineValue,
    pub(crate) plots: Vec<PlotSeries>,
    pub(crate) plot_chars: Vec<PlotCharSeries>,
    pub(crate) plot_shapes: Vec<PlotShapeSeries>,
    pub(crate) plot_arrows: Vec<PlotArrowSeries>,
    pub(crate) plot_bars: Vec<PlotBarSeries>,
    pub(crate) plot_candles: Vec<PlotCandleSeries>,
    pub(crate) bg_colors: Vec<ColorSeries>,
    pub(crate) bar_colors: Vec<ColorSeries>,
    pub(crate) hlines: Vec<HLineOutput>,
    pub(crate) fills: Vec<FillOutput>,
    pub(crate) labels: Vec<LabelOutput>,
    pub(crate) lines: Vec<LineOutput>,
    pub(crate) line_fills: Vec<LineFillOutput>,
    pub(crate) polylines: Vec<PolylineOutput>,
    pub(crate) boxes: Vec<BoxOutput>,
    pub(crate) tables: Vec<TableOutput>,
    pub(crate) alerts: Vec<AlertEvent>,
    pub(crate) alert_once_per_bar_calls: HashSet<CallSiteId>,
    pub(crate) strategy_broker: BrokerState,
    pub(crate) strategy_scheduler: super::strategy_scheduler::StrategySchedulerState,
    strategy_eval_checkpoint: Option<StrategyEvalCheckpoint>,
    magnifier_diagnostics: Vec<RuntimeDiagnostic>,
    #[cfg(test)]
    pub(crate) strategy_phase_trace: Vec<crate::runtime::strategy_scheduler::StrategyBarPhase>,
    #[cfg(test)]
    pub(crate) strategy_path_trace: Vec<crate::runtime::strategy_scheduler::StrategyPathTraceEntry>,
    pub(crate) next_label_id: u32,
    pub(crate) next_line_id: u32,
    pub(crate) next_line_fill_id: u32,
    pub(crate) next_polyline_id: u32,
    pub(crate) next_box_id: u32,
    pub(crate) next_table_id: u32,
}

pub fn run_historical(program: &HirProgram, bars: &[Bar]) -> Result<RuntimeResult, RuntimeError> {
    HistoricalRuntime::new(program).run(bars)
}

pub fn run_historical_with_execution_times(
    program: &HirProgram,
    bars: &[Bar],
    execution_times: &[i64],
) -> Result<RuntimeResult, RuntimeError> {
    HistoricalRuntime::new(program).run_with_execution_times(bars, execution_times)
}

pub fn run_historical_with_request_environment(
    program: &HirProgram,
    bars: &[Bar],
    request_environment: RequestEnvironment,
) -> Result<RuntimeResult, RuntimeError> {
    HistoricalRuntime::with_request_environment(program, request_environment).run(bars)
}

pub fn run_historical_with_input_overrides(
    program: &HirProgram,
    bars: &[Bar],
    input_overrides: InputOverrides,
) -> Result<RuntimeResult, RuntimeError> {
    HistoricalRuntime::with_input_overrides(program, input_overrides).run(bars)
}

pub fn run_historical_with_request_environment_and_input_overrides(
    program: &HirProgram,
    bars: &[Bar],
    request_environment: RequestEnvironment,
    input_overrides: InputOverrides,
) -> Result<RuntimeResult, RuntimeError> {
    HistoricalRuntime::with_request_environment_and_input_overrides(
        program,
        request_environment,
        input_overrides,
    )
    .run(bars)
}

pub fn run_historical_with_request_environment_and_input_overrides_and_execution_times(
    program: &HirProgram,
    bars: &[Bar],
    request_environment: RequestEnvironment,
    input_overrides: InputOverrides,
    execution_times: &[i64],
) -> Result<RuntimeResult, RuntimeError> {
    HistoricalRuntime::with_request_environment_and_input_overrides(
        program,
        request_environment,
        input_overrides,
    )
    .run_with_execution_times(bars, execution_times)
}

pub fn run_historical_profiled(
    program: &HirProgram,
    bars: &[Bar],
) -> Result<RuntimeProfiledResult, RuntimeError> {
    HistoricalRuntime::new(program).run_profiled(bars)
}

pub fn run_historical_profiled_with_execution_times(
    program: &HirProgram,
    bars: &[Bar],
    execution_times: &[i64],
) -> Result<RuntimeProfiledResult, RuntimeError> {
    HistoricalRuntime::new(program).run_profiled_with_execution_times(bars, execution_times)
}

pub fn run_historical_profiled_with_request_environment(
    program: &HirProgram,
    bars: &[Bar],
    request_environment: RequestEnvironment,
) -> Result<RuntimeProfiledResult, RuntimeError> {
    HistoricalRuntime::with_request_environment(program, request_environment).run_profiled(bars)
}

pub fn run_historical_profiled_with_request_environment_and_input_overrides(
    program: &HirProgram,
    bars: &[Bar],
    request_environment: RequestEnvironment,
    input_overrides: InputOverrides,
) -> Result<RuntimeProfiledResult, RuntimeError> {
    HistoricalRuntime::with_request_environment_and_input_overrides(
        program,
        request_environment,
        input_overrides,
    )
    .run_profiled(bars)
}

pub fn run_historical_profiled_with_request_environment_and_input_overrides_and_execution_times(
    program: &HirProgram,
    bars: &[Bar],
    request_environment: RequestEnvironment,
    input_overrides: InputOverrides,
    execution_times: &[i64],
) -> Result<RuntimeProfiledResult, RuntimeError> {
    HistoricalRuntime::with_request_environment_and_input_overrides(
        program,
        request_environment,
        input_overrides,
    )
    .run_profiled_with_execution_times(bars, execution_times)
}

impl<'a> HistoricalRuntime<'a> {
    pub(crate) fn uses_v6_semantics(&self) -> bool {
        self.program
            .language_version
            .is_some_and(|version| version >= 6)
    }

    #[must_use]
    pub fn new(program: &'a HirProgram) -> Self {
        Self::with_request_environment(program, RequestEnvironment::default())
    }

    #[must_use]
    pub fn with_request_environment(
        program: &'a HirProgram,
        request_environment: RequestEnvironment,
    ) -> Self {
        Self::with_runtime_program(RuntimeProgram::Borrowed(program), request_environment)
    }

    pub(crate) fn with_runtime_program(
        program: RuntimeProgram<'a>,
        request_environment: RequestEnvironment,
    ) -> Self {
        let series_retention = SeriesRetention::from_program(&program);
        let strategy_broker = BrokerState::new_with_account_settings_and_pyramiding(
            program.strategy_settings.initial_capital,
            program.strategy_settings.commission,
            program.strategy_settings.slippage_ticks * request_environment.chart().min_tick(),
            program.strategy_settings.backtest_fill_limit_ticks
                * request_environment.chart().min_tick(),
            program.strategy_settings.margin_long,
            program.strategy_settings.margin_short,
            program.strategy_settings.pyramiding_limit,
        )
        .with_quantity_scale(request_environment.chart().quantity_scale())
        .with_close_entries_rule(program.strategy_settings.close_entries_rule)
        .with_calc_on_order_fills(program.strategy_settings.calc_on_order_fills);
        Self {
            program,
            input_overrides: InputOverrides::new(),
            magnifier_input: MagnifierInput::new(),
            magnifier_chart_bar_count: None,
            session_windows: crate::SessionWindowInput::new(),
            bars: 0,
            historical_end: None,
            current_bar_update_kind: BarUpdateKind::Historical,
            current_bar_is_new: true,
            current_bar: None,
            current_execution_time: None,
            last_bar_index: None,
            last_bar_time: None,
            chart_visible_left_time: None,
            chart_visible_right_time: None,
            first_bar_close: None,
            request_environment,
            request_cache: HashMap::new(),
            legacy_security_repaint_warnings: HashMap::new(),
            eval_expr_depth: 0,
            series_store: SeriesStore::new(),
            series_retention,
            history_dynamic_retention_misses: 0,
            history_dynamic_retention_max_bars_back: None,
            history_dynamic_retention_max_missed_offset: None,
            current_symbols: HashMap::new(),
            current_series: HashMap::new(),
            active_series: HashSet::new(),
            var_store: HashMap::new(),
            array_store: HashMap::new(),
            array_kinds: HashMap::new(),
            array_user_types: HashMap::new(),
            array_slices: HashMap::new(),
            next_array_id: 0,
            matrix_store: HashMap::new(),
            next_matrix_id: 0,
            map_store: HashMap::new(),
            next_map_id: 0,
            call_state: HashMap::new(),
            valuewhen_state: HashMap::new(),
            rolling_windows: HashMap::new(),
            rsi_state: HashMap::new(),
            macd_state: HashMap::new(),
            vwap_call_state: HashMap::new(),
            pivot_point_state: HashMap::new(),
            random_state: HashMap::new(),
            previous_bar_time: None,
            price_flow_previous_close: None,
            price_flow_previous_volume: None,
            accdist_state: PineValue::Na,
            accdist_current: PineValue::Na,
            iii_current: PineValue::Na,
            nvi_state: PineValue::Na,
            nvi_current: PineValue::Na,
            obv_state: PineValue::Na,
            obv_current: PineValue::Na,
            pvi_state: PineValue::Na,
            pvi_current: PineValue::Na,
            pvt_state: PineValue::Na,
            pvt_current: PineValue::Na,
            vwap_weighted_sum: 0.0,
            vwap_volume_sum: 0.0,
            vwap_current: PineValue::Na,
            wad_state: PineValue::Na,
            wad_current: PineValue::Na,
            wvad_current: PineValue::Na,
            plots: Vec::new(),
            plot_chars: Vec::new(),
            plot_shapes: Vec::new(),
            plot_arrows: Vec::new(),
            plot_bars: Vec::new(),
            plot_candles: Vec::new(),
            bg_colors: Vec::new(),
            bar_colors: Vec::new(),
            hlines: Vec::new(),
            fills: Vec::new(),
            labels: Vec::new(),
            lines: Vec::new(),
            line_fills: Vec::new(),
            polylines: Vec::new(),
            boxes: Vec::new(),
            tables: Vec::new(),
            alerts: Vec::new(),
            alert_once_per_bar_calls: HashSet::new(),
            strategy_broker,
            strategy_scheduler: super::strategy_scheduler::StrategySchedulerState::new(),
            strategy_eval_checkpoint: None,
            magnifier_diagnostics: Vec::new(),
            #[cfg(test)]
            strategy_phase_trace: Vec::new(),
            #[cfg(test)]
            strategy_path_trace: Vec::new(),
            next_label_id: 1,
            next_line_id: 1,
            next_line_fill_id: 1,
            next_polyline_id: 1,
            next_box_id: 1,
            next_table_id: 1,
        }
    }

    #[must_use]
    pub fn with_input_overrides(program: &'a HirProgram, input_overrides: InputOverrides) -> Self {
        let mut runtime = Self::new(program);
        runtime.input_overrides = input_overrides;
        runtime
    }

    #[must_use]
    pub fn with_request_environment_and_input_overrides(
        program: &'a HirProgram,
        request_environment: RequestEnvironment,
        input_overrides: InputOverrides,
    ) -> Self {
        let mut runtime = Self::with_request_environment(program, request_environment);
        runtime.input_overrides = input_overrides;
        runtime
    }

    #[must_use]
    pub fn request_environment(&self) -> &RequestEnvironment {
        &self.request_environment
    }

    #[must_use]
    pub fn with_magnifier_input(mut self, input: MagnifierInput) -> Self {
        self.magnifier_input = input;
        self.magnifier_chart_bar_count = None;
        self
    }

    #[must_use]
    pub fn magnifier_input(&self) -> &MagnifierInput {
        &self.magnifier_input
    }

    pub fn with_session_windows(
        mut self,
        input: crate::SessionWindowInput,
    ) -> Result<Self, RuntimeError> {
        self.session_windows
            .validate_replacement(&input, self.bars)
            .map_err(crate::SessionWindowInputError::runtime_error)?;
        self.session_windows = input;
        Ok(self)
    }

    pub fn extend_session_windows(
        &mut self,
        input: crate::SessionWindowInput,
    ) -> Result<(), RuntimeError> {
        self.session_windows
            .validate_extension(&input, self.bars)
            .map_err(crate::SessionWindowInputError::runtime_error)?;
        self.session_windows.extend_validated(input);
        Ok(())
    }

    #[must_use]
    pub fn session_windows(&self) -> &crate::SessionWindowInput {
        &self.session_windows
    }

    /// Validate the complete chart range before using the one-bar streaming API.
    ///
    /// Batch APIs derive this value from their complete input slice. Streaming
    /// callers must provide it before bar zero so sparse future groups can be
    /// distinguished from out-of-range input without inspecting partial chunks.
    pub fn prepare_magnifier_chart_bar_count(
        &mut self,
        chart_bar_count: usize,
    ) -> Result<(), RuntimeError> {
        if self.bars != 0 {
            return Err(MagnifierInputError::ChartBarCountRequired.runtime_error());
        }
        self.magnifier_input
            .validate_chart_bar_range(chart_bar_count)
            .map_err(|error| error.runtime_error())?;
        self.magnifier_chart_bar_count = Some(chart_bar_count);
        Ok(())
    }

    pub(crate) fn push_magnifier_diagnostic(&mut self, diagnostic: RuntimeDiagnostic) {
        if !self
            .magnifier_diagnostics
            .iter()
            .any(|existing| existing == &diagnostic)
        {
            self.magnifier_diagnostics.push(diagnostic);
        }
    }

    pub(crate) fn fork_with_request_environment(
        &self,
        request_environment: RequestEnvironment,
    ) -> Self {
        Self::with_runtime_program(self.program.clone(), request_environment)
    }

    pub(crate) fn run(mut self, bars: &[Bar]) -> Result<RuntimeResult, RuntimeError> {
        self.append_bars(bars)?;

        Ok(self.result())
    }

    pub(crate) fn run_with_execution_times(
        mut self,
        bars: &[Bar],
        execution_times: &[i64],
    ) -> Result<RuntimeResult, RuntimeError> {
        self.append_bars_with_execution_times(bars, execution_times)?;

        Ok(self.result())
    }

    pub(crate) fn run_profiled(
        mut self,
        bars: &[Bar],
    ) -> Result<RuntimeProfiledResult, RuntimeError> {
        self.append_bars(bars)?;

        Ok(RuntimeProfiledResult {
            result: self.result(),
            profile: self.profile(),
        })
    }

    pub(crate) fn run_profiled_with_execution_times(
        mut self,
        bars: &[Bar],
        execution_times: &[i64],
    ) -> Result<RuntimeProfiledResult, RuntimeError> {
        self.append_bars_with_execution_times(bars, execution_times)?;

        Ok(RuntimeProfiledResult {
            result: self.result(),
            profile: self.profile(),
        })
    }

    pub fn append_bars(&mut self, bars: &[Bar]) -> Result<(), RuntimeError> {
        self.append_bars_inner(bars, None)
    }

    pub fn append_bars_with_execution_times(
        &mut self,
        bars: &[Bar],
        execution_times: &[i64],
    ) -> Result<(), RuntimeError> {
        if bars.len() != execution_times.len() {
            return Err(RuntimeError {
                message: format!(
                    "execution timestamp count {} does not match bar count {}",
                    execution_times.len(),
                    bars.len()
                ),
            });
        }
        self.append_bars_inner(bars, Some(execution_times))
    }

    fn append_bars_inner(
        &mut self,
        bars: &[Bar],
        execution_times: Option<&[i64]>,
    ) -> Result<(), RuntimeError> {
        if self.program.script_mode == ScriptMode::Strategy {
            self.session_windows
                .validate_range(self.bars, self.bars + bars.len())
                .map_err(crate::SessionWindowInputError::runtime_error)?;
        }
        if self.bars == 0 && self.magnifier_chart_bar_count.is_none() {
            self.prepare_magnifier_chart_bar_count(bars.len())?;
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
        let result = (|| {
            for (index, bar) in bars.iter().enumerate() {
                self.append_bar_with_context(
                    *bar,
                    BarUpdateKind::Historical,
                    true,
                    execution_times.map(|times| times[index]),
                )?;
            }
            Ok(())
        })();
        self.historical_end = previous_historical_end;
        result
    }

    pub fn append_bar(&mut self, bar: Bar) -> Result<(), RuntimeError> {
        self.append_bar_with_kind(bar, BarUpdateKind::Historical)
    }

    pub fn append_bar_with_execution_time(
        &mut self,
        bar: Bar,
        execution_time: i64,
    ) -> Result<(), RuntimeError> {
        self.append_bar_with_context(bar, BarUpdateKind::Historical, true, Some(execution_time))
    }

    pub(crate) fn append_bar_with_kind(
        &mut self,
        bar: Bar,
        update_kind: BarUpdateKind,
    ) -> Result<(), RuntimeError> {
        self.append_bar_with_context(bar, update_kind, true, None)
    }

    pub(crate) fn append_bar_with_context(
        &mut self,
        bar: Bar,
        update_kind: BarUpdateKind,
        is_new_bar: bool,
        execution_time: Option<i64>,
    ) -> Result<(), RuntimeError> {
        let bar_index = self.bars;
        if self.program.script_mode == ScriptMode::Strategy {
            self.session_windows
                .validate_range(bar_index, bar_index + 1)
                .map_err(crate::SessionWindowInputError::runtime_error)?;
        }
        if update_kind == BarUpdateKind::Historical
            && bar_index == 0
            && !self.magnifier_input.is_empty()
            && self.magnifier_chart_bar_count.is_none()
        {
            return Err(MagnifierInputError::ChartBarCountRequired.runtime_error());
        }
        if update_kind == BarUpdateKind::Forming
            && self.magnifier_input.bars_for_chart_bar(bar_index).is_some()
        {
            return Err(MagnifierInputError::FormingBar {
                chart_bar_index: bar_index,
            }
            .runtime_error());
        }
        self.current_bar_update_kind = update_kind;
        self.current_bar_is_new = is_new_bar;
        self.current_bar = Some(bar);
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
        self.set_builtin_symbols(&bar, bar_index)?;
        // Evaluation checkpoints are only consumed by fill-triggered re-execution.
        // Single-pass strategies keep live state and need no clone/restore cycle.
        if self.program.script_mode == ScriptMode::Strategy
            && self.program.strategy_settings.calc_on_order_fills
        {
            self.snapshot_strategy_eval_checkpoint();
        }
        let passes_before_tick = self.strategy_scheduler.script_passes();
        self.run_pre_script_strategy_phases(bar_index, bar)?;
        let skip_normal_strategy_pass = self.program.script_mode == ScriptMode::Strategy
            && ((update_kind == BarUpdateKind::Forming
                && !self.program.strategy_settings.calc_on_every_tick)
                // A realtime observation with fills has already executed the
                // strategy. Do not execute it again for the same feed update.
                // Historical path ticks retain their separate closing pass.
                || (update_kind != BarUpdateKind::Historical
                    && self.strategy_scheduler.script_passes() > passes_before_tick));
        if skip_normal_strategy_pass
            && self.strategy_scheduler.script_passes() == passes_before_tick
        {
            self.strategy_broker.record_equity(bar_index, bar.close);
            self.strategy_eval_checkpoint = None;
            self.current_bar_update_kind = BarUpdateKind::Historical;
            self.current_bar_is_new = true;
            self.current_bar = None;
            self.current_execution_time = None;
            return Ok(());
        }
        if self.program.script_mode == ScriptMode::Strategy {
            self.trace_strategy_phase(
                crate::runtime::strategy_scheduler::StrategyBarPhase::BuiltinRefresh,
            );
            if !skip_normal_strategy_pass {
                let filled = self.run_strategy_script_pass()?;
                self.recalculate_after_fill(filled)?;
            }
        } else {
            let program = self.program.clone();
            for statement in &program.statements {
                match self.eval_stmt(statement) {
                    Ok(StmtControl::None) => {}
                    Ok(StmtControl::Break | StmtControl::Continue) => {
                        return Err(RuntimeError::escaped_loop_control());
                    }
                    Err(error) if error.loop_control().is_some() => {
                        return Err(RuntimeError::escaped_loop_control());
                    }
                    Err(error) => return Err(error),
                }
            }
        }

        self.run_post_script_strategy_phases(bar_index, bar)?;
        if self.program.script_mode == ScriptMode::Strategy {
            self.trace_strategy_phase(
                crate::runtime::strategy_scheduler::StrategyBarPhase::OutputCommit,
            );
        }
        self.strategy_eval_checkpoint = None;
        self.finalize_series_outputs();
        self.commit_current_series()?;
        self.previous_bar_time = Some(bar.time);
        self.bars += 1;
        self.current_bar_update_kind = BarUpdateKind::Historical;
        self.current_bar_is_new = true;
        self.current_bar = None;
        self.current_execution_time = None;
        Ok(())
    }

    #[must_use]
    pub fn result(&self) -> RuntimeResult {
        RuntimeResult {
            plots: self.plots.clone(),
            plot_chars: self.plot_chars.clone(),
            plot_shapes: self.plot_shapes.clone(),
            plot_arrows: self.plot_arrows.clone(),
            plot_bars: self.plot_bars.clone(),
            plot_candles: self.plot_candles.clone(),
            bg_colors: self.bg_colors.clone(),
            bar_colors: self.bar_colors.clone(),
            hlines: self.hlines.clone(),
            fills: self.fills.clone(),
            labels: self.labels.clone(),
            lines: self.lines.clone(),
            line_fills: self.line_fills.clone(),
            polylines: self.polylines.clone(),
            boxes: self.boxes.clone(),
            tables: self.tables.clone(),
            alerts: self.alerts.clone(),
            strategy: (self.program.script_mode == ScriptMode::Strategy)
                .then(|| self.strategy_broker.result()),
            diagnostics: self.runtime_diagnostics(),
        }
    }

    fn runtime_diagnostics(&self) -> Vec<RuntimeDiagnostic> {
        let mut diagnostics = self.magnifier_diagnostics.clone();
        let mut lookahead = self
            .legacy_security_repaint_warnings
            .iter()
            .map(|(callsite, (start, end))| {
                (
                    callsite.0,
                    RuntimeDiagnostic {
                        code: "W_LEGACY_SECURITY_LOOKAHEAD".to_owned(),
                        message: format!(
                            "legacy security at source span {start}..{end} uses historical lookahead_on behavior and can repaint"
                        ),
                    },
                )
            })
            .collect::<Vec<_>>();
        lookahead.sort_by_key(|(callsite, _)| *callsite);
        diagnostics.extend(lookahead.into_iter().map(|(_, diagnostic)| diagnostic));

        if self.history_dynamic_retention_misses == 0 {
            return diagnostics;
        }

        let Some(max_bars_back) = self
            .history_dynamic_retention_max_bars_back
            .or_else(|| self.program.max_bars_back.map(|value| value as usize))
        else {
            return diagnostics;
        };

        let max_missed_offset = self
            .history_dynamic_retention_max_missed_offset
            .unwrap_or(max_bars_back + 1);

        diagnostics.push(RuntimeDiagnostic {
            code: "W_HISTORY_MAX_BARS_BACK".to_owned(),
            message: format!(
                "dynamic history offsets exceeded max_bars_back={max_bars_back}; {} reads returned na, maximum requested offset was {max_missed_offset}",
                self.history_dynamic_retention_misses
            ),
        });
        diagnostics
    }

    #[must_use]
    pub fn profile(&self) -> RuntimeProfile {
        let request_cache_contexts = self
            .request_cache
            .keys()
            .map(RequestCacheKey::context)
            .collect::<HashSet<_>>()
            .len();
        let request_cache_values = self.request_cache.values().map(Vec::len).sum::<usize>();
        let request_cache_value_capacity = self
            .request_cache
            .values()
            .map(Vec::capacity)
            .sum::<usize>();
        let series_buffers = self.series_store.buffers.len();
        let series_values = self
            .series_store
            .buffers
            .values()
            .map(Vec::len)
            .sum::<usize>();
        let series_capacity = self
            .series_store
            .buffers
            .values()
            .map(Vec::capacity)
            .sum::<usize>();
        let plot_values = self
            .plots
            .iter()
            .map(|plot| plot.values.len())
            .sum::<usize>();
        let plot_capacity = self
            .plots
            .iter()
            .map(|plot| plot.values.capacity())
            .sum::<usize>();
        let plot_char_values = self
            .plot_chars
            .iter()
            .map(|plot_char| plot_char.values.len())
            .sum::<usize>();
        let plot_char_capacity = self
            .plot_chars
            .iter()
            .map(|plot_char| {
                plot_char.values.capacity()
                    + plot_char.chars.capacity()
                    + plot_char.colors.capacity()
            })
            .sum::<usize>();
        let plot_shape_values = self
            .plot_shapes
            .iter()
            .map(|plot_shape| plot_shape.values.len())
            .sum::<usize>();
        let plot_shape_capacity = self
            .plot_shapes
            .iter()
            .map(|plot_shape| {
                plot_shape.values.capacity()
                    + plot_shape.styles.capacity()
                    + plot_shape.locations.capacity()
                    + plot_shape.colors.capacity()
                    + plot_shape.texts.capacity()
                    + plot_shape.text_colors.capacity()
                    + plot_shape.sizes.capacity()
            })
            .sum::<usize>();
        let plot_arrow_values = self
            .plot_arrows
            .iter()
            .map(|plot_arrow| plot_arrow.values.len())
            .sum::<usize>();
        let plot_arrow_capacity = self
            .plot_arrows
            .iter()
            .map(|plot_arrow| {
                plot_arrow.values.capacity()
                    + plot_arrow.color_ups.capacity()
                    + plot_arrow.color_downs.capacity()
                    + plot_arrow.min_heights.capacity()
                    + plot_arrow.max_heights.capacity()
            })
            .sum::<usize>();
        let plot_bar_values = self
            .plot_bars
            .iter()
            .map(|plot_bar| plot_bar.opens.len())
            .sum::<usize>();
        let plot_bar_capacity = self
            .plot_bars
            .iter()
            .map(|plot_bar| {
                plot_bar.opens.capacity()
                    + plot_bar.highs.capacity()
                    + plot_bar.lows.capacity()
                    + plot_bar.closes.capacity()
                    + plot_bar.colors.capacity()
            })
            .sum::<usize>();
        let plot_candle_values = self
            .plot_candles
            .iter()
            .map(|plot_candle| plot_candle.opens.len())
            .sum::<usize>();
        let plot_candle_capacity = self
            .plot_candles
            .iter()
            .map(|plot_candle| {
                plot_candle.opens.capacity()
                    + plot_candle.highs.capacity()
                    + plot_candle.lows.capacity()
                    + plot_candle.closes.capacity()
                    + plot_candle.colors.capacity()
                    + plot_candle.wick_colors.capacity()
                    + plot_candle.border_colors.capacity()
            })
            .sum::<usize>();
        let bg_color_values = self
            .bg_colors
            .iter()
            .map(|colors| colors.values.len())
            .sum::<usize>();
        let bg_color_capacity = self
            .bg_colors
            .iter()
            .map(|colors| colors.values.capacity())
            .sum::<usize>();
        let bar_color_values = self
            .bar_colors
            .iter()
            .map(|colors| colors.values.len())
            .sum::<usize>();
        let bar_color_capacity = self
            .bar_colors
            .iter()
            .map(|colors| colors.values.capacity())
            .sum::<usize>();
        let rolling_window_values = self
            .rolling_windows
            .values()
            .map(RollingWindowState::retained_values)
            .sum::<usize>();
        let rolling_window_value_capacity = self
            .rolling_windows
            .values()
            .map(RollingWindowState::retained_capacity)
            .sum::<usize>();
        let valuewhen_state_values = self
            .valuewhen_state
            .values()
            .map(VecDeque::len)
            .sum::<usize>();
        let valuewhen_state_value_capacity = self
            .valuewhen_state
            .values()
            .map(VecDeque::capacity)
            .sum::<usize>();
        let array_values = self.array_store.values().map(Vec::len).sum::<usize>();
        let array_value_capacity = self.array_store.values().map(Vec::capacity).sum::<usize>();
        let matrix_profile = self.matrix_store_profile();
        let label_snapshots = self
            .labels
            .iter()
            .map(|label| label.snapshots.len())
            .sum::<usize>();
        let label_snapshot_capacity = self
            .labels
            .iter()
            .map(|label| label.snapshots.capacity())
            .sum::<usize>();
        let line_snapshots = self
            .lines
            .iter()
            .map(|line| line.snapshots.len())
            .sum::<usize>();
        let line_snapshot_capacity = self
            .lines
            .iter()
            .map(|line| line.snapshots.capacity())
            .sum::<usize>();
        let line_fill_snapshots = self
            .line_fills
            .iter()
            .map(|line_fill| line_fill.snapshots.len())
            .sum::<usize>();
        let line_fill_snapshot_capacity = self
            .line_fills
            .iter()
            .map(|line_fill| line_fill.snapshots.capacity())
            .sum::<usize>();
        let polyline_snapshots = self
            .polylines
            .iter()
            .map(|polyline| polyline.snapshots.len())
            .sum::<usize>();
        let polyline_snapshot_capacity = self
            .polylines
            .iter()
            .map(|polyline| polyline.snapshots.capacity())
            .sum::<usize>();
        let polyline_points = self
            .polylines
            .iter()
            .flat_map(|polyline| polyline.snapshots.iter())
            .map(|snapshot| snapshot.points.len())
            .sum::<usize>();
        let polyline_point_capacity = self
            .polylines
            .iter()
            .flat_map(|polyline| polyline.snapshots.iter())
            .map(|snapshot| snapshot.points.capacity())
            .sum::<usize>();
        let box_snapshots = self
            .boxes
            .iter()
            .map(|box_output| box_output.snapshots.len())
            .sum::<usize>();
        let box_snapshot_capacity = self
            .boxes
            .iter()
            .map(|box_output| box_output.snapshots.capacity())
            .sum::<usize>();
        let table_cells = self
            .tables
            .iter()
            .flat_map(|table| table.snapshots.iter())
            .map(|snapshot| snapshot.cells.len())
            .sum::<usize>();
        let table_snapshot_capacity = self
            .tables
            .iter()
            .map(|table| table.snapshots.capacity())
            .sum::<usize>();
        let table_cell_capacity = self
            .tables
            .iter()
            .flat_map(|table| table.snapshots.iter())
            .map(|snapshot| snapshot.cells.capacity())
            .sum::<usize>();

        RuntimeProfile {
            bars: self.bars,
            series_buffers,
            series_values,
            series_capacity,
            max_series_depth: self.series_store.max_depth(),
            history_retention_mode: self.series_retention.mode(),
            history_max_constant_offset: self.program.history.max_constant_offset,
            history_max_bars_back: self.program.max_bars_back,
            history_has_dynamic_offsets: self.program.history.has_dynamic_offsets,
            history_dynamic_retention_misses: self.history_dynamic_retention_misses,
            history_dynamic_retention_max_missed_offset: self
                .history_dynamic_retention_max_missed_offset,
            request_cache_entries: self.request_cache.len(),
            request_cache_contexts,
            request_cache_values,
            request_cache_value_capacity,
            symbol_slots: self.current_symbols.len(),
            symbol_capacity: self.current_symbols.capacity(),
            current_series_slots: self.current_series.len(),
            current_series_capacity: self.current_series.capacity(),
            var_slots: self.var_store.len(),
            var_capacity: self.var_store.capacity(),
            array_slots: self.array_store.len(),
            array_capacity: self.array_store.capacity(),
            array_values,
            array_value_capacity,
            matrix_slots: matrix_profile.slots,
            matrix_capacity: matrix_profile.capacity,
            matrix_cells: matrix_profile.cells,
            matrix_cell_capacity: matrix_profile.cell_capacity,
            call_state_slots: self.call_state.len(),
            call_state_capacity: self.call_state.capacity(),
            valuewhen_state_slots: self.valuewhen_state.len(),
            valuewhen_state_capacity: self.valuewhen_state.capacity(),
            valuewhen_state_values,
            valuewhen_state_value_capacity,
            rolling_window_slots: self.rolling_windows.len(),
            rolling_window_capacity: self.rolling_windows.capacity(),
            rolling_window_values,
            rolling_window_value_capacity,
            rsi_state_slots: self.rsi_state.len(),
            rsi_state_capacity: self.rsi_state.capacity(),
            macd_state_slots: self.macd_state.len(),
            macd_state_capacity: self.macd_state.capacity(),
            plots: self.plots.len(),
            plot_values,
            plot_capacity,
            plot_chars: self.plot_chars.len(),
            plot_char_values,
            plot_char_capacity,
            plot_shapes: self.plot_shapes.len(),
            plot_shape_values,
            plot_shape_capacity,
            plot_arrows: self.plot_arrows.len(),
            plot_arrow_values,
            plot_arrow_capacity,
            plot_bars: self.plot_bars.len(),
            plot_bar_values,
            plot_bar_capacity,
            plot_candles: self.plot_candles.len(),
            plot_candle_values,
            plot_candle_capacity,
            bg_colors: self.bg_colors.len(),
            bg_color_values,
            bg_color_capacity,
            bar_colors: self.bar_colors.len(),
            bar_color_values,
            bar_color_capacity,
            hlines: self.hlines.len(),
            hline_capacity: self.hlines.capacity(),
            fills: self.fills.len(),
            fill_capacity: self.fills.capacity(),
            labels: self.labels.len(),
            label_snapshots,
            label_capacity: self.labels.capacity(),
            label_snapshot_capacity,
            lines: self.lines.len(),
            line_snapshots,
            line_capacity: self.lines.capacity(),
            line_snapshot_capacity,
            line_fills: self.line_fills.len(),
            line_fill_snapshots,
            line_fill_capacity: self.line_fills.capacity(),
            line_fill_snapshot_capacity,
            polylines: self.polylines.len(),
            polyline_snapshots,
            polyline_points,
            polyline_capacity: self.polylines.capacity(),
            polyline_snapshot_capacity,
            polyline_point_capacity,
            boxes: self.boxes.len(),
            box_snapshots,
            box_capacity: self.boxes.capacity(),
            box_snapshot_capacity,
            tables: self.tables.len(),
            table_cells,
            table_capacity: self.tables.capacity(),
            table_snapshot_capacity,
            table_cell_capacity,
            strategy_script_passes: if self.program.script_mode == ScriptMode::Strategy {
                self.strategy_scheduler.script_passes()
            } else {
                0
            },
            strategy_recalculation_passes: if self.program.script_mode == ScriptMode::Strategy {
                self.strategy_scheduler.recalculation_passes()
            } else {
                0
            },
            strategy_max_passes_on_bar: if self.program.script_mode == ScriptMode::Strategy {
                self.strategy_scheduler.max_passes_on_bar() as usize
            } else {
                0
            },
            strategy_max_recalculation_passes: if self.program.script_mode == ScriptMode::Strategy {
                self.strategy_scheduler.max_recalculation_passes() as usize
            } else {
                0
            },
        }
    }

    fn snapshot_strategy_eval_checkpoint(&mut self) {
        self.strategy_eval_checkpoint = Some(StrategyEvalCheckpoint {
            rolling_windows: self.rolling_windows.clone(),
            rsi_state: self.rsi_state.clone(),
            macd_state: self.macd_state.clone(),
            call_state: self.call_state.clone(),
            valuewhen_state: self.valuewhen_state.clone(),
            vwap_call_state: self.vwap_call_state.clone(),
            pivot_point_state: self.pivot_point_state.clone(),
            random_state: self.random_state.clone(),
            current_symbols: self.current_symbols.clone(),
            current_series: self.current_series.clone(),
            active_series: self.active_series.clone(),
        });
    }

    fn restore_strategy_eval_checkpoint(&mut self) {
        let Some(checkpoint) = self.strategy_eval_checkpoint.as_ref() else {
            return;
        };
        self.rolling_windows.clone_from(&checkpoint.rolling_windows);
        self.rsi_state.clone_from(&checkpoint.rsi_state);
        self.macd_state.clone_from(&checkpoint.macd_state);
        self.call_state.clone_from(&checkpoint.call_state);
        self.valuewhen_state.clone_from(&checkpoint.valuewhen_state);
        self.vwap_call_state.clone_from(&checkpoint.vwap_call_state);
        self.pivot_point_state
            .clone_from(&checkpoint.pivot_point_state);
        self.random_state.clone_from(&checkpoint.random_state);
        self.current_symbols.clone_from(&checkpoint.current_symbols);
        self.current_series.clone_from(&checkpoint.current_series);
        self.active_series.clone_from(&checkpoint.active_series);
    }

    fn run_strategy_script_pass(&mut self) -> Result<bool, RuntimeError> {
        let before = self.strategy_broker.public_order_event_count();
        self.restore_strategy_eval_checkpoint();
        self.strategy_scheduler.begin_script_pass()?;
        self.trace_strategy_phase(
            crate::runtime::strategy_scheduler::StrategyBarPhase::ScriptStatements,
        );
        let program = self.program.clone();
        for statement in &program.statements {
            match self.eval_stmt(statement) {
                Ok(StmtControl::None) => {}
                Ok(StmtControl::Break | StmtControl::Continue) => {
                    return Err(RuntimeError::escaped_loop_control());
                }
                Err(error) if error.loop_control().is_some() => {
                    return Err(RuntimeError::escaped_loop_control());
                }
                Err(error) => return Err(error),
            }
        }
        Ok(self.strategy_broker.public_order_event_count() > before)
    }

    pub(crate) fn recalculate_after_fill(&mut self, mut filled: bool) -> Result<(), RuntimeError> {
        if !self.program.strategy_settings.calc_on_order_fills {
            return Ok(());
        }
        while filled {
            filled = self.run_strategy_script_pass()?;
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn snapshot_strategy_broker(&self) -> BrokerState {
        self.strategy_broker.snapshot()
    }

    #[cfg(test)]
    pub(crate) fn restore_strategy_broker(&mut self, snapshot: BrokerState) {
        self.strategy_broker.restore(snapshot);
    }

    pub(crate) fn finalize_series_outputs(&mut self) {
        finalize_plot_values(&mut self.plots, self.bars);
        finalize_bar_aligned_outputs(&mut self.plot_chars, self.bars);
        finalize_bar_aligned_outputs(&mut self.plot_shapes, self.bars);
        finalize_bar_aligned_outputs(&mut self.plot_arrows, self.bars);
        finalize_bar_aligned_outputs(&mut self.plot_bars, self.bars);
        finalize_bar_aligned_outputs(&mut self.plot_candles, self.bars);
        finalize_series_values(&mut self.bg_colors, self.bars);
        finalize_series_values(&mut self.bar_colors, self.bars);
        for fill in &mut self.fills {
            while fill.colors.len() < self.bars {
                fill.colors.push(PineValue::Na);
            }
            if fill.colors.len() == self.bars {
                fill.colors.push(PineValue::Na);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn push_hline(
        &mut self,
        id: u32,
        price: PineValue,
        title: PineValue,
        color: PineValue,
        style: PineValue,
        linewidth: PineValue,
        editable: PineValue,
        display: PineValue,
    ) {
        if let Some(hline) = self.hlines.iter_mut().find(|hline| hline.id == id) {
            hline.price = price;
            hline.title = title;
            hline.color = color;
            hline.style = style;
            hline.linewidth = linewidth;
            hline.editable = editable;
            hline.display = display;
        } else {
            self.hlines.push(HLineOutput {
                id,
                price,
                title,
                color,
                style,
                linewidth,
                editable,
                display,
            });
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn push_fill(
        &mut self,
        id: u32,
        first: PineValue,
        second: PineValue,
        color: PineValue,
        title: PineValue,
        editable: PineValue,
        show_last: PineValue,
        fill_gaps: PineValue,
        display: PineValue,
    ) {
        let first_is_hline = matches!(first, PineValue::HLine(_));
        let second_is_hline = matches!(second, PineValue::HLine(_));
        let Some(first_id) = output_id(first) else {
            return;
        };
        let Some(second_id) = output_id(second) else {
            return;
        };
        if let Some(fill) = self.fills.iter_mut().find(|fill| fill.id == id) {
            while fill.colors.len() < self.bars {
                fill.colors.push(PineValue::Na);
            }
            if fill.colors.len() == self.bars {
                fill.colors.push(color);
            } else if let Some(current) = fill.colors.last_mut() {
                *current = color;
            }
            fill.first_id = first_id;
            fill.second_id = second_id;
            fill.first_is_hline = first_is_hline;
            fill.second_is_hline = second_is_hline;
            fill.title = title;
            fill.editable = editable;
            fill.show_last = show_last;
            fill.fill_gaps = fill_gaps;
            fill.display = display;
            return;
        }
        let mut colors = vec![PineValue::Na; self.bars];
        colors.push(color);
        self.fills.push(FillOutput {
            id,
            first_id,
            second_id,
            first_is_hline,
            second_is_hline,
            colors,
            title,
            editable,
            show_last,
            fill_gaps,
            display,
        });
    }
}

impl HistoricalRuntime<'static> {
    pub(crate) fn with_owned_program_and_request_environment_and_input_overrides(
        program: HirProgram,
        request_environment: RequestEnvironment,
        input_overrides: InputOverrides,
    ) -> Self {
        let mut runtime = Self::with_runtime_program(
            RuntimeProgram::Owned(Arc::new(program)),
            request_environment,
        );
        runtime.input_overrides = input_overrides;
        runtime
    }
}

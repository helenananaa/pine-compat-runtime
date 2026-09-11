use pine_ir::HirProgram;
use pine_runtime::{
    Bar, BarUpdate, OutputRetention, RealtimeRuntime, RealtimeUpdateContext, RequestKey,
    RequestTimeframe, RuntimeReplica, public_runtime_changes_json, public_runtime_result_json,
    runtime_changes_from_json, runtime_result_from_json, session_window_input_from_json,
};
use serde_json::Value;
use wasm_bindgen::prelude::*;

use crate::input_overrides::input_overrides_from_json;
use crate::request_bars::{
    bar_from_json, execution_times_from_json, request_environment_and_execution_times_from_json,
};
use crate::run::{WasmProgram, parse_bars_csv};

pub(crate) const REALTIME_SESSION_SCHEMA_VERSION: u32 = 1;

#[wasm_bindgen(js_name = realtimeSessionSchemaVersion)]
pub fn realtime_session_schema_version() -> u32 {
    REALTIME_SESSION_SCHEMA_VERSION
}

#[wasm_bindgen(js_name = runtimeChangesSchemaVersion)]
pub fn runtime_changes_schema_version() -> u32 {
    pine_runtime::PUBLIC_RUNTIME_CHANGES_SCHEMA_VERSION
}

#[wasm_bindgen(js_name = RealtimeSession)]
pub struct WasmRealtimeSession {
    runtime: RealtimeRuntime<'static>,
    seeded: bool,
    confirmed_bars: usize,
    last_confirmed_time: Option<i64>,
    forming_time: Option<i64>,
}

#[wasm_bindgen(js_name = RuntimeReplica)]
pub struct WasmRuntimeReplica {
    inner: RuntimeReplica,
}

impl WasmRealtimeSession {
    pub(crate) fn from_program(
        hir: HirProgram,
        request_bars_json: &str,
        input_overrides_json: &str,
    ) -> Result<Self, String> {
        let request_json = if request_bars_json.trim().is_empty() {
            "{}"
        } else {
            request_bars_json
        };
        let overrides_json = if input_overrides_json.trim().is_empty() {
            "{}"
        } else {
            input_overrides_json
        };
        let parsed = request_environment_and_execution_times_from_json(request_json)?;
        if parsed.execution_times.is_some() {
            return Err(
                "realtime session request bars must not include `$executionTimes`; pass them to seedWithExecutionTimes"
                    .to_owned(),
            );
        }
        let input_overrides = input_overrides_from_json(overrides_json, &hir)?;
        let mut runtime =
            RealtimeRuntime::from_program_with_request_environment_and_input_overrides(
                hir,
                parsed.environment,
                input_overrides,
            );
        if let Some(magnifier) = parsed.magnifier {
            runtime = runtime.with_magnifier_input(magnifier);
        }
        if let Some(session_windows) = parsed.session_windows {
            runtime = runtime
                .with_session_windows(session_windows)
                .map_err(|err| err.message)?;
        }
        Ok(Self {
            runtime,
            seeded: false,
            confirmed_bars: 0,
            last_confirmed_time: None,
            forming_time: None,
        })
    }

    fn require_seeded(&self) -> Result<(), String> {
        if self.seeded {
            Ok(())
        } else {
            Err("realtime session must be seeded before updates".to_owned())
        }
    }

    fn validate_next_bar(&self, bar: &Bar, forming: bool) -> Result<(), String> {
        if let Some(last_confirmed_time) = self.last_confirmed_time
            && bar.time <= last_confirmed_time
        {
            return Err(format!(
                "realtime bar time `{}` must be later than confirmed time `{last_confirmed_time}`",
                bar.time
            ));
        }
        if let Some(forming_time) = self.forming_time
            && bar.time != forming_time
        {
            let action = if forming { "replace" } else { "confirm" };
            return Err(format!(
                "realtime {action} time `{}` does not match forming time `{forming_time}`",
                bar.time
            ));
        }
        Ok(())
    }

    pub(crate) fn seed_internal(
        &mut self,
        bars_csv: &str,
        execution_times: Option<&[i64]>,
    ) -> Result<String, String> {
        if self.seeded {
            return Err("realtime session history has already been seeded".to_owned());
        }
        let bars = parse_bars_csv(bars_csv)?;
        let result = match execution_times {
            Some(times) => self
                .runtime
                .seed_historical_with_execution_times(&bars, times),
            None => self.runtime.seed_historical(&bars),
        }
        .map_err(|err| err.message)?;
        self.seeded = true;
        self.confirmed_bars = self.runtime.confirmed_bar_count();
        self.last_confirmed_time = self.runtime.last_confirmed_bar_time();
        Ok(public_runtime_result_json(&result))
    }

    pub(crate) fn replay_internal(
        &mut self,
        bars_csv: &str,
        execution_times: Option<&[i64]>,
    ) -> Result<String, String> {
        self.require_seeded()?;
        let bars = parse_bars_csv(bars_csv)?;
        let result = match execution_times {
            Some(times) => self
                .runtime
                .replay_historical_with_execution_times(&bars, times),
            None => self.runtime.replay_historical(&bars),
        }
        .map_err(|err| err.message)?;
        self.confirmed_bars = self.runtime.confirmed_bar_count();
        self.last_confirmed_time = self.runtime.last_confirmed_bar_time();
        self.forming_time = None;
        Ok(public_runtime_result_json(&result))
    }

    pub(crate) fn correct_internal(
        &mut self,
        from_time: i64,
        bars_csv: &str,
        execution_times: Option<&[i64]>,
    ) -> Result<String, String> {
        self.require_seeded()?;
        let bars = parse_bars_csv(bars_csv)?;
        let result = match execution_times {
            Some(times) => self
                .runtime
                .correct_historical_with_execution_times(from_time, &bars, times),
            None => self.runtime.correct_historical(from_time, &bars),
        }
        .map_err(|err| err.message)?;
        self.confirmed_bars = self.runtime.confirmed_bar_count();
        self.last_confirmed_time = self.runtime.last_confirmed_bar_time();
        self.forming_time = None;
        Ok(public_runtime_result_json(&result))
    }

    fn parse_from_time(from_time: f64) -> Result<i64, String> {
        if !from_time.is_finite() || from_time.fract() != 0.0 {
            return Err("fromTime must be an integer millisecond timestamp".to_owned());
        }
        Ok(from_time as i64)
    }

    pub(crate) fn apply_chart_update(
        &mut self,
        bar_json: &str,
        context_json: &str,
        forming: bool,
        changes: bool,
    ) -> Result<String, String> {
        self.require_seeded()?;
        let bar = bar_from_json(bar_json)?;
        self.validate_next_bar(&bar, forming)?;
        let context = context_from_json(context_json)?;
        let update = if forming {
            BarUpdate::forming(bar)
        } else {
            BarUpdate::confirmed(bar)
        };
        let output = if changes {
            let changes = self
                .runtime
                .apply_update_with_context(update, context)
                .map_err(|err| err.message)?;
            public_runtime_changes_json(&changes)
        } else {
            let result = self
                .runtime
                .update_with_context(update, context)
                .map_err(|err| err.message)?;
            public_runtime_result_json(&result)
        };
        if forming {
            self.forming_time = Some(bar.time);
        } else {
            self.confirmed_bars = self.runtime.confirmed_bar_count();
            self.last_confirmed_time = self.runtime.last_confirmed_bar_time();
            self.forming_time = None;
        }
        Ok(output)
    }

    pub(crate) fn apply_request_update(
        &mut self,
        symbol: &str,
        timeframe: &str,
        bar_json: &str,
        forming: bool,
    ) -> Result<String, String> {
        self.require_seeded()?;
        let timeframe = RequestTimeframe::parse(timeframe).map_err(|err| err.to_string())?;
        let key = RequestKey::new(symbol, timeframe);
        let bar = bar_from_json(bar_json)?;
        let update = if forming {
            BarUpdate::forming(bar)
        } else {
            BarUpdate::confirmed(bar)
        };
        match self
            .runtime
            .apply_request_update(key, update)
            .map_err(|err| err.message)?
        {
            Some(changes) => Ok(public_runtime_changes_json(&changes)),
            None => Ok("null".to_owned()),
        }
    }
}

#[wasm_bindgen]
impl WasmRealtimeSession {
    #[wasm_bindgen(js_name = seed)]
    pub fn seed(&mut self, bars_csv: &str) -> Result<String, JsValue> {
        self.seed_internal(bars_csv, None)
            .map_err(|err| JsValue::from_str(&err))
    }

    #[wasm_bindgen(js_name = seedWithExecutionTimes)]
    pub fn seed_with_execution_times(
        &mut self,
        bars_csv: &str,
        execution_times_json: &str,
    ) -> Result<String, JsValue> {
        let times = execution_times_from_json(execution_times_json)
            .map_err(|err| JsValue::from_str(&err))?;
        self.seed_internal(bars_csv, Some(&times))
            .map_err(|err| JsValue::from_str(&err))
    }

    #[wasm_bindgen(js_name = replay)]
    pub fn replay(&mut self, bars_csv: &str) -> Result<String, JsValue> {
        self.replay_internal(bars_csv, None)
            .map_err(|err| JsValue::from_str(&err))
    }

    #[wasm_bindgen(js_name = replayWithExecutionTimes)]
    pub fn replay_with_execution_times(
        &mut self,
        bars_csv: &str,
        execution_times_json: &str,
    ) -> Result<String, JsValue> {
        let times = execution_times_from_json(execution_times_json)
            .map_err(|err| JsValue::from_str(&err))?;
        self.replay_internal(bars_csv, Some(&times))
            .map_err(|err| JsValue::from_str(&err))
    }

    #[wasm_bindgen(js_name = correct)]
    pub fn correct(&mut self, from_time: f64, bars_csv: &str) -> Result<String, JsValue> {
        let from_time = Self::parse_from_time(from_time).map_err(|err| JsValue::from_str(&err))?;
        self.correct_internal(from_time, bars_csv, None)
            .map_err(|err| JsValue::from_str(&err))
    }

    #[wasm_bindgen(js_name = correctWithExecutionTimes)]
    pub fn correct_with_execution_times(
        &mut self,
        from_time: f64,
        bars_csv: &str,
        execution_times_json: &str,
    ) -> Result<String, JsValue> {
        let from_time = Self::parse_from_time(from_time).map_err(|err| JsValue::from_str(&err))?;
        let times = execution_times_from_json(execution_times_json)
            .map_err(|err| JsValue::from_str(&err))?;
        self.correct_internal(from_time, bars_csv, Some(&times))
            .map_err(|err| JsValue::from_str(&err))
    }

    #[wasm_bindgen(js_name = applyForming)]
    pub fn apply_forming(&mut self, bar_json: &str) -> Result<String, JsValue> {
        self.apply_forming_with_context(bar_json, "{}")
    }

    #[wasm_bindgen(js_name = applyFormingWithContext)]
    pub fn apply_forming_with_context(
        &mut self,
        bar_json: &str,
        context_json: &str,
    ) -> Result<String, JsValue> {
        self.apply_chart_update(bar_json, context_json, true, true)
            .map_err(|err| JsValue::from_str(&err))
    }

    #[wasm_bindgen(js_name = applyConfirmed)]
    pub fn apply_confirmed(&mut self, bar_json: &str) -> Result<String, JsValue> {
        self.apply_confirmed_with_context(bar_json, "{}")
    }

    #[wasm_bindgen(js_name = applyConfirmedWithContext)]
    pub fn apply_confirmed_with_context(
        &mut self,
        bar_json: &str,
        context_json: &str,
    ) -> Result<String, JsValue> {
        self.apply_chart_update(bar_json, context_json, false, true)
            .map_err(|err| JsValue::from_str(&err))
    }

    #[wasm_bindgen(js_name = updateForming)]
    pub fn update_forming(&mut self, bar_json: &str) -> Result<String, JsValue> {
        self.update_forming_with_context(bar_json, "{}")
    }

    #[wasm_bindgen(js_name = updateFormingWithContext)]
    pub fn update_forming_with_context(
        &mut self,
        bar_json: &str,
        context_json: &str,
    ) -> Result<String, JsValue> {
        self.apply_chart_update(bar_json, context_json, true, false)
            .map_err(|err| JsValue::from_str(&err))
    }

    #[wasm_bindgen(js_name = updateConfirmed)]
    pub fn update_confirmed(&mut self, bar_json: &str) -> Result<String, JsValue> {
        self.update_confirmed_with_context(bar_json, "{}")
    }

    #[wasm_bindgen(js_name = updateConfirmedWithContext)]
    pub fn update_confirmed_with_context(
        &mut self,
        bar_json: &str,
        context_json: &str,
    ) -> Result<String, JsValue> {
        self.apply_chart_update(bar_json, context_json, false, false)
            .map_err(|err| JsValue::from_str(&err))
    }

    #[wasm_bindgen(js_name = applyRequestForming)]
    pub fn apply_request_forming(
        &mut self,
        symbol: &str,
        timeframe: &str,
        bar_json: &str,
    ) -> Result<String, JsValue> {
        self.apply_request_update(symbol, timeframe, bar_json, true)
            .map_err(|err| JsValue::from_str(&err))
    }

    #[wasm_bindgen(js_name = applyRequestConfirmed)]
    pub fn apply_request_confirmed(
        &mut self,
        symbol: &str,
        timeframe: &str,
        bar_json: &str,
    ) -> Result<String, JsValue> {
        self.apply_request_update(symbol, timeframe, bar_json, false)
            .map_err(|err| JsValue::from_str(&err))
    }

    #[wasm_bindgen(js_name = replica)]
    pub fn replica(&self) -> WasmRuntimeReplica {
        WasmRuntimeReplica {
            inner: self.runtime.replica(),
        }
    }

    #[wasm_bindgen(js_name = streamSnapshot)]
    pub fn stream_snapshot(&self) -> String {
        format!(
            "{{\"revision\":{},\"retainedFrom\":{},\"result\":{}}}",
            self.runtime.revision(),
            self.runtime.display_origin(),
            public_runtime_result_json(&self.runtime.result())
        )
    }

    #[wasm_bindgen(js_name = setOutputRetention)]
    pub fn set_output_retention(&mut self, keep_confirmed_bars: u32) {
        self.runtime
            .set_output_retention(OutputRetention::keep_confirmed_bars(
                keep_confirmed_bars as usize,
            ));
    }

    #[wasm_bindgen(js_name = clearOutputRetention)]
    pub fn clear_output_retention(&mut self) {
        self.runtime
            .set_output_retention(OutputRetention::unlimited());
    }

    #[wasm_bindgen(js_name = extendSessionWindows)]
    pub fn extend_session_windows(&mut self, session_windows_json: &str) -> Result<(), JsValue> {
        let input = session_window_input_from_json(session_windows_json)
            .map_err(|err| JsValue::from_str(&err))?;
        self.runtime
            .extend_session_windows(input)
            .map_err(|err| JsValue::from_str(&err.message))
    }

    #[wasm_bindgen(js_name = result)]
    pub fn result(&self) -> String {
        public_runtime_result_json(&self.runtime.result())
    }

    #[wasm_bindgen(js_name = confirmedResult)]
    pub fn confirmed_result(&self) -> String {
        public_runtime_result_json(&self.runtime.confirmed_result())
    }

    #[wasm_bindgen(js_name = lastChanges)]
    pub fn last_changes(&self) -> String {
        match self.runtime.last_changes() {
            Some(changes) => public_runtime_changes_json(changes),
            None => "null".to_owned(),
        }
    }

    #[wasm_bindgen(getter, js_name = schemaVersion)]
    pub fn schema_version(&self) -> u32 {
        REALTIME_SESSION_SCHEMA_VERSION
    }

    #[wasm_bindgen(getter, js_name = isSeeded)]
    pub fn is_seeded(&self) -> bool {
        self.seeded
    }

    #[wasm_bindgen(getter, js_name = confirmedBars)]
    pub fn confirmed_bars(&self) -> usize {
        self.confirmed_bars
    }

    #[wasm_bindgen(getter, js_name = lastConfirmedTime)]
    pub fn last_confirmed_time(&self) -> Option<f64> {
        self.last_confirmed_time.map(|value| value as f64)
    }

    #[wasm_bindgen(getter, js_name = formingTime)]
    pub fn forming_time(&self) -> Option<f64> {
        self.forming_time.map(|value| value as f64)
    }

    #[wasm_bindgen(getter)]
    pub fn revision(&self) -> u64 {
        self.runtime.revision()
    }

    #[wasm_bindgen(getter, js_name = displayOrigin)]
    pub fn display_origin(&self) -> usize {
        self.runtime.display_origin()
    }
}

#[wasm_bindgen]
impl WasmRuntimeReplica {
    #[wasm_bindgen(constructor)]
    pub fn new(
        result_json: &str,
        revision: u64,
        retained_from: u32,
    ) -> Result<WasmRuntimeReplica, JsValue> {
        let result =
            runtime_result_from_json(result_json).map_err(|err| JsValue::from_str(&err))?;
        Ok(Self {
            inner: RuntimeReplica::with_retained_from(result, revision, retained_from as usize),
        })
    }

    #[wasm_bindgen]
    pub fn apply(&mut self, changes_json: &str) -> Result<bool, JsValue> {
        self.apply_internal(changes_json)
            .map_err(|err| JsValue::from_str(&err))
    }

    #[wasm_bindgen]
    pub fn result(&self) -> String {
        public_runtime_result_json(self.inner.result())
    }

    #[wasm_bindgen]
    pub fn reset(
        &mut self,
        result_json: &str,
        revision: u64,
        retained_from: u32,
    ) -> Result<(), JsValue> {
        self.reset_internal(result_json, revision, retained_from)
            .map_err(|err| JsValue::from_str(&err))
    }

    #[wasm_bindgen(getter)]
    pub fn revision(&self) -> u64 {
        self.inner.revision()
    }

    #[wasm_bindgen(getter, js_name = retainedFrom)]
    pub fn retained_from(&self) -> usize {
        self.inner.retained_from()
    }
}

impl WasmRuntimeReplica {
    pub(crate) fn apply_internal(&mut self, changes_json: &str) -> Result<bool, String> {
        let changes = runtime_changes_from_json(changes_json)?;
        self.inner.apply(&changes).map_err(|err| err.message)
    }

    pub(crate) fn reset_internal(
        &mut self,
        result_json: &str,
        revision: u64,
        retained_from: u32,
    ) -> Result<(), String> {
        let result = runtime_result_from_json(result_json)?;
        self.inner
            .reset_with_retained_from(result, revision, retained_from as usize);
        Ok(())
    }
}

#[wasm_bindgen]
impl WasmProgram {
    #[wasm_bindgen(js_name = realtimeSession)]
    pub fn realtime_session(&self) -> Result<WasmRealtimeSession, JsValue> {
        self.realtime_session_with_request_bars_and_input_overrides("{}", "{}")
    }

    #[wasm_bindgen(js_name = realtimeSessionWithRequestBars)]
    pub fn realtime_session_with_request_bars(
        &self,
        request_bars_json: &str,
    ) -> Result<WasmRealtimeSession, JsValue> {
        self.realtime_session_with_request_bars_and_input_overrides(request_bars_json, "{}")
    }

    #[wasm_bindgen(js_name = realtimeSessionWithRequestBarsAndInputOverrides)]
    pub fn realtime_session_with_request_bars_and_input_overrides(
        &self,
        request_bars_json: &str,
        input_overrides_json: &str,
    ) -> Result<WasmRealtimeSession, JsValue> {
        WasmRealtimeSession::from_program(self.hir.clone(), request_bars_json, input_overrides_json)
            .map_err(|err| JsValue::from_str(&err))
    }
}

fn context_from_json(json: &str) -> Result<RealtimeUpdateContext, String> {
    let trimmed = json.trim();
    if trimmed.is_empty() || trimmed == "{}" {
        return Ok(RealtimeUpdateContext::default());
    }
    let value: Value = serde_json::from_str(trimmed)
        .map_err(|err| format!("realtime update context must be a JSON object: {err}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| "realtime update context must be a JSON object".to_owned())?;
    if let Some(field) = object
        .keys()
        .find(|field| !matches!(field.as_str(), "executionTime" | "openingUpdate"))
    {
        return Err(format!(
            "realtime update context has unknown field `{field}`"
        ));
    }
    let mut context = RealtimeUpdateContext::default();
    if let Some(value) = object.get("executionTime") {
        context.execution_time =
            Some(value.as_i64().ok_or_else(|| {
                "executionTime must be an integer millisecond timestamp".to_owned()
            })?);
    }
    if let Some(value) = object.get("openingUpdate") {
        context.opening_update = Some(
            value
                .as_bool()
                .ok_or_else(|| "openingUpdate must be a bool".to_owned())?,
        );
    }
    Ok(context)
}

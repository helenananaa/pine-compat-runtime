//! Host-neutral external evaluator binding.
use crate::{
    PyProgram, parse_bars, parse_execution_times, parse_input_overrides, parse_request_environment,
    runtime_result_view_to_py,
};
use pine_runtime::HistoricalRuntime;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBool, PyDict, PyInt, PyModule};

pub(crate) fn register_schemas(module: &Bound<'_, PyModule>) -> PyResult<()> {
    for (name, version) in [
        ("EXTERNAL_CLOCK_SCHEMA_VERSION", 1),
        (
            "EXTERNAL_REQUEST_UPDATE_SCHEMA_VERSION",
            pine_runtime::EXTERNAL_REQUEST_UPDATE_SCHEMA_VERSION,
        ),
        (
            "EXTERNAL_SCHEDULE_SCHEMA_VERSION",
            pine_runtime::EXTERNAL_SCHEDULE_SCHEMA_VERSION,
        ),
        (
            "STRATEGY_RISK_CONFIGURATION_SCHEMA_VERSION",
            pine_runtime::STRATEGY_RISK_CONFIGURATION_SCHEMA_VERSION,
        ),
        (
            "EXTERNAL_CLOSING_TICK_SCHEMA_VERSION",
            pine_runtime::EXTERNAL_CLOSING_TICK_SCHEMA_VERSION,
        ),
        (
            "EXTERNAL_ORDER_SCHEMA_VERSION",
            pine_runtime::EXTERNAL_ORDER_SCHEMA_VERSION,
        ),
        (
            "EXTERNAL_SIZING_SCHEMA_VERSION",
            pine_runtime::EXTERNAL_SIZING_SCHEMA_VERSION,
        ),
        (
            "EXTERNAL_ORDER_METADATA_SCHEMA_VERSION",
            pine_runtime::EXTERNAL_ORDER_METADATA_SCHEMA_VERSION,
        ),
    ] {
        module.add(name, version)?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn run(
    program: &PyProgram,
    py: Python<'_>,
    bars: &Bound<'_, PyAny>,
    accounts: &Bound<'_, PyAny>,
    input_overrides: Option<&Bound<'_, PyAny>>,
    chart_symbol: Option<&str>,
    chart_timeframe: Option<&str>,
    execution_passes: Option<&Bound<'_, PyAny>>,
    request_bars: Option<&Bound<'_, PyAny>>,
    execution_times: Option<&Bound<'_, PyAny>>,
    closing_ticks: bool,
    risk_rules: bool,
    execution_schedule: Option<&Bound<'_, PyAny>>,
    order_metadata: bool,
) -> PyResult<Py<PyAny>> {
    let bars = parse_bars(bars)?;
    let execution_times = parse_execution_times(execution_times)?;
    let json = PyModule::import(py, "json")?;
    let raw: String = json.call_method1("dumps", (accounts,))?.extract()?;
    let frames: Vec<pine_runtime::ExternalAccountFrame> =
        serde_json::from_str(&raw).map_err(|err| PyValueError::new_err(err.to_string()))?;
    if frames.len() != bars.len() {
        return Err(PyValueError::new_err("E_EXTERNAL_FEEDBACK_COUNT"));
    }
    let passes = execution_passes
        .map(|value| -> PyResult<Vec<Vec<pine_runtime::ExternalPass>>> {
            let raw: String = json.call_method1("dumps", (value,))?.extract()?;
            serde_json::from_str(&raw).map_err(|err| PyValueError::new_err(err.to_string()))
        })
        .transpose()?;
    if execution_passes.is_some() && execution_schedule.is_some() {
        return Err(PyValueError::new_err(
            "E_EXTERNAL_UNSUPPORTED: pass formats are mutually exclusive",
        ));
    }
    let schedule = parse_schedule(&json, execution_schedule)?;
    let runtime = HistoricalRuntime::from_prepared_with_request_environment_and_input_overrides(
        &program.hir,
        parse_request_environment(request_bars, chart_symbol, chart_timeframe)?,
        parse_input_overrides(input_overrides, &program.hir)?,
    );
    let options = pine_runtime::ExternalExecutionOptions {
        closing_ticks,
        risk_rules,
        order_metadata,
    };
    let mut runtime = match schedule {
        Some(schedule) => runtime.with_external_schedule(frames, schedule, options),
        None => runtime.with_external_accounts_and_options(frames, passes, options),
    }
    .map_err(|err| PyValueError::new_err(err.message))?;
    py.detach(|| match execution_times.as_deref() {
        Some(times) => runtime.append_bars_with_execution_times(&bars, times),
        None => runtime.append_bars(&bars),
    })
    .map_err(|err| PyValueError::new_err(err.message))?;
    let intents = runtime
        .external_intents()
        .map_err(|err| PyValueError::new_err(err.message))?;
    let result = PyDict::new(py);
    if let Some(configuration) = runtime.external_risk_configuration() {
        result.set_item(
            "risk_rules",
            json.call_method1(
                "loads",
                (serde_json::to_string(configuration)
                    .map_err(|err| PyValueError::new_err(err.to_string()))?,),
            )?,
        )?;
    }
    result.set_item("protocol", "external-broker/1")?;
    result.set_item("pyramiding", program.hir.strategy_settings.pyramiding_limit)?;
    result.set_item(
        "intents",
        json.call_method1(
            "loads",
            (serde_json::to_string(intents)
                .map_err(|err| PyValueError::new_err(err.to_string()))?,),
        )?,
    )?;
    let mut output = runtime.result_view();
    output.strategy = None;
    result.set_item("output", runtime_result_view_to_py(py, &output)?)?;
    Ok(result.into_any().unbind())
}

fn parse_schedule(
    json: &Bound<'_, PyModule>,
    value: Option<&Bound<'_, PyAny>>,
) -> PyResult<Option<Vec<Vec<pine_runtime::ExternalScheduledPass>>>> {
    value
        .map(|value| {
            let raw: String = json.call_method1("dumps", (value,))?.extract()?;
            serde_json::from_str(&raw).map_err(|error| PyValueError::new_err(error.to_string()))
        })
        .transpose()
}

/// Process-local external evaluator. Failed execution poisons the session.
#[pyclass(name = "ExternalSession", skip_from_py_object)]
pub(crate) struct ExternalSession {
    runtime: HistoricalRuntime<'static>,
    pyramiding: usize,
    cursor: usize,
    failed: bool,
    scheduled: bool,
}
impl ExternalSession {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        program: &PyProgram,
        overrides: Option<&Bound<'_, PyAny>>,
        symbol: Option<&str>,
        timeframe: Option<&str>,
        requests: Option<&Bound<'_, PyAny>>,
        closing_ticks: bool,
        risk_rules: bool,
        scheduled: bool,
        order_metadata: bool,
    ) -> PyResult<Self> {
        let runtime = HistoricalRuntime::from_prepared_with_request_environment_and_input_overrides(
            &program.hir,
            parse_request_environment(requests, symbol, timeframe)?,
            parse_input_overrides(overrides, &program.hir)?,
        );
        let options = pine_runtime::ExternalExecutionOptions {
            closing_ticks,
            risk_rules,
            order_metadata,
        };
        let runtime = if scheduled {
            runtime.with_external_schedule(vec![], vec![], options)
        } else {
            runtime.with_external_accounts_and_options(
                vec![],
                program
                    .hir
                    .strategy_settings
                    .calc_on_order_fills
                    .then(Vec::new),
                options,
            )
        }
        .map_err(|err| PyValueError::new_err(err.message))?;
        Ok(Self {
            runtime,
            pyramiding: program.hir.strategy_settings.pyramiding_limit,
            cursor: 0,
            failed: false,
            scheduled,
        })
    }
}
#[pymethods]
impl ExternalSession {
    /// Feed-only transaction; invalid input does not poison the session.
    fn apply_request_updates(
        &mut self,
        py: Python<'_>,
        updates: &Bound<'_, PyAny>,
        observed_at_ms: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        if self.failed {
            return Err(PyValueError::new_err(
                "E_RUNTIME_POISONED: external session failed; create a new session",
            ));
        }
        if !observed_at_ms.is_instance_of::<PyInt>() || observed_at_ms.is_instance_of::<PyBool>() {
            return Err(PyValueError::new_err(
                "requested-data observation clock must be an integer millisecond timestamp",
            ));
        }
        let observed_at_ms = observed_at_ms.extract::<i64>()?;
        let json = PyModule::import(py, "json")?;
        let raw: String = json.call_method1("dumps", (updates,))?.extract()?;
        let updates: Vec<pine_runtime::ObservedRequestUpdate> =
            serde_json::from_str(&raw).map_err(|error| PyValueError::new_err(error.to_string()))?;
        py.detach(|| {
            self.runtime
                .apply_external_request_updates(&updates, observed_at_ms)
        })
        .map_err(|error| PyValueError::new_err(error.message))
    }

    #[pyo3(signature=(bars, accounts, execution_passes=None, execution_times=None, *, execution_schedule=None))]
    fn append(
        &mut self,
        py: Python<'_>,
        bars: &Bound<'_, PyAny>,
        accounts: &Bound<'_, PyAny>,
        execution_passes: Option<&Bound<'_, PyAny>>,
        execution_times: Option<&Bound<'_, PyAny>>,
        execution_schedule: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Py<PyAny>> {
        if self.failed {
            return Err(PyValueError::new_err(
                "E_RUNTIME_POISONED: external session failed; create a new session",
            ));
        }
        let bars = parse_bars(bars)?;
        let execution_times = parse_execution_times(execution_times)?;
        let json = PyModule::import(py, "json")?;
        let raw: String = json.call_method1("dumps", (accounts,))?.extract()?;
        let frames =
            serde_json::from_str(&raw).map_err(|err| PyValueError::new_err(err.to_string()))?;
        let passes = execution_passes
            .map(|value| -> PyResult<Vec<Vec<pine_runtime::ExternalPass>>> {
                let raw: String = json.call_method1("dumps", (value,))?.extract()?;
                serde_json::from_str(&raw).map_err(|err| PyValueError::new_err(err.to_string()))
            })
            .transpose()?;
        if self.scheduled != execution_schedule.is_some()
            || (self.scheduled && execution_passes.is_some())
        {
            return Err(PyValueError::new_err(
                "E_EXTERNAL_UNSUPPORTED: external schedule mode changed",
            ));
        }
        let schedule = parse_schedule(&json, execution_schedule)?;
        if let Err(err) = py.detach(|| match schedule {
            Some(schedule) => self.runtime.append_external_schedule(
                &bars,
                frames,
                schedule,
                execution_times.as_deref(),
            ),
            None => match execution_times.as_deref() {
                Some(times) => self
                    .runtime
                    .append_external_bars_with_execution_times(&bars, frames, passes, times),
                None => self.runtime.append_external_bars(&bars, frames, passes),
            },
        }) {
            self.failed = true;
            return Err(PyValueError::new_err(err.message));
        }
        self.cursor += bars.len();
        let result = PyDict::new(py);
        if let Some(configuration) = self.runtime.external_risk_configuration() {
            result.set_item(
                "risk_rules",
                json.call_method1(
                    "loads",
                    (serde_json::to_string(configuration)
                        .map_err(|err| PyValueError::new_err(err.to_string()))?,),
                )?,
            )?;
        }
        result.set_item("protocol", "external-broker/1")?;
        result.set_item("pyramiding", self.pyramiding)?;
        let intents = self.runtime.external_intents().map_err(|err| {
            self.failed = true;
            PyValueError::new_err(err.message)
        })?;
        result.set_item(
            "intents",
            json.call_method1(
                "loads",
                (serde_json::to_string(intents)
                    .map_err(|err| PyValueError::new_err(err.to_string()))?,),
            )?,
        )?;
        let mut output = self.runtime.result_view();
        output.strategy = None;
        result.set_item("output", runtime_result_view_to_py(py, &output)?)?;
        Ok(result.into_any().unbind())
    }
    #[getter]
    fn cursor(&self) -> usize {
        self.cursor
    }
}

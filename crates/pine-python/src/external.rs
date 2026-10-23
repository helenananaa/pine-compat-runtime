//! Host-neutral external evaluator binding.
use crate::{
    PyProgram, parse_bars, parse_input_overrides, parse_request_environment,
    runtime_result_view_to_py,
};
use pine_runtime::HistoricalRuntime;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyModule};

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
) -> PyResult<Py<PyAny>> {
    let bars = parse_bars(bars)?;
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
    let mut runtime =
        HistoricalRuntime::from_prepared_with_request_environment_and_input_overrides(
            &program.hir,
            parse_request_environment(request_bars, chart_symbol, chart_timeframe)?,
            parse_input_overrides(input_overrides, &program.hir)?,
        )
        .with_external_accounts_and_passes(frames, passes)
        .map_err(|err| PyValueError::new_err(err.message))?;
    py.detach(|| runtime.append_bars(&bars))
        .map_err(|err| PyValueError::new_err(err.message))?;
    let intents = runtime
        .external_intents()
        .map_err(|err| PyValueError::new_err(err.message))?;
    let result = PyDict::new(py);
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

/// Process-local external evaluator. Failed execution poisons the session.
#[pyclass(name = "ExternalSession", skip_from_py_object)]
pub(crate) struct ExternalSession {
    runtime: HistoricalRuntime<'static>,
    pyramiding: usize,
    cursor: usize,
    failed: bool,
}
impl ExternalSession {
    pub(crate) fn new(
        program: &PyProgram,
        overrides: Option<&Bound<'_, PyAny>>,
        symbol: Option<&str>,
        timeframe: Option<&str>,
        requests: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        let runtime =
            HistoricalRuntime::from_prepared_with_request_environment_and_input_overrides(
                &program.hir,
                parse_request_environment(requests, symbol, timeframe)?,
                parse_input_overrides(overrides, &program.hir)?,
            )
            .with_external_accounts_and_passes(
                vec![],
                if program.hir.strategy_settings.calc_on_order_fills {
                    Some(vec![])
                } else {
                    None
                },
            )
            .map_err(|err| PyValueError::new_err(err.message))?;
        Ok(Self {
            runtime,
            pyramiding: program.hir.strategy_settings.pyramiding_limit,
            cursor: 0,
            failed: false,
        })
    }
}
#[pymethods]
impl ExternalSession {
    #[pyo3(signature=(bars, accounts, execution_passes=None))]
    fn append(
        &mut self,
        py: Python<'_>,
        bars: &Bound<'_, PyAny>,
        accounts: &Bound<'_, PyAny>,
        execution_passes: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Py<PyAny>> {
        if self.failed {
            return Err(PyValueError::new_err(
                "E_RUNTIME_POISONED: external session failed; create a new session",
            ));
        }
        let bars = parse_bars(bars)?;
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
        if let Err(err) = py.detach(|| self.runtime.append_external_bars(&bars, frames, passes)) {
            self.failed = true;
            return Err(PyValueError::new_err(err.message));
        }
        self.cursor += bars.len();
        let result = PyDict::new(py);
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

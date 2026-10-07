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

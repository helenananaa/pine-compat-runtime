//! Side-effect-free input and risk metadata bindings.
use crate::outputs::value_to_py;
use crate::{PyProgram, parse_input_overrides, parse_request_environment};
use pine_runtime::input_calls;
use pine_sema::Analysis;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyList, PyModule};

pub(crate) fn risk_configuration(
    program: &PyProgram,
    py: Python<'_>,
    input_overrides: Option<&Bound<'_, PyAny>>,
    chart_symbol: Option<&str>,
    chart_timeframe: Option<&str>,
    request_bars: Option<&Bound<'_, PyAny>>,
) -> PyResult<Py<PyAny>> {
    let environment = parse_request_environment(request_bars, chart_symbol, chart_timeframe)?;
    let overrides = parse_input_overrides(input_overrides, &program.hir)?;
    let value =
        pine_runtime::strategy_risk_configuration(&program.hir, &overrides, environment.chart());
    Ok(PyModule::import(py, "json")?
        .call_method1(
            "loads",
            (serde_json::to_string(&value)
                .map_err(|err| PyValueError::new_err(err.to_string()))?,),
        )?
        .unbind())
}

pub(crate) fn inputs_to_py(py: Python<'_>, analysis: &Analysis) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    if let Some(hir) = &analysis.hir {
        for input in input_calls(hir) {
            let item = PyDict::new(py);
            item.set_item("callSiteId", input.call_site_id)?;
            item.set_item("name", input.name)?;
            item.set_item("isSource", input.is_source)?;
            item.set_item("title", input.title)?;
            match &input.default_value {
                Some(value) => item.set_item("default", value_to_py(py, value)?)?,
                None => item.set_item("default", py.None())?,
            }
            if let Some(value) = &input.min_value {
                item.set_item("min", value_to_py(py, value)?)?;
            }
            if let Some(value) = &input.max_value {
                item.set_item("max", value_to_py(py, value)?)?;
            }
            if let Some(value) = &input.step {
                item.set_item("step", value_to_py(py, value)?)?;
            }
            if !input.options.is_empty() {
                let options = PyList::empty(py);
                for value in &input.options {
                    options.append(value_to_py(py, value)?)?;
                }
                item.set_item("options", options)?;
            }
            output.append(item)?;
        }
    }
    Ok(output.into_any().unbind())
}

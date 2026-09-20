//! Process-local historical state, independent of host storage and scheduling.
use pine_runtime::{Bar, HistoricalRuntime};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use std::sync::Arc;

#[pyclass(name = "HistoricalSession", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct HistoricalSession {
    runtime: HistoricalRuntime<'static>,
    bars: Arc<Vec<Bar>>,
    cursor: usize,
}
impl HistoricalSession {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        program: &crate::PyProgram,
        bars: &Bound<'_, PyAny>,
        overrides: Option<&Bound<'_, PyAny>>,
        symbol: Option<&str>,
        timeframe: Option<&str>,
        request_bars: Option<&Bound<'_, PyAny>>,
        magnifier_bars: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        let py = bars.py();
        let bars = crate::parse_bars(bars)?;
        if bars.is_empty() {
            return Err(PyValueError::new_err("history cannot be empty"));
        }
        let mut runtime =
            HistoricalRuntime::with_owned_program_and_request_environment_and_input_overrides(
                program.hir.clone(),
                crate::parse_request_environment(request_bars, symbol, timeframe)?,
                crate::parse_input_overrides(overrides, &program.hir)?,
            );
        if let Some(magnifier) = crate::parse_magnifier_bars(py, magnifier_bars)? {
            runtime = runtime.with_magnifier_input(magnifier);
        }
        runtime
            .set_historical_horizon(bars.len(), bars.last().unwrap().time)
            .map_err(|e| PyValueError::new_err(e.message))?;
        Ok(Self {
            runtime,
            bars: Arc::new(bars),
            cursor: 0,
        })
    }
}
#[pymethods]
impl HistoricalSession {
    fn advance(&mut self, py: Python<'_>, target: usize) -> PyResult<Py<PyAny>> {
        if target < self.cursor || target > self.bars.len() {
            return Err(PyValueError::new_err(
                "historical cursor outside forward range",
            ));
        }
        while self.cursor < target {
            self.runtime
                .append_bar(self.bars[self.cursor])
                .map_err(|e| PyValueError::new_err(e.message))?;
            self.cursor += 1;
        }
        crate::outputs::runtime_result_to_py(py, &self.runtime.result())
    }
    fn fork(&self) -> Self {
        self.clone()
    }
    #[getter]
    fn cursor(&self) -> usize {
        self.cursor
    }
}

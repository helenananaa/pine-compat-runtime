//! Process-local historical state, independent of host storage and scheduling.
use pine_runtime::{Bar, HistoricalRuntime};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBool};
use std::sync::Arc;

#[pyclass(name = "HistoricalSession", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct HistoricalSession {
    runtime: HistoricalRuntime<'static>,
    bars: Arc<Vec<Bar>>,
    execution_times: Option<Arc<Vec<i64>>>,
    first_execution_index: usize,
    cursor: usize,
    failed: bool,
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
        execution_times: Option<&Bound<'_, PyAny>>,
        session_windows: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        let py = bars.py();
        let bars = crate::parse_bars(bars)?;
        if bars.is_empty() {
            return Err(PyValueError::new_err("history cannot be empty"));
        }
        let execution_times = parse_execution_times(execution_times)?;
        if execution_times
            .as_ref()
            .is_some_and(|times| times.len() != bars.len())
        {
            return Err(PyValueError::new_err(
                "execution_times must match the historical bar count",
            ));
        }
        let first_execution_index = program
            .hir
            .calc_bars_count
            .filter(|count| *count > 0)
            .map_or(0, |count| bars.len().saturating_sub(count as usize));
        let mut runtime =
            HistoricalRuntime::from_prepared_with_request_environment_and_input_overrides(
                &program.hir,
                crate::parse_request_environment(request_bars, symbol, timeframe)?,
                crate::parse_input_overrides(overrides, &program.hir)?,
            );
        if let Some(magnifier) = crate::parse_magnifier_bars(py, magnifier_bars)? {
            runtime = runtime.with_magnifier_input(magnifier);
        }
        if let Some(windows) = crate::parse_session_windows(py, session_windows)? {
            runtime = runtime
                .with_session_windows(windows)
                .map_err(|err| PyValueError::new_err(err.message))?;
        }
        runtime
            .set_historical_horizon(
                bars.len() - first_execution_index,
                bars.last().unwrap().time,
            )
            .map_err(|e| PyValueError::new_err(e.message))?;
        Ok(Self {
            runtime,
            bars: Arc::new(bars),
            execution_times: execution_times.map(Arc::new),
            first_execution_index,
            cursor: 0,
            failed: false,
        })
    }
}
#[pymethods]
impl HistoricalSession {
    fn advance(&mut self, py: Python<'_>, target: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        self.require_ready()?;
        let target = if target.is_instance_of::<PyBool>() {
            None
        } else {
            target.extract::<usize>().ok()
        }
        .ok_or_else(|| PyValueError::new_err("historical cursor must be a nonnegative integer"))?;
        if target < self.cursor || target > self.bars.len() {
            return Err(PyValueError::new_err(
                "historical cursor outside forward range",
            ));
        }
        let result: Result<(), pine_runtime::RuntimeError> = py.detach(|| {
            while self.cursor < target {
                if self.cursor >= self.first_execution_index {
                    let bar = self.bars[self.cursor];
                    match &self.execution_times {
                        Some(times) => self
                            .runtime
                            .append_bar_with_execution_time(bar, times[self.cursor])?,
                        None => self.runtime.append_bar(bar)?,
                    }
                }
                self.cursor += 1;
            }
            Ok(())
        });
        if let Err(error) = result {
            self.failed = true;
            return Err(PyValueError::new_err(error.message));
        }
        crate::outputs::runtime_result_view_to_py(py, &self.runtime.result_view())
    }
    fn fork(&self) -> PyResult<Self> {
        self.require_ready()?;
        Ok(self.clone())
    }
    #[getter]
    fn cursor(&self) -> usize {
        self.cursor
    }
}

impl HistoricalSession {
    fn require_ready(&self) -> PyResult<()> {
        if self.failed {
            Err(PyValueError::new_err(
                "E_RUNTIME_POISONED: historical session failed; create a new session",
            ))
        } else {
            Ok(())
        }
    }
}

pub(crate) fn parse_execution_times(
    execution_times: Option<&Bound<'_, PyAny>>,
) -> PyResult<Option<Vec<i64>>> {
    let Some(execution_times) = execution_times else {
        return Ok(None);
    };
    let iterator = execution_times.try_iter().map_err(|_| {
        PyValueError::new_err(
            "execution_times must be a sequence of integer millisecond timestamps",
        )
    })?;
    let mut parsed = Vec::new();
    for (index, value) in iterator.enumerate() {
        let value = value?;
        if value.is_instance_of::<PyBool>() {
            return Err(PyValueError::new_err(format!(
                "execution_times[{index}] must be an integer millisecond timestamp"
            )));
        }
        parsed.push(value.extract::<i64>().map_err(|_| {
            PyValueError::new_err(format!(
                "execution_times[{index}] must be an integer millisecond timestamp"
            ))
        })?);
    }
    Ok(Some(parsed))
}

pub(crate) fn parse_bars(bars: &Bound<'_, PyAny>) -> PyResult<Vec<Bar>> {
    let mut parsed = Vec::new();
    for item in bars.try_iter()? {
        let item = item?;
        parsed.push(crate::parse_bar(&item)?);
    }
    validate_bar_times(&parsed)?;
    Ok(parsed)
}

pub(crate) fn validate_bar_times(bars: &[Bar]) -> PyResult<()> {
    let mut previous_time = None;
    for bar in bars {
        if let Some(previous) = previous_time {
            if bar.time == previous {
                return Err(PyValueError::new_err(format!(
                    "duplicate bar time `{}`",
                    bar.time
                )));
            }
            if bar.time < previous {
                return Err(PyValueError::new_err(format!(
                    "bars are not sorted: `{}` follows `{previous}`",
                    bar.time
                )));
            }
        }
        previous_time = Some(bar.time);
    }
    Ok(())
}

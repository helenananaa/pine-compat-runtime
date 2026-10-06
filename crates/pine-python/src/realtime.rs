use pine_runtime::{
    BarUpdate, InputOverrides, PreparedProgram, RealtimeRuntime, RealtimeUpdateContext,
    RequestEnvironment, RequestKey, RequestTimeframe, ValueWhenLimits,
};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBool, PyModule};

use crate::changes::runtime_changes_to_py;
use crate::outputs::runtime_result_view_to_py;
use crate::{compile_script, parse_bar, parse_bars, parse_execution_times};

fn execution_timestamp(value: Option<&Bound<'_, PyAny>>) -> PyResult<Option<i64>> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_instance_of::<PyBool>() {
        return Err(PyValueError::new_err(
            "execution_time must be an integer millisecond timestamp",
        ));
    }
    value.extract::<i64>().map(Some).map_err(|_| {
        PyValueError::new_err("execution_time must be an integer millisecond timestamp")
    })
}

pub(crate) const REALTIME_SESSION_SCHEMA_VERSION: u32 = 1;

fn opening_update_flag(value: Option<&Bound<'_, PyAny>>) -> PyResult<Option<bool>> {
    let Some(value) = value else {
        return Ok(None);
    };
    if !value.is_instance_of::<PyBool>() {
        return Err(PyValueError::new_err("opening_update must be a bool"));
    }
    value.extract::<bool>().map(Some)
}

#[pyclass(name = "RealtimeSession", skip_from_py_object)]
pub(crate) struct PyRealtimeSession {
    runtime: RealtimeRuntime<'static>,
    seeded: bool,
}

impl PyRealtimeSession {
    pub(crate) fn new(
        hir: PreparedProgram,
        request_environment: RequestEnvironment,
        input_overrides: InputOverrides,
        magnifier: Option<pine_runtime::MagnifierInput>,
        session_windows: Option<pine_runtime::SessionWindowInput>,
    ) -> PyResult<Self> {
        let mut runtime =
            RealtimeRuntime::from_prepared_with_request_environment_and_input_overrides(
                &hir,
                request_environment,
                input_overrides,
            );
        if let Some(magnifier) = magnifier {
            runtime = runtime.with_magnifier_input(magnifier);
        }
        if let Some(session_windows) = session_windows {
            runtime = runtime
                .with_session_windows(session_windows)
                .map_err(|err| PyValueError::new_err(err.message))?;
        }
        Ok(Self {
            runtime,
            seeded: false,
        })
    }

    fn require_seeded(&self) -> PyResult<()> {
        if self.seeded {
            Ok(())
        } else {
            Err(PyValueError::new_err(
                "realtime session must be seeded before updates",
            ))
        }
    }

    fn apply_request_update(
        &mut self,
        py: Python<'_>,
        symbol: &str,
        timeframe: &str,
        bar: &Bound<'_, PyAny>,
        forming: bool,
    ) -> PyResult<Py<PyAny>> {
        self.require_seeded()?;
        let timeframe = RequestTimeframe::parse(timeframe)
            .map_err(|err| PyValueError::new_err(err.to_string()))?;
        let key = RequestKey::new(symbol, timeframe);
        let bar = parse_bar(bar)?;
        let update = if forming {
            BarUpdate::forming(bar)
        } else {
            BarUpdate::confirmed(bar)
        };
        match py
            .detach(|| self.runtime.apply_request_update_ref(key, update))
            .map_err(|err| PyValueError::new_err(err.message))?
        {
            Some(changes) => runtime_changes_to_py(py, changes),
            None => Ok(py.None()),
        }
    }
}

#[pymethods]
impl PyRealtimeSession {
    /// Bound logical valuewhen events; None keeps aggregate retention unlimited.
    #[pyo3(signature = (limit=None))]
    fn set_valuewhen_limit(&mut self, limit: Option<&Bound<'_, PyAny>>) -> PyResult<()> {
        let max_retained_values = limit
            .map(|value| {
                if value.is_instance_of::<PyBool>() {
                    return Err(PyValueError::new_err(
                        "limit must be a nonnegative integer that fits this platform",
                    ));
                }
                value.extract::<usize>().map_err(|_| {
                    PyValueError::new_err(
                        "limit must be a nonnegative integer that fits this platform",
                    )
                })
            })
            .transpose()?;
        self.runtime
            .set_valuewhen_limits(ValueWhenLimits {
                max_retained_values,
            })
            .map_err(|error| PyValueError::new_err(error.message))
    }

    #[getter]
    fn valuewhen_limit(&self) -> Option<usize> {
        self.runtime.valuewhen_limits().max_retained_values
    }

    #[getter]
    fn valuewhen_retained_values(&self) -> usize {
        self.runtime.valuewhen_retained_values()
    }

    #[getter]
    fn confirmed_valuewhen_retained_values(&self) -> usize {
        self.runtime.confirmed_valuewhen_retained_values()
    }

    fn extend_session_windows(
        &mut self,
        py: Python<'_>,
        session_windows: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let input = crate::parse_session_windows(py, Some(session_windows))?
            .ok_or_else(|| PyValueError::new_err("session window input is required"))?;
        self.runtime
            .extend_session_windows(input)
            .map_err(|err| PyValueError::new_err(err.message))
    }

    #[pyo3(signature = (bars, *, execution_times=None))]
    fn seed(
        &mut self,
        py: Python<'_>,
        bars: &Bound<'_, PyAny>,
        execution_times: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Py<PyAny>> {
        if self.seeded {
            return Err(PyValueError::new_err(
                "realtime session history has already been seeded",
            ));
        }
        let bars = parse_bars(bars)?;
        let times = parse_execution_times(execution_times)?;
        py.detach(|| match times {
            Some(times) => self
                .runtime
                .seed_historical_with_execution_times_without_output(&bars, &times),
            None => self.runtime.seed_historical_without_output(&bars),
        })
        .map_err(|err| PyValueError::new_err(err.message))?;
        self.seeded = true;
        runtime_result_view_to_py(py, &self.runtime.result_view())
    }

    /// Seed a session without constructing an owned Python snapshot.
    #[pyo3(signature = (bars, *, execution_times=None))]
    fn seed_state(
        &mut self,
        py: Python<'_>,
        bars: &Bound<'_, PyAny>,
        execution_times: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<()> {
        if self.seeded {
            return Err(PyValueError::new_err(
                "realtime session history has already been seeded",
            ));
        }
        let bars = parse_bars(bars)?;
        let times = parse_execution_times(execution_times)?;
        py.detach(|| match times {
            Some(times) => self
                .runtime
                .seed_historical_with_execution_times_without_output(&bars, &times),
            None => self.runtime.seed_historical_without_output(&bars),
        })
        .map_err(|err| PyValueError::new_err(err.message))?;
        self.seeded = true;
        Ok(())
    }

    #[pyo3(signature = (bars, *, execution_times=None))]
    fn replay(
        &mut self,
        py: Python<'_>,
        bars: &Bound<'_, PyAny>,
        execution_times: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Py<PyAny>> {
        self.require_seeded()?;
        let bars = parse_bars(bars)?;
        let times = parse_execution_times(execution_times)?;
        py.detach(|| match times {
            Some(times) => self
                .runtime
                .replay_historical_with_execution_times_without_output(&bars, &times),
            None => self.runtime.replay_historical_without_output(&bars),
        })
        .map_err(|err| PyValueError::new_err(err.message))?;
        runtime_result_view_to_py(py, &self.runtime.result_view())
    }

    #[pyo3(signature = (from_time, bars, *, execution_times=None))]
    fn correct(
        &mut self,
        py: Python<'_>,
        from_time: &Bound<'_, PyAny>,
        bars: &Bound<'_, PyAny>,
        execution_times: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Py<PyAny>> {
        self.require_seeded()?;
        if from_time.is_instance_of::<PyBool>() {
            return Err(PyValueError::new_err(
                "from_time must be an integer millisecond timestamp",
            ));
        }
        let from_time = from_time.extract::<i64>().map_err(|_| {
            PyValueError::new_err("from_time must be an integer millisecond timestamp")
        })?;
        let bars = parse_bars(bars)?;
        let times = parse_execution_times(execution_times)?;
        py.detach(|| match times {
            Some(times) => self
                .runtime
                .correct_historical_with_execution_times_without_output(from_time, &bars, &times),
            None => self
                .runtime
                .correct_historical_without_output(from_time, &bars),
        })
        .map_err(|err| PyValueError::new_err(err.message))?;
        runtime_result_view_to_py(py, &self.runtime.result_view())
    }

    #[pyo3(signature = (bar, *, execution_time=None, opening_update=None))]
    fn update_forming(
        &mut self,
        py: Python<'_>,
        bar: &Bound<'_, PyAny>,
        execution_time: Option<&Bound<'_, PyAny>>,
        opening_update: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Py<PyAny>> {
        self.require_seeded()?;
        let bar = parse_bar(bar)?;
        let context = RealtimeUpdateContext {
            execution_time: execution_timestamp(execution_time)?,
            opening_update: opening_update_flag(opening_update)?,
        };
        py.detach(|| {
            self.runtime
                .update_with_context_without_output(BarUpdate::forming(bar), context)
        })
        .map_err(|err| PyValueError::new_err(err.message))?;
        runtime_result_view_to_py(py, &self.runtime.result_view())
    }

    #[pyo3(signature = (bar, *, execution_time=None, opening_update=None))]
    fn update_confirmed(
        &mut self,
        py: Python<'_>,
        bar: &Bound<'_, PyAny>,
        execution_time: Option<&Bound<'_, PyAny>>,
        opening_update: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Py<PyAny>> {
        self.require_seeded()?;
        let bar = parse_bar(bar)?;
        let context = RealtimeUpdateContext {
            execution_time: execution_timestamp(execution_time)?,
            opening_update: opening_update_flag(opening_update)?,
        };
        py.detach(|| {
            self.runtime
                .update_with_context_without_output(BarUpdate::confirmed(bar), context)
        })
        .map_err(|err| PyValueError::new_err(err.message))?;
        runtime_result_view_to_py(py, &self.runtime.result_view())
    }

    #[pyo3(signature = (bar, *, execution_time=None, opening_update=None))]
    fn apply_forming(
        &mut self,
        py: Python<'_>,
        bar: &Bound<'_, PyAny>,
        execution_time: Option<&Bound<'_, PyAny>>,
        opening_update: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Py<PyAny>> {
        self.require_seeded()?;
        let bar = parse_bar(bar)?;
        let context = RealtimeUpdateContext {
            execution_time: execution_timestamp(execution_time)?,
            opening_update: opening_update_flag(opening_update)?,
        };
        let changes = py
            .detach(|| {
                self.runtime
                    .apply_update_with_context_ref(BarUpdate::forming(bar), context)
            })
            .map_err(|err| PyValueError::new_err(err.message))?;
        runtime_changes_to_py(py, changes)
    }

    #[pyo3(signature = (bar, *, execution_time=None, opening_update=None))]
    fn apply_confirmed(
        &mut self,
        py: Python<'_>,
        bar: &Bound<'_, PyAny>,
        execution_time: Option<&Bound<'_, PyAny>>,
        opening_update: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Py<PyAny>> {
        self.require_seeded()?;
        let bar = parse_bar(bar)?;
        let context = RealtimeUpdateContext {
            execution_time: execution_timestamp(execution_time)?,
            opening_update: opening_update_flag(opening_update)?,
        };
        let changes = py
            .detach(|| {
                self.runtime
                    .apply_update_with_context_ref(BarUpdate::confirmed(bar), context)
            })
            .map_err(|err| PyValueError::new_err(err.message))?;
        runtime_changes_to_py(py, changes)
    }

    fn apply_request_forming(
        &mut self,
        py: Python<'_>,
        symbol: &str,
        timeframe: &str,
        bar: &Bound<'_, PyAny>,
    ) -> PyResult<Py<PyAny>> {
        self.apply_request_update(py, symbol, timeframe, bar, true)
    }

    fn apply_request_confirmed(
        &mut self,
        py: Python<'_>,
        symbol: &str,
        timeframe: &str,
        bar: &Bound<'_, PyAny>,
    ) -> PyResult<Py<PyAny>> {
        self.apply_request_update(py, symbol, timeframe, bar, false)
    }

    fn replica(&self, py: Python<'_>) -> crate::replica::PyRuntimeReplica {
        crate::replica::PyRuntimeReplica {
            inner: py.detach(|| self.runtime.replica()),
        }
    }

    fn stream_snapshot(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let envelope = pyo3::types::PyDict::new(py);
        envelope.set_item("revision", self.runtime.revision())?;
        envelope.set_item("retainedFrom", self.runtime.display_origin())?;
        envelope.set_item(
            "result",
            runtime_result_view_to_py(py, &self.runtime.result_view())?,
        )?;
        Ok(envelope.into_any().unbind())
    }

    fn set_output_retention(&mut self, keep_confirmed_bars: Option<usize>) {
        self.runtime
            .set_output_retention(match keep_confirmed_bars {
                Some(count) => pine_runtime::OutputRetention::keep_confirmed_bars(count),
                None => pine_runtime::OutputRetention::unlimited(),
            });
    }

    #[getter]
    fn display_origin(&self) -> usize {
        self.runtime.display_origin()
    }

    fn result(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        runtime_result_view_to_py(py, &self.runtime.result_view())
    }

    fn confirmed_result(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        runtime_result_view_to_py(py, &self.runtime.confirmed_result_view())
    }

    #[getter]
    fn schema_version(&self) -> u32 {
        REALTIME_SESSION_SCHEMA_VERSION
    }

    #[getter]
    fn is_seeded(&self) -> bool {
        self.seeded
    }

    #[getter]
    fn confirmed_bars(&self) -> usize {
        self.runtime.confirmed_bar_count()
    }

    #[getter]
    fn last_confirmed_time(&self) -> Option<i64> {
        self.runtime.last_confirmed_bar_time()
    }

    #[getter]
    fn forming_time(&self) -> Option<i64> {
        self.runtime.forming_bar_time()
    }

    #[getter]
    fn revision(&self) -> u64 {
        self.runtime.revision()
    }

    fn last_changes(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        match self.runtime.last_changes() {
            Some(changes) => runtime_changes_to_py(py, changes),
            None => Ok(py.None()),
        }
    }
}

#[pyfunction(signature = (
    source,
    request_bars=None,
    library_sources=None,
    input_overrides=None,
    chart_symbol=None,
    chart_timeframe=None,
    magnifier_bars=None,
    session_windows=None
))]
#[allow(clippy::too_many_arguments)]
fn create_realtime_session(
    py: Python<'_>,
    source: &str,
    request_bars: Option<&Bound<'_, PyAny>>,
    library_sources: Option<&Bound<'_, PyAny>>,
    input_overrides: Option<&Bound<'_, PyAny>>,
    chart_symbol: Option<&str>,
    chart_timeframe: Option<&str>,
    magnifier_bars: Option<&Bound<'_, PyAny>>,
    session_windows: Option<&Bound<'_, PyAny>>,
) -> PyResult<PyRealtimeSession> {
    compile_script(source, library_sources)?.realtime_session(
        py,
        request_bars,
        input_overrides,
        chart_symbol,
        chart_timeframe,
        magnifier_bars,
        session_windows,
    )
}

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "REALTIME_SESSION_SCHEMA_VERSION",
        REALTIME_SESSION_SCHEMA_VERSION,
    )?;
    module.add_class::<PyRealtimeSession>()?;
    module.add_function(wrap_pyfunction!(create_realtime_session, module)?)?;
    Ok(())
}

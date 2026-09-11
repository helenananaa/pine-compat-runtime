use pine_runtime::RuntimeReplica;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyModule};

use crate::{
    changes::runtime_changes_from_py, outputs::runtime_result_to_py,
    result_parse::runtime_result_from_py,
};

#[pyclass(name = "RuntimeReplica", skip_from_py_object)]
pub(crate) struct PyRuntimeReplica {
    pub(crate) inner: RuntimeReplica,
}

#[pymethods]
impl PyRuntimeReplica {
    #[new]
    #[pyo3(signature = (result, *, revision))]
    fn new(py: Python<'_>, result: &Bound<'_, PyAny>, revision: u64) -> PyResult<Self> {
        Ok(Self {
            inner: RuntimeReplica::new(runtime_result_from_py(py, result)?, revision),
        })
    }

    /// Parse only this update and mutate the native result in place.
    fn apply(&mut self, py: Python<'_>, changes: &Bound<'_, PyAny>) -> PyResult<bool> {
        let changes = runtime_changes_from_py(py, changes)?;
        self.inner
            .apply(&changes)
            .map_err(|e| PyValueError::new_err(e.message))
    }

    #[getter]
    fn revision(&self) -> u64 {
        self.inner.revision()
    }

    /// Full conversion is opt-in, never part of apply().
    fn result(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        runtime_result_to_py(py, self.inner.result())
    }

    #[pyo3(signature = (result, *, revision))]
    fn reset(&mut self, py: Python<'_>, result: &Bound<'_, PyAny>, revision: u64) -> PyResult<()> {
        let result = runtime_result_from_py(py, result)?;
        self.inner.reset(result, revision);
        Ok(())
    }
}

/// The unreleased dictionary helper now requires a cursor-bearing replica.
#[pyfunction(name = "apply_runtime_changes")]
fn apply_runtime_changes_py(
    mut replica: PyRefMut<'_, PyRuntimeReplica>,
    changes: &Bound<'_, PyAny>,
) -> PyResult<bool> {
    replica.apply(changes.py(), changes)
}

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyRuntimeReplica>()?;
    module.add_function(wrap_pyfunction!(apply_runtime_changes_py, module)?)?;
    Ok(())
}

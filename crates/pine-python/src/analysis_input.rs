//! Compile inputs and supplied library sources.
use pine_sema::AnalysisInput;
use pine_syntax::SourceFile;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict};

pub(crate) fn analysis_input_from_python(
    source: &str,
    library_sources: Option<&Bound<'_, PyAny>>,
) -> PyResult<AnalysisInput> {
    let root = SourceFile::new("<python>", source);
    let Some(library_sources) = library_sources else {
        return Ok(AnalysisInput::new(root));
    };
    let dict = library_sources.cast::<PyDict>().map_err(|_| {
        PyValueError::new_err("library_sources must be a dict mapping import key to source text")
    })?;
    let mut sources = Vec::with_capacity(dict.len());
    for (key, value) in dict {
        let key: String = key.extract()?;
        let text: String = value.extract().map_err(|_| {
            PyValueError::new_err("library_sources values must be source text strings")
        })?;
        sources.push((
            key.clone(),
            SourceFile::new(format!("<python:{key}>"), text),
        ));
    }
    AnalysisInput::with_library_sources(root, sources)
        .map_err(|err| PyValueError::new_err(err.to_string()))
}

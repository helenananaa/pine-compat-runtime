use pine_runtime::FillGradientSample;
use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyAny, PyBool, PyDict, PyList},
};

pub(crate) fn samples_to_py<'a>(
    py: Python<'_>,
    samples: impl IntoIterator<Item = &'a FillGradientSample>,
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for sample in samples {
        let item = PyDict::new(py);
        item.set_item("topValue", sample.top_value)?;
        item.set_item("bottomValue", sample.bottom_value)?;
        item.set_item("topColor", sample.top_color)?;
        item.set_item("bottomColor", sample.bottom_color)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

pub(crate) fn samples_from_py(value: &Bound<'_, PyAny>) -> PyResult<Vec<FillGradientSample>> {
    let list = value
        .cast::<PyList>()
        .map_err(|_| PyValueError::new_err("gradient samples must be a list"))?;
    let mut samples = Vec::with_capacity(list.len());
    for value in list {
        let item = value
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("gradient sample must be a dict"))?;
        if item.len() != 4 {
            return Err(PyValueError::new_err(
                "gradient sample requires exactly four fields",
            ));
        }
        let sample = FillGradientSample {
            top_value: number_field(item, "topValue")?,
            bottom_value: number_field(item, "bottomValue")?,
            top_color: color_field(item, "topColor")?,
            bottom_color: color_field(item, "bottomColor")?,
        };
        if !sample.is_valid() {
            return Err(PyValueError::new_err("invalid gradient sample"));
        }
        samples.push(sample);
    }
    Ok(samples)
}

fn field<'py>(item: &Bound<'py, PyDict>, name: &str) -> PyResult<Bound<'py, PyAny>> {
    let value = item
        .get_item(name)?
        .ok_or_else(|| PyValueError::new_err(format!("gradient sample missing {name}")))?;
    if value.is_instance_of::<PyBool>() {
        return Err(PyValueError::new_err("gradient fields cannot be bool"));
    }
    Ok(value)
}

fn number_field(item: &Bound<'_, PyDict>, name: &str) -> PyResult<Option<f64>> {
    let value = field(item, name)?;
    if value.is_none() {
        Ok(None)
    } else {
        Ok(Some(value.extract()?))
    }
}

fn color_field(item: &Bound<'_, PyDict>, name: &str) -> PyResult<Option<u64>> {
    let value = field(item, name)?;
    if value.is_none() {
        Ok(None)
    } else {
        Ok(Some(value.extract()?))
    }
}

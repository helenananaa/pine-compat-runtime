use pine_runtime::ChartContext;
use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyAny, PyBool, PyDict},
};

pub(crate) fn parse_chart_metadata(
    mut chart: ChartContext,
    value: &Bound<'_, PyAny>,
) -> PyResult<ChartContext> {
    let grid = value
        .cast::<PyDict>()
        .map_err(|_| PyValueError::new_err("$chart must be a dict with minMove and priceScale"))?;
    let has_quantity_precision = grid.contains("quantityPrecision")?;
    if grid.len() != 2 + usize::from(has_quantity_precision)
        || !grid.contains("minMove")?
        || !grid.contains("priceScale")?
    {
        return Err(PyValueError::new_err(
            "$chart requires minMove and priceScale, optionally quantityPrecision; use chart_symbol/chart_timeframe for identity",
        ));
    }
    let integer = |key: &str| -> PyResult<u32> {
        let value = grid.get_item(key)?.expect("checked key");
        if value.is_instance_of::<PyBool>() {
            return Err(PyValueError::new_err("price grid does not accept booleans"));
        }
        value
            .extract::<u32>()
            .map_err(|_| PyValueError::new_err("price grid must contain positive integers"))
    };
    chart = chart
        .with_price_grid(integer("minMove")?, integer("priceScale")?)
        .map_err(PyValueError::new_err)?;
    if has_quantity_precision {
        let value = grid.get_item("quantityPrecision")?.expect("checked key");
        if value.is_instance_of::<PyBool>() {
            return Err(PyValueError::new_err(
                "quantityPrecision must be an integer between 0 and 9",
            ));
        }
        let precision = value.extract::<u32>().map_err(|_| {
            PyValueError::new_err("quantityPrecision must be an integer between 0 and 9")
        })?;
        chart = chart
            .with_quantity_precision(precision)
            .map_err(PyValueError::new_err)?;
    }
    Ok(chart)
}

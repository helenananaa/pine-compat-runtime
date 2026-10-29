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
    let has_quantity_step = grid.contains("quantityStep")?;
    let has_min_contract = grid.contains("minContract")?;
    let has_point_value = grid.contains("pointValue")?;
    let has_currency = grid.contains("currency")?;
    let has_chart_type = grid.contains("chartType")?;
    if grid.len()
        != 2 + usize::from(has_quantity_precision)
            + usize::from(has_quantity_step)
            + usize::from(has_min_contract)
            + usize::from(has_point_value)
            + usize::from(has_currency)
            + usize::from(has_chart_type)
        || !grid.contains("minMove")?
        || !grid.contains("priceScale")?
    {
        return Err(PyValueError::new_err(
            "$chart requires minMove and priceScale, optionally quantityPrecision, quantityStep, minContract, pointValue, currency and chartType; use chart_symbol/chart_timeframe for identity",
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
    if has_chart_type {
        let value = grid
            .get_item("chartType")?
            .expect("checked key")
            .extract::<String>()?;
        chart = chart.with_chart_type(
            pine_runtime::ChartType::parse(&value).map_err(PyValueError::new_err)?,
        );
    }
    if has_currency {
        let value = grid.get_item("currency")?.expect("checked key");
        let currency = value
            .extract::<String>()
            .map_err(|_| PyValueError::new_err("chart currency must be a string"))?;
        chart = chart
            .with_currency(currency)
            .map_err(PyValueError::new_err)?;
    }
    if has_point_value {
        let value = grid.get_item("pointValue")?.expect("checked key");
        if value.is_instance_of::<PyBool>() {
            return Err(PyValueError::new_err(
                "pointValue must be numeric, not boolean",
            ));
        }
        let value = value
            .extract::<f64>()
            .map_err(|_| PyValueError::new_err("pointValue must be numeric"))?;
        chart = chart
            .with_point_value(value)
            .map_err(PyValueError::new_err)?;
    }
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
    for (key, present) in [
        ("quantityStep", has_quantity_step),
        ("minContract", has_min_contract),
    ] {
        if present {
            let value = grid
                .get_item(key)?
                .expect("checked key")
                .extract::<String>()
                .map_err(|_| PyValueError::new_err(format!("{key} must be a decimal string")))?;
            chart = if key == "quantityStep" {
                chart.with_quantity_step(&value)
            } else {
                chart.with_min_contract(&value)
            }
            .map_err(|error| PyValueError::new_err(format!("{key}: {error}")))?;
        }
    }
    Ok(chart)
}

use super::*;

/// Convert an owned snapshot while releasing each source series as it is copied.
pub(crate) fn runtime_result_into_py(
    py: Python<'_>,
    result: pine_runtime::RuntimeResult,
) -> PyResult<Py<PyAny>> {
    let output = PyDict::new(py);
    output.set_item("schemaVersion", PUBLIC_RUNTIME_SCHEMA_VERSION)?;
    output.set_item("renderMetadataVersion", PUBLIC_RENDER_METADATA_VERSION)?;
    macro_rules! family {
        ($name:literal, $values:expr, $converter:ident) => {
            output.set_item($name, owned_list(py, $values, $converter)?)?;
        };
    }
    family!("plots", result.plots, plots_to_py);
    family!("plotChars", result.plot_chars, plot_chars_to_py);
    family!("plotShapes", result.plot_shapes, plot_shapes_to_py);
    family!("plotArrows", result.plot_arrows, plot_arrows_to_py);
    family!("plotBars", result.plot_bars, plot_bars_to_py);
    family!("plotCandles", result.plot_candles, plot_candles_to_py);
    family!("bgColors", result.bg_colors, colors_to_py);
    family!("barColors", result.bar_colors, colors_to_py);
    family!("hlines", result.hlines, hlines_to_py);
    family!("fills", result.fills, fills_to_py);
    family!("labels", result.labels, labels_to_py);
    family!("lines", result.lines, lines_to_py);
    family!("lineFills", result.line_fills, line_fills_to_py);
    family!("polylines", result.polylines, polylines_to_py);
    family!("boxes", result.boxes, boxes_to_py);
    family!("tables", result.tables, tables_to_py);
    family!("alerts", result.alerts, alerts_to_py);
    if let Some(strategy) = result.strategy {
        let value = PyDict::new(py);
        macro_rules! strategy_family {
            ($name:literal, $values:expr, $converter:ident) => {
                value.set_item($name, owned_list(py, $values, $converter)?)?;
            };
        }
        strategy_family!("orders", strategy.orders, strategy_orders_to_py);
        strategy_family!("trades", strategy.trades, strategy_trades_to_py);
        strategy_family!("position", strategy.position, strategy_position_to_py);
        strategy_family!("equity", strategy.equity, strategy_equity_to_py);
        strategy_family!("alerts", strategy.alerts, strategy_order_fill_alerts_to_py);
        strategy_family!(
            "diagnostics",
            strategy.diagnostics,
            runtime_diagnostics_to_py
        );
        output.set_item("strategy", value)?;
    }
    family!("diagnostics", result.diagnostics, runtime_diagnostics_to_py);
    Ok(output.into_any().unbind())
}

fn owned_list<T>(
    py: Python<'_>,
    values: Vec<T>,
    convert: impl Fn(Python<'_>, &[T]) -> PyResult<Py<PyAny>>,
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for value in values {
        let item = convert(py, std::slice::from_ref(&value))?;
        output.append(item.bind(py).get_item(0)?)?;
    }
    Ok(output.into_any().unbind())
}

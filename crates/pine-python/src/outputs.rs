use crate::tables::tables_to_py;
use pine_runtime::{
    BoxOutputView, ColorSeriesView, FillOutputView, LabelOutputView, LineFillOutputView,
    LineOutputView, PlotArrowSeriesView, PlotBarSeriesView, PlotCandleSeriesView,
    PlotCharSeriesView, PlotSeriesView, PlotShapeSeriesView, PolylineOutputView, RuntimeResultView,
    StrategyResultView,
};
use pine_runtime::{PUBLIC_RENDER_METADATA_VERSION, PUBLIC_RUNTIME_SCHEMA_VERSION, PineValue};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyList};

pub(crate) fn runtime_result_to_py(
    py: Python<'_>,
    result: &pine_runtime::RuntimeResult,
) -> PyResult<Py<PyAny>> {
    runtime_result_view_to_py(py, &result.view())
}

pub(crate) fn runtime_result_view_to_py(
    py: Python<'_>,
    result: &RuntimeResultView<'_>,
) -> PyResult<Py<PyAny>> {
    let output = PyDict::new(py);
    output.set_item("schemaVersion", PUBLIC_RUNTIME_SCHEMA_VERSION)?;
    output.set_item("renderMetadataVersion", PUBLIC_RENDER_METADATA_VERSION)?;
    output.set_item("plots", plots_to_py(py, &result.plots)?)?;
    output.set_item("plotChars", plot_chars_to_py(py, &result.plot_chars)?)?;
    output.set_item("plotShapes", plot_shapes_to_py(py, &result.plot_shapes)?)?;
    output.set_item("plotArrows", plot_arrows_to_py(py, &result.plot_arrows)?)?;
    output.set_item("plotBars", plot_bars_to_py(py, &result.plot_bars)?)?;
    output.set_item("plotCandles", plot_candles_to_py(py, &result.plot_candles)?)?;
    output.set_item("bgColors", colors_to_py(py, &result.bg_colors)?)?;
    output.set_item("barColors", colors_to_py(py, &result.bar_colors)?)?;
    output.set_item("hlines", hlines_to_py(py, result.hlines)?)?;
    output.set_item("fills", fills_to_py(py, &result.fills)?)?;
    output.set_item("labels", labels_to_py(py, &result.labels)?)?;
    output.set_item("lines", lines_to_py(py, &result.lines)?)?;
    output.set_item("lineFills", line_fills_to_py(py, &result.line_fills)?)?;
    output.set_item("polylines", polylines_to_py(py, &result.polylines)?)?;
    output.set_item("boxes", boxes_to_py(py, &result.boxes)?)?;
    output.set_item("tables", tables_to_py(py, &result.tables)?)?;
    output.set_item("alerts", alerts_to_py(py, result.alerts)?)?;
    if let Some(strategy) = &result.strategy {
        output.set_item("strategy", strategy_result_view_to_py(py, strategy)?)?;
    }
    output.set_item(
        "diagnostics",
        runtime_diagnostics_to_py(py, result.diagnostics.as_ref())?,
    )?;
    Ok(output.into_any().unbind())
}

pub(crate) fn strategy_result_view_to_py(
    py: Python<'_>,
    strategy: &StrategyResultView<'_>,
) -> PyResult<Py<PyAny>> {
    let output = PyDict::new(py);
    output.set_item("orders", strategy_orders_to_py(py, strategy.orders)?)?;
    output.set_item("trades", strategy_trades_to_py(py, strategy.trades)?)?;
    output.set_item("position", strategy_position_to_py(py, strategy.position)?)?;
    output.set_item("equity", strategy_equity_to_py(py, strategy.equity)?)?;
    output.set_item(
        "alerts",
        strategy_order_fill_alerts_to_py(py, strategy.alerts)?,
    )?;
    output.set_item(
        "diagnostics",
        runtime_diagnostics_to_py(py, strategy.diagnostics)?,
    )?;
    Ok(output.into_any().unbind())
}

pub(crate) fn strategy_order_fill_alerts_to_py<'a>(
    py: Python<'_>,
    alerts: impl IntoIterator<Item = &'a pine_runtime::StrategyOrderFillAlertOutput>,
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for alert in alerts {
        let item = PyDict::new(py);
        item.set_item("id", &alert.id)?;
        item.set_item("barIndex", alert.bar_index)?;
        item.set_item("time", alert.time)?;
        item.set_item("direction", &alert.direction)?;
        set_finite_f64(py, &item, "qty", alert.qty)?;
        set_finite_f64(py, &item, "price", alert.price)?;
        match &alert.entry_id {
            Some(entry_id) => item.set_item("entryId", entry_id)?,
            None => item.set_item("entryId", py.None())?,
        }
        match &alert.exit_id {
            Some(exit_id) => item.set_item("exitId", exit_id)?,
            None => item.set_item("exitId", py.None())?,
        }
        item.set_item("message", &alert.message)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

pub(crate) fn runtime_diagnostics_to_py(
    py: Python<'_>,
    diagnostics: &[pine_runtime::RuntimeDiagnostic],
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for diagnostic in diagnostics {
        let item = PyDict::new(py);
        item.set_item("code", &diagnostic.code)?;
        item.set_item("message", &diagnostic.message)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

pub(crate) fn strategy_trades_to_py<'a>(
    py: Python<'_>,
    trades: impl IntoIterator<Item = &'a pine_runtime::StrategyTrade>,
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for trade in trades {
        let item = PyDict::new(py);
        item.set_item("id", &trade.id)?;
        item.set_item("entryBarIndex", trade.entry_bar_index)?;
        item.set_item("exitBarIndex", trade.exit_bar_index)?;
        item.set_item("entryTime", trade.entry_time)?;
        item.set_item("exitTime", trade.exit_time)?;
        set_finite_f64(py, &item, "entryPrice", trade.entry_price)?;
        set_finite_f64(py, &item, "exitPrice", trade.exit_price)?;
        set_finite_f64(py, &item, "qty", trade.qty)?;
        set_finite_f64(py, &item, "profit", trade.profit)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

pub(crate) fn strategy_orders_to_py<'a>(
    py: Python<'_>,
    orders: impl IntoIterator<Item = &'a pine_runtime::StrategyOrderEvent>,
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for order in orders {
        let item = PyDict::new(py);
        item.set_item("id", &order.id)?;
        item.set_item("barIndex", order.bar_index)?;
        item.set_item("time", order.time)?;
        item.set_item("direction", &order.direction)?;
        set_finite_f64(py, &item, "qty", order.qty)?;
        set_finite_f64(py, &item, "price", order.price)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

pub(crate) fn strategy_position_to_py<'a>(
    py: Python<'_>,
    position: impl IntoIterator<Item = &'a pine_runtime::StrategyPositionSnapshot>,
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for snapshot in position {
        let item = PyDict::new(py);
        item.set_item("barIndex", snapshot.bar_index)?;
        set_finite_f64(py, &item, "size", snapshot.size)?;
        set_option_finite_f64(py, &item, "avgPrice", snapshot.avg_price)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

pub(crate) fn strategy_equity_to_py<'a>(
    py: Python<'_>,
    equity: impl IntoIterator<Item = &'a pine_runtime::StrategyEquitySnapshot>,
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for snapshot in equity {
        let item = PyDict::new(py);
        item.set_item("barIndex", snapshot.bar_index)?;
        set_finite_f64(py, &item, "cash", snapshot.cash)?;
        set_finite_f64(py, &item, "marketValue", snapshot.market_value)?;
        set_finite_f64(py, &item, "equity", snapshot.equity)?;
        set_finite_f64(py, &item, "netProfit", snapshot.net_profit)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn plots_to_py<'a>(
    py: Python<'_>,
    plots: impl IntoIterator<Item = impl Into<PlotSeriesView<'a>>>,
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for plot in plots.into_iter().map(Into::into) {
        let item = PyDict::new(py);
        item.set_item("id", plot.id)?;
        item.set_item("values", values_to_py(py, plot.values)?)?;
        if plot.colors.iter().any(|value| *value != PineValue::Na) {
            item.set_item("colors", values_to_py(py, plot.colors)?)?;
        }
        set_non_default_value(py, &item, "linewidth", plot.linewidth, &PineValue::Int(1))?;
        set_non_default_value(
            py,
            &item,
            "style",
            plot.style,
            &PineValue::String("plot.style_line".to_owned()),
        )?;
        set_non_default_value(
            py,
            &item,
            "trackPrice",
            plot.track_price,
            &PineValue::Bool(false),
        )?;
        set_non_default_value(py, &item, "histBase", plot.hist_base, &PineValue::Int(0))?;
        set_non_default_value(py, &item, "join", plot.join, &PineValue::Bool(false))?;
        set_non_default_value(
            py,
            &item,
            "format",
            plot.format,
            &PineValue::String("format.inherit".to_owned()),
        )?;
        set_non_default_value(py, &item, "precision", plot.precision, &PineValue::Na)?;
        set_non_default_value(
            py,
            &item,
            "linestyle",
            plot.linestyle,
            &PineValue::String("plot.linestyle_solid".to_owned()),
        )?;
        set_output_metadata(py, &item, plot.metadata)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn colors_to_py<'a>(
    py: Python<'_>,
    colors: impl IntoIterator<Item = impl Into<ColorSeriesView<'a>>>,
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for colors in colors.into_iter().map(Into::into) {
        let item = PyDict::new(py);
        item.set_item("id", colors.id)?;
        item.set_item("values", values_to_py(py, colors.values)?)?;
        set_output_metadata(py, &item, colors.metadata)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn plot_chars_to_py<'a>(
    py: Python<'_>,
    plot_chars: impl IntoIterator<Item = impl Into<PlotCharSeriesView<'a>>>,
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for plot_char in plot_chars.into_iter().map(Into::into) {
        let item = PyDict::new(py);
        item.set_item("id", plot_char.id)?;
        item.set_item("values", values_to_py(py, plot_char.values)?)?;
        item.set_item("chars", values_to_py(py, plot_char.chars)?)?;
        item.set_item("colors", values_to_py(py, plot_char.colors)?)?;
        item.set_item("locations", values_to_py(py, plot_char.locations)?)?;
        item.set_item("texts", values_to_py(py, plot_char.texts)?)?;
        item.set_item("textColors", values_to_py(py, plot_char.text_colors)?)?;
        item.set_item("sizes", values_to_py(py, plot_char.sizes)?)?;
        set_output_metadata(py, &item, plot_char.metadata)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn plot_shapes_to_py<'a>(
    py: Python<'_>,
    plot_shapes: impl IntoIterator<Item = impl Into<PlotShapeSeriesView<'a>>>,
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for plot_shape in plot_shapes.into_iter().map(Into::into) {
        let item = PyDict::new(py);
        item.set_item("id", plot_shape.id)?;
        item.set_item("values", values_to_py(py, plot_shape.values)?)?;
        item.set_item("styles", values_to_py(py, plot_shape.styles)?)?;
        item.set_item("locations", values_to_py(py, plot_shape.locations)?)?;
        item.set_item("colors", values_to_py(py, plot_shape.colors)?)?;
        item.set_item("texts", values_to_py(py, plot_shape.texts)?)?;
        item.set_item("textColors", values_to_py(py, plot_shape.text_colors)?)?;
        item.set_item("sizes", values_to_py(py, plot_shape.sizes)?)?;
        set_output_metadata(py, &item, plot_shape.metadata)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn plot_arrows_to_py<'a>(
    py: Python<'_>,
    plot_arrows: impl IntoIterator<Item = impl Into<PlotArrowSeriesView<'a>>>,
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for plot_arrow in plot_arrows.into_iter().map(Into::into) {
        let item = PyDict::new(py);
        item.set_item("id", plot_arrow.id)?;
        item.set_item("values", values_to_py(py, plot_arrow.values)?)?;
        item.set_item("colorUps", values_to_py(py, plot_arrow.color_ups)?)?;
        item.set_item("colorDowns", values_to_py(py, plot_arrow.color_downs)?)?;
        item.set_item("minHeights", values_to_py(py, plot_arrow.min_heights)?)?;
        item.set_item("maxHeights", values_to_py(py, plot_arrow.max_heights)?)?;
        set_output_metadata(py, &item, plot_arrow.metadata)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn plot_bars_to_py<'a>(
    py: Python<'_>,
    plot_bars: impl IntoIterator<Item = impl Into<PlotBarSeriesView<'a>>>,
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for plot_bar in plot_bars.into_iter().map(Into::into) {
        let item = PyDict::new(py);
        item.set_item("id", plot_bar.id)?;
        item.set_item("opens", values_to_py(py, plot_bar.opens)?)?;
        item.set_item("highs", values_to_py(py, plot_bar.highs)?)?;
        item.set_item("lows", values_to_py(py, plot_bar.lows)?)?;
        item.set_item("closes", values_to_py(py, plot_bar.closes)?)?;
        item.set_item("colors", values_to_py(py, plot_bar.colors)?)?;
        set_output_metadata(py, &item, plot_bar.metadata)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn plot_candles_to_py<'a>(
    py: Python<'_>,
    plot_candles: impl IntoIterator<Item = impl Into<PlotCandleSeriesView<'a>>>,
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for plot_candle in plot_candles.into_iter().map(Into::into) {
        let item = PyDict::new(py);
        item.set_item("id", plot_candle.id)?;
        item.set_item("opens", values_to_py(py, plot_candle.opens)?)?;
        item.set_item("highs", values_to_py(py, plot_candle.highs)?)?;
        item.set_item("lows", values_to_py(py, plot_candle.lows)?)?;
        item.set_item("closes", values_to_py(py, plot_candle.closes)?)?;
        item.set_item("colors", values_to_py(py, plot_candle.colors)?)?;
        item.set_item("wickColors", values_to_py(py, plot_candle.wick_colors)?)?;
        item.set_item("borderColors", values_to_py(py, plot_candle.border_colors)?)?;
        set_output_metadata(py, &item, plot_candle.metadata)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

pub(crate) fn hlines_to_py(
    py: Python<'_>,
    hlines: &[pine_runtime::HLineOutput],
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for hline in hlines {
        let item = PyDict::new(py);
        item.set_item("id", hline.id)?;
        item.set_item("price", value_to_py(py, &hline.price)?)?;
        set_non_default_value(
            py,
            &item,
            "title",
            &hline.title,
            &PineValue::String(String::new()),
        )?;
        set_non_default_value(
            py,
            &item,
            "color",
            &hline.color,
            &PineValue::Color(0x787B86),
        )?;
        set_non_default_value(
            py,
            &item,
            "style",
            &hline.style,
            &PineValue::String("hline.style_solid".to_owned()),
        )?;
        set_non_default_value(py, &item, "linewidth", &hline.linewidth, &PineValue::Int(1))?;
        set_non_default_value(
            py,
            &item,
            "editable",
            &hline.editable,
            &PineValue::Bool(true),
        )?;
        set_non_default_value(
            py,
            &item,
            "display",
            &hline.display,
            &PineValue::String("display.all".to_owned()),
        )?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

pub(crate) fn fills_to_py<'a>(
    py: Python<'_>,
    fills: impl IntoIterator<Item = impl Into<FillOutputView<'a>>>,
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for fill in fills.into_iter().map(Into::into) {
        let item = PyDict::new(py);
        item.set_item("id", fill.id)?;
        item.set_item("firstId", fill.first_id)?;
        item.set_item("secondId", fill.second_id)?;
        item.set_item("firstIsHLine", fill.first_is_hline)?;
        item.set_item("secondIsHLine", fill.second_is_hline)?;
        item.set_item("colors", values_to_py(py, fill.colors)?)?;
        if let Some(samples) = &fill.gradient {
            item.set_item("gradient", crate::gradient::samples_to_py(py, samples)?)?;
        }
        set_non_default_value(
            py,
            &item,
            "title",
            fill.title,
            &PineValue::String(String::new()),
        )?;
        set_non_default_value(py, &item, "editable", fill.editable, &PineValue::Bool(true))?;
        set_non_default_value(py, &item, "showLast", fill.show_last, &PineValue::Na)?;
        set_non_default_value(
            py,
            &item,
            "fillGaps",
            fill.fill_gaps,
            &PineValue::Bool(true),
        )?;
        set_non_default_value(
            py,
            &item,
            "display",
            fill.display,
            &PineValue::String("display.all".to_owned()),
        )?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

pub(crate) fn set_non_default_value(
    py: Python<'_>,
    item: &Bound<'_, PyDict>,
    name: &str,
    value: &PineValue,
    default: &PineValue,
) -> PyResult<()> {
    if value != default {
        item.set_item(name, value_to_py(py, value)?)?;
    }
    Ok(())
}

pub(crate) fn set_output_metadata(
    py: Python<'_>,
    item: &Bound<'_, PyDict>,
    metadata: &pine_runtime::OutputMetadata,
) -> PyResult<()> {
    set_non_default_value(
        py,
        item,
        "title",
        &metadata.title,
        &PineValue::String(String::new()),
    )?;
    set_non_default_value(py, item, "offset", &metadata.offset, &PineValue::Int(0))?;
    set_non_default_value(
        py,
        item,
        "editable",
        &metadata.editable,
        &PineValue::Bool(true),
    )?;
    set_non_default_value(py, item, "showLast", &metadata.show_last, &PineValue::Na)?;
    set_non_default_value(
        py,
        item,
        "display",
        &metadata.display,
        &PineValue::String("display.all".to_owned()),
    )?;
    set_non_default_value(
        py,
        item,
        "forceOverlay",
        &metadata.force_overlay,
        &PineValue::Bool(false),
    )
}

pub(crate) fn labels_to_py<'a>(
    py: Python<'_>,
    labels: impl IntoIterator<Item = impl Into<LabelOutputView<'a>>>,
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for label in labels.into_iter().map(Into::into) {
        let item = PyDict::new(py);
        item.set_item("id", label.id)?;
        item.set_item("snapshots", label_snapshots_to_py(py, label.snapshots)?)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn label_snapshots_to_py<'a>(
    py: Python<'_>,
    snapshots: impl IntoIterator<Item = &'a pine_runtime::LabelSnapshot>,
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for snapshot in snapshots {
        let item = PyDict::new(py);
        item.set_item("barIndex", snapshot.bar_index)?;
        item.set_item("exists", snapshot.exists)?;
        if snapshot.exists {
            item.set_item("x", value_to_py(py, &snapshot.x)?)?;
            item.set_item("y", value_to_py(py, &snapshot.y)?)?;
            item.set_item("text", value_to_py(py, &snapshot.text)?)?;
            item.set_item("xloc", value_to_py(py, &snapshot.xloc)?)?;
            item.set_item("yloc", value_to_py(py, &snapshot.yloc)?)?;
            item.set_item("color", value_to_py(py, &snapshot.color)?)?;
            item.set_item("style", value_to_py(py, &snapshot.style)?)?;
            item.set_item("textColor", value_to_py(py, &snapshot.text_color)?)?;
            item.set_item("size", value_to_py(py, &snapshot.size)?)?;
            item.set_item("tooltip", value_to_py(py, &snapshot.tooltip)?)?;
            item.set_item("textAlign", value_to_py(py, &snapshot.text_align)?)?;
            item.set_item(
                "textFontFamily",
                value_to_py(py, &snapshot.text_font_family)?,
            )?;
            item.set_item(
                "textFormatting",
                value_to_py(py, &snapshot.text_formatting)?,
            )?;
        }
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

pub(crate) fn lines_to_py<'a>(
    py: Python<'_>,
    lines: impl IntoIterator<Item = impl Into<LineOutputView<'a>>>,
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for line in lines.into_iter().map(Into::into) {
        let item = PyDict::new(py);
        item.set_item("id", line.id)?;
        item.set_item("snapshots", line_snapshots_to_py(py, line.snapshots)?)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn line_snapshots_to_py<'a>(
    py: Python<'_>,
    snapshots: impl IntoIterator<Item = &'a pine_runtime::LineSnapshot>,
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for snapshot in snapshots {
        let item = PyDict::new(py);
        item.set_item("barIndex", snapshot.bar_index)?;
        item.set_item("exists", snapshot.exists)?;
        if snapshot.exists {
            item.set_item("x1", value_to_py(py, &snapshot.x1)?)?;
            item.set_item("y1", value_to_py(py, &snapshot.y1)?)?;
            item.set_item("x2", value_to_py(py, &snapshot.x2)?)?;
            item.set_item("y2", value_to_py(py, &snapshot.y2)?)?;
            item.set_item("xloc", value_to_py(py, &snapshot.xloc)?)?;
            item.set_item("color", value_to_py(py, &snapshot.color)?)?;
            item.set_item("width", value_to_py(py, &snapshot.width)?)?;
            item.set_item("style", value_to_py(py, &snapshot.style)?)?;
            item.set_item("extend", value_to_py(py, &snapshot.extend)?)?;
        }
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

pub(crate) fn line_fills_to_py<'a>(
    py: Python<'_>,
    line_fills: impl IntoIterator<Item = impl Into<LineFillOutputView<'a>>>,
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for line_fill in line_fills.into_iter().map(Into::into) {
        let item = PyDict::new(py);
        item.set_item("id", line_fill.id)?;
        item.set_item(
            "snapshots",
            line_fill_snapshots_to_py(py, line_fill.snapshots)?,
        )?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn line_fill_snapshots_to_py<'a>(
    py: Python<'_>,
    snapshots: impl IntoIterator<Item = &'a pine_runtime::LineFillSnapshot>,
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for snapshot in snapshots {
        let item = PyDict::new(py);
        item.set_item("barIndex", snapshot.bar_index)?;
        item.set_item("exists", snapshot.exists)?;
        if snapshot.exists {
            item.set_item("line1", snapshot.line1)?;
            item.set_item("line2", snapshot.line2)?;
            item.set_item("color", value_to_py(py, &snapshot.color)?)?;
        }
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

pub(crate) fn polylines_to_py<'a>(
    py: Python<'_>,
    polylines: impl IntoIterator<Item = impl Into<PolylineOutputView<'a>>>,
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for polyline in polylines.into_iter().map(Into::into) {
        let item = PyDict::new(py);
        item.set_item("id", polyline.id)?;
        item.set_item(
            "snapshots",
            polyline_snapshots_to_py(py, polyline.snapshots)?,
        )?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn polyline_snapshots_to_py<'a>(
    py: Python<'_>,
    snapshots: impl IntoIterator<Item = &'a pine_runtime::PolylineSnapshot>,
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for snapshot in snapshots {
        let item = PyDict::new(py);
        item.set_item("barIndex", snapshot.bar_index)?;
        item.set_item("exists", snapshot.exists)?;
        if snapshot.exists {
            item.set_item("points", values_to_py(py, &snapshot.points)?)?;
            item.set_item("curved", value_to_py(py, &snapshot.curved)?)?;
            item.set_item("closed", value_to_py(py, &snapshot.closed)?)?;
            item.set_item("xloc", value_to_py(py, &snapshot.xloc)?)?;
            item.set_item("lineColor", value_to_py(py, &snapshot.line_color)?)?;
            item.set_item("fillColor", value_to_py(py, &snapshot.fill_color)?)?;
            item.set_item("lineStyle", value_to_py(py, &snapshot.line_style)?)?;
            item.set_item("lineWidth", value_to_py(py, &snapshot.line_width)?)?;
            item.set_item("forceOverlay", value_to_py(py, &snapshot.force_overlay)?)?;
        }
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

pub(crate) fn boxes_to_py<'a>(
    py: Python<'_>,
    boxes: impl IntoIterator<Item = impl Into<BoxOutputView<'a>>>,
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for box_output in boxes.into_iter().map(Into::into) {
        let item = PyDict::new(py);
        item.set_item("id", box_output.id)?;
        item.set_item("snapshots", box_snapshots_to_py(py, box_output.snapshots)?)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn box_snapshots_to_py<'a>(
    py: Python<'_>,
    snapshots: impl IntoIterator<Item = &'a pine_runtime::BoxSnapshot>,
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for snapshot in snapshots {
        let item = PyDict::new(py);
        item.set_item("barIndex", snapshot.bar_index)?;
        item.set_item("exists", snapshot.exists)?;
        if snapshot.exists {
            item.set_item("left", value_to_py(py, &snapshot.left)?)?;
            item.set_item("top", value_to_py(py, &snapshot.top)?)?;
            item.set_item("right", value_to_py(py, &snapshot.right)?)?;
            item.set_item("bottom", value_to_py(py, &snapshot.bottom)?)?;
            item.set_item("xloc", value_to_py(py, &snapshot.xloc)?)?;
            item.set_item("bgColor", value_to_py(py, &snapshot.bg_color)?)?;
            item.set_item("borderColor", value_to_py(py, &snapshot.border_color)?)?;
            item.set_item("borderWidth", value_to_py(py, &snapshot.border_width)?)?;
            item.set_item("borderStyle", value_to_py(py, &snapshot.border_style)?)?;
            item.set_item("extend", value_to_py(py, &snapshot.extend)?)?;
            item.set_item("text", value_to_py(py, &snapshot.text)?)?;
            item.set_item("textColor", value_to_py(py, &snapshot.text_color)?)?;
            item.set_item("textSize", value_to_py(py, &snapshot.text_size)?)?;
            item.set_item("textHalign", value_to_py(py, &snapshot.text_halign)?)?;
            item.set_item("textValign", value_to_py(py, &snapshot.text_valign)?)?;
            item.set_item("textWrap", value_to_py(py, &snapshot.text_wrap)?)?;
            item.set_item(
                "textFontFamily",
                value_to_py(py, &snapshot.text_font_family)?,
            )?;
            item.set_item(
                "textFormatting",
                value_to_py(py, &snapshot.text_formatting)?,
            )?;
        }
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

pub(crate) fn alerts_to_py<'a>(
    py: Python<'_>,
    alerts: impl IntoIterator<Item = &'a pine_runtime::AlertEvent>,
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for alert in alerts {
        let item = PyDict::new(py);
        item.set_item("id", alert.id)?;
        item.set_item("barIndex", alert.bar_index)?;
        item.set_item("time", alert.time)?;
        item.set_item("message", &alert.message)?;
        item.set_item("source", &alert.source)?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

pub(crate) fn values_to_py<'a>(
    py: Python<'_>,
    values: impl IntoIterator<Item = &'a PineValue>,
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    // Repeated immutable scalars (especially color histories) can share one
    // Python object. Containers must remain independently mutable, and float
    // identity uses bits so negative zero is not replaced with positive zero.
    let mut previous: Option<(&PineValue, Py<PyAny>)> = None;
    let mut values = values.into_iter().peekable();
    let mut index = 0;
    while let Some(value) = values.next() {
        if let Some((old, item)) = &previous
            && same_python_scalar(old, value)
        {
            output.append(item.bind(py))?;
        } else {
            append_value(py, &output, value)?;
            previous = None;
        }
        if previous.is_none()
            && values
                .peek()
                .is_some_and(|next| same_python_scalar(value, next))
        {
            previous = Some((value, output.get_item(index)?.unbind()));
        }
        index += 1;
    }
    Ok(output.into_any().unbind())
}

fn same_python_scalar(left: &PineValue, right: &PineValue) -> bool {
    match (left, right) {
        (PineValue::Int(left), PineValue::Int(right)) => left == right,
        (PineValue::Float(left), PineValue::Float(right)) => left.to_bits() == right.to_bits(),
        (PineValue::Color(left), PineValue::Color(right)) => left == right,
        (PineValue::String(left), PineValue::String(right)) => left == right,
        _ => false,
    }
}

pub(crate) fn value_to_py(py: Python<'_>, value: &PineValue) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    append_value(py, &output, value)?;
    Ok(output.get_item(0)?.unbind())
}

fn append_value(py: Python<'_>, output: &Bound<'_, PyList>, value: &PineValue) -> PyResult<()> {
    match value {
        PineValue::Int(value) => output.append(*value),
        PineValue::Float(value) if value.is_finite() => output.append(*value),
        PineValue::Float(_) => output.append(py.None()),
        PineValue::Bool(value) => output.append(*value),
        PineValue::String(value) => output.append(value),
        PineValue::Color(value) => output.append(*value),
        PineValue::Plot(value)
        | PineValue::HLine(value)
        | PineValue::Label(value)
        | PineValue::Line(value)
        | PineValue::LineFill(value)
        | PineValue::Polyline(value)
        | PineValue::Box(value)
        | PineValue::Table(value) => output.append(*value),
        PineValue::ChartPoint(point) => {
            let item = PyDict::new(py);
            item.set_item("time", value_to_py(py, &point.time)?)?;
            item.set_item("index", value_to_py(py, &point.index)?)?;
            item.set_item("price", value_to_py(py, &point.price)?)?;
            output.append(item)
        }
        PineValue::UserType(values) | PineValue::Tuple(values) => {
            output.append(values_to_py(py, values)?)
        }
        PineValue::Array(_)
        | PineValue::UserTypeRef(_)
        | PineValue::Matrix(_)
        | PineValue::Map(_)
        | PineValue::Na
        | PineValue::Void => output.append(py.None()),
    }
}

fn set_finite_f64(
    py: Python<'_>,
    output: &Bound<'_, PyDict>,
    name: &str,
    value: f64,
) -> PyResult<()> {
    if value.is_finite() {
        output.set_item(name, value)
    } else {
        output.set_item(name, py.None())
    }
}

fn set_option_finite_f64(
    py: Python<'_>,
    output: &Bound<'_, PyDict>,
    name: &str,
    value: Option<f64>,
) -> PyResult<()> {
    match value {
        Some(value) if value.is_finite() => output.set_item(name, value),
        Some(_) | None => output.set_item(name, py.None()),
    }
}

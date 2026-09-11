use pine_runtime::{
    AlertEvent, BoxOutput, BoxSnapshot, ChartPointValue, ColorSeries, FillOutput, HLineOutput,
    LabelOutput, LabelSnapshot, LineFillOutput, LineFillSnapshot, LineOutput, LineSnapshot,
    OutputMetadata, PineValue, PlotArrowSeries, PlotBarSeries, PlotCandleSeries, PlotCharSeries,
    PlotSeries, PlotShapeSeries, PolylineOutput, PolylineSnapshot, RuntimeDiagnostic,
    RuntimeResult, SeriesHeader, StrategyEquitySnapshot, StrategyOrderEvent,
    StrategyOrderFillAlertOutput, StrategyPositionSnapshot, StrategyResult, StrategyTrade,
    TableCellSnapshot, TableMergedCellSnapshot, TableOutput, TableSnapshot,
};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBool, PyDict, PyList};

use crate::outputs::value_to_py;

pub(crate) fn runtime_result_from_py(
    py: Python<'_>,
    value: &Bound<'_, PyAny>,
) -> PyResult<RuntimeResult> {
    let dict = value
        .cast::<PyDict>()
        .map_err(|_| PyValueError::new_err("runtime result must be a dict"))?;
    Ok(RuntimeResult {
        plots: plots_from_py(py, dict_list(dict, "plots")?)?,
        plot_chars: plot_chars_from_py(py, dict_list(dict, "plotChars")?)?,
        plot_shapes: plot_shapes_from_py(py, dict_list(dict, "plotShapes")?)?,
        plot_arrows: plot_arrows_from_py(py, dict_list(dict, "plotArrows")?)?,
        plot_bars: plot_bars_from_py(py, dict_list(dict, "plotBars")?)?,
        plot_candles: plot_candles_from_py(py, dict_list(dict, "plotCandles")?)?,
        bg_colors: colors_from_py(py, dict_list(dict, "bgColors")?)?,
        bar_colors: colors_from_py(py, dict_list(dict, "barColors")?)?,
        hlines: hlines_from_py(py, dict_list(dict, "hlines")?)?,
        fills: fills_from_py(py, dict_list(dict, "fills")?)?,
        labels: labels_from_py(py, dict_list(dict, "labels")?)?,
        lines: lines_from_py(py, dict_list(dict, "lines")?)?,
        line_fills: line_fills_from_py(py, dict_list(dict, "lineFills")?)?,
        polylines: polylines_from_py(py, dict_list(dict, "polylines")?)?,
        boxes: boxes_from_py(py, dict_list(dict, "boxes")?)?,
        tables: tables_from_py(py, dict_list(dict, "tables")?)?,
        alerts: alerts_from_py(dict_list(dict, "alerts")?)?,
        strategy: match dict.get_item("strategy")? {
            Some(value) if !value.is_none() => Some(strategy_from_py(py, &value)?),
            _ => None,
        },
        diagnostics: diagnostics_from_py(dict_list(dict, "diagnostics")?)?,
    })
}

pub(crate) fn pine_value_from_py(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<PineValue> {
    if value.is_none() {
        return Ok(PineValue::Na);
    }
    if let Ok(value) = value.cast::<PyBool>() {
        return Ok(PineValue::Bool(value.extract()?));
    }
    if let Ok(value) = value.extract::<i64>() {
        return Ok(PineValue::Int(value));
    }
    if let Ok(value) = value.extract::<f64>() {
        if value.is_finite() {
            return Ok(PineValue::Float(value));
        }
        return Ok(PineValue::Na);
    }
    if let Ok(value) = value.extract::<String>() {
        return Ok(PineValue::String(value));
    }
    if let Ok(dict) = value.cast::<PyDict>()
        && dict.contains("time")?
        && dict.contains("index")?
        && dict.contains("price")?
    {
        return Ok(PineValue::ChartPoint(ChartPointValue::new(
            required_pine_value(py, dict, "time")?,
            required_pine_value(py, dict, "index")?,
            required_pine_value(py, dict, "price")?,
        )));
    }
    if let Ok(list) = value.cast::<PyList>() {
        let mut values = Vec::with_capacity(list.len());
        for item in list {
            values.push(pine_value_from_py(py, &item)?);
        }
        return Ok(PineValue::Tuple(values));
    }
    Err(PyValueError::new_err(format!(
        "unsupported Pine value {value:?}"
    )))
}

pub(crate) fn values_from_py(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Vec<PineValue>> {
    let list = value
        .cast::<PyList>()
        .map_err(|_| PyValueError::new_err("expected a list of values"))?;
    let mut values = Vec::with_capacity(list.len());
    for item in list {
        values.push(pine_value_from_py(py, &item)?);
    }
    Ok(values)
}

pub(crate) fn first_item(py: Python<'_>, list: Py<PyAny>) -> PyResult<Py<PyAny>> {
    Ok(list.bind(py).cast::<PyList>()?.get_item(0)?.unbind())
}

pub(crate) fn dict_u32(dict: &Bound<'_, PyDict>, key: &str) -> PyResult<u32> {
    dict.get_item(key)?
        .ok_or_else(|| PyValueError::new_err(format!("missing `{key}`")))?
        .extract()
}

pub(crate) fn dict_usize(dict: &Bound<'_, PyDict>, key: &str) -> PyResult<usize> {
    dict.get_item(key)?
        .ok_or_else(|| PyValueError::new_err(format!("missing `{key}`")))?
        .extract()
}

pub(crate) fn dict_string(dict: &Bound<'_, PyDict>, key: &str) -> PyResult<String> {
    dict.get_item(key)?
        .ok_or_else(|| PyValueError::new_err(format!("missing `{key}`")))?
        .extract()
}

pub(crate) fn optional_values(
    py: Python<'_>,
    dict: &Bound<'_, PyDict>,
    key: &str,
) -> PyResult<Vec<PineValue>> {
    match dict.get_item(key)? {
        Some(value) if !value.is_none() => values_from_py(py, &value),
        _ => Ok(Vec::new()),
    }
}

fn dict_list<'a>(dict: &Bound<'a, PyDict>, key: &str) -> PyResult<Bound<'a, PyList>> {
    match dict.get_item(key)? {
        Some(value) if !value.is_none() => value
            .cast::<PyList>()
            .map(|list| list.to_owned())
            .map_err(|_| PyValueError::new_err(format!("`{key}` must be a list"))),
        _ => Ok(PyList::empty(dict.py())),
    }
}

fn required_pine_value(py: Python<'_>, dict: &Bound<'_, PyDict>, key: &str) -> PyResult<PineValue> {
    let value = dict
        .get_item(key)?
        .ok_or_else(|| PyValueError::new_err(format!("chart point missing `{key}`")))?;
    pine_value_from_py(py, &value)
}

pub(crate) fn dict_opt_value(
    py: Python<'_>,
    dict: &Bound<'_, PyDict>,
    key: &str,
    default: PineValue,
) -> PyResult<PineValue> {
    match dict.get_item(key)? {
        Some(value) => pine_value_from_py(py, &value),
        None => Ok(default),
    }
}

pub(crate) fn metadata_from_py(
    py: Python<'_>,
    dict: &Bound<'_, PyDict>,
) -> PyResult<OutputMetadata> {
    Ok(OutputMetadata {
        title: dict_opt_value(py, dict, "title", PineValue::String(String::new()))?,
        offset: dict_opt_value(py, dict, "offset", PineValue::Int(0))?,
        editable: dict_opt_value(py, dict, "editable", PineValue::Bool(true))?,
        show_last: dict_opt_value(py, dict, "showLast", PineValue::Na)?,
        display: dict_opt_value(
            py,
            dict,
            "display",
            PineValue::String("display.all".to_owned()),
        )?,
        force_overlay: dict_opt_value(py, dict, "forceOverlay", PineValue::Bool(false))?,
    })
}

pub(crate) fn series_header_from_py(
    py: Python<'_>,
    dict: &Bound<'_, PyDict>,
) -> PyResult<SeriesHeader> {
    Ok(SeriesHeader {
        metadata: metadata_from_py(py, dict)?,
        linewidth: dict_opt_value(py, dict, "linewidth", PineValue::Int(1))?,
        style: dict_opt_value(
            py,
            dict,
            "style",
            PineValue::String("plot.style_line".to_owned()),
        )?,
        track_price: dict_opt_value(py, dict, "trackPrice", PineValue::Bool(false))?,
        hist_base: dict_opt_value(py, dict, "histBase", PineValue::Int(0))?,
        join: dict_opt_value(py, dict, "join", PineValue::Bool(false))?,
        format: dict_opt_value(
            py,
            dict,
            "format",
            PineValue::String("format.inherit".to_owned()),
        )?,
        precision: dict_opt_value(py, dict, "precision", PineValue::Na)?,
    })
}

fn plots_from_py(py: Python<'_>, list: Bound<'_, PyList>) -> PyResult<Vec<PlotSeries>> {
    let mut plots = Vec::with_capacity(list.len());
    for item in list {
        let dict = item
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("plot entries must be dicts"))?;
        let values = values_from_py(
            py,
            &dict
                .get_item("values")?
                .ok_or_else(|| PyValueError::new_err("plot is missing `values`"))?,
        )?;
        let mut plot = PlotSeries::new(dict_u32(dict, "id")?, values);
        if let Some(colors) = dict.get_item("colors")? {
            plot.colors = values_from_py(py, &colors)?;
        }
        plot.linewidth = dict_opt_value(py, dict, "linewidth", PineValue::Int(1))?;
        plot.style = dict_opt_value(
            py,
            dict,
            "style",
            PineValue::String("plot.style_line".to_owned()),
        )?;
        plot.track_price = dict_opt_value(py, dict, "trackPrice", PineValue::Bool(false))?;
        plot.hist_base = dict_opt_value(py, dict, "histBase", PineValue::Int(0))?;
        plot.join = dict_opt_value(py, dict, "join", PineValue::Bool(false))?;
        plot.format = dict_opt_value(
            py,
            dict,
            "format",
            PineValue::String("format.inherit".to_owned()),
        )?;
        plot.precision = dict_opt_value(py, dict, "precision", PineValue::Na)?;
        plot.metadata = metadata_from_py(py, dict)?;
        plots.push(plot);
    }
    Ok(plots)
}

fn plot_chars_from_py(py: Python<'_>, list: Bound<'_, PyList>) -> PyResult<Vec<PlotCharSeries>> {
    let mut items = Vec::with_capacity(list.len());
    for item in list {
        let dict = item
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("plotChar entries must be dicts"))?;
        items.push(PlotCharSeries {
            id: dict_u32(dict, "id")?,
            values: optional_values(py, dict, "values")?,
            chars: optional_values(py, dict, "chars")?,
            colors: optional_values(py, dict, "colors")?,
            locations: optional_values(py, dict, "locations")?,
            texts: optional_values(py, dict, "texts")?,
            text_colors: optional_values(py, dict, "textColors")?,
            sizes: optional_values(py, dict, "sizes")?,
            metadata: metadata_from_py(py, dict)?,
        });
    }
    Ok(items)
}

fn plot_shapes_from_py(py: Python<'_>, list: Bound<'_, PyList>) -> PyResult<Vec<PlotShapeSeries>> {
    let mut items = Vec::with_capacity(list.len());
    for item in list {
        let dict = item
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("plotShape entries must be dicts"))?;
        items.push(PlotShapeSeries {
            id: dict_u32(dict, "id")?,
            values: optional_values(py, dict, "values")?,
            styles: optional_values(py, dict, "styles")?,
            locations: optional_values(py, dict, "locations")?,
            colors: optional_values(py, dict, "colors")?,
            texts: optional_values(py, dict, "texts")?,
            text_colors: optional_values(py, dict, "textColors")?,
            sizes: optional_values(py, dict, "sizes")?,
            metadata: metadata_from_py(py, dict)?,
        });
    }
    Ok(items)
}

fn plot_arrows_from_py(py: Python<'_>, list: Bound<'_, PyList>) -> PyResult<Vec<PlotArrowSeries>> {
    let mut items = Vec::with_capacity(list.len());
    for item in list {
        let dict = item
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("plotArrow entries must be dicts"))?;
        items.push(PlotArrowSeries {
            id: dict_u32(dict, "id")?,
            values: optional_values(py, dict, "values")?,
            color_ups: optional_values(py, dict, "colorUps")?,
            color_downs: optional_values(py, dict, "colorDowns")?,
            min_heights: optional_values(py, dict, "minHeights")?,
            max_heights: optional_values(py, dict, "maxHeights")?,
            metadata: metadata_from_py(py, dict)?,
        });
    }
    Ok(items)
}

fn plot_bars_from_py(py: Python<'_>, list: Bound<'_, PyList>) -> PyResult<Vec<PlotBarSeries>> {
    let mut items = Vec::with_capacity(list.len());
    for item in list {
        let dict = item
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("plotBar entries must be dicts"))?;
        items.push(PlotBarSeries {
            id: dict_u32(dict, "id")?,
            opens: optional_values(py, dict, "opens")?,
            highs: optional_values(py, dict, "highs")?,
            lows: optional_values(py, dict, "lows")?,
            closes: optional_values(py, dict, "closes")?,
            colors: optional_values(py, dict, "colors")?,
            metadata: metadata_from_py(py, dict)?,
        });
    }
    Ok(items)
}

fn plot_candles_from_py(
    py: Python<'_>,
    list: Bound<'_, PyList>,
) -> PyResult<Vec<PlotCandleSeries>> {
    let mut items = Vec::with_capacity(list.len());
    for item in list {
        let dict = item
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("plotCandle entries must be dicts"))?;
        items.push(PlotCandleSeries {
            id: dict_u32(dict, "id")?,
            opens: optional_values(py, dict, "opens")?,
            highs: optional_values(py, dict, "highs")?,
            lows: optional_values(py, dict, "lows")?,
            closes: optional_values(py, dict, "closes")?,
            colors: optional_values(py, dict, "colors")?,
            wick_colors: optional_values(py, dict, "wickColors")?,
            border_colors: optional_values(py, dict, "borderColors")?,
            metadata: metadata_from_py(py, dict)?,
        });
    }
    Ok(items)
}

fn colors_from_py(py: Python<'_>, list: Bound<'_, PyList>) -> PyResult<Vec<ColorSeries>> {
    let mut items = Vec::with_capacity(list.len());
    for item in list {
        let dict = item
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("color series entries must be dicts"))?;
        items.push(ColorSeries {
            id: dict_u32(dict, "id")?,
            values: optional_values(py, dict, "values")?,
            metadata: metadata_from_py(py, dict)?,
        });
    }
    Ok(items)
}

fn hlines_from_py(py: Python<'_>, list: Bound<'_, PyList>) -> PyResult<Vec<HLineOutput>> {
    let mut items = Vec::with_capacity(list.len());
    for item in list {
        let dict = item
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("hline entries must be dicts"))?;
        items.push(HLineOutput {
            id: dict_u32(dict, "id")?,
            price: dict_opt_value(py, dict, "price", PineValue::Na)?,
            title: dict_opt_value(py, dict, "title", PineValue::String(String::new()))?,
            color: dict_opt_value(py, dict, "color", PineValue::Color(0x787B86))?,
            style: dict_opt_value(
                py,
                dict,
                "style",
                PineValue::String("hline.style_solid".to_owned()),
            )?,
            linewidth: dict_opt_value(py, dict, "linewidth", PineValue::Int(1))?,
            editable: dict_opt_value(py, dict, "editable", PineValue::Bool(true))?,
            display: dict_opt_value(
                py,
                dict,
                "display",
                PineValue::String("display.all".to_owned()),
            )?,
        });
    }
    Ok(items)
}

fn fills_from_py(py: Python<'_>, list: Bound<'_, PyList>) -> PyResult<Vec<FillOutput>> {
    let mut items = Vec::with_capacity(list.len());
    for item in list {
        let dict = item
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("fill entries must be dicts"))?;
        items.push(FillOutput {
            id: dict_u32(dict, "id")?,
            first_id: dict_u32(dict, "firstId")?,
            second_id: dict_u32(dict, "secondId")?,
            first_is_hline: dict
                .get_item("firstIsHLine")?
                .and_then(|value| value.extract().ok())
                .unwrap_or(false),
            second_is_hline: dict
                .get_item("secondIsHLine")?
                .and_then(|value| value.extract().ok())
                .unwrap_or(false),
            colors: optional_values(py, dict, "colors")?,
            title: dict_opt_value(py, dict, "title", PineValue::String(String::new()))?,
            editable: dict_opt_value(py, dict, "editable", PineValue::Bool(true))?,
            show_last: dict_opt_value(py, dict, "showLast", PineValue::Na)?,
            fill_gaps: dict_opt_value(py, dict, "fillGaps", PineValue::Bool(true))?,
            display: dict_opt_value(
                py,
                dict,
                "display",
                PineValue::String("display.all".to_owned()),
            )?,
        });
    }
    Ok(items)
}

fn na_snapshot_value() -> PineValue {
    PineValue::Na
}

fn labels_from_py(py: Python<'_>, list: Bound<'_, PyList>) -> PyResult<Vec<LabelOutput>> {
    let mut items = Vec::with_capacity(list.len());
    for item in list {
        let dict = item
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("label entries must be dicts"))?;
        items.push(LabelOutput {
            id: dict_u32(dict, "id")?,
            snapshots: label_snapshots_from_py(py, dict_list(dict, "snapshots")?)?,
        });
    }
    Ok(items)
}

fn label_snapshots_from_py(
    py: Python<'_>,
    list: Bound<'_, PyList>,
) -> PyResult<Vec<LabelSnapshot>> {
    let mut items = Vec::with_capacity(list.len());
    for item in list {
        let dict = item
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("label snapshots must be dicts"))?;
        let exists = dict
            .get_item("exists")?
            .and_then(|value| value.extract().ok())
            .unwrap_or(true);
        items.push(LabelSnapshot {
            bar_index: dict_usize(dict, "barIndex")?,
            exists,
            x: dict_opt_value(py, dict, "x", na_snapshot_value())?,
            y: dict_opt_value(py, dict, "y", na_snapshot_value())?,
            text: dict_opt_value(py, dict, "text", PineValue::String(String::new()))?,
            xloc: dict_opt_value(py, dict, "xloc", PineValue::String(String::new()))?,
            yloc: dict_opt_value(py, dict, "yloc", PineValue::String(String::new()))?,
            color: dict_opt_value(py, dict, "color", PineValue::Na)?,
            style: dict_opt_value(py, dict, "style", PineValue::String(String::new()))?,
            text_color: dict_opt_value(py, dict, "textColor", PineValue::Na)?,
            size: dict_opt_value(py, dict, "size", PineValue::String(String::new()))?,
            tooltip: dict_opt_value(py, dict, "tooltip", PineValue::String(String::new()))?,
            text_align: dict_opt_value(py, dict, "textAlign", PineValue::String(String::new()))?,
            text_font_family: dict_opt_value(
                py,
                dict,
                "textFontFamily",
                PineValue::String(String::new()),
            )?,
            text_formatting: dict_opt_value(py, dict, "textFormatting", PineValue::Int(0))?,
        });
    }
    Ok(items)
}

fn lines_from_py(py: Python<'_>, list: Bound<'_, PyList>) -> PyResult<Vec<LineOutput>> {
    let mut items = Vec::with_capacity(list.len());
    for item in list {
        let dict = item
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("line entries must be dicts"))?;
        items.push(LineOutput {
            id: dict_u32(dict, "id")?,
            snapshots: line_snapshots_from_py(py, dict_list(dict, "snapshots")?)?,
        });
    }
    Ok(items)
}

fn line_snapshots_from_py(py: Python<'_>, list: Bound<'_, PyList>) -> PyResult<Vec<LineSnapshot>> {
    let mut items = Vec::with_capacity(list.len());
    for item in list {
        let dict = item
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("line snapshots must be dicts"))?;
        let exists = dict
            .get_item("exists")?
            .and_then(|value| value.extract().ok())
            .unwrap_or(true);
        items.push(LineSnapshot {
            bar_index: dict_usize(dict, "barIndex")?,
            exists,
            x1: dict_opt_value(py, dict, "x1", PineValue::Na)?,
            y1: dict_opt_value(py, dict, "y1", PineValue::Na)?,
            x2: dict_opt_value(py, dict, "x2", PineValue::Na)?,
            y2: dict_opt_value(py, dict, "y2", PineValue::Na)?,
            xloc: dict_opt_value(py, dict, "xloc", PineValue::String(String::new()))?,
            color: dict_opt_value(py, dict, "color", PineValue::Na)?,
            width: dict_opt_value(py, dict, "width", PineValue::Int(1))?,
            style: dict_opt_value(py, dict, "style", PineValue::String(String::new()))?,
            extend: dict_opt_value(py, dict, "extend", PineValue::String(String::new()))?,
        });
    }
    Ok(items)
}

fn line_fills_from_py(py: Python<'_>, list: Bound<'_, PyList>) -> PyResult<Vec<LineFillOutput>> {
    let mut items = Vec::with_capacity(list.len());
    for item in list {
        let dict = item
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("lineFill entries must be dicts"))?;
        items.push(LineFillOutput {
            id: dict_u32(dict, "id")?,
            snapshots: line_fill_snapshots_from_py(py, dict_list(dict, "snapshots")?)?,
        });
    }
    Ok(items)
}

fn line_fill_snapshots_from_py(
    py: Python<'_>,
    list: Bound<'_, PyList>,
) -> PyResult<Vec<LineFillSnapshot>> {
    let mut items = Vec::with_capacity(list.len());
    for item in list {
        let dict = item
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("lineFill snapshots must be dicts"))?;
        let exists = dict
            .get_item("exists")?
            .and_then(|value| value.extract().ok())
            .unwrap_or(true);
        items.push(LineFillSnapshot {
            bar_index: dict_usize(dict, "barIndex")?,
            exists,
            line1: dict
                .get_item("line1")?
                .and_then(|value| value.extract().ok())
                .unwrap_or(0),
            line2: dict
                .get_item("line2")?
                .and_then(|value| value.extract().ok())
                .unwrap_or(0),
            color: dict_opt_value(py, dict, "color", PineValue::Na)?,
        });
    }
    Ok(items)
}

fn polylines_from_py(py: Python<'_>, list: Bound<'_, PyList>) -> PyResult<Vec<PolylineOutput>> {
    let mut items = Vec::with_capacity(list.len());
    for item in list {
        let dict = item
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("polyline entries must be dicts"))?;
        items.push(PolylineOutput {
            id: dict_u32(dict, "id")?,
            snapshots: polyline_snapshots_from_py(py, dict_list(dict, "snapshots")?)?,
        });
    }
    Ok(items)
}

fn polyline_snapshots_from_py(
    py: Python<'_>,
    list: Bound<'_, PyList>,
) -> PyResult<Vec<PolylineSnapshot>> {
    let mut items = Vec::with_capacity(list.len());
    for item in list {
        let dict = item
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("polyline snapshots must be dicts"))?;
        let exists = dict
            .get_item("exists")?
            .and_then(|value| value.extract().ok())
            .unwrap_or(true);
        items.push(PolylineSnapshot {
            bar_index: dict_usize(dict, "barIndex")?,
            exists,
            points: optional_values(py, dict, "points")?,
            curved: dict_opt_value(py, dict, "curved", PineValue::Bool(false))?,
            closed: dict_opt_value(py, dict, "closed", PineValue::Bool(false))?,
            xloc: dict_opt_value(py, dict, "xloc", PineValue::String(String::new()))?,
            line_color: dict_opt_value(py, dict, "lineColor", PineValue::Na)?,
            fill_color: dict_opt_value(py, dict, "fillColor", PineValue::Na)?,
            line_style: dict_opt_value(py, dict, "lineStyle", PineValue::String(String::new()))?,
            line_width: dict_opt_value(py, dict, "lineWidth", PineValue::Int(1))?,
            force_overlay: dict_opt_value(py, dict, "forceOverlay", PineValue::Bool(false))?,
        });
    }
    Ok(items)
}

fn boxes_from_py(py: Python<'_>, list: Bound<'_, PyList>) -> PyResult<Vec<BoxOutput>> {
    let mut items = Vec::with_capacity(list.len());
    for item in list {
        let dict = item
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("box entries must be dicts"))?;
        items.push(BoxOutput {
            id: dict_u32(dict, "id")?,
            snapshots: box_snapshots_from_py(py, dict_list(dict, "snapshots")?)?,
        });
    }
    Ok(items)
}

fn box_snapshots_from_py(py: Python<'_>, list: Bound<'_, PyList>) -> PyResult<Vec<BoxSnapshot>> {
    let mut items = Vec::with_capacity(list.len());
    for item in list {
        let dict = item
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("box snapshots must be dicts"))?;
        let exists = dict
            .get_item("exists")?
            .and_then(|value| value.extract().ok())
            .unwrap_or(true);
        items.push(BoxSnapshot {
            bar_index: dict_usize(dict, "barIndex")?,
            exists,
            left: dict_opt_value(py, dict, "left", PineValue::Na)?,
            top: dict_opt_value(py, dict, "top", PineValue::Na)?,
            right: dict_opt_value(py, dict, "right", PineValue::Na)?,
            bottom: dict_opt_value(py, dict, "bottom", PineValue::Na)?,
            xloc: dict_opt_value(py, dict, "xloc", PineValue::String(String::new()))?,
            bg_color: dict_opt_value(py, dict, "bgColor", PineValue::Na)?,
            border_color: dict_opt_value(py, dict, "borderColor", PineValue::Na)?,
            border_width: dict_opt_value(py, dict, "borderWidth", PineValue::Int(1))?,
            border_style: dict_opt_value(
                py,
                dict,
                "borderStyle",
                PineValue::String(String::new()),
            )?,
            extend: dict_opt_value(py, dict, "extend", PineValue::String(String::new()))?,
            text: dict_opt_value(py, dict, "text", PineValue::String(String::new()))?,
            text_color: dict_opt_value(py, dict, "textColor", PineValue::Na)?,
            text_size: dict_opt_value(py, dict, "textSize", PineValue::String(String::new()))?,
            text_halign: dict_opt_value(py, dict, "textHalign", PineValue::String(String::new()))?,
            text_valign: dict_opt_value(py, dict, "textValign", PineValue::String(String::new()))?,
            text_wrap: dict_opt_value(py, dict, "textWrap", PineValue::String(String::new()))?,
            text_font_family: dict_opt_value(
                py,
                dict,
                "textFontFamily",
                PineValue::String(String::new()),
            )?,
            text_formatting: dict_opt_value(py, dict, "textFormatting", PineValue::Int(0))?,
        });
    }
    Ok(items)
}

fn tables_from_py(py: Python<'_>, list: Bound<'_, PyList>) -> PyResult<Vec<TableOutput>> {
    let mut items = Vec::with_capacity(list.len());
    for item in list {
        let dict = item
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("table entries must be dicts"))?;
        items.push(TableOutput {
            id: dict_u32(dict, "id")?,
            position: dict_opt_value(py, dict, "position", PineValue::String(String::new()))?,
            bg_color: dict_opt_value(py, dict, "bgColor", PineValue::Na)?,
            frame_color: dict_opt_value(py, dict, "frameColor", PineValue::Na)?,
            frame_width: dict_opt_value(py, dict, "frameWidth", PineValue::Int(0))?,
            border_color: dict_opt_value(py, dict, "borderColor", PineValue::Na)?,
            border_width: dict_opt_value(py, dict, "borderWidth", PineValue::Int(0))?,
            columns: dict
                .get_item("columns")?
                .and_then(|value| value.extract().ok())
                .unwrap_or(0),
            rows: dict
                .get_item("rows")?
                .and_then(|value| value.extract().ok())
                .unwrap_or(0),
            snapshots: table_snapshots_from_py(py, dict_list(dict, "snapshots")?)?,
        });
    }
    Ok(items)
}

fn table_snapshots_from_py(
    py: Python<'_>,
    list: Bound<'_, PyList>,
) -> PyResult<Vec<TableSnapshot>> {
    let mut items = Vec::with_capacity(list.len());
    for item in list {
        let dict = item
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("table snapshots must be dicts"))?;
        items.push(TableSnapshot {
            bar_index: dict_usize(dict, "barIndex")?,
            exists: dict
                .get_item("exists")?
                .and_then(|value| value.extract().ok())
                .unwrap_or(true),
            cells: table_cells_from_py(py, dict)?,
            merged_cells: table_merged_from_py(dict)?,
        });
    }
    Ok(items)
}

fn table_cells_from_py(
    py: Python<'_>,
    dict: &Bound<'_, PyDict>,
) -> PyResult<Vec<TableCellSnapshot>> {
    let Some(list) = dict.get_item("cells")? else {
        return Ok(Vec::new());
    };
    if list.is_none() {
        return Ok(Vec::new());
    }
    let list = list
        .cast::<PyList>()
        .map_err(|_| PyValueError::new_err("table cells must be a list"))?;
    let mut items = Vec::with_capacity(list.len());
    for item in list {
        let cell = item
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("table cell entries must be dicts"))?;
        items.push(TableCellSnapshot {
            column: cell
                .get_item("column")?
                .and_then(|value| value.extract().ok())
                .unwrap_or(0),
            row: cell
                .get_item("row")?
                .and_then(|value| value.extract().ok())
                .unwrap_or(0),
            text: dict_opt_value(py, cell, "text", PineValue::String(String::new()))?,
            bg_color: dict_opt_value(py, cell, "bgColor", PineValue::Na)?,
            text_color: dict_opt_value(py, cell, "textColor", PineValue::Na)?,
            width: dict_opt_value(py, cell, "width", PineValue::Na)?,
            height: dict_opt_value(py, cell, "height", PineValue::Na)?,
            text_size: dict_opt_value(py, cell, "textSize", PineValue::String(String::new()))?,
            text_halign: dict_opt_value(py, cell, "textHalign", PineValue::String(String::new()))?,
            text_valign: dict_opt_value(py, cell, "textValign", PineValue::String(String::new()))?,
            text_wrap: dict_opt_value(py, cell, "textWrap", PineValue::String(String::new()))?,
            tooltip: dict_opt_value(py, cell, "tooltip", PineValue::String(String::new()))?,
            text_font_family: dict_opt_value(
                py,
                cell,
                "textFontFamily",
                PineValue::String(String::new()),
            )?,
            text_formatting: dict_opt_value(py, cell, "textFormatting", PineValue::Int(0))?,
        });
    }
    Ok(items)
}

fn table_merged_from_py(dict: &Bound<'_, PyDict>) -> PyResult<Vec<TableMergedCellSnapshot>> {
    let Some(list) = dict.get_item("mergedCells")? else {
        return Ok(Vec::new());
    };
    if list.is_none() {
        return Ok(Vec::new());
    }
    let list = list
        .cast::<PyList>()
        .map_err(|_| PyValueError::new_err("mergedCells must be a list"))?;
    let mut items = Vec::with_capacity(list.len());
    for item in list {
        let cell = item
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("merged cell entries must be dicts"))?;
        items.push(TableMergedCellSnapshot {
            start_column: cell
                .get_item("startColumn")?
                .and_then(|value| value.extract().ok())
                .unwrap_or(0),
            start_row: cell
                .get_item("startRow")?
                .and_then(|value| value.extract().ok())
                .unwrap_or(0),
            end_column: cell
                .get_item("endColumn")?
                .and_then(|value| value.extract().ok())
                .unwrap_or(0),
            end_row: cell
                .get_item("endRow")?
                .and_then(|value| value.extract().ok())
                .unwrap_or(0),
        });
    }
    Ok(items)
}

pub(crate) fn alerts_from_py(list: Bound<'_, PyList>) -> PyResult<Vec<AlertEvent>> {
    let mut items = Vec::with_capacity(list.len());
    for item in list {
        let dict = item
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("alert entries must be dicts"))?;
        items.push(alert_from_py(dict)?);
    }
    Ok(items)
}

pub(crate) fn alert_from_py(dict: &Bound<'_, PyDict>) -> PyResult<AlertEvent> {
    Ok(AlertEvent {
        id: dict_u32(dict, "id")?,
        bar_index: dict_usize(dict, "barIndex")?,
        time: dict
            .get_item("time")?
            .ok_or_else(|| PyValueError::new_err("alert is missing `time`"))?
            .extract()?,
        message: dict_string(dict, "message")?,
        source: dict_string(dict, "source")?,
    })
}

pub(crate) fn diagnostics_from_py(list: Bound<'_, PyList>) -> PyResult<Vec<RuntimeDiagnostic>> {
    let mut items = Vec::with_capacity(list.len());
    for item in list {
        let dict = item
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("diagnostic entries must be dicts"))?;
        items.push(RuntimeDiagnostic {
            code: dict_string(dict, "code")?,
            message: dict_string(dict, "message")?,
        });
    }
    Ok(items)
}

fn strategy_from_py(_py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<StrategyResult> {
    let dict = value
        .cast::<PyDict>()
        .map_err(|_| PyValueError::new_err("strategy result must be a dict"))?;
    Ok(StrategyResult {
        orders: strategy_orders_from_py(dict_list(dict, "orders")?)?,
        trades: strategy_trades_from_py(dict_list(dict, "trades")?)?,
        position: strategy_position_from_py(dict_list(dict, "position")?)?,
        equity: strategy_equity_from_py(dict_list(dict, "equity")?)?,
        alerts: strategy_alerts_from_py(dict_list(dict, "alerts")?)?,
        diagnostics: diagnostics_from_py(dict_list(dict, "diagnostics")?)?,
    })
}

pub(crate) fn strategy_orders_from_py(
    list: Bound<'_, PyList>,
) -> PyResult<Vec<StrategyOrderEvent>> {
    let mut items = Vec::with_capacity(list.len());
    for item in list {
        let dict = item
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("strategy order entries must be dicts"))?;
        items.push(strategy_order_from_py(dict)?);
    }
    Ok(items)
}

pub(crate) fn strategy_order_from_py(dict: &Bound<'_, PyDict>) -> PyResult<StrategyOrderEvent> {
    Ok(StrategyOrderEvent {
        id: dict_string(dict, "id")?,
        bar_index: dict_usize(dict, "barIndex")?,
        time: dict
            .get_item("time")?
            .ok_or_else(|| PyValueError::new_err("order is missing `time`"))?
            .extract()?,
        direction: dict_string(dict, "direction")?,
        qty: dict_f64(dict, "qty")?,
        price: dict_f64(dict, "price")?,
    })
}

pub(crate) fn strategy_trades_from_py(list: Bound<'_, PyList>) -> PyResult<Vec<StrategyTrade>> {
    let mut items = Vec::with_capacity(list.len());
    for item in list {
        let dict = item
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("strategy trade entries must be dicts"))?;
        items.push(StrategyTrade {
            id: dict_string(dict, "id")?,
            exit_id: dict
                .get_item("exitId")?
                .and_then(|value| value.extract().ok())
                .unwrap_or_default(),
            entry_bar_index: dict_usize(dict, "entryBarIndex")?,
            exit_bar_index: dict_usize(dict, "exitBarIndex")?,
            entry_time: dict
                .get_item("entryTime")?
                .and_then(|value| value.extract().ok())
                .unwrap_or(0),
            exit_time: dict
                .get_item("exitTime")?
                .and_then(|value| value.extract().ok())
                .unwrap_or(0),
            entry_price: dict_f64(dict, "entryPrice")?,
            exit_price: dict_f64(dict, "exitPrice")?,
            qty: dict_f64(dict, "qty")?,
            profit: dict
                .get_item("profit")?
                .and_then(|value| value.extract().ok())
                .unwrap_or(0.0),
        });
    }
    Ok(items)
}

fn strategy_position_from_py(list: Bound<'_, PyList>) -> PyResult<Vec<StrategyPositionSnapshot>> {
    let mut items = Vec::with_capacity(list.len());
    for item in list {
        let dict = item
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("strategy position entries must be dicts"))?;
        items.push(StrategyPositionSnapshot {
            bar_index: dict_usize(dict, "barIndex")?,
            size: dict_f64(dict, "size")?,
            avg_price: dict.get_item("avgPrice")?.and_then(|value| {
                if value.is_none() {
                    None
                } else {
                    value.extract().ok()
                }
            }),
        });
    }
    Ok(items)
}

fn strategy_equity_from_py(list: Bound<'_, PyList>) -> PyResult<Vec<StrategyEquitySnapshot>> {
    let mut items = Vec::with_capacity(list.len());
    for item in list {
        let dict = item
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("strategy equity entries must be dicts"))?;
        items.push(StrategyEquitySnapshot {
            bar_index: dict_usize(dict, "barIndex")?,
            cash: dict_f64(dict, "cash")?,
            market_value: dict_f64(dict, "marketValue")?,
            equity: dict_f64(dict, "equity")?,
            net_profit: dict_f64(dict, "netProfit")?,
        });
    }
    Ok(items)
}

pub(crate) fn strategy_alerts_from_py(
    list: Bound<'_, PyList>,
) -> PyResult<Vec<StrategyOrderFillAlertOutput>> {
    let mut items = Vec::with_capacity(list.len());
    for item in list {
        let dict = item
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("strategy alert entries must be dicts"))?;
        items.push(strategy_fill_alert_from_py(dict)?);
    }
    Ok(items)
}

pub(crate) fn strategy_fill_alert_from_py(
    dict: &Bound<'_, PyDict>,
) -> PyResult<StrategyOrderFillAlertOutput> {
    Ok(StrategyOrderFillAlertOutput {
        id: dict_string(dict, "id")?,
        bar_index: dict_usize(dict, "barIndex")?,
        time: dict
            .get_item("time")?
            .and_then(|value| value.extract().ok())
            .unwrap_or(0),
        direction: dict_string(dict, "direction")?,
        qty: dict_f64(dict, "qty")?,
        price: dict_f64(dict, "price")?,
        entry_id: dict.get_item("entryId")?.and_then(|value| {
            if value.is_none() {
                None
            } else {
                value.extract().ok()
            }
        }),
        exit_id: dict.get_item("exitId")?.and_then(|value| {
            if value.is_none() {
                None
            } else {
                value.extract().ok()
            }
        }),
        message: dict
            .get_item("message")?
            .and_then(|value| value.extract().ok())
            .unwrap_or_default(),
    })
}

fn dict_f64(dict: &Bound<'_, PyDict>, key: &str) -> PyResult<f64> {
    let value = dict
        .get_item(key)?
        .ok_or_else(|| PyValueError::new_err(format!("missing `{key}`")))?;
    if value.is_none() {
        return Ok(f64::NAN);
    }
    value.extract()
}

#[allow(dead_code)]
pub(crate) fn value_roundtrip_ok(py: Python<'_>, value: &PineValue) -> PyResult<bool> {
    let converted = value_to_py(py, value)?;
    Ok(pine_value_from_py(py, converted.bind(py))? == *value)
}

use pine_runtime::{
    DrawingAction, DrawingChange, DrawingFamily, DrawingObject, EventAction, EventChange,
    FillAction, FillChange, HLineAction, HLineChange, ListSplice,
    PUBLIC_RUNTIME_CHANGES_SCHEMA_VERSION, RuntimeChanges, SeriesChange, SeriesChangeOp,
    SeriesFamily, SeriesFields, SeriesHeader, StrategyChanges, StreamingVisibility,
};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyList, PyModule};

use crate::outputs::{
    alerts_to_py, boxes_to_py, fills_to_py, hlines_to_py, labels_to_py, line_fills_to_py,
    lines_to_py, polylines_to_py, runtime_diagnostics_to_py, set_non_default_value,
    set_output_metadata, strategy_equity_to_py, strategy_order_fill_alerts_to_py,
    strategy_orders_to_py, strategy_position_to_py, strategy_trades_to_py, values_to_py,
};
use crate::result_parse::{
    alert_from_py, diagnostics_from_py, dict_string, dict_u32, dict_usize, first_item,
    optional_values, runtime_result_from_py, series_header_from_py, strategy_alerts_from_py,
    strategy_orders_from_py, strategy_trades_from_py,
};
use crate::tables::tables_to_py;

pub(crate) fn runtime_changes_to_py(
    py: Python<'_>,
    changes: &RuntimeChanges,
) -> PyResult<Py<PyAny>> {
    let output = PyDict::new(py);
    output.set_item("schemaVersion", changes.schema_version)?;
    output.set_item("revision", changes.revision)?;
    output.set_item("baseRevision", changes.base_revision)?;
    output.set_item("visibility", changes.visibility.as_str())?;
    output.set_item("series", series_changes_to_py(py, &changes.series)?)?;
    output.set_item("hlines", hline_changes_to_py(py, &changes.hlines)?)?;
    output.set_item("fills", fill_changes_to_py(py, &changes.fills)?)?;
    output.set_item("drawings", drawing_changes_to_py(py, &changes.drawings)?)?;
    output.set_item("alerts", event_changes_to_py(py, &changes.alerts)?)?;
    if let Some(strategy) = &changes.strategy {
        output.set_item("strategy", strategy_changes_to_py(py, strategy)?)?;
    }
    output.set_item(
        "diagnostics",
        runtime_diagnostics_to_py(py, &changes.diagnostics)?,
    )?;
    Ok(output.into_any().unbind())
}

fn series_changes_to_py(py: Python<'_>, changes: &[SeriesChange]) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for change in changes {
        let item = PyDict::new(py);
        item.set_item("family", change.family.as_str())?;
        item.set_item("id", change.id)?;
        item.set_item("op", change.op.as_str())?;
        item.set_item("start", change.start)?;
        set_values_if_present(py, &item, "values", &change.fields.values)?;
        set_values_if_present(py, &item, "colors", &change.fields.colors)?;
        set_values_if_present(py, &item, "chars", &change.fields.chars)?;
        set_values_if_present(py, &item, "locations", &change.fields.locations)?;
        set_values_if_present(py, &item, "texts", &change.fields.texts)?;
        set_values_if_present(py, &item, "textColors", &change.fields.text_colors)?;
        set_values_if_present(py, &item, "sizes", &change.fields.sizes)?;
        set_values_if_present(py, &item, "styles", &change.fields.styles)?;
        set_values_if_present(py, &item, "colorUps", &change.fields.color_ups)?;
        set_values_if_present(py, &item, "colorDowns", &change.fields.color_downs)?;
        set_values_if_present(py, &item, "minHeights", &change.fields.min_heights)?;
        set_values_if_present(py, &item, "maxHeights", &change.fields.max_heights)?;
        set_values_if_present(py, &item, "opens", &change.fields.opens)?;
        set_values_if_present(py, &item, "highs", &change.fields.highs)?;
        set_values_if_present(py, &item, "lows", &change.fields.lows)?;
        set_values_if_present(py, &item, "closes", &change.fields.closes)?;
        set_values_if_present(py, &item, "wickColors", &change.fields.wick_colors)?;
        set_values_if_present(py, &item, "borderColors", &change.fields.border_colors)?;
        if let Some(header) = &change.header {
            item.set_item("header", series_header_to_py(py, header)?)?;
        }
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn series_header_to_py(py: Python<'_>, header: &SeriesHeader) -> PyResult<Py<PyAny>> {
    let item = PyDict::new(py);
    let defaults = SeriesHeader::default();
    set_output_metadata(py, &item, &header.metadata)?;
    set_non_default_value(
        py,
        &item,
        "linewidth",
        &header.linewidth,
        &defaults.linewidth,
    )?;
    set_non_default_value(py, &item, "style", &header.style, &defaults.style)?;
    set_non_default_value(
        py,
        &item,
        "trackPrice",
        &header.track_price,
        &defaults.track_price,
    )?;
    set_non_default_value(
        py,
        &item,
        "histBase",
        &header.hist_base,
        &defaults.hist_base,
    )?;
    set_non_default_value(py, &item, "join", &header.join, &defaults.join)?;
    set_non_default_value(py, &item, "format", &header.format, &defaults.format)?;
    set_non_default_value(
        py,
        &item,
        "precision",
        &header.precision,
        &defaults.precision,
    )?;
    Ok(item.into_any().unbind())
}

fn set_values_if_present(
    py: Python<'_>,
    item: &Bound<'_, PyDict>,
    name: &str,
    values: &[pine_runtime::PineValue],
) -> PyResult<()> {
    if !values.is_empty() {
        item.set_item(name, values_to_py(py, values)?)?;
    }
    Ok(())
}

fn hline_changes_to_py(py: Python<'_>, changes: &[HLineChange]) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for change in changes {
        let item = PyDict::new(py);
        item.set_item("id", change.id)?;
        match &change.action {
            HLineAction::Add(hline) => {
                item.set_item("action", "add")?;
                item.set_item(
                    "object",
                    first_item(py, hlines_to_py(py, std::slice::from_ref(hline))?)?,
                )?;
            }
            HLineAction::Replace(hline) => {
                item.set_item("action", "replace")?;
                item.set_item(
                    "object",
                    first_item(py, hlines_to_py(py, std::slice::from_ref(hline))?)?,
                )?;
            }
            HLineAction::Delete => item.set_item("action", "delete")?,
        }
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn fill_changes_to_py(py: Python<'_>, changes: &[FillChange]) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for change in changes {
        let item = PyDict::new(py);
        item.set_item("id", change.id)?;
        match &change.action {
            FillAction::Add(fill) => {
                item.set_item("action", "add")?;
                item.set_item(
                    "object",
                    first_item(py, fills_to_py(py, std::slice::from_ref(fill))?)?,
                )?;
            }
            FillAction::Delete => item.set_item("action", "delete")?,
            FillAction::SetColors { start, values } => {
                item.set_item("action", "setColors")?;
                item.set_item("start", *start)?;
                item.set_item("values", values_to_py(py, values)?)?;
            }
        }
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn drawing_changes_to_py(py: Python<'_>, changes: &[DrawingChange]) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for change in changes {
        let item = PyDict::new(py);
        item.set_item("family", change.family.as_str())?;
        item.set_item("id", change.id)?;
        item.set_item("action", change.action.as_str())?;
        match &change.action {
            DrawingAction::Add(object) => {
                item.set_item("object", drawing_object_to_py(py, object)?)?;
            }
            DrawingAction::SetTail { start, object } => {
                item.set_item("start", *start)?;
                item.set_item("object", drawing_object_to_py(py, object)?)?;
            }
            DrawingAction::Delete => {}
        }
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn drawing_object_to_py(py: Python<'_>, object: &DrawingObject) -> PyResult<Py<PyAny>> {
    match object {
        DrawingObject::Label(item) => first_item(py, labels_to_py(py, std::slice::from_ref(item))?),
        DrawingObject::Line(item) => first_item(py, lines_to_py(py, std::slice::from_ref(item))?),
        DrawingObject::LineFill(item) => {
            first_item(py, line_fills_to_py(py, std::slice::from_ref(item))?)
        }
        DrawingObject::Polyline(item) => {
            first_item(py, polylines_to_py(py, std::slice::from_ref(item))?)
        }
        DrawingObject::Box(item) => first_item(py, boxes_to_py(py, std::slice::from_ref(item))?),
        DrawingObject::Table(item) => first_item(py, tables_to_py(py, std::slice::from_ref(item))?),
    }
}

fn event_changes_to_py(
    py: Python<'_>,
    changes: &[EventChange<pine_runtime::AlertEvent>],
) -> PyResult<Py<PyAny>> {
    let output = PyList::empty(py);
    for change in changes {
        let item = PyDict::new(py);
        item.set_item("action", change.action.as_str())?;
        item.set_item(
            "event",
            first_item(py, alerts_to_py(py, std::slice::from_ref(&change.event))?)?,
        )?;
        output.append(item)?;
    }
    Ok(output.into_any().unbind())
}

fn strategy_changes_to_py(py: Python<'_>, changes: &StrategyChanges) -> PyResult<Py<PyAny>> {
    let output = PyDict::new(py);
    if let Some(orders) = &changes.orders {
        output.set_item(
            "orders",
            list_splice_to_py(py, orders.start, &strategy_orders_to_py(py, &orders.items)?)?,
        )?;
    }
    if let Some(trades) = &changes.trades {
        output.set_item(
            "trades",
            list_splice_to_py(py, trades.start, &strategy_trades_to_py(py, &trades.items)?)?,
        )?;
    }
    if let Some(alerts) = &changes.alerts {
        output.set_item(
            "alerts",
            list_splice_to_py(
                py,
                alerts.start,
                &strategy_order_fill_alerts_to_py(py, &alerts.items)?,
            )?,
        )?;
    }
    if let Some(position) = &changes.position {
        output.set_item(
            "position",
            list_splice_to_py(
                py,
                position.start,
                &strategy_position_to_py(py, &position.items)?,
            )?,
        )?;
    }
    if let Some(equity) = &changes.equity {
        output.set_item(
            "equity",
            list_splice_to_py(py, equity.start, &strategy_equity_to_py(py, &equity.items)?)?,
        )?;
    }
    if let Some(diagnostics) = &changes.diagnostics {
        output.set_item("diagnostics", runtime_diagnostics_to_py(py, diagnostics)?)?;
    }
    Ok(output.into_any().unbind())
}

fn list_splice_to_py(py: Python<'_>, start: usize, items: &Py<PyAny>) -> PyResult<Py<PyAny>> {
    let output = PyDict::new(py);
    output.set_item("start", start)?;
    output.set_item("items", items)?;
    Ok(output.into_any().unbind())
}

pub(crate) fn runtime_changes_from_py(
    py: Python<'_>,
    value: &Bound<'_, PyAny>,
) -> PyResult<RuntimeChanges> {
    let dict = value
        .cast::<PyDict>()
        .map_err(|_| PyValueError::new_err("runtime changes must be a dict"))?;
    let visibility = match dict_string(dict, "visibility")?.as_str() {
        "preview" => StreamingVisibility::Preview,
        "confirmed" => StreamingVisibility::Confirmed,
        other => {
            return Err(PyValueError::new_err(format!(
                "unsupported visibility `{other}`"
            )));
        }
    };
    let mut changes = RuntimeChanges::new(
        dict.get_item("revision")?
            .ok_or_else(|| PyValueError::new_err("changes missing revision"))?
            .extract()?,
        visibility,
    );
    changes.base_revision = dict
        .get_item("baseRevision")?
        .ok_or_else(|| PyValueError::new_err("changes missing baseRevision"))?
        .extract()?;
    changes.schema_version = dict
        .get_item("schemaVersion")?
        .ok_or_else(|| PyValueError::new_err("changes missing schemaVersion"))?
        .extract()?;
    if let Some(series) = dict.get_item("series")? {
        changes.series = series_changes_from_py(py, &series)?;
    }
    if let Some(hlines) = dict.get_item("hlines")? {
        changes.hlines = hline_changes_from_py(py, &hlines)?;
    }
    if let Some(fills) = dict.get_item("fills")? {
        changes.fills = fill_changes_from_py(py, &fills)?;
    }
    if let Some(drawings) = dict.get_item("drawings")? {
        changes.drawings = drawing_changes_from_py(py, &drawings)?;
    }
    if let Some(alerts) = dict.get_item("alerts")? {
        changes.alerts = event_changes_from_py(&alerts)?;
    }
    match dict.get_item("strategy")? {
        Some(value) if !value.is_none() => {
            changes.strategy = Some(strategy_changes_from_py(py, &value)?);
        }
        _ => {}
    }
    if let Some(diagnostics) = dict.get_item("diagnostics")? {
        let list = diagnostics
            .cast::<PyList>()
            .map_err(|_| PyValueError::new_err("diagnostics must be a list"))?;
        changes.diagnostics = diagnostics_from_py(list.to_owned())?;
    }
    Ok(changes)
}

fn series_changes_from_py(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Vec<SeriesChange>> {
    let list = value
        .cast::<PyList>()
        .map_err(|_| PyValueError::new_err("series changes must be a list"))?;
    let mut items = Vec::with_capacity(list.len());
    for item in list {
        let dict = item
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("series change must be a dict"))?;
        items.push(SeriesChange {
            family: series_family(&dict_string(dict, "family")?)?,
            id: dict_u32(dict, "id")?,
            op: match dict_string(dict, "op")?.as_str() {
                "append" => SeriesChangeOp::Append,
                "replaceLast" => SeriesChangeOp::ReplaceLast,
                other => {
                    return Err(PyValueError::new_err(format!(
                        "unsupported series op `{other}`"
                    )));
                }
            },
            start: dict_usize(dict, "start")?,
            fields: SeriesFields {
                values: optional_values(py, dict, "values")?,
                colors: optional_values(py, dict, "colors")?,
                chars: optional_values(py, dict, "chars")?,
                locations: optional_values(py, dict, "locations")?,
                texts: optional_values(py, dict, "texts")?,
                text_colors: optional_values(py, dict, "textColors")?,
                sizes: optional_values(py, dict, "sizes")?,
                styles: optional_values(py, dict, "styles")?,
                color_ups: optional_values(py, dict, "colorUps")?,
                color_downs: optional_values(py, dict, "colorDowns")?,
                min_heights: optional_values(py, dict, "minHeights")?,
                max_heights: optional_values(py, dict, "maxHeights")?,
                opens: optional_values(py, dict, "opens")?,
                highs: optional_values(py, dict, "highs")?,
                lows: optional_values(py, dict, "lows")?,
                closes: optional_values(py, dict, "closes")?,
                wick_colors: optional_values(py, dict, "wickColors")?,
                border_colors: optional_values(py, dict, "borderColors")?,
            },
            header: match dict.get_item("header")? {
                Some(value) if !value.is_none() => {
                    let header = value.cast::<PyDict>().map_err(|_| {
                        PyValueError::new_err("series change header must be a dict")
                    })?;
                    Some(series_header_from_py(py, header)?)
                }
                _ => None,
            },
        });
    }
    Ok(items)
}

fn series_family(value: &str) -> PyResult<SeriesFamily> {
    Ok(match value {
        "plot" => SeriesFamily::Plot,
        "plotChar" => SeriesFamily::PlotChar,
        "plotShape" => SeriesFamily::PlotShape,
        "plotArrow" => SeriesFamily::PlotArrow,
        "plotBar" => SeriesFamily::PlotBar,
        "plotCandle" => SeriesFamily::PlotCandle,
        "bgColor" => SeriesFamily::BgColor,
        "barColor" => SeriesFamily::BarColor,
        other => {
            return Err(PyValueError::new_err(format!(
                "unsupported series family `{other}`"
            )));
        }
    })
}

fn hline_changes_from_py(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Vec<HLineChange>> {
    let list = value
        .cast::<PyList>()
        .map_err(|_| PyValueError::new_err("hline changes must be a list"))?;
    let mut items = Vec::with_capacity(list.len());
    for item in list {
        let dict = item
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("hline change must be a dict"))?;
        let id = dict_u32(dict, "id")?;
        let action = match dict_string(dict, "action")?.as_str() {
            "delete" => HLineAction::Delete,
            "add" | "replace" => {
                let object = dict
                    .get_item("object")?
                    .ok_or_else(|| PyValueError::new_err("hline change missing object"))?;
                let wrapper = hline_wrapper(py, &object)?;
                let parsed = runtime_result_from_py(py, wrapper.as_any())?;
                let hline = parsed
                    .hlines
                    .into_iter()
                    .next()
                    .ok_or_else(|| PyValueError::new_err("hline object did not parse"))?;
                if dict_string(dict, "action")? == "add" {
                    HLineAction::Add(hline)
                } else {
                    HLineAction::Replace(hline)
                }
            }
            other => {
                return Err(PyValueError::new_err(format!(
                    "unsupported hline action `{other}`"
                )));
            }
        };
        items.push(HLineChange { id, action });
    }
    Ok(items)
}

fn hline_wrapper<'py>(py: Python<'py>, object: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyDict>> {
    let wrapper = PyDict::new(py);
    let list = PyList::empty(py);
    list.append(object)?;
    wrapper.set_item("hlines", list)?;
    Ok(wrapper)
}

fn fill_changes_from_py(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Vec<FillChange>> {
    let list = value
        .cast::<PyList>()
        .map_err(|_| PyValueError::new_err("fill changes must be a list"))?;
    let mut items = Vec::with_capacity(list.len());
    for item in list {
        let dict = item
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("fill change must be a dict"))?;
        let id = dict_u32(dict, "id")?;
        let action = match dict_string(dict, "action")?.as_str() {
            "delete" => FillAction::Delete,
            "setColors" => FillAction::SetColors {
                start: dict_usize(dict, "start")?,
                values: optional_values(py, dict, "values")?,
            },
            "add" => {
                let object = dict
                    .get_item("object")?
                    .ok_or_else(|| PyValueError::new_err("fill change missing object"))?;
                let wrapper = PyDict::new(py);
                let list = PyList::empty(py);
                list.append(object)?;
                wrapper.set_item("fills", list)?;
                let parsed = runtime_result_from_py(py, wrapper.as_any())?;
                FillAction::Add(
                    parsed
                        .fills
                        .into_iter()
                        .next()
                        .ok_or_else(|| PyValueError::new_err("fill object did not parse"))?,
                )
            }
            other => {
                return Err(PyValueError::new_err(format!(
                    "unsupported fill action `{other}`"
                )));
            }
        };
        items.push(FillChange { id, action });
    }
    Ok(items)
}

fn drawing_changes_from_py(
    py: Python<'_>,
    value: &Bound<'_, PyAny>,
) -> PyResult<Vec<DrawingChange>> {
    let list = value
        .cast::<PyList>()
        .map_err(|_| PyValueError::new_err("drawing changes must be a list"))?;
    let mut items = Vec::with_capacity(list.len());
    for item in list {
        let dict = item
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("drawing change must be a dict"))?;
        let family = drawing_family(&dict_string(dict, "family")?)?;
        let id = dict_u32(dict, "id")?;
        let action = match dict_string(dict, "action")?.as_str() {
            "delete" => DrawingAction::Delete,
            "add" => DrawingAction::Add(drawing_object_from_py(
                py,
                family,
                &dict
                    .get_item("object")?
                    .ok_or_else(|| PyValueError::new_err("drawing add missing object"))?,
            )?),
            "setTail" => DrawingAction::SetTail {
                start: dict_usize(dict, "start")?,
                object: drawing_object_from_py(
                    py,
                    family,
                    &dict
                        .get_item("object")?
                        .ok_or_else(|| PyValueError::new_err("drawing setTail missing object"))?,
                )?,
            },
            other => {
                return Err(PyValueError::new_err(format!(
                    "unsupported drawing action `{other}`"
                )));
            }
        };
        items.push(DrawingChange { family, id, action });
    }
    Ok(items)
}

fn drawing_family(value: &str) -> PyResult<DrawingFamily> {
    Ok(match value {
        "label" => DrawingFamily::Label,
        "line" => DrawingFamily::Line,
        "lineFill" => DrawingFamily::LineFill,
        "polyline" => DrawingFamily::Polyline,
        "box" => DrawingFamily::Box,
        "table" => DrawingFamily::Table,
        other => {
            return Err(PyValueError::new_err(format!(
                "unsupported drawing family `{other}`"
            )));
        }
    })
}

fn drawing_object_from_py(
    py: Python<'_>,
    family: DrawingFamily,
    object: &Bound<'_, PyAny>,
) -> PyResult<DrawingObject> {
    let wrapper = PyDict::new(py);
    let list = PyList::empty(py);
    list.append(object)?;
    let key = match family {
        DrawingFamily::Label => "labels",
        DrawingFamily::Line => "lines",
        DrawingFamily::LineFill => "lineFills",
        DrawingFamily::Polyline => "polylines",
        DrawingFamily::Box => "boxes",
        DrawingFamily::Table => "tables",
    };
    wrapper.set_item(key, list)?;
    let parsed = runtime_result_from_py(py, wrapper.as_any())?;
    Ok(match family {
        DrawingFamily::Label => DrawingObject::Label(
            parsed
                .labels
                .into_iter()
                .next()
                .ok_or_else(|| PyValueError::new_err("label object did not parse"))?,
        ),
        DrawingFamily::Line => DrawingObject::Line(
            parsed
                .lines
                .into_iter()
                .next()
                .ok_or_else(|| PyValueError::new_err("line object did not parse"))?,
        ),
        DrawingFamily::LineFill => DrawingObject::LineFill(
            parsed
                .line_fills
                .into_iter()
                .next()
                .ok_or_else(|| PyValueError::new_err("lineFill object did not parse"))?,
        ),
        DrawingFamily::Polyline => DrawingObject::Polyline(
            parsed
                .polylines
                .into_iter()
                .next()
                .ok_or_else(|| PyValueError::new_err("polyline object did not parse"))?,
        ),
        DrawingFamily::Box => DrawingObject::Box(
            parsed
                .boxes
                .into_iter()
                .next()
                .ok_or_else(|| PyValueError::new_err("box object did not parse"))?,
        ),
        DrawingFamily::Table => DrawingObject::Table(Box::new(
            parsed
                .tables
                .into_iter()
                .next()
                .ok_or_else(|| PyValueError::new_err("table object did not parse"))?,
        )),
    })
}

fn event_changes_from_py(
    value: &Bound<'_, PyAny>,
) -> PyResult<Vec<EventChange<pine_runtime::AlertEvent>>> {
    let list = value
        .cast::<PyList>()
        .map_err(|_| PyValueError::new_err("alert changes must be a list"))?;
    let mut items = Vec::with_capacity(list.len());
    for item in list {
        let dict = item
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("alert change must be a dict"))?;
        let action = match dict_string(dict, "action")?.as_str() {
            "add" => EventAction::Add,
            "remove" => EventAction::Remove,
            other => {
                return Err(PyValueError::new_err(format!(
                    "unsupported event action `{other}`"
                )));
            }
        };
        let event = dict
            .get_item("event")?
            .ok_or_else(|| PyValueError::new_err("alert change missing event"))?;
        let event = event
            .cast::<PyDict>()
            .map_err(|_| PyValueError::new_err("alert event must be a dict"))?;
        items.push(EventChange {
            action,
            event: alert_from_py(event)?,
        });
    }
    Ok(items)
}

fn strategy_changes_from_py(
    _py: Python<'_>,
    value: &Bound<'_, PyAny>,
) -> PyResult<StrategyChanges> {
    let dict = value
        .cast::<PyDict>()
        .map_err(|_| PyValueError::new_err("strategy changes must be a dict"))?;
    Ok(StrategyChanges {
        orders: optional_order_splice(dict)?,
        trades: optional_trade_splice(dict)?,
        alerts: optional_fill_alert_splice(dict)?,
        position: optional_position_splice(dict)?,
        equity: optional_equity_splice(dict)?,
        diagnostics: match dict.get_item("diagnostics")? {
            Some(value) if !value.is_none() => {
                let list = value
                    .cast::<PyList>()
                    .map_err(|_| PyValueError::new_err("strategy diagnostics must be a list"))?;
                Some(diagnostics_from_py(list.to_owned())?)
            }
            _ => None,
        },
    })
}

fn splice_dict<'a>(dict: &'a Bound<'_, PyDict>, key: &str) -> PyResult<Option<Bound<'a, PyDict>>> {
    match dict.get_item(key)? {
        Some(value) if !value.is_none() => Ok(Some(
            value
                .cast::<PyDict>()
                .map_err(|_| PyValueError::new_err(format!("`{key}` splice must be a dict")))?
                .to_owned(),
        )),
        _ => Ok(None),
    }
}

fn optional_order_splice(
    dict: &Bound<'_, PyDict>,
) -> PyResult<Option<ListSplice<pine_runtime::StrategyOrderEvent>>> {
    let Some(splice) = splice_dict(dict, "orders")? else {
        return Ok(None);
    };
    let items = splice
        .get_item("items")?
        .ok_or_else(|| PyValueError::new_err("orders splice missing items"))?;
    let list = items
        .cast::<PyList>()
        .map_err(|_| PyValueError::new_err("orders items must be a list"))?;
    Ok(Some(ListSplice {
        start: dict_usize(&splice, "start")?,
        items: strategy_orders_from_py(list.to_owned())?,
    }))
}

fn optional_trade_splice(
    dict: &Bound<'_, PyDict>,
) -> PyResult<Option<ListSplice<pine_runtime::StrategyTrade>>> {
    let Some(splice) = splice_dict(dict, "trades")? else {
        return Ok(None);
    };
    let items = splice
        .get_item("items")?
        .ok_or_else(|| PyValueError::new_err("trades splice missing items"))?;
    let list = items
        .cast::<PyList>()
        .map_err(|_| PyValueError::new_err("trades items must be a list"))?;
    Ok(Some(ListSplice {
        start: dict_usize(&splice, "start")?,
        items: strategy_trades_from_py(list.to_owned())?,
    }))
}

fn optional_fill_alert_splice(
    dict: &Bound<'_, PyDict>,
) -> PyResult<Option<ListSplice<pine_runtime::StrategyOrderFillAlertOutput>>> {
    let Some(splice) = splice_dict(dict, "alerts")? else {
        return Ok(None);
    };
    let items = splice
        .get_item("items")?
        .ok_or_else(|| PyValueError::new_err("strategy alerts splice missing items"))?;
    let list = items
        .cast::<PyList>()
        .map_err(|_| PyValueError::new_err("strategy alert items must be a list"))?;
    Ok(Some(ListSplice {
        start: dict_usize(&splice, "start")?,
        items: strategy_alerts_from_py(list.to_owned())?,
    }))
}

fn optional_position_splice(
    dict: &Bound<'_, PyDict>,
) -> PyResult<Option<ListSplice<pine_runtime::StrategyPositionSnapshot>>> {
    let Some(splice) = splice_dict(dict, "position")? else {
        return Ok(None);
    };
    let items = splice
        .get_item("items")?
        .ok_or_else(|| PyValueError::new_err("position splice missing items"))?;
    let wrapper = PyDict::new(dict.py());
    wrapper.set_item("position", items)?;
    let result = PyDict::new(dict.py());
    result.set_item("strategy", wrapper)?;
    let parsed = runtime_result_from_py(dict.py(), result.as_any())?;
    Ok(Some(ListSplice {
        start: dict_usize(&splice, "start")?,
        items: parsed
            .strategy
            .map(|item| item.position)
            .unwrap_or_default(),
    }))
}

fn optional_equity_splice(
    dict: &Bound<'_, PyDict>,
) -> PyResult<Option<ListSplice<pine_runtime::StrategyEquitySnapshot>>> {
    let Some(splice) = splice_dict(dict, "equity")? else {
        return Ok(None);
    };
    let items = splice
        .get_item("items")?
        .ok_or_else(|| PyValueError::new_err("equity splice missing items"))?;
    let wrapper = PyDict::new(dict.py());
    wrapper.set_item("equity", items)?;
    let result = PyDict::new(dict.py());
    result.set_item("strategy", wrapper)?;
    let parsed = runtime_result_from_py(dict.py(), result.as_any())?;
    Ok(Some(ListSplice {
        start: dict_usize(&splice, "start")?,
        items: parsed.strategy.map(|item| item.equity).unwrap_or_default(),
    }))
}

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "RUNTIME_CHANGES_SCHEMA_VERSION",
        PUBLIC_RUNTIME_CHANGES_SCHEMA_VERSION,
    )?;
    crate::replica::register(module)
}

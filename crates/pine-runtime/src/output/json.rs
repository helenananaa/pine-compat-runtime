use crate::output::view::*;
use crate::{PineValue, RuntimeProfile};

mod drawings_writer;
mod profile;
mod series_writer;
mod value_writer;
mod writer;
pub use writer::{
    into_public_runtime_result_json, public_runtime_result_view_json,
    write_public_runtime_result_json, write_public_runtime_result_view_json,
};

use super::alerts::AlertEvent;
use super::changes::{
    DrawingAction, DrawingObject, FillAction, HLineAction, RuntimeChanges, SeriesChange,
    SeriesHeader, StrategyChanges,
};
use super::model::{
    ColorSeries, FillOutput, HLineOutput, PUBLIC_RENDER_METADATA_VERSION,
    PUBLIC_RUNTIME_SCHEMA_VERSION, PlotArrowSeries, PlotBarSeries, PlotCandleSeries,
    PlotCharSeries, PlotSeries, PlotShapeSeries, RuntimeResult,
};
use profile::profile_json;

/// Serialize the public owned result without complete per-family buffers.
pub fn public_runtime_result_json(result: &RuntimeResult) -> String {
    public_runtime_result_view_json(&result.view())
}

pub fn public_runtime_profiled_result_json(
    result: &RuntimeResult,
    profile: &RuntimeProfile,
) -> String {
    let mut output = public_runtime_result_json(result);
    output.pop();
    output.push_str(",\"profile\":");
    output.push_str(&profile_json(profile));
    output.push('}');
    output
}

pub fn public_runtime_changes_json(changes: &RuntimeChanges) -> String {
    let mut output = format!(
        "{{\"schemaVersion\":{},\"revision\":{},\"baseRevision\":{},\"retainedFrom\":{},\"visibility\":\"{}\",\"series\":",
        changes.schema_version,
        changes.revision,
        changes.base_revision,
        changes.retained_from,
        changes.visibility.as_str()
    );
    output.push_str(&series_changes_json(&changes.series));
    output.push_str(",\"hlines\":");
    output.push_str(&hline_changes_json(&changes.hlines));
    output.push_str(",\"fills\":");
    output.push_str(&fill_changes_json(&changes.fills));
    output.push_str(",\"drawings\":");
    output.push_str(&drawing_changes_json(&changes.drawings));
    output.push_str(",\"alerts\":");
    output.push_str(&event_changes_json(&changes.alerts));
    if let Some(strategy) = &changes.strategy {
        output.push_str(",\"strategy\":");
        output.push_str(&strategy_changes_json(strategy));
    }
    output.push_str(",\"diagnostics\":");
    output.push_str(&runtime_diagnostics_json(&changes.diagnostics));
    output.push('}');
    output
}

fn series_changes_json(changes: &[SeriesChange]) -> String {
    let mut output = String::from("[");
    for (index, change) in changes.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!(
            "{{\"family\":\"{}\",\"id\":{},\"op\":\"{}\",\"start\":{}",
            change.family.as_str(),
            change.id,
            change.op.as_str(),
            change.start
        ));
        push_optional_values_field(&mut output, "values", &change.fields.values);
        push_optional_values_field(&mut output, "colors", &change.fields.colors);
        push_optional_values_field(&mut output, "chars", &change.fields.chars);
        push_optional_values_field(&mut output, "locations", &change.fields.locations);
        push_optional_values_field(&mut output, "texts", &change.fields.texts);
        push_optional_values_field(&mut output, "textColors", &change.fields.text_colors);
        push_optional_values_field(&mut output, "sizes", &change.fields.sizes);
        push_optional_values_field(&mut output, "styles", &change.fields.styles);
        push_optional_values_field(&mut output, "colorUps", &change.fields.color_ups);
        push_optional_values_field(&mut output, "colorDowns", &change.fields.color_downs);
        push_optional_values_field(&mut output, "minHeights", &change.fields.min_heights);
        push_optional_values_field(&mut output, "maxHeights", &change.fields.max_heights);
        push_optional_values_field(&mut output, "opens", &change.fields.opens);
        push_optional_values_field(&mut output, "highs", &change.fields.highs);
        push_optional_values_field(&mut output, "lows", &change.fields.lows);
        push_optional_values_field(&mut output, "closes", &change.fields.closes);
        push_optional_values_field(&mut output, "wickColors", &change.fields.wick_colors);
        push_optional_values_field(&mut output, "borderColors", &change.fields.border_colors);
        if let Some(header) = &change.header {
            output.push_str(",\"header\":");
            output.push_str(&series_header_json(header));
        }
        output.push('}');
    }
    output.push(']');
    output
}

fn series_header_json(header: &SeriesHeader) -> String {
    let mut output = String::from("{");
    let defaults = SeriesHeader::default();
    let mut first = true;
    push_object_value(
        &mut output,
        &mut first,
        "title",
        &header.metadata.title,
        &defaults.metadata.title,
    );
    push_object_value(
        &mut output,
        &mut first,
        "offset",
        &header.metadata.offset,
        &defaults.metadata.offset,
    );
    push_object_value(
        &mut output,
        &mut first,
        "editable",
        &header.metadata.editable,
        &defaults.metadata.editable,
    );
    push_object_value(
        &mut output,
        &mut first,
        "showLast",
        &header.metadata.show_last,
        &defaults.metadata.show_last,
    );
    push_object_value(
        &mut output,
        &mut first,
        "display",
        &header.metadata.display,
        &defaults.metadata.display,
    );
    push_object_value(
        &mut output,
        &mut first,
        "forceOverlay",
        &header.metadata.force_overlay,
        &defaults.metadata.force_overlay,
    );
    push_object_value(
        &mut output,
        &mut first,
        "linewidth",
        &header.linewidth,
        &defaults.linewidth,
    );
    push_object_value(
        &mut output,
        &mut first,
        "style",
        &header.style,
        &defaults.style,
    );
    push_object_value(
        &mut output,
        &mut first,
        "trackPrice",
        &header.track_price,
        &defaults.track_price,
    );
    push_object_value(
        &mut output,
        &mut first,
        "histBase",
        &header.hist_base,
        &defaults.hist_base,
    );
    push_object_value(
        &mut output,
        &mut first,
        "join",
        &header.join,
        &defaults.join,
    );
    push_object_value(
        &mut output,
        &mut first,
        "format",
        &header.format,
        &defaults.format,
    );
    push_object_value(
        &mut output,
        &mut first,
        "precision",
        &header.precision,
        &defaults.precision,
    );
    push_object_value(
        &mut output,
        &mut first,
        "linestyle",
        &header.linestyle,
        &defaults.linestyle,
    );
    output.push('}');
    output
}

fn push_object_value(
    output: &mut String,
    first: &mut bool,
    name: &str,
    value: &PineValue,
    default: &PineValue,
) {
    if value == default {
        return;
    }
    if !*first {
        output.push(',');
    }
    *first = false;
    output.push_str(&format!("\"{name}\":"));
    output.push_str(&value_json(value));
}

fn push_optional_values_field(output: &mut String, name: &str, values: &[PineValue]) {
    if !values.is_empty() {
        push_values_field(output, name, values);
    }
}

fn hline_changes_json(changes: &[super::changes::HLineChange]) -> String {
    let mut output = String::from("[");
    for (index, change) in changes.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!(
            "{{\"id\":{},\"action\":\"{}\"",
            change.id,
            match &change.action {
                HLineAction::Add(_) => "add",
                HLineAction::Replace(_) => "replace",
                HLineAction::Delete => "delete",
            }
        ));
        match &change.action {
            HLineAction::Add(hline) | HLineAction::Replace(hline) => {
                output.push_str(",\"object\":");
                output.push_str(&first_object_json(&hlines_json(std::slice::from_ref(
                    hline,
                ))));
            }
            HLineAction::Delete => {}
        }
        output.push('}');
    }
    output.push(']');
    output
}

fn fill_changes_json(changes: &[super::changes::FillChange]) -> String {
    let mut output = String::from("[");
    for (index, change) in changes.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!(
            "{{\"id\":{},\"action\":\"{}\"",
            change.id,
            match &change.action {
                FillAction::Add(_) => "add",
                FillAction::Delete => "delete",
                FillAction::SetColors { .. } => "setColors",
                FillAction::SetGradient { .. } => "setGradient",
            }
        ));
        match &change.action {
            FillAction::Add(fill) => {
                output.push_str(",\"object\":");
                output.push_str(&first_object_json(&fills_json(std::slice::from_ref(fill))));
            }
            FillAction::Delete => {}
            FillAction::SetColors { start, values } => {
                output.push_str(&format!(",\"start\":{start}"));
                push_values_field(&mut output, "values", values);
            }
            FillAction::SetGradient { start, values } => {
                output.push_str(&format!(",\"start\":{start},\"values\":"));
                output.push_str(&serde_json::to_string(values).expect("finite gradient samples"));
            }
        }
        output.push('}');
    }
    output.push(']');
    output
}

fn drawing_changes_json(changes: &[super::changes::DrawingChange]) -> String {
    let mut output = String::from("[");
    for (index, change) in changes.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!(
            "{{\"family\":\"{}\",\"id\":{},\"action\":\"{}\"",
            change.family.as_str(),
            change.id,
            change.action.as_str()
        ));
        match &change.action {
            DrawingAction::Add(object) => {
                output.push_str(",\"object\":");
                output.push_str(&drawing_object_json(object));
            }
            DrawingAction::SetTail { start, object } => {
                output.push_str(&format!(",\"start\":{start},\"object\":"));
                output.push_str(&drawing_object_json(object));
            }
            DrawingAction::Delete => {}
        }
        output.push('}');
    }
    output.push(']');
    output
}

fn drawing_object_json(object: &DrawingObject) -> String {
    match object {
        DrawingObject::Label(item) => first_object_json(&labels_json(std::slice::from_ref(item))),
        DrawingObject::Line(item) => first_object_json(&lines_json(std::slice::from_ref(item))),
        DrawingObject::LineFill(item) => {
            first_object_json(&line_fills_json(std::slice::from_ref(item)))
        }
        DrawingObject::Polyline(item) => {
            first_object_json(&polylines_json(std::slice::from_ref(item)))
        }
        DrawingObject::Box(item) => first_object_json(&boxes_json(std::slice::from_ref(item))),
        DrawingObject::Table(item) => {
            first_object_json(&tables_json(std::slice::from_ref(item.as_ref())))
        }
    }
}

fn event_changes_json(changes: &[super::changes::EventChange<AlertEvent>]) -> String {
    let mut output = String::from("[");
    for (index, change) in changes.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!(
            "{{\"action\":\"{}\",\"event\":{}",
            change.action.as_str(),
            first_object_json(&alerts_json(std::slice::from_ref(&change.event)))
        ));
        output.push('}');
    }
    output.push(']');
    output
}

fn strategy_changes_json(changes: &StrategyChanges) -> String {
    let mut output = String::from("{");
    let mut first = true;
    push_splice(
        &mut output,
        &mut first,
        "orders",
        changes.orders.as_ref(),
        strategy_orders_json,
    );
    push_splice(
        &mut output,
        &mut first,
        "trades",
        changes.trades.as_ref(),
        strategy_trades_json,
    );
    push_splice(
        &mut output,
        &mut first,
        "alerts",
        changes.alerts.as_ref(),
        strategy_order_fill_alerts_json,
    );
    push_splice(
        &mut output,
        &mut first,
        "position",
        changes.position.as_ref(),
        strategy_position_json,
    );
    push_splice(
        &mut output,
        &mut first,
        "equity",
        changes.equity.as_ref(),
        strategy_equity_json,
    );
    if let Some(diagnostics) = &changes.diagnostics {
        if !first {
            output.push(',');
        }
        output.push_str("\"diagnostics\":");
        output.push_str(&runtime_diagnostics_json(diagnostics));
    }
    output.push('}');
    output
}

fn push_splice<T>(
    output: &mut String,
    first: &mut bool,
    name: &str,
    splice: Option<&super::changes::ListSplice<T>>,
    items_json: fn(&[T]) -> String,
) {
    let Some(splice) = splice else {
        return;
    };
    if !*first {
        output.push(',');
    }
    *first = false;
    output.push_str(&format!(
        "\"{name}\":{{\"start\":{},\"items\":{}}}",
        splice.start,
        items_json(&splice.items)
    ));
}

fn first_object_json(array_json: &str) -> String {
    let trimmed = array_json.trim();
    match trimmed.strip_prefix('[') {
        Some(rest) if rest.ends_with(']') => rest[..rest.len() - 1].to_owned(),
        _ => trimmed.to_owned(),
    }
}

fn values_json_into(output: &mut String, values: &[PineValue]) {
    for (value_index, value) in values.iter().enumerate() {
        if value_index > 0 {
            output.push(',');
        }
        output.push_str(&value_json(value));
    }
}

fn push_values_field(output: &mut String, name: &str, values: &[PineValue]) {
    output.push_str(&format!(",\"{name}\":["));
    values_json_into(output, values);
    output.push(']');
}

fn push_value_field(output: &mut String, name: &str, value: &PineValue) {
    output.push_str(&format!(",\"{name}\":"));
    output.push_str(&value_json(value));
}

fn push_non_default_value_field(
    output: &mut String,
    name: &str,
    value: &PineValue,
    default: &PineValue,
) {
    if value != default {
        push_value_field(output, name, value);
    }
}

fn output_metadata_json_into(output: &mut String, metadata: &super::model::OutputMetadata) {
    push_non_default_value_field(
        output,
        "title",
        &metadata.title,
        &PineValue::String(String::new()),
    );
    push_non_default_value_field(output, "offset", &metadata.offset, &PineValue::Int(0));
    push_non_default_value_field(
        output,
        "editable",
        &metadata.editable,
        &PineValue::Bool(true),
    );
    push_non_default_value_field(output, "showLast", &metadata.show_last, &PineValue::Na);
    push_non_default_value_field(
        output,
        "display",
        &metadata.display,
        &PineValue::String("display.all".to_owned()),
    );
    push_non_default_value_field(
        output,
        "forceOverlay",
        &metadata.force_overlay,
        &PineValue::Bool(false),
    );
}

fn hlines_json(hlines: &[HLineOutput]) -> String {
    let mut output = String::from("[");
    for (index, hline) in hlines.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!(
            "{{\"id\":{},\"price\":{}",
            hline.id,
            value_json(&hline.price)
        ));
        push_non_default_value_field(
            &mut output,
            "title",
            &hline.title,
            &PineValue::String(String::new()),
        );
        push_non_default_value_field(
            &mut output,
            "color",
            &hline.color,
            &PineValue::Color(0x787B86),
        );
        push_non_default_value_field(
            &mut output,
            "style",
            &hline.style,
            &PineValue::String("hline.style_solid".to_owned()),
        );
        push_non_default_value_field(
            &mut output,
            "linewidth",
            &hline.linewidth,
            &PineValue::Int(1),
        );
        push_non_default_value_field(
            &mut output,
            "editable",
            &hline.editable,
            &PineValue::Bool(true),
        );
        push_non_default_value_field(
            &mut output,
            "display",
            &hline.display,
            &PineValue::String("display.all".to_owned()),
        );
        output.push('}');
    }
    output.push(']');
    output
}

fn fills_json(fills: &[FillOutput]) -> String {
    let mut output = String::from("[");
    for (index, fill) in fills.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!(
            "{{\"id\":{},\"firstId\":{},\"secondId\":{},\"firstIsHLine\":{},\"secondIsHLine\":{}",
            fill.id, fill.first_id, fill.second_id, fill.first_is_hline, fill.second_is_hline
        ));
        push_values_field(&mut output, "colors", &fill.colors);
        if let Some(samples) = &fill.gradient {
            output.push_str(",\"gradient\":");
            output.push_str(&serde_json::to_string(samples).expect("finite gradient samples"));
        }
        push_non_default_value_field(
            &mut output,
            "title",
            &fill.title,
            &PineValue::String(String::new()),
        );
        push_non_default_value_field(
            &mut output,
            "editable",
            &fill.editable,
            &PineValue::Bool(true),
        );
        push_non_default_value_field(&mut output, "showLast", &fill.show_last, &PineValue::Na);
        push_non_default_value_field(
            &mut output,
            "fillGaps",
            &fill.fill_gaps,
            &PineValue::Bool(true),
        );
        push_non_default_value_field(
            &mut output,
            "display",
            &fill.display,
            &PineValue::String("display.all".to_owned()),
        );
        output.push('}');
    }
    output.push(']');
    output
}

fn labels_json<'a>(labels: impl IntoIterator<Item = impl Into<LabelOutputView<'a>>>) -> String {
    let mut output = String::from("[");
    for (index, label) in labels.into_iter().map(Into::into).enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!("{{\"id\":{},\"snapshots\":[", label.id));
        for (snapshot_index, snapshot) in label.snapshots.iter().enumerate() {
            if snapshot_index > 0 {
                output.push(',');
            }
            output.push_str(&format!(
                "{{\"barIndex\":{},\"exists\":{}",
                snapshot.bar_index, snapshot.exists
            ));
            if snapshot.exists {
                output.push_str(",\"x\":");
                output.push_str(&value_json(&snapshot.x));
                output.push_str(",\"y\":");
                output.push_str(&value_json(&snapshot.y));
                output.push_str(",\"text\":");
                output.push_str(&value_json(&snapshot.text));
                output.push_str(",\"xloc\":");
                output.push_str(&value_json(&snapshot.xloc));
                output.push_str(",\"yloc\":");
                output.push_str(&value_json(&snapshot.yloc));
                output.push_str(",\"color\":");
                output.push_str(&value_json(&snapshot.color));
                output.push_str(",\"style\":");
                output.push_str(&value_json(&snapshot.style));
                output.push_str(",\"textColor\":");
                output.push_str(&value_json(&snapshot.text_color));
                output.push_str(",\"size\":");
                output.push_str(&value_json(&snapshot.size));
                output.push_str(",\"tooltip\":");
                output.push_str(&value_json(&snapshot.tooltip));
                output.push_str(",\"textAlign\":");
                output.push_str(&value_json(&snapshot.text_align));
                output.push_str(",\"textFontFamily\":");
                output.push_str(&value_json(&snapshot.text_font_family));
                output.push_str(",\"textFormatting\":");
                output.push_str(&value_json(&snapshot.text_formatting));
            }
            output.push('}');
        }
        output.push_str("]}");
    }
    output.push(']');
    output
}

fn lines_json<'a>(lines: impl IntoIterator<Item = impl Into<LineOutputView<'a>>>) -> String {
    let mut output = String::from("[");
    for (index, line) in lines.into_iter().map(Into::into).enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!("{{\"id\":{},\"snapshots\":[", line.id));
        for (snapshot_index, snapshot) in line.snapshots.iter().enumerate() {
            if snapshot_index > 0 {
                output.push(',');
            }
            output.push_str(&format!(
                "{{\"barIndex\":{},\"exists\":{}",
                snapshot.bar_index, snapshot.exists
            ));
            if snapshot.exists {
                output.push_str(",\"x1\":");
                output.push_str(&value_json(&snapshot.x1));
                output.push_str(",\"y1\":");
                output.push_str(&value_json(&snapshot.y1));
                output.push_str(",\"x2\":");
                output.push_str(&value_json(&snapshot.x2));
                output.push_str(",\"y2\":");
                output.push_str(&value_json(&snapshot.y2));
                output.push_str(",\"xloc\":");
                output.push_str(&value_json(&snapshot.xloc));
                output.push_str(",\"color\":");
                output.push_str(&value_json(&snapshot.color));
                output.push_str(",\"width\":");
                output.push_str(&value_json(&snapshot.width));
                output.push_str(",\"style\":");
                output.push_str(&value_json(&snapshot.style));
                output.push_str(",\"extend\":");
                output.push_str(&value_json(&snapshot.extend));
            }
            output.push('}');
        }
        output.push_str("]}");
    }
    output.push(']');
    output
}

fn line_fills_json<'a>(
    line_fills: impl IntoIterator<Item = impl Into<LineFillOutputView<'a>>>,
) -> String {
    let mut output = String::from("[");
    for (index, line_fill) in line_fills.into_iter().map(Into::into).enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!("{{\"id\":{},\"snapshots\":[", line_fill.id));
        for (snapshot_index, snapshot) in line_fill.snapshots.iter().enumerate() {
            if snapshot_index > 0 {
                output.push(',');
            }
            output.push_str(&format!(
                "{{\"barIndex\":{},\"exists\":{}",
                snapshot.bar_index, snapshot.exists
            ));
            if snapshot.exists {
                output.push_str(",\"line1\":");
                output.push_str(&snapshot.line1.to_string());
                output.push_str(",\"line2\":");
                output.push_str(&snapshot.line2.to_string());
                output.push_str(",\"color\":");
                output.push_str(&value_json(&snapshot.color));
            }
            output.push('}');
        }
        output.push_str("]}");
    }
    output.push(']');
    output
}

fn polylines_json<'a>(
    polylines: impl IntoIterator<Item = impl Into<PolylineOutputView<'a>>>,
) -> String {
    let mut output = String::from("[");
    for (index, polyline) in polylines.into_iter().map(Into::into).enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!("{{\"id\":{},\"snapshots\":[", polyline.id));
        for (snapshot_index, snapshot) in polyline.snapshots.iter().enumerate() {
            if snapshot_index > 0 {
                output.push(',');
            }
            output.push_str(&format!(
                "{{\"barIndex\":{},\"exists\":{}",
                snapshot.bar_index, snapshot.exists
            ));
            if snapshot.exists {
                output.push_str(",\"points\":");
                output.push_str(&values_json(&snapshot.points));
                output.push_str(",\"curved\":");
                output.push_str(&value_json(&snapshot.curved));
                output.push_str(",\"closed\":");
                output.push_str(&value_json(&snapshot.closed));
                output.push_str(",\"xloc\":");
                output.push_str(&value_json(&snapshot.xloc));
                output.push_str(",\"lineColor\":");
                output.push_str(&value_json(&snapshot.line_color));
                output.push_str(",\"fillColor\":");
                output.push_str(&value_json(&snapshot.fill_color));
                output.push_str(",\"lineStyle\":");
                output.push_str(&value_json(&snapshot.line_style));
                output.push_str(",\"lineWidth\":");
                output.push_str(&value_json(&snapshot.line_width));
                output.push_str(",\"forceOverlay\":");
                output.push_str(&value_json(&snapshot.force_overlay));
            }
            output.push('}');
        }
        output.push_str("]}");
    }
    output.push(']');
    output
}

fn boxes_json<'a>(boxes: impl IntoIterator<Item = impl Into<BoxOutputView<'a>>>) -> String {
    let mut output = String::from("[");
    for (index, box_output) in boxes.into_iter().map(Into::into).enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!("{{\"id\":{},\"snapshots\":[", box_output.id));
        for (snapshot_index, snapshot) in box_output.snapshots.iter().enumerate() {
            if snapshot_index > 0 {
                output.push(',');
            }
            output.push_str(&format!(
                "{{\"barIndex\":{},\"exists\":{}",
                snapshot.bar_index, snapshot.exists
            ));
            if snapshot.exists {
                output.push_str(",\"left\":");
                output.push_str(&value_json(&snapshot.left));
                output.push_str(",\"top\":");
                output.push_str(&value_json(&snapshot.top));
                output.push_str(",\"right\":");
                output.push_str(&value_json(&snapshot.right));
                output.push_str(",\"bottom\":");
                output.push_str(&value_json(&snapshot.bottom));
                output.push_str(",\"xloc\":");
                output.push_str(&value_json(&snapshot.xloc));
                output.push_str(",\"bgColor\":");
                output.push_str(&value_json(&snapshot.bg_color));
                output.push_str(",\"borderColor\":");
                output.push_str(&value_json(&snapshot.border_color));
                output.push_str(",\"borderWidth\":");
                output.push_str(&value_json(&snapshot.border_width));
                output.push_str(",\"borderStyle\":");
                output.push_str(&value_json(&snapshot.border_style));
                output.push_str(",\"extend\":");
                output.push_str(&value_json(&snapshot.extend));
                output.push_str(",\"text\":");
                output.push_str(&value_json(&snapshot.text));
                output.push_str(",\"textColor\":");
                output.push_str(&value_json(&snapshot.text_color));
                output.push_str(",\"textSize\":");
                output.push_str(&value_json(&snapshot.text_size));
                output.push_str(",\"textHalign\":");
                output.push_str(&value_json(&snapshot.text_halign));
                output.push_str(",\"textValign\":");
                output.push_str(&value_json(&snapshot.text_valign));
                output.push_str(",\"textWrap\":");
                output.push_str(&value_json(&snapshot.text_wrap));
                output.push_str(",\"textFontFamily\":");
                output.push_str(&value_json(&snapshot.text_font_family));
                output.push_str(",\"textFormatting\":");
                output.push_str(&value_json(&snapshot.text_formatting));
            }
            output.push('}');
        }
        output.push_str("]}");
    }
    output.push(']');
    output
}

fn tables_json<'a>(tables: impl IntoIterator<Item = impl Into<TableOutputView<'a>>>) -> String {
    let mut output = String::from("[");
    for (index, table) in tables.into_iter().map(Into::into).enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!("{{\"id\":{},\"position\":", table.id));
        output.push_str(&value_json(table.position));
        output.push_str(",\"bgColor\":");
        output.push_str(&value_json(table.bg_color));
        output.push_str(",\"frameColor\":");
        output.push_str(&value_json(table.frame_color));
        output.push_str(",\"frameWidth\":");
        output.push_str(&value_json(table.frame_width));
        output.push_str(",\"borderColor\":");
        output.push_str(&value_json(table.border_color));
        output.push_str(",\"borderWidth\":");
        output.push_str(&value_json(table.border_width));
        output.push_str(&format!(
            ",\"columns\":{},\"rows\":{},\"snapshots\":[",
            table.columns, table.rows
        ));
        for (snapshot_index, snapshot) in table.snapshots.iter().enumerate() {
            if snapshot_index > 0 {
                output.push(',');
            }
            output.push_str(&format!(
                "{{\"barIndex\":{},\"exists\":{}",
                snapshot.bar_index, snapshot.exists
            ));
            if snapshot.exists {
                output.push_str(",\"cells\":[");
                for (cell_index, cell) in snapshot.cells.iter().enumerate() {
                    if cell_index > 0 {
                        output.push(',');
                    }
                    output.push_str(&format!(
                        "{{\"column\":{},\"row\":{},\"text\":",
                        cell.column, cell.row
                    ));
                    output.push_str(&value_json(&cell.text));
                    output.push_str(",\"bgColor\":");
                    output.push_str(&value_json(&cell.bg_color));
                    output.push_str(",\"textColor\":");
                    output.push_str(&value_json(&cell.text_color));
                    output.push_str(",\"width\":");
                    output.push_str(&value_json(&cell.width));
                    output.push_str(",\"height\":");
                    output.push_str(&value_json(&cell.height));
                    output.push_str(",\"textSize\":");
                    output.push_str(&value_json(&cell.text_size));
                    output.push_str(",\"textHalign\":");
                    output.push_str(&value_json(&cell.text_halign));
                    output.push_str(",\"textValign\":");
                    output.push_str(&value_json(&cell.text_valign));
                    output.push_str(",\"textWrap\":");
                    output.push_str(&value_json(&cell.text_wrap));
                    output.push_str(",\"tooltip\":");
                    output.push_str(&value_json(&cell.tooltip));
                    output.push_str(",\"textFontFamily\":");
                    output.push_str(&value_json(&cell.text_font_family));
                    output.push_str(",\"textFormatting\":");
                    output.push_str(&value_json(&cell.text_formatting));
                    output.push('}');
                }
                output.push(']');
                output.push_str(",\"mergedCells\":[");
                for (merge_index, merged_cell) in snapshot.merged_cells.iter().enumerate() {
                    if merge_index > 0 {
                        output.push(',');
                    }
                    output.push_str(&format!(
                        "{{\"startColumn\":{},\"startRow\":{},\"endColumn\":{},\"endRow\":{}}}",
                        merged_cell.start_column,
                        merged_cell.start_row,
                        merged_cell.end_column,
                        merged_cell.end_row
                    ));
                }
                output.push(']');
            }
            output.push('}');
        }
        output.push_str("]}");
    }
    output.push(']');
    output
}

fn alerts_json(alerts: &[AlertEvent]) -> String {
    let mut output = String::from("[");
    for (index, alert) in alerts.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!(
            "{{\"id\":{},\"barIndex\":{},\"time\":{},\"message\":\"{}\",\"source\":\"{}\"}}",
            alert.id,
            alert.bar_index,
            alert.time,
            json_escape(&alert.message),
            json_escape(&alert.source)
        ));
    }
    output.push(']');
    output
}

fn strategy_orders_json(orders: &[crate::StrategyOrderEvent]) -> String {
    let mut output = String::from("[");
    for (index, order) in orders.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!(
            "{{\"id\":\"{}\",\"barIndex\":{},\"time\":{},\"direction\":\"{}\",\"qty\":{},\"price\":{}}}",
            json_escape(&order.id),
            order.bar_index,
            order.time,
            json_escape(&order.direction),
            f64_json(order.qty),
            f64_json(order.price)
        ));
    }
    output.push(']');
    output
}

fn strategy_order_fill_alerts_json(alerts: &[crate::StrategyOrderFillAlertOutput]) -> String {
    let mut output = String::from("[");
    for (index, alert) in alerts.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!(
            "{{\"id\":\"{}\",\"barIndex\":{},\"time\":{},\"direction\":\"{}\",\"qty\":{},\"price\":{},\"entryId\":{},\"exitId\":{},\"message\":\"{}\"}}",
            json_escape(&alert.id),
            alert.bar_index,
            alert.time,
            json_escape(&alert.direction),
            f64_json(alert.qty),
            f64_json(alert.price),
            option_string_json(alert.entry_id.as_deref()),
            option_string_json(alert.exit_id.as_deref()),
            json_escape(&alert.message)
        ));
    }
    output.push(']');
    output
}

fn strategy_trades_json(trades: &[crate::StrategyTrade]) -> String {
    let mut output = String::from("[");
    for (index, trade) in trades.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!(
            "{{\"id\":\"{}\",\"entryBarIndex\":{},\"exitBarIndex\":{},\"entryTime\":{},\"exitTime\":{},\"entryPrice\":{},\"exitPrice\":{},\"qty\":{},\"profit\":{}}}",
            json_escape(&trade.id),
            trade.entry_bar_index,
            trade.exit_bar_index,
            trade.entry_time,
            trade.exit_time,
            f64_json(trade.entry_price),
            f64_json(trade.exit_price),
            f64_json(trade.qty),
            f64_json(trade.profit)
        ));
    }
    output.push(']');
    output
}

fn strategy_position_json(position: &[crate::StrategyPositionSnapshot]) -> String {
    let mut output = String::from("[");
    for (index, snapshot) in position.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!(
            "{{\"barIndex\":{},\"size\":{},\"avgPrice\":{}}}",
            snapshot.bar_index,
            f64_json(snapshot.size),
            option_f64_json(snapshot.avg_price)
        ));
    }
    output.push(']');
    output
}

fn strategy_equity_json(equity: &[crate::StrategyEquitySnapshot]) -> String {
    let mut output = String::from("[");
    for (index, snapshot) in equity.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!(
            "{{\"barIndex\":{},\"cash\":{},\"marketValue\":{},\"equity\":{},\"netProfit\":{}}}",
            snapshot.bar_index,
            f64_json(snapshot.cash),
            f64_json(snapshot.market_value),
            f64_json(snapshot.equity),
            f64_json(snapshot.net_profit)
        ));
    }
    output.push(']');
    output
}

fn option_f64_json(value: Option<f64>) -> String {
    value.map_or_else(|| "null".to_owned(), f64_json)
}

fn option_string_json(value: Option<&str>) -> String {
    value.map_or_else(
        || "null".to_owned(),
        |value| format!("\"{}\"", json_escape(value)),
    )
}

fn f64_json(value: f64) -> String {
    if value.is_finite() {
        value.to_string()
    } else {
        "null".to_owned()
    }
}

fn runtime_diagnostics_json(diagnostics: &[crate::RuntimeDiagnostic]) -> String {
    let mut output = String::from("[");
    for (index, diagnostic) in diagnostics.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!(
            "{{\"code\":\"{}\",\"message\":\"{}\"}}",
            json_escape(&diagnostic.code),
            json_escape(&diagnostic.message)
        ));
    }
    output.push(']');
    output
}

fn values_json(values: &[PineValue]) -> String {
    let mut output = String::from("[");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&value_json(value));
    }
    output.push(']');
    output
}

fn value_json(value: &PineValue) -> String {
    let mut bytes = Vec::new();
    value_writer::value(value, &mut bytes).expect("writing to a Vec cannot fail");
    String::from_utf8(bytes).expect("JSON encoders produce UTF-8")
}

fn json_escape(value: &str) -> String {
    let mut bytes = Vec::with_capacity(value.len());
    value_writer::escaped(value, &mut bytes).expect("writing to a Vec cannot fail");
    String::from_utf8(bytes).expect("JSON encoders produce UTF-8")
}

#[cfg(test)]
mod tests;

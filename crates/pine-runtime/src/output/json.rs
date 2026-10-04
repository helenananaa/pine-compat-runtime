use crate::output::view::*;
use crate::{PineValue, RuntimeProfile};

mod drawings_writer;
mod profile;
mod records_writer;
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
    let mut bytes = Vec::new();
    match object {
        DrawingObject::Label(item) => drawings_writer::labels(&item.into(), &mut bytes),
        DrawingObject::Line(item) => drawings_writer::lines(&item.into(), &mut bytes),
        DrawingObject::LineFill(item) => drawings_writer::line_fills(&item.into(), &mut bytes),
        DrawingObject::Polyline(item) => drawings_writer::polylines(&item.into(), &mut bytes),
        DrawingObject::Box(item) => drawings_writer::boxes(&item.into(), &mut bytes),
        DrawingObject::Table(item) => drawings_writer::tables(&item.as_ref().into(), &mut bytes),
    }
    .expect("writing to a Vec cannot fail");
    String::from_utf8(bytes).expect("JSON encoders produce UTF-8")
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

// String-returning delta helpers use the same record encoders as full output.
macro_rules! records_json {
    ($function:ident, $kind:ty, $encoder:ident) => {
        fn $function(items: &[$kind]) -> String {
            let mut bytes = Vec::new();
            writer::write_series_array(&mut bytes, items, records_writer::$encoder)
                .expect("writing to a Vec cannot fail");
            String::from_utf8(bytes).expect("JSON encoders produce UTF-8")
        }
    };
}
records_json!(alerts_json, AlertEvent, alerts);
records_json!(strategy_orders_json, crate::StrategyOrderEvent, orders);
records_json!(
    strategy_order_fill_alerts_json,
    crate::StrategyOrderFillAlertOutput,
    order_fill_alerts
);
records_json!(strategy_trades_json, crate::StrategyTrade, trades);
records_json!(
    strategy_position_json,
    crate::StrategyPositionSnapshot,
    position
);
records_json!(strategy_equity_json, crate::StrategyEquitySnapshot, equity);
records_json!(
    runtime_diagnostics_json,
    crate::RuntimeDiagnostic,
    diagnostics
);

fn value_json(value: &PineValue) -> String {
    let mut bytes = Vec::new();
    value_writer::value(value, &mut bytes).expect("writing to a Vec cannot fail");
    String::from_utf8(bytes).expect("JSON encoders produce UTF-8")
}

#[cfg(test)]
mod tests;

//! Sink encoding of history fields without a complete per-series buffer.
use super::*;
use std::io::{self, Write};

fn values<W: Write + ?Sized>(output: &mut W, name: &str, values: &[PineValue]) -> io::Result<()> {
    write!(output, ",\"{name}\":[")?;
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            output.write_all(b",")?;
        }
        output.write_all(value_json(value).as_bytes())?;
    }
    output.write_all(b"]")
}

fn metadata<W: Write + ?Sized>(
    output: &mut W,
    metadata: &super::super::model::OutputMetadata,
) -> io::Result<()> {
    let mut suffix = String::new();
    output_metadata_json_into(&mut suffix, metadata);
    output.write_all(suffix.as_bytes())?;
    output.write_all(b"}")
}

pub(super) fn plots<W: Write + ?Sized>(plot: &PlotSeries, output: &mut W) -> io::Result<()> {
    write!(output, "{{\"id\":{}", plot.id)?;
    values(output, "values", &plot.values)?;
    if plot.colors.iter().any(|value| *value != PineValue::Na) {
        values(output, "colors", &plot.colors)?;
    }
    let mut suffix = String::new();
    push_non_default_value_field(
        &mut suffix,
        "linewidth",
        &plot.linewidth,
        &PineValue::Int(1),
    );
    push_non_default_value_field(
        &mut suffix,
        "style",
        &plot.style,
        &PineValue::String("plot.style_line".to_owned()),
    );
    push_non_default_value_field(
        &mut suffix,
        "trackPrice",
        &plot.track_price,
        &PineValue::Bool(false),
    );
    push_non_default_value_field(&mut suffix, "histBase", &plot.hist_base, &PineValue::Int(0));
    push_non_default_value_field(&mut suffix, "join", &plot.join, &PineValue::Bool(false));
    push_non_default_value_field(
        &mut suffix,
        "format",
        &plot.format,
        &PineValue::String("format.inherit".to_owned()),
    );
    push_non_default_value_field(&mut suffix, "precision", &plot.precision, &PineValue::Na);
    push_non_default_value_field(
        &mut suffix,
        "linestyle",
        &plot.linestyle,
        &PineValue::String("plot.linestyle_solid".to_owned()),
    );

    output.write_all(suffix.as_bytes())?;
    metadata(output, &plot.metadata)
}

macro_rules! series {
    ($name:ident, $kind:ty, $( $field:literal => $member:ident ),+ ) => {
        pub(super) fn $name<W: Write + ?Sized>(item: &$kind, output: &mut W) -> io::Result<()> {
            write!(output, "{{\"id\":{}", item.id)?;
            $(values(output, $field, &item.$member)?;)+
            metadata(output, &item.metadata)
        }
    };
}
series!(colors, ColorSeries, "values" => values);
series!(chars, PlotCharSeries, "values" => values, "chars" => chars, "colors" => colors,
    "locations" => locations, "texts" => texts, "textColors" => text_colors, "sizes" => sizes);
series!(shapes, PlotShapeSeries, "values" => values, "styles" => styles, "locations" => locations,
    "colors" => colors, "texts" => texts, "textColors" => text_colors, "sizes" => sizes);
series!(arrows, PlotArrowSeries, "values" => values, "colorUps" => color_ups, "colorDowns" => color_downs,
    "minHeights" => min_heights, "maxHeights" => max_heights);
series!(bars, PlotBarSeries, "opens" => opens, "highs" => highs, "lows" => lows, "closes" => closes, "colors" => colors);
series!(candles, PlotCandleSeries, "opens" => opens, "highs" => highs, "lows" => lows, "closes" => closes,
    "colors" => colors, "wickColors" => wick_colors, "borderColors" => border_colors);

pub(super) fn fills<W: Write + ?Sized>(fill: &FillOutput, output: &mut W) -> io::Result<()> {
    write!(
        output,
        "{{\"id\":{},\"firstId\":{},\"secondId\":{},\"firstIsHLine\":{},\"secondIsHLine\":{}",
        fill.id, fill.first_id, fill.second_id, fill.first_is_hline, fill.second_is_hline
    )?;
    values(output, "colors", &fill.colors)?;
    if let Some(samples) = &fill.gradient {
        output.write_all(b",\"gradient\":[")?;
        for (index, sample) in samples.iter().enumerate() {
            if index > 0 {
                output.write_all(b",")?;
            }
            let text = serde_json::to_string(sample).expect("finite gradient samples");
            output.write_all(text.as_bytes())?;
        }
        output.write_all(b"]")?;
    }
    let mut suffix = String::new();
    push_non_default_value_field(
        &mut suffix,
        "title",
        &fill.title,
        &PineValue::String(String::new()),
    );
    push_non_default_value_field(
        &mut suffix,
        "editable",
        &fill.editable,
        &PineValue::Bool(true),
    );
    push_non_default_value_field(&mut suffix, "showLast", &fill.show_last, &PineValue::Na);
    push_non_default_value_field(
        &mut suffix,
        "fillGaps",
        &fill.fill_gaps,
        &PineValue::Bool(true),
    );
    push_non_default_value_field(
        &mut suffix,
        "display",
        &fill.display,
        &PineValue::String("display.all".to_owned()),
    );

    output.write_all(suffix.as_bytes())?;
    output.write_all(b"}")
}

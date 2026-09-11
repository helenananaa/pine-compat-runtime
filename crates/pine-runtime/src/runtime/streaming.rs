use super::historical::HistoricalRuntime;
use crate::output::changes::{
    DrawingAction, DrawingChange, DrawingFamily, DrawingObject, EventAction, EventChange,
    FillAction, FillChange, HLineAction, HLineChange, ListSplice, RuntimeChanges, SeriesFamily,
    SeriesFields, SeriesHeader, StrategyChanges, StreamingVisibility, series_change_from_lens,
};
use crate::{PineValue, StrategyOrderFillAlertOutput};

#[derive(Debug, Clone, Default)]
pub(crate) struct OutputCursor {
    plots: Vec<(u32, usize)>,
    plot_chars: Vec<(u32, usize)>,
    plot_shapes: Vec<(u32, usize)>,
    plot_arrows: Vec<(u32, usize)>,
    plot_bars: Vec<(u32, usize)>,
    plot_candles: Vec<(u32, usize)>,
    bg_colors: Vec<(u32, usize)>,
    bar_colors: Vec<(u32, usize)>,
    hlines: Vec<crate::HLineOutput>,
    fills: Vec<(u32, usize)>,
    labels: Vec<(u32, usize)>,
    lines: Vec<(u32, usize)>,
    line_fills: Vec<(u32, usize)>,
    polylines: Vec<(u32, usize)>,
    boxes: Vec<(u32, usize)>,
    tables: Vec<(u32, usize)>,
    alerts: Vec<crate::AlertEvent>,
    alert_bar: usize,
    drawing_bar: usize,
    orders: usize,
    trades: usize,
    fill_alerts: usize,
    position: usize,
    equity: usize,
    has_strategy: bool,
}

impl OutputCursor {
    pub(crate) fn capture(runtime: &HistoricalRuntime<'_>) -> Self {
        let alert_start = runtime
            .alerts
            .partition_point(|event| event.bar_index < runtime.bars.saturating_sub(1));
        Self {
            plots: runtime
                .plots
                .iter()
                .map(|item| (item.id, abs_len(runtime, item.values.len())))
                .collect(),
            plot_chars: runtime
                .plot_chars
                .iter()
                .map(|item| (item.id, abs_len(runtime, item.values.len())))
                .collect(),
            plot_shapes: runtime
                .plot_shapes
                .iter()
                .map(|item| (item.id, abs_len(runtime, item.values.len())))
                .collect(),
            plot_arrows: runtime
                .plot_arrows
                .iter()
                .map(|item| (item.id, abs_len(runtime, item.values.len())))
                .collect(),
            plot_bars: runtime
                .plot_bars
                .iter()
                .map(|item| (item.id, abs_len(runtime, item.closes.len())))
                .collect(),
            plot_candles: runtime
                .plot_candles
                .iter()
                .map(|item| (item.id, abs_len(runtime, item.closes.len())))
                .collect(),
            bg_colors: runtime
                .bg_colors
                .iter()
                .map(|item| (item.id, abs_len(runtime, item.values.len())))
                .collect(),
            bar_colors: runtime
                .bar_colors
                .iter()
                .map(|item| (item.id, abs_len(runtime, item.values.len())))
                .collect(),
            hlines: runtime.hlines.clone(),
            fills: runtime
                .fills
                .iter()
                .map(|item| (item.id, abs_len(runtime, item.colors.len())))
                .collect(),
            labels: runtime
                .labels
                .iter()
                .map(|item| {
                    (
                        item.id,
                        item.snapshots
                            .partition_point(|s| s.bar_index < runtime.bars.saturating_sub(1)),
                    )
                })
                .collect(),
            lines: runtime
                .lines
                .iter()
                .map(|item| {
                    (
                        item.id,
                        item.snapshots
                            .partition_point(|s| s.bar_index < runtime.bars.saturating_sub(1)),
                    )
                })
                .collect(),
            line_fills: runtime
                .line_fills
                .iter()
                .map(|item| {
                    (
                        item.id,
                        item.snapshots
                            .partition_point(|s| s.bar_index < runtime.bars.saturating_sub(1)),
                    )
                })
                .collect(),
            polylines: runtime
                .polylines
                .iter()
                .map(|item| {
                    (
                        item.id,
                        item.snapshots
                            .partition_point(|s| s.bar_index < runtime.bars.saturating_sub(1)),
                    )
                })
                .collect(),
            boxes: runtime
                .boxes
                .iter()
                .map(|item| {
                    (
                        item.id,
                        item.snapshots
                            .partition_point(|s| s.bar_index < runtime.bars.saturating_sub(1)),
                    )
                })
                .collect(),
            tables: runtime
                .tables
                .iter()
                .map(|item| {
                    (
                        item.id,
                        item.snapshots
                            .partition_point(|s| s.bar_index < runtime.bars.saturating_sub(1)),
                    )
                })
                .collect(),
            alerts: runtime.alerts.tail(alert_start),
            alert_bar: runtime.bars.saturating_sub(1),
            drawing_bar: runtime.bars.saturating_sub(1),
            orders: runtime.strategy_broker.order_len(),
            trades: runtime.strategy_broker.trade_len(),
            fill_alerts: runtime.strategy_broker.fill_alert_len(),
            position: runtime.strategy_broker.position_len(),
            equity: runtime.strategy_broker.equity_len(),
            has_strategy: runtime.program.script_mode == pine_ir::ScriptMode::Strategy,
        }
    }

    pub(crate) fn diff(
        &self,
        runtime: &HistoricalRuntime<'_>,
        revision: u64,
        visibility: StreamingVisibility,
    ) -> RuntimeChanges {
        let mut changes = RuntimeChanges::new(revision, visibility);
        diff_plots(&mut changes, &self.plots, runtime);
        diff_plot_chars(&mut changes, &self.plot_chars, runtime);
        diff_plot_shapes(&mut changes, &self.plot_shapes, runtime);
        diff_plot_arrows(&mut changes, &self.plot_arrows, runtime);
        diff_plot_bars(&mut changes, &self.plot_bars, runtime);
        diff_plot_candles(&mut changes, &self.plot_candles, runtime);
        diff_color_series(
            &mut changes,
            SeriesFamily::BgColor,
            &self.bg_colors,
            runtime,
            &runtime.bg_colors,
        );
        diff_color_series(
            &mut changes,
            SeriesFamily::BarColor,
            &self.bar_colors,
            runtime,
            &runtime.bar_colors,
        );
        diff_hlines(&mut changes, &self.hlines, &runtime.hlines);
        diff_fills(&mut changes, &self.fills, runtime);
        diff_label_drawings(
            &mut changes,
            &self.labels,
            &runtime.labels,
            self.drawing_bar,
        );
        diff_line_drawings(&mut changes, &self.lines, &runtime.lines, self.drawing_bar);
        diff_line_fill_drawings(
            &mut changes,
            &self.line_fills,
            &runtime.line_fills,
            self.drawing_bar,
        );
        diff_polyline_drawings(
            &mut changes,
            &self.polylines,
            &runtime.polylines,
            self.drawing_bar,
        );
        diff_box_drawings(&mut changes, &self.boxes, &runtime.boxes, self.drawing_bar);
        diff_table_drawings(
            &mut changes,
            &self.tables,
            &runtime.tables,
            self.drawing_bar,
        );
        diff_alerts(
            &mut changes,
            &self.alerts,
            &runtime.alerts.tail(
                runtime
                    .alerts
                    .partition_point(|event| event.bar_index < self.alert_bar),
            ),
        );
        if runtime.program.script_mode == pine_ir::ScriptMode::Strategy {
            changes.strategy = Some(diff_strategy(self, runtime));
        } else if self.has_strategy {
            changes.strategy = Some(StrategyChanges::default());
        }
        changes.diagnostics = runtime.runtime_diagnostics();
        changes
    }
}

fn lens_get(items: &[(u32, usize)], id: u32) -> Option<usize> {
    items
        .iter()
        .find_map(|(item_id, len)| (*item_id == id).then_some(*len))
}

fn abs_len(runtime: &HistoricalRuntime<'_>, local: usize) -> usize {
    runtime.stored_origin + local
}

fn slice_from(
    values: &super::append_history::AppendHistory<PineValue>,
    start: usize,
    stored_origin: usize,
) -> Vec<PineValue> {
    values.tail(start.saturating_sub(stored_origin))
}

fn plot_header(plot: &super::plot_history::RuntimePlot) -> SeriesHeader {
    SeriesHeader {
        metadata: plot.metadata.clone(),
        linewidth: plot.linewidth.clone(),
        style: plot.style.clone(),
        track_price: plot.track_price.clone(),
        hist_base: plot.hist_base.clone(),
        join: plot.join.clone(),
        format: plot.format.clone(),
        precision: plot.precision.clone(),
    }
}

fn metadata_header(metadata: &crate::OutputMetadata) -> SeriesHeader {
    SeriesHeader {
        metadata: metadata.clone(),
        ..SeriesHeader::default()
    }
}

fn diff_plots(
    changes: &mut RuntimeChanges,
    cursor: &[(u32, usize)],
    runtime: &HistoricalRuntime<'_>,
) {
    for plot in runtime.plots.iter() {
        if let Some(change) = series_change_from_lens(
            SeriesFamily::Plot,
            plot.id,
            lens_get(cursor, plot.id),
            abs_len(runtime, plot.values.len()),
            runtime.display_origin,
            |start| SeriesFields {
                values: plot
                    .values
                    .tail(start.saturating_sub(runtime.stored_origin)),
                colors: plot
                    .colors
                    .tail(start.saturating_sub(runtime.stored_origin)),
                ..SeriesFields::default()
            },
            Some(plot_header(plot)),
        ) {
            changes.series.push(change);
        }
    }
}

fn diff_plot_chars(
    changes: &mut RuntimeChanges,
    cursor: &[(u32, usize)],
    runtime: &HistoricalRuntime<'_>,
) {
    for item in &runtime.plot_chars {
        if let Some(change) = series_change_from_lens(
            SeriesFamily::PlotChar,
            item.id,
            lens_get(cursor, item.id),
            abs_len(runtime, item.values.len()),
            runtime.display_origin,
            |start| SeriesFields {
                values: slice_from(&item.values, start, runtime.stored_origin),
                colors: slice_from(&item.colors, start, runtime.stored_origin),
                chars: slice_from(&item.chars, start, runtime.stored_origin),
                locations: slice_from(&item.locations, start, runtime.stored_origin),
                texts: slice_from(&item.texts, start, runtime.stored_origin),
                text_colors: slice_from(&item.text_colors, start, runtime.stored_origin),
                sizes: slice_from(&item.sizes, start, runtime.stored_origin),
                ..SeriesFields::default()
            },
            Some(metadata_header(&item.metadata)),
        ) {
            changes.series.push(change);
        }
    }
}

fn diff_plot_shapes(
    changes: &mut RuntimeChanges,
    cursor: &[(u32, usize)],
    runtime: &HistoricalRuntime<'_>,
) {
    for item in &runtime.plot_shapes {
        if let Some(change) = series_change_from_lens(
            SeriesFamily::PlotShape,
            item.id,
            lens_get(cursor, item.id),
            abs_len(runtime, item.values.len()),
            runtime.display_origin,
            |start| SeriesFields {
                values: slice_from(&item.values, start, runtime.stored_origin),
                styles: slice_from(&item.styles, start, runtime.stored_origin),
                locations: slice_from(&item.locations, start, runtime.stored_origin),
                colors: slice_from(&item.colors, start, runtime.stored_origin),
                texts: slice_from(&item.texts, start, runtime.stored_origin),
                text_colors: slice_from(&item.text_colors, start, runtime.stored_origin),
                sizes: slice_from(&item.sizes, start, runtime.stored_origin),
                ..SeriesFields::default()
            },
            Some(metadata_header(&item.metadata)),
        ) {
            changes.series.push(change);
        }
    }
}

fn diff_plot_arrows(
    changes: &mut RuntimeChanges,
    cursor: &[(u32, usize)],
    runtime: &HistoricalRuntime<'_>,
) {
    for item in &runtime.plot_arrows {
        if let Some(change) = series_change_from_lens(
            SeriesFamily::PlotArrow,
            item.id,
            lens_get(cursor, item.id),
            abs_len(runtime, item.values.len()),
            runtime.display_origin,
            |start| SeriesFields {
                values: slice_from(&item.values, start, runtime.stored_origin),
                color_ups: slice_from(&item.color_ups, start, runtime.stored_origin),
                color_downs: slice_from(&item.color_downs, start, runtime.stored_origin),
                min_heights: slice_from(&item.min_heights, start, runtime.stored_origin),
                max_heights: slice_from(&item.max_heights, start, runtime.stored_origin),
                ..SeriesFields::default()
            },
            Some(metadata_header(&item.metadata)),
        ) {
            changes.series.push(change);
        }
    }
}

fn diff_plot_bars(
    changes: &mut RuntimeChanges,
    cursor: &[(u32, usize)],
    runtime: &HistoricalRuntime<'_>,
) {
    for item in &runtime.plot_bars {
        if let Some(change) = series_change_from_lens(
            SeriesFamily::PlotBar,
            item.id,
            lens_get(cursor, item.id),
            abs_len(runtime, item.closes.len()),
            runtime.display_origin,
            |start| SeriesFields {
                opens: slice_from(&item.opens, start, runtime.stored_origin),
                highs: slice_from(&item.highs, start, runtime.stored_origin),
                lows: slice_from(&item.lows, start, runtime.stored_origin),
                closes: slice_from(&item.closes, start, runtime.stored_origin),
                colors: slice_from(&item.colors, start, runtime.stored_origin),
                ..SeriesFields::default()
            },
            Some(metadata_header(&item.metadata)),
        ) {
            changes.series.push(change);
        }
    }
}

fn diff_plot_candles(
    changes: &mut RuntimeChanges,
    cursor: &[(u32, usize)],
    runtime: &HistoricalRuntime<'_>,
) {
    for item in &runtime.plot_candles {
        if let Some(change) = series_change_from_lens(
            SeriesFamily::PlotCandle,
            item.id,
            lens_get(cursor, item.id),
            abs_len(runtime, item.closes.len()),
            runtime.display_origin,
            |start| SeriesFields {
                opens: slice_from(&item.opens, start, runtime.stored_origin),
                highs: slice_from(&item.highs, start, runtime.stored_origin),
                lows: slice_from(&item.lows, start, runtime.stored_origin),
                closes: slice_from(&item.closes, start, runtime.stored_origin),
                colors: slice_from(&item.colors, start, runtime.stored_origin),
                wick_colors: slice_from(&item.wick_colors, start, runtime.stored_origin),
                border_colors: slice_from(&item.border_colors, start, runtime.stored_origin),
                ..SeriesFields::default()
            },
            Some(metadata_header(&item.metadata)),
        ) {
            changes.series.push(change);
        }
    }
}

fn diff_color_series(
    changes: &mut RuntimeChanges,
    family: SeriesFamily,
    cursor: &[(u32, usize)],
    runtime: &HistoricalRuntime<'_>,
    items: &[super::plot_history::RuntimeColorSeries],
) {
    for item in items {
        if let Some(change) = series_change_from_lens(
            family,
            item.id,
            lens_get(cursor, item.id),
            abs_len(runtime, item.values.len()),
            runtime.display_origin,
            |start| SeriesFields {
                values: slice_from(&item.values, start, runtime.stored_origin),
                ..SeriesFields::default()
            },
            Some(metadata_header(&item.metadata)),
        ) {
            changes.series.push(change);
        }
    }
}

fn diff_hlines(
    changes: &mut RuntimeChanges,
    cursor: &[crate::HLineOutput],
    items: &[crate::HLineOutput],
) {
    for item in items {
        match cursor.iter().find(|hline| hline.id == item.id) {
            None => changes.hlines.push(HLineChange {
                id: item.id,
                action: HLineAction::Add(item.clone()),
            }),
            Some(previous) if previous != item => changes.hlines.push(HLineChange {
                id: item.id,
                action: HLineAction::Replace(item.clone()),
            }),
            Some(_) => {}
        }
    }
    for previous in cursor {
        if !items.iter().any(|item| item.id == previous.id) {
            changes.hlines.push(HLineChange {
                id: previous.id,
                action: HLineAction::Delete,
            });
        }
    }
}

fn diff_fills(
    changes: &mut RuntimeChanges,
    cursor: &[(u32, usize)],
    runtime: &HistoricalRuntime<'_>,
) {
    for item in &runtime.fills {
        match lens_get(cursor, item.id) {
            None => changes.fills.push(FillChange {
                id: item.id,
                action: FillAction::Add(item.snapshot_from(runtime.display_skip())),
            }),
            Some(old_len) => {
                let new_len = abs_len(runtime, item.colors.len());
                let start = delta_start(old_len, new_len).max(runtime.display_origin);
                let values = slice_from(&item.colors, start, runtime.stored_origin);
                if !values.is_empty() || start < old_len {
                    changes.fills.push(FillChange {
                        id: item.id,
                        action: FillAction::SetColors { start, values },
                    });
                }
            }
        }
    }
    for (id, _) in cursor {
        if !runtime.fills.iter().any(|item| item.id == *id) {
            changes.fills.push(FillChange {
                id: *id,
                action: FillAction::Delete,
            });
        }
    }
}

fn delta_start(old_len: usize, new_len: usize) -> usize {
    if new_len > old_len {
        old_len
    } else if new_len == old_len && new_len > 0 {
        new_len - 1
    } else {
        0
    }
}

fn push_drawing(
    changes: &mut RuntimeChanges,
    family: DrawingFamily,
    cursor: &[(u32, usize)],
    id: u32,
    stable_len: usize,
    tail: impl FnOnce(usize) -> DrawingObject,
) {
    let action = match lens_get(cursor, id) {
        None => DrawingAction::Add(tail(0)),
        Some(_) => {
            let start = stable_len;
            DrawingAction::SetTail {
                start,
                object: tail(start),
            }
        }
    };
    changes.drawings.push(DrawingChange { family, id, action });
}

fn delete_missing(
    changes: &mut RuntimeChanges,
    family: DrawingFamily,
    cursor: &[(u32, usize)],
    present: impl Fn(u32) -> bool,
) {
    for (id, _) in cursor {
        if !present(*id) {
            changes.drawings.push(DrawingChange {
                family,
                id: *id,
                action: DrawingAction::Delete,
            });
        }
    }
}

fn diff_label_drawings(
    changes: &mut RuntimeChanges,
    cursor: &[(u32, usize)],
    items: &[super::drawing_history::RuntimeLabel],
    bar: usize,
) {
    for item in items {
        push_drawing(
            changes,
            DrawingFamily::Label,
            cursor,
            item.id,
            item.snapshots
                .partition_point(|snapshot| snapshot.bar_index < bar),
            |start| {
                DrawingObject::Label(crate::LabelOutput {
                    id: item.id,
                    snapshots: item.snapshots.tail(start),
                })
            },
        );
    }
    delete_missing(changes, DrawingFamily::Label, cursor, |id| {
        items.iter().any(|item| item.id == id)
    });
}

fn diff_line_drawings(
    changes: &mut RuntimeChanges,
    cursor: &[(u32, usize)],
    items: &[super::drawing_history::RuntimeLine],
    bar: usize,
) {
    for item in items {
        push_drawing(
            changes,
            DrawingFamily::Line,
            cursor,
            item.id,
            item.snapshots
                .partition_point(|snapshot| snapshot.bar_index < bar),
            |start| {
                DrawingObject::Line(crate::LineOutput {
                    id: item.id,
                    snapshots: item.snapshots.tail(start),
                })
            },
        );
    }
    delete_missing(changes, DrawingFamily::Line, cursor, |id| {
        items.iter().any(|item| item.id == id)
    });
}

fn diff_line_fill_drawings(
    changes: &mut RuntimeChanges,
    cursor: &[(u32, usize)],
    items: &[super::drawing_history::RuntimeLineFill],
    bar: usize,
) {
    for item in items {
        push_drawing(
            changes,
            DrawingFamily::LineFill,
            cursor,
            item.id,
            item.snapshots
                .partition_point(|snapshot| snapshot.bar_index < bar),
            |start| {
                DrawingObject::LineFill(crate::LineFillOutput {
                    id: item.id,
                    snapshots: item.snapshots.tail(start),
                })
            },
        );
    }
    delete_missing(changes, DrawingFamily::LineFill, cursor, |id| {
        items.iter().any(|item| item.id == id)
    });
}

fn diff_polyline_drawings(
    changes: &mut RuntimeChanges,
    cursor: &[(u32, usize)],
    items: &[super::drawing_history::RuntimePolyline],
    bar: usize,
) {
    for item in items {
        push_drawing(
            changes,
            DrawingFamily::Polyline,
            cursor,
            item.id,
            item.snapshots
                .partition_point(|snapshot| snapshot.bar_index < bar),
            |start| {
                DrawingObject::Polyline(crate::PolylineOutput {
                    id: item.id,
                    snapshots: item.snapshots.tail(start),
                })
            },
        );
    }
    delete_missing(changes, DrawingFamily::Polyline, cursor, |id| {
        items.iter().any(|item| item.id == id)
    });
}

fn diff_box_drawings(
    changes: &mut RuntimeChanges,
    cursor: &[(u32, usize)],
    items: &[super::drawing_history::RuntimeBox],
    bar: usize,
) {
    for item in items {
        push_drawing(
            changes,
            DrawingFamily::Box,
            cursor,
            item.id,
            item.snapshots
                .partition_point(|snapshot| snapshot.bar_index < bar),
            |start| {
                DrawingObject::Box(crate::BoxOutput {
                    id: item.id,
                    snapshots: item.snapshots.tail(start),
                })
            },
        );
    }
    delete_missing(changes, DrawingFamily::Box, cursor, |id| {
        items.iter().any(|item| item.id == id)
    });
}

fn diff_table_drawings(
    changes: &mut RuntimeChanges,
    cursor: &[(u32, usize)],
    items: &[super::drawing_history::RuntimeTable],
    bar: usize,
) {
    for item in items {
        push_drawing(
            changes,
            DrawingFamily::Table,
            cursor,
            item.id,
            item.snapshots
                .partition_point(|snapshot| snapshot.bar_index < bar),
            |start| {
                DrawingObject::Table(Box::new(crate::TableOutput {
                    id: item.id,
                    snapshots: item.snapshots.tail(start),
                    position: item.position.clone(),
                    bg_color: item.bg_color.clone(),
                    frame_color: item.frame_color.clone(),
                    frame_width: item.frame_width.clone(),
                    border_color: item.border_color.clone(),
                    border_width: item.border_width.clone(),
                    columns: item.columns,
                    rows: item.rows,
                }))
            },
        );
    }
    delete_missing(changes, DrawingFamily::Table, cursor, |id| {
        items.iter().any(|item| item.id == id)
    });
}

fn diff_alerts(
    changes: &mut RuntimeChanges,
    previous: &[crate::AlertEvent],
    current: &[crate::AlertEvent],
) {
    let prefix = previous
        .iter()
        .zip(current)
        .take_while(|(a, b)| a == b)
        .count();
    // Preserve occurrence counts for alert.freq_all calls with identical payloads.
    for event in previous[prefix..].iter().rev() {
        changes.alerts.push(EventChange {
            action: EventAction::Remove,
            event: event.clone(),
        });
    }
    for event in &current[prefix..] {
        changes.alerts.push(EventChange {
            action: EventAction::Add,
            event: event.clone(),
        });
    }
}

fn diff_strategy(cursor: &OutputCursor, runtime: &HistoricalRuntime<'_>) -> StrategyChanges {
    let broker = &runtime.strategy_broker;
    StrategyChanges {
        orders: list_splice(cursor.orders, broker.order_len(), |start| {
            broker.order_tail(start)
        }),
        trades: list_splice(cursor.trades, broker.trade_len(), |start| {
            broker.trade_tail(start)
        }),
        alerts: fill_alert_splice(cursor.fill_alerts, broker),
        position: list_splice(cursor.position, broker.position_len(), |start| {
            broker.position_tail(start)
        }),
        equity: list_splice(cursor.equity, broker.equity_len(), |start| {
            broker.equity_tail(start)
        }),
        diagnostics: Some(broker.diagnostics_slice().to_vec()),
    }
}

fn list_splice<T: Clone>(
    old_len: usize,
    new_len: usize,
    suffix: impl FnOnce(usize) -> Vec<T>,
) -> Option<ListSplice<T>> {
    if new_len == old_len && new_len == 0 {
        return None;
    }
    if new_len > old_len {
        return Some(ListSplice {
            start: old_len,
            items: suffix(old_len),
        });
    }
    if new_len == old_len {
        return Some(ListSplice {
            start: new_len - 1,
            items: suffix(new_len - 1),
        });
    }
    Some(ListSplice {
        start: new_len,
        items: Vec::new(),
    })
}

fn fill_alert_splice(
    old_len: usize,
    broker: &crate::BrokerState,
) -> Option<ListSplice<StrategyOrderFillAlertOutput>> {
    let new_len = broker.fill_alert_len();
    if new_len == old_len && new_len == 0 {
        return None;
    }
    if new_len < old_len {
        return Some(ListSplice {
            start: new_len,
            items: Vec::new(),
        });
    }
    let start = delta_start(old_len, new_len);
    Some(ListSplice {
        start,
        items: broker.fill_alerts_from(start),
    })
}

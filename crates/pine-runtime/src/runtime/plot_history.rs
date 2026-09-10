use std::sync::Arc;

use super::historical::HistoricalRuntime;
use crate::{PineValue, PlotSeries};

fn copy_with_next_value(values: &[PineValue]) -> Vec<PineValue> {
    let mut copy = Vec::with_capacity(values.len().saturating_add(1));
    copy.extend_from_slice(values);
    copy
}

fn copy_for_next_bar(plot: &PlotSeries) -> PlotSeries {
    PlotSeries {
        id: plot.id,
        values: copy_with_next_value(&plot.values),
        colors: copy_with_next_value(&plot.colors),
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

impl HistoricalRuntime<'_> {
    pub(crate) fn plots_mut(&mut self) -> &mut Vec<PlotSeries> {
        if Arc::get_mut(&mut self.plots).is_none() {
            // A normal Vec clone has no guaranteed spare capacity. Detach once
            // with room for the pending bar, avoiding another allocation/copy
            // when the first plot statement appends its value and color.
            self.plots = Arc::new(self.plots.iter().map(copy_for_next_bar).collect());
        }
        Arc::get_mut(&mut self.plots).expect("plot history was detached")
    }
}

use super::{append_history::AppendHistory, historical::HistoricalRuntime};
use crate::{OutputMetadata, PineValue, PlotSeries};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub(crate) struct RuntimePlot {
    pub(crate) id: u32,
    pub(crate) values: AppendHistory<PineValue>,
    pub(crate) colors: AppendHistory<PineValue>,
    pub(crate) metadata: OutputMetadata,
    pub(crate) linewidth: PineValue,
    pub(crate) style: PineValue,
    pub(crate) track_price: PineValue,
    pub(crate) hist_base: PineValue,
    pub(crate) join: PineValue,
    pub(crate) format: PineValue,
    pub(crate) precision: PineValue,
}
impl RuntimePlot {
    pub(crate) fn new(id: u32, values: Vec<PineValue>) -> Self {
        let public = PlotSeries::new(id, values);
        Self {
            id: public.id,
            values: history(public.values),
            colors: history(public.colors),
            metadata: public.metadata,
            linewidth: public.linewidth,
            style: public.style,
            track_price: public.track_price,
            hist_base: public.hist_base,
            join: public.join,
            format: public.format,
            precision: public.precision,
        }
    }
    pub(crate) fn snapshot(&self) -> PlotSeries {
        PlotSeries {
            id: self.id,
            values: self.values.to_vec(),
            colors: self.colors.to_vec(),
            metadata: self.metadata.clone(),
            linewidth: self.linewidth.clone(),
            style: self.style.clone(),
            track_price: self.track_price.clone(),
            hist_base: self.hist_base.clone(),
            join: self.join.clone(),
            format: self.format.clone(),
            precision: self.precision.clone(),
        }
    }
}
fn history(values: Vec<PineValue>) -> AppendHistory<PineValue> {
    let mut result = AppendHistory::default();
    for value in values {
        result.push(value);
    }
    result
}
impl HistoricalRuntime<'_> {
    pub(crate) fn plots_mut(&mut self) -> &mut Vec<RuntimePlot> {
        Arc::make_mut(&mut self.plots)
    }
}

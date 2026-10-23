//! Stateful price/volume builtins are recomputed from the previous bar on
//! every external observation, without accumulating discarded tick values.
use super::*;

macro_rules! price_flow_state {
    ($($field:ident: $kind:ty),+ $(,)?) => {
        #[derive(Clone)]
        pub(super) struct PriceFlowCheckpoint { $($field: $kind),+ }

        impl PriceFlowCheckpoint {
            pub(super) fn capture(runtime: &HistoricalRuntime<'_>) -> Self {
                Self { $($field: runtime.$field.clone()),+ }
            }

            pub(super) fn restore(&self, runtime: &mut HistoricalRuntime<'_>) {
                $(runtime.$field.clone_from(&self.$field);)+
            }
        }
    }
}

price_flow_state! {
    price_flow_previous_close: Option<f64>,
    price_flow_previous_volume: Option<f64>,
    accdist_state: PineValue,
    accdist_current: PineValue,
    iii_current: PineValue,
    nvi_state: PineValue,
    nvi_current: PineValue,
    obv_state: PineValue,
    obv_current: PineValue,
    pvi_state: PineValue,
    pvi_current: PineValue,
    pvt_state: PineValue,
    pvt_current: PineValue,
    vwap_weighted_sum: f64,
    vwap_volume_sum: f64,
    vwap_current: PineValue,
    wad_state: PineValue,
    wad_current: PineValue,
    wvad_current: PineValue,
}

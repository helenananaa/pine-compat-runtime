//! Host-provided metadata for symbols evaluated inside request expressions.
use std::collections::BTreeMap;

use serde::Deserialize;

use super::{ChartContext, RequestTimeframe};

pub const SYMBOL_METADATA_SCHEMA_VERSION: u32 = 1;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Metadata {
    min_move: u32,
    price_scale: u32,
    quantity_precision: Option<u32>,
    quantity_step: Option<String>,
    min_contract: Option<String>,
    point_value: Option<f64>,
    currency: Option<String>,
}

/// Parse a symbol-to-metadata object. Identity comes from the exact map key;
/// timeframe comes from the request, never from this registry.
pub fn symbol_contexts_from_json(json: &str) -> Result<Vec<ChartContext>, String> {
    let entries: BTreeMap<String, Metadata> =
        serde_json::from_str(json).map_err(|error| format!("invalid symbol metadata: {error}"))?;
    entries
        .into_iter()
        .map(|(symbol, metadata)| {
            if symbol.trim().is_empty() || symbol.trim() != symbol {
                return Err("symbol metadata requires exact nonempty symbol keys".to_owned());
            }
            let mut chart = ChartContext::new(symbol, RequestTimeframe::default())
                .with_price_grid(metadata.min_move, metadata.price_scale)
                .map_err(str::to_owned)?;
            if let Some(precision) = metadata.quantity_precision {
                chart = chart
                    .with_quantity_precision(precision)
                    .map_err(str::to_owned)?;
            }
            if let Some(value) = metadata.point_value {
                chart = chart.with_point_value(value).map_err(str::to_owned)?;
            }
            if let Some(value) = metadata.quantity_step {
                chart = chart
                    .with_quantity_step(&value)
                    .map_err(|error| format!("quantityStep: {error}"))?;
            }
            if let Some(value) = metadata.min_contract {
                chart = chart
                    .with_min_contract(&value)
                    .map_err(|error| format!("minContract: {error}"))?;
            }
            if let Some(currency) = metadata.currency {
                chart = chart.with_currency(currency).map_err(str::to_owned)?;
            }
            Ok(chart)
        })
        .collect()
}

use super::{CurrencyRateError, CurrencyRateFrame, InMemoryCurrencyRateProvider};
use serde::Deserialize;

pub const CURRENCY_RATE_SCHEMA_VERSION: u32 = 1;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Input {
    schema_version: u32,
    pairs: Vec<Pair>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Pair {
    from: String,
    to: String,
    rates: Option<Vec<Frame>>,
    #[serde(default)]
    unavailable: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Frame {
    effective_time: i64,
    rate: f64,
}

/// Parse explicit effective conversion data; no market symbols or network
/// connections are resolved by the interpreter. An empty `rates` array denotes
/// a known stream with no available history, while `unavailable` denotes a
/// host-resolved pair for which no conversion can be calculated.
pub fn currency_rates_from_json(
    json: &str,
) -> Result<InMemoryCurrencyRateProvider, CurrencyRateError> {
    let input: Input = serde_json::from_str(json)
        .map_err(|error| CurrencyRateError::InvalidInput(error.to_string()))?;
    if input.schema_version != CURRENCY_RATE_SCHEMA_VERSION {
        return Err(CurrencyRateError::InvalidInput(format!(
            "unsupported schemaVersion {}",
            input.schema_version
        )));
    }
    let mut provider = InMemoryCurrencyRateProvider::default();
    for pair in input.pairs {
        match (pair.unavailable, pair.rates) {
            (true, None) => provider.insert_unavailable(pair.from, pair.to)?,
            (false, Some(frames)) => provider.insert(
                pair.from,
                pair.to,
                frames
                    .into_iter()
                    .map(|frame| CurrencyRateFrame {
                        effective_time: frame.effective_time,
                        rate: frame.rate,
                    })
                    .collect(),
            )?,
            _ => {
                return Err(CurrencyRateError::InvalidInput(
                    "each pair must provide either rates or unavailable=true".to_owned(),
                ));
            }
        }
    }
    Ok(provider)
}

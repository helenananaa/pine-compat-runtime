//! Host-neutral standard prices paired with the exact visible chart observation.
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{Bar, BarUpdateKind, HistoricalRuntime, RuntimeError};

pub const STANDARD_OHLC_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChartType {
    #[default]
    Standard,
    #[serde(rename = "heikinashi")]
    HeikinAshi,
}

impl ChartType {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "standard" => Ok(Self::Standard),
            "heikinashi" => Ok(Self::HeikinAshi),
            _ => Err(format!(
                "unsupported chartType `{value}`; expected standard or heikinashi"
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StandardOhlcObservation {
    pub chart: Bar,
    pub kind: BarUpdateKind,
    pub execution_time: Option<i64>,
}

/// Supplies only prices visible at the requested observation. Implementations
/// must be deterministic under rollback/replay and must not substitute final
/// historical OHLC for a forming observation. Acquisition belongs to the host.
pub trait StandardOhlcProvider: Send + Sync {
    fn standard_bar(&self, observation: StandardOhlcObservation) -> Option<Bar>;
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StandardOhlcPair {
    pub observation: StandardOhlcObservation,
    pub standard: Bar,
}

type ObservationKey = (i64, [u64; 5], u8, Option<i64>);

fn key(observation: StandardOhlcObservation) -> ObservationKey {
    let bar = observation.chart;
    (
        bar.time,
        [bar.open, bar.high, bar.low, bar.close, bar.volume].map(f64::to_bits),
        match observation.kind {
            BarUpdateKind::Historical => 0,
            BarUpdateKind::Forming => 1,
            BarUpdateKind::Confirmed => 2,
        },
        observation.execution_time,
    )
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryStandardOhlcProvider {
    bars: HashMap<ObservationKey, Bar>,
}

impl InMemoryStandardOhlcProvider {
    pub fn from_pairs(pairs: impl IntoIterator<Item = StandardOhlcPair>) -> Result<Self, String> {
        let mut provider = Self::default();
        for pair in pairs {
            validate_bar(pair.observation.chart)?;
            validate_pair(pair.observation.chart, pair.standard)?;
            if pair.observation.kind == BarUpdateKind::Forming
                && pair.observation.execution_time.is_none()
            {
                return Err("forming standard OHLC observations require executionTime".to_owned());
            }
            if provider
                .bars
                .insert(key(pair.observation), pair.standard)
                .is_some()
            {
                return Err("duplicate standard OHLC observation".to_owned());
            }
        }
        Ok(provider)
    }
}

impl StandardOhlcProvider for InMemoryStandardOhlcProvider {
    fn standard_bar(&self, observation: StandardOhlcObservation) -> Option<Bar> {
        self.bars.get(&key(observation)).copied()
    }
}

pub fn standard_ohlc_from_json(json: &str) -> Result<InMemoryStandardOhlcProvider, String> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct Input {
        schema_version: u32,
        observations: Vec<StandardOhlcPair>,
    }
    let input: Input =
        serde_json::from_str(json).map_err(|e| format!("invalid standard OHLC input: {e}"))?;
    if input.schema_version != STANDARD_OHLC_SCHEMA_VERSION {
        return Err("unsupported standard OHLC schemaVersion".to_owned());
    }
    InMemoryStandardOhlcProvider::from_pairs(input.observations)
}

fn validate_bar(bar: Bar) -> Result<(), String> {
    if ![bar.open, bar.high, bar.low, bar.close, bar.volume]
        .into_iter()
        .all(f64::is_finite)
        || bar.high < bar.open.max(bar.close)
        || bar.low > bar.open.min(bar.close)
        || bar.low > bar.high
        || bar.volume < 0.0
    {
        return Err(
            "standard OHLC input requires finite, ordered OHLC and nonnegative volume".to_owned(),
        );
    }
    Ok(())
}

fn validate_pair(chart: Bar, standard: Bar) -> Result<(), String> {
    validate_bar(standard)?;
    if chart.time != standard.time {
        return Err("standard OHLC time must equal the chart observation time".to_owned());
    }
    Ok(())
}

impl HistoricalRuntime<'_> {
    pub(crate) fn standard_bar_for_observation(
        &self,
        chart: Bar,
        kind: BarUpdateKind,
        execution_time: Option<i64>,
    ) -> Result<Option<Bar>, RuntimeError> {
        if self.program.script_mode != pine_ir::ScriptMode::Strategy
            || !self.program.strategy_settings.fill_orders_on_standard_ohlc
            || self.request_environment.chart().chart_type() != ChartType::HeikinAshi
        {
            return Ok(None);
        }
        let error = |message: String| RuntimeError {
            message: format!("E_STANDARD_OHLC_INPUT: {message}"),
        };
        if self.program.strategy_settings.calc_on_every_history_tick {
            return Err(error("calc_on_every_history_tick with separate Heikin-Ashi and standard price paths is not supported".to_owned()));
        }
        if self.external_execution.is_some() {
            return Err(error("external execution must implement standard OHLC fills in its broker; this native dual-price profile is not supported in external mode".to_owned()));
        }
        let standard = self
            .request_environment
            .standard_ohlc_bar(StandardOhlcObservation {
                chart,
                kind,
                execution_time,
            })
            .ok_or_else(|| {
                error(format!(
                    "missing standard prices for {:?} observation at {}",
                    kind, chart.time
                ))
            })?;
        validate_pair(chart, standard).map_err(error)?;
        Ok(Some(standard))
    }
}

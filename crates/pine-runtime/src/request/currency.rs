//! Host-supplied, effective conversion rates. Acquisition and source selection
//! belong to the host; this module performs deterministic as-of lookup only.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    sync::Arc,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CurrencyRateFrame {
    /// First calculation timestamp at which this confirmed rate is available.
    /// This is availability time, not the opening timestamp of its source bar.
    pub effective_time: i64,
    /// Units of the destination currency per unit of the source currency.
    pub rate: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CurrencyRateError {
    InvalidCurrency(String),
    InvalidRate { index: usize },
    InvalidResolvedRate,
    InvalidInput(String),
    UnorderedTimes { index: usize },
    DuplicatePair { from: String, to: String },
    MissingPair { from: String, to: String },
    UnavailablePair { from: String, to: String },
    MissingHistory { from: String, to: String, time: i64 },
}

impl fmt::Display for CurrencyRateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(message) => write!(f, "invalid currency rate input: {message}"),
            Self::InvalidCurrency(code) => write!(
                f,
                "invalid currency code `{code}`: expected nonempty uppercase letters"
            ),
            Self::InvalidRate { index } => write!(
                f,
                "currency rate at index {index} must be finite and positive"
            ),
            Self::InvalidResolvedRate => write!(
                f,
                "currency rate provider returned a rate that is not finite and positive"
            ),
            Self::UnorderedTimes { index } => write!(
                f,
                "currency rate effective times must strictly increase at index {index}"
            ),
            Self::DuplicatePair { from, to } => {
                write!(f, "duplicate currency rate stream `{from}` to `{to}`")
            }
            Self::MissingPair { from, to } => {
                write!(f, "missing currency rate data from `{from}` to `{to}`")
            }
            Self::UnavailablePair { from, to } => write!(
                f,
                "no currency conversion rate is available from `{from}` to `{to}`"
            ),
            Self::MissingHistory { from, to, time } => write!(
                f,
                "missing currency rate history from `{from}` to `{to}` at {time}"
            ),
        }
    }
}

impl std::error::Error for CurrencyRateError {}

pub(super) fn validate_currency(code: &str) -> Result<(), CurrencyRateError> {
    if code.is_empty() || !code.bytes().all(|c| c.is_ascii_uppercase()) {
        return Err(CurrencyRateError::InvalidCurrency(code.to_owned()));
    }
    Ok(())
}

/// Supplies a conversion rate already available at the requested timestamp.
/// Implementations must not use later observations, access application state
/// implicitly, or silently replace a missing foreign pair with an identity.
pub trait CurrencyRateProvider: Send + Sync {
    fn rate(&self, from: &str, to: &str, time: i64) -> Result<f64, CurrencyRateError>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NoCurrencyRateProvider;

impl CurrencyRateProvider for NoCurrencyRateProvider {
    fn rate(&self, from: &str, to: &str, _time: i64) -> Result<f64, CurrencyRateError> {
        validate_currency(from)?;
        validate_currency(to)?;
        if from == to {
            Ok(1.0)
        } else {
            Err(CurrencyRateError::MissingPair {
                from: from.to_owned(),
                to: to.to_owned(),
            })
        }
    }
}

/// Immutable streams may contain future frames; lookup excludes them by time.
/// No inverse or cross-pair market observations are invented. A host can supply
/// each required direction explicitly, or implement the provider trait itself.
#[derive(Debug, Clone, Default)]
pub struct InMemoryCurrencyRateProvider {
    streams: BTreeMap<String, BTreeMap<String, Arc<[CurrencyRateFrame]>>>,
    unavailable: BTreeMap<String, BTreeSet<String>>,
}

impl InMemoryCurrencyRateProvider {
    pub fn insert(
        &mut self,
        from: impl Into<String>,
        to: impl Into<String>,
        frames: Vec<CurrencyRateFrame>,
    ) -> Result<(), CurrencyRateError> {
        let (from, to) = (from.into(), to.into());
        validate_currency(&from)?;
        validate_currency(&to)?;
        for (index, frame) in frames.iter().enumerate() {
            if !frame.rate.is_finite() || frame.rate <= 0.0 {
                return Err(CurrencyRateError::InvalidRate { index });
            }
            if index > 0 && frame.effective_time <= frames[index - 1].effective_time {
                return Err(CurrencyRateError::UnorderedTimes { index });
            }
        }
        if self
            .streams
            .get(&from)
            .is_some_and(|pairs| pairs.contains_key(&to))
            || self
                .unavailable
                .get(&from)
                .is_some_and(|pairs| pairs.contains(&to))
        {
            return Err(CurrencyRateError::DuplicatePair { from, to });
        }
        self.streams
            .entry(from)
            .or_default()
            .insert(to, frames.into());
        Ok(())
    }

    /// Record a host-resolved unavailable pair, distinct from missing input.
    pub fn insert_unavailable(
        &mut self,
        from: impl Into<String>,
        to: impl Into<String>,
    ) -> Result<(), CurrencyRateError> {
        let (from, to) = (from.into(), to.into());
        validate_currency(&from)?;
        validate_currency(&to)?;
        if self
            .streams
            .get(&from)
            .is_some_and(|pairs| pairs.contains_key(&to))
            || self
                .unavailable
                .get(&from)
                .is_some_and(|pairs| pairs.contains(&to))
        {
            return Err(CurrencyRateError::DuplicatePair { from, to });
        }
        self.unavailable.entry(from).or_default().insert(to);
        Ok(())
    }
}

impl CurrencyRateProvider for InMemoryCurrencyRateProvider {
    fn rate(&self, from: &str, to: &str, time: i64) -> Result<f64, CurrencyRateError> {
        validate_currency(from)?;
        validate_currency(to)?;
        if from == to {
            return Ok(1.0);
        }
        if self
            .unavailable
            .get(from)
            .is_some_and(|pairs| pairs.contains(to))
        {
            return Err(CurrencyRateError::UnavailablePair {
                from: from.into(),
                to: to.into(),
            });
        }
        let frames = self
            .streams
            .get(from)
            .and_then(|pairs| pairs.get(to))
            .ok_or_else(|| CurrencyRateError::MissingPair {
                from: from.to_owned(),
                to: to.to_owned(),
            })?;
        let index = frames.partition_point(|frame| frame.effective_time <= time);
        index
            .checked_sub(1)
            .map(|index| frames[index].rate)
            .ok_or_else(|| CurrencyRateError::MissingHistory {
                from: from.to_owned(),
                to: to.to_owned(),
                time,
            })
    }
}

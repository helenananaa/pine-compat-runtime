//! Immutable host-supplied trading boundaries. The host supplies facts; Pine
//! interval selection, offsets and request confirmation remain in the runtime.
use std::{collections::BTreeMap, fmt};

use chrono::{DateTime, Utc};
use serde::Deserialize;

use super::RequestTimeframe;

pub const BAR_CALENDAR_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BarCalendarError(pub(crate) String);

impl fmt::Display for BarCalendarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for BarCalendarError {}

/// UTC millisecond timestamps. `trading_day` is the UTC midnight identifying
/// the session's trading date, including sessions opening the previous day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct BarTimeInterval {
    open: i64,
    close: i64,
    trading_day: i64,
}

impl BarTimeInterval {
    #[must_use]
    pub fn new(open: i64, close: i64, trading_day: i64) -> Self {
        Self {
            open,
            close,
            trading_day,
        }
    }

    #[must_use]
    pub fn open(self) -> i64 {
        self.open
    }
    #[must_use]
    pub fn close(self) -> i64 {
        self.close
    }
    #[must_use]
    pub fn trading_day(self) -> i64 {
        self.trading_day
    }
}

#[derive(Debug, Clone)]
pub struct BarCalendar {
    symbol: String,
    timezone: String,
    timeframes: BTreeMap<String, Vec<BarTimeInterval>>,
}

impl BarCalendar {
    pub fn new(
        symbol: impl Into<String>,
        timezone: impl Into<String>,
        streams: impl IntoIterator<Item = (RequestTimeframe, Vec<BarTimeInterval>)>,
    ) -> Result<Self, BarCalendarError> {
        let symbol = symbol.into();
        let timezone = timezone.into();
        if symbol.is_empty() || symbol.trim() != symbol {
            return Err(BarCalendarError(
                "bar calendar symbol must be nonempty without surrounding whitespace".into(),
            ));
        }
        if crate::builtins::time::timezone_offset_seconds(&timezone, &DateTime::<Utc>::UNIX_EPOCH)
            .is_none()
        {
            return Err(BarCalendarError(format!(
                "bar calendar `{symbol}` has unsupported timezone `{timezone}`"
            )));
        }
        let mut timeframes = BTreeMap::new();
        for (timeframe, intervals) in streams {
            let key = canonical_timeframe(&timeframe);
            if intervals.is_empty() {
                return Err(BarCalendarError(format!(
                    "bar calendar `{symbol}` timeframe `{key}` has no intervals"
                )));
            }
            let mut previous: Option<BarTimeInterval> = None;
            for interval in &intervals {
                if interval.open >= interval.close
                    || interval.trading_day.rem_euclid(86_400_000) != 0
                    || [interval.open, interval.close, interval.trading_day]
                        .iter()
                        .any(|t| DateTime::<Utc>::from_timestamp_millis(*t).is_none())
                {
                    return Err(BarCalendarError(format!(
                        "bar calendar `{symbol}` timeframe `{key}` has invalid interval timestamps"
                    )));
                }
                if previous.is_some_and(|prev| {
                    prev.close > interval.open || prev.trading_day > interval.trading_day
                }) {
                    return Err(BarCalendarError(format!(
                        "bar calendar `{symbol}` timeframe `{key}` intervals overlap or are not sorted"
                    )));
                }
                previous = Some(*interval);
            }
            if timeframes.insert(key.clone(), intervals).is_some() {
                return Err(BarCalendarError(format!(
                    "duplicate bar calendar `{symbol}` timeframe `{key}`"
                )));
            }
        }
        if timeframes.is_empty() {
            return Err(BarCalendarError(format!(
                "bar calendar `{symbol}` has no timeframes"
            )));
        }
        Ok(Self {
            symbol,
            timezone,
            timeframes,
        })
    }

    #[must_use]
    pub fn symbol(&self) -> &str {
        &self.symbol
    }
    #[must_use]
    pub fn timezone(&self) -> &str {
        &self.timezone
    }

    fn intervals(
        &self,
        timeframe: &RequestTimeframe,
    ) -> Result<&[BarTimeInterval], BarCalendarError> {
        self.timeframes
            .get(&canonical_timeframe(timeframe))
            .map(Vec::as_slice)
            .ok_or_else(|| {
                BarCalendarError(format!(
                    "missing bar calendar timeframe `{}` for symbol `{}`",
                    timeframe.value(),
                    self.symbol
                ))
            })
    }

    fn coverage_error(&self, timeframe: &RequestTimeframe, time: i64) -> BarCalendarError {
        BarCalendarError(format!(
            "missing bar calendar coverage for symbol `{}` timeframe `{}` at `{time}`",
            self.symbol,
            timeframe.value()
        ))
    }

    /// `None` is a known closure inside the supplied coverage. Outside coverage
    /// is an error, never an invented continuous-UTC interval.
    pub(crate) fn select(
        &self,
        timeframe: &RequestTimeframe,
        time: i64,
        bars_back: i64,
    ) -> Result<Option<BarTimeInterval>, BarCalendarError> {
        let intervals = self.intervals(timeframe)?;
        if time < intervals[0].open || time >= intervals[intervals.len() - 1].close {
            return Err(self.coverage_error(timeframe, time));
        }
        let index = intervals.partition_point(|interval| interval.open <= time) - 1;
        if time >= intervals[index].close {
            return Ok(None);
        }
        let selected = i64::try_from(index)
            .ok()
            .and_then(|index| index.checked_sub(bars_back))
            .and_then(|index| usize::try_from(index).ok())
            .and_then(|index| intervals.get(index))
            .copied();
        selected
            .map(Some)
            .ok_or_else(|| self.coverage_error(timeframe, time))
    }

    pub(crate) fn exact(
        &self,
        timeframe: &RequestTimeframe,
        time: i64,
    ) -> Result<BarTimeInterval, BarCalendarError> {
        self.select(timeframe, time, 0)?
            .filter(|interval| interval.open == time)
            .ok_or_else(|| {
                BarCalendarError(format!(
                    "bar calendar `{}` timeframe `{}` has no bar opening at `{time}`",
                    self.symbol,
                    timeframe.value()
                ))
            })
    }

    pub(crate) fn previous_open(
        &self,
        timeframe: &RequestTimeframe,
        time: i64,
    ) -> Result<i64, BarCalendarError> {
        self.exact(timeframe, time)?;
        let intervals = self.intervals(timeframe)?;
        let index = intervals.partition_point(|interval| interval.open < time);
        // No predecessor is needed to emit the first covered gapped value.
        Ok(index
            .checked_sub(1)
            .map_or(i64::MIN, |index| intervals[index].open))
    }

    pub(crate) fn merge_bucket(
        &self,
        timeframe: &RequestTimeframe,
        time: i64,
    ) -> Result<Option<i64>, BarCalendarError> {
        let intervals = self.intervals(timeframe)?;
        Ok(intervals
            .partition_point(|interval| interval.open <= time)
            .checked_sub(1)
            .map(|index| intervals[index].open))
    }
}

fn canonical_timeframe(timeframe: &RequestTimeframe) -> String {
    let value = timeframe.value();
    let (count, unit) = match value
        .as_bytes()
        .last()
        .copied()
        .filter(u8::is_ascii_alphabetic)
    {
        Some(_) => (&value[..value.len() - 1], &value[value.len() - 1..]),
        None => (value, ""),
    };
    let count = if count.is_empty() {
        1
    } else {
        count.parse::<u64>().expect("validated timeframe")
    };
    if count == 1 && matches!(unit, "D" | "W" | "M") {
        unit.to_owned()
    } else {
        format!("{count}{unit}")
    }
}

#[derive(Debug, Clone, Default)]
pub struct BarCalendarRegistry {
    calendars: BTreeMap<String, BarCalendar>,
}

impl BarCalendarRegistry {
    pub fn new(calendars: impl IntoIterator<Item = BarCalendar>) -> Result<Self, BarCalendarError> {
        let mut registry = Self::default();
        for calendar in calendars {
            let symbol = calendar.symbol.clone();
            if registry
                .calendars
                .insert(symbol.clone(), calendar)
                .is_some()
            {
                return Err(BarCalendarError(format!(
                    "duplicate bar calendar symbol `{symbol}`"
                )));
            }
        }
        Ok(registry)
    }

    #[must_use]
    pub fn get(&self, symbol: &str) -> Option<&BarCalendar> {
        self.calendars.get(symbol)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct CalendarInput {
    schema_version: u32,
    symbols: Vec<SymbolInput>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SymbolInput {
    symbol: String,
    timezone: String,
    timeframes: Vec<TimeframeInput>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TimeframeInput {
    timeframe: String,
    intervals: Vec<BarTimeInterval>,
}

pub fn bar_calendars_from_json(json: &str) -> Result<BarCalendarRegistry, BarCalendarError> {
    let input: CalendarInput = serde_json::from_str(json)
        .map_err(|err| BarCalendarError(format!("invalid bar calendars: {err}")))?;
    if input.schema_version != BAR_CALENDAR_SCHEMA_VERSION {
        return Err(BarCalendarError(format!(
            "unsupported bar calendar schema version {}",
            input.schema_version
        )));
    }
    BarCalendarRegistry::new(
        input
            .symbols
            .into_iter()
            .map(|symbol| {
                let streams = symbol
                    .timeframes
                    .into_iter()
                    .map(|stream| {
                        RequestTimeframe::parse(stream.timeframe)
                            .map(|tf| (tf, stream.intervals))
                            .map_err(|err| BarCalendarError(err.to_string()))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                BarCalendar::new(symbol.symbol, symbol.timezone, streams)
            })
            .collect::<Result<Vec<_>, _>>()?,
    )
}

use std::{collections::HashMap, fmt, sync::Arc};

use pine_ir::CallSiteId;

use crate::Bar;

use super::{
    BarCalendar, BarCalendarRegistry, ChartContext, RequestTimeframe, bars::validate_requested_bars,
};
use crate::RuntimeError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RequestKey {
    symbol: String,
    timeframe: RequestTimeframe,
}

impl RequestKey {
    #[must_use]
    pub fn new(symbol: impl Into<String>, timeframe: RequestTimeframe) -> Self {
        Self {
            symbol: symbol.into(),
            timeframe,
        }
    }

    #[must_use]
    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    #[must_use]
    pub fn timeframe(&self) -> &RequestTimeframe {
        &self.timeframe
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct RequestCacheKey {
    call_site_id: CallSiteId,
    symbol: String,
    timeframe: String,
}

impl RequestCacheKey {
    #[must_use]
    pub(crate) fn new(
        call_site_id: CallSiteId,
        symbol: impl Into<String>,
        timeframe: impl Into<String>,
    ) -> Self {
        Self {
            call_site_id,
            symbol: symbol.into(),
            timeframe: timeframe.into(),
        }
    }

    #[must_use]
    pub(crate) fn context(&self) -> (&str, &str) {
        (&self.symbol, &self.timeframe)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestDataError {
    MissingData { symbol: String, timeframe: String },
    DuplicateKey { symbol: String, timeframe: String },
    DuplicateBars { time: i64 },
    UnsortedBars { previous_time: i64, time: i64 },
}

impl fmt::Display for RequestDataError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingData { symbol, timeframe } => {
                write!(
                    formatter,
                    "missing request data for symbol `{symbol}` timeframe `{timeframe}`"
                )
            }
            Self::DuplicateKey { symbol, timeframe } => {
                write!(
                    formatter,
                    "duplicate request data for symbol `{symbol}` timeframe `{timeframe}`"
                )
            }
            Self::DuplicateBars { time } => {
                write!(formatter, "duplicate requested bar time `{time}`")
            }
            Self::UnsortedBars {
                previous_time,
                time,
            } => {
                write!(
                    formatter,
                    "requested bars are not sorted: `{time}` follows `{previous_time}`"
                )
            }
        }
    }
}

pub trait RequestDataProvider: Send + Sync {
    fn bars<'a>(&'a self, key: &RequestKey) -> Result<&'a [Bar], RequestDataError>;
}

#[derive(Debug, Clone, Default)]
pub struct NoRequestDataProvider;

impl RequestDataProvider for NoRequestDataProvider {
    fn bars<'a>(&'a self, key: &RequestKey) -> Result<&'a [Bar], RequestDataError> {
        Err(RequestDataError::MissingData {
            symbol: key.symbol().to_owned(),
            timeframe: key.timeframe().value().to_owned(),
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryRequestDataProvider {
    streams: HashMap<RequestKey, Vec<Bar>>,
}

impl InMemoryRequestDataProvider {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: RequestKey, bars: Vec<Bar>) -> Result<(), RequestDataError> {
        validate_requested_bars(&bars)?;
        if self.streams.contains_key(&key) {
            return Err(RequestDataError::DuplicateKey {
                symbol: key.symbol().to_owned(),
                timeframe: key.timeframe().value().to_owned(),
            });
        }
        self.streams.insert(key, bars);
        Ok(())
    }

    pub fn from_streams(
        streams: impl IntoIterator<Item = (RequestKey, Vec<Bar>)>,
    ) -> Result<Self, RequestDataError> {
        let mut provider = Self::new();
        for (key, bars) in streams {
            provider.insert(key, bars)?;
        }
        Ok(provider)
    }
}

impl RequestDataProvider for InMemoryRequestDataProvider {
    fn bars<'a>(&'a self, key: &RequestKey) -> Result<&'a [Bar], RequestDataError> {
        self.streams
            .get(key)
            .map(Vec::as_slice)
            .ok_or_else(|| RequestDataError::MissingData {
                symbol: key.symbol().to_owned(),
                timeframe: key.timeframe().value().to_owned(),
            })
    }
}

#[derive(Clone)]
pub struct RequestEnvironment {
    chart: ChartContext,
    main_timeframe: RequestTimeframe,
    provider: Arc<dyn RequestDataProvider>,
    calendars: Arc<BarCalendarRegistry>,
}

impl RequestEnvironment {
    #[must_use]
    pub fn new(chart: ChartContext, provider: Arc<dyn RequestDataProvider>) -> Self {
        let main_timeframe = chart.timeframe().clone();
        Self {
            chart,
            main_timeframe,
            provider,
            calendars: Arc::new(BarCalendarRegistry::default()),
        }
    }

    #[must_use]
    pub fn chart(&self) -> &ChartContext {
        &self.chart
    }

    #[must_use]
    pub fn main_timeframe(&self) -> &RequestTimeframe {
        &self.main_timeframe
    }

    #[must_use]
    pub fn provider(&self) -> &dyn RequestDataProvider {
        self.provider.as_ref()
    }

    #[must_use]
    pub fn with_bar_calendars(mut self, calendars: Arc<BarCalendarRegistry>) -> Self {
        self.calendars = calendars;
        self
    }

    #[must_use]
    pub fn bar_calendars(&self) -> &BarCalendarRegistry {
        &self.calendars
    }

    pub(crate) fn chart_calendar(&self) -> Option<&BarCalendar> {
        self.calendars.get(self.chart.symbol())
    }

    pub(crate) fn chart_timezone(&self) -> &str {
        self.chart_calendar()
            .map_or("Etc/UTC", BarCalendar::timezone)
    }

    pub(crate) fn nominal_close(
        &self,
        symbol: &str,
        timeframe: &RequestTimeframe,
        open: i64,
    ) -> Result<i64, RuntimeError> {
        match self.calendars.get(symbol) {
            Some(calendar) => calendar
                .exact(timeframe, open)
                .map(|interval| interval.close())
                .map_err(|err| RuntimeError {
                    message: err.to_string(),
                }),
            None => Ok(timeframe.nominal_close(open)),
        }
    }

    pub(crate) fn chart_close(&self, open: i64) -> Result<i64, RuntimeError> {
        if self.chart_calendar().is_some() {
            self.nominal_close(self.chart.symbol(), self.chart.timeframe(), open)
        } else {
            crate::builtins::time::chart_timeframe_close(
                open,
                self.chart.timeframe().value(),
                self.chart.timeframe().seconds(),
            )
            .ok_or_else(|| RuntimeError {
                message: format!("time_close timestamp is out of range: {open}"),
            })
        }
    }

    pub(crate) fn previous_chart_open(&self, open: i64) -> Result<i64, RuntimeError> {
        match self.chart_calendar() {
            Some(calendar) => calendar
                .previous_open(self.chart.timeframe(), open)
                .map_err(|err| RuntimeError {
                    message: err.to_string(),
                }),
            None => Ok(self.chart.timeframe().previous_open(open)),
        }
    }

    pub(crate) fn chart_merge_bucket(&self, time: i64) -> Result<Option<i64>, RuntimeError> {
        match self.chart_calendar() {
            Some(calendar) => calendar
                .merge_bucket(self.chart.timeframe(), time)
                .map_err(|err| RuntimeError {
                    message: err.to_string(),
                }),
            None => Ok(crate::builtins::time::timeframe_change_bucket(
                time,
                self.chart.timeframe().value(),
                self.chart.timeframe().seconds(),
            )),
        }
    }

    #[must_use]
    pub fn for_chart(&self, chart: ChartContext) -> Self {
        Self {
            chart,
            main_timeframe: self.main_timeframe.clone(),
            provider: Arc::clone(&self.provider),
            calendars: Arc::clone(&self.calendars),
        }
    }
}

impl Default for RequestEnvironment {
    fn default() -> Self {
        let chart = ChartContext::default();
        let main_timeframe = chart.timeframe().clone();
        Self {
            chart,
            main_timeframe,
            provider: Arc::new(NoRequestDataProvider),
            calendars: Arc::new(BarCalendarRegistry::default()),
        }
    }
}

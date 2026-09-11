use std::collections::HashMap;
use std::fmt;

use crate::{Bar, BarUpdate, BarUpdateKind, RuntimeError};

use super::RequestKey;
use crate::runtime::append_history::AppendHistory;

#[derive(Debug, Clone, Default)]
pub(crate) struct RequestFeed {
    streams: HashMap<RequestKey, RequestStream>,
}

#[derive(Debug, Clone, Default)]
struct RequestStream {
    confirmed: AppendHistory<Bar>,
    forming: Option<Bar>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestFeedError {
    DuplicateBars { time: i64 },
    FormingMismatch { expected: i64, time: i64 },
    FormingOpen { time: i64 },
    Stale { time: i64, last_confirmed: i64 },
}

impl RequestFeed {
    pub(crate) fn contains(&self, key: &RequestKey) -> bool {
        self.streams
            .get(key)
            .is_some_and(|stream| !stream.confirmed.is_empty() || stream.forming.is_some())
    }

    pub(crate) fn has_forming(&self, key: &RequestKey) -> bool {
        self.streams
            .get(key)
            .is_some_and(|stream| stream.forming.is_some())
    }

    pub(crate) fn resolved_len(
        &self,
        key: &RequestKey,
        provider_len: usize,
        include_forming: bool,
    ) -> usize {
        provider_len
            + self.streams.get(key).map_or(0, |stream| {
                stream.confirmed.len() + usize::from(include_forming && stream.forming.is_some())
            })
    }

    pub(crate) fn resolved_from(
        &self,
        key: &RequestKey,
        provider: &[Bar],
        include_forming: bool,
        start: usize,
    ) -> Vec<Bar> {
        let mut bars = provider.get(start..).unwrap_or(&[]).to_vec();
        if let Some(stream) = self.streams.get(key) {
            bars.extend(stream.confirmed.tail(start.saturating_sub(provider.len())));
            if include_forming
                && start <= provider.len() + stream.confirmed.len()
                && let Some(forming) = stream.forming
            {
                bars.push(forming);
            }
        }
        bars
    }

    /// Drop confirmed extras that close after `last_time`. Forming extras at or
    /// before `last_time` are stale; later forming extras stay for the next
    /// chart forming bar.
    pub(crate) fn trim_after(&mut self, last_time: Option<i64>) {
        let Some(last_time) = last_time else {
            self.streams.clear();
            return;
        };
        self.streams.retain(|_, stream| {
            stream.confirmed = AppendHistory::from_values(
                stream
                    .confirmed
                    .iter()
                    .copied()
                    .take_while(|bar| bar.time <= last_time),
            );
            if stream.forming.is_some_and(|bar| bar.time <= last_time) {
                stream.forming = None;
            }
            !stream.confirmed.is_empty() || stream.forming.is_some()
        });
    }

    pub(crate) fn apply(
        &mut self,
        key: RequestKey,
        update: BarUpdate,
        provider_last: Option<i64>,
    ) -> Result<(), RequestFeedError> {
        let last_confirmed = self
            .streams
            .get(&key)
            .and_then(|stream| stream.confirmed.last().map(|bar| bar.time))
            .or(provider_last);
        let stream = self.streams.entry(key).or_default();
        match update.kind {
            BarUpdateKind::Historical => {
                if stream.forming.is_some() {
                    return Err(RequestFeedError::FormingOpen {
                        time: update.bar.time,
                    });
                }
                append_confirmed(&mut stream.confirmed, update.bar, last_confirmed)
            }
            BarUpdateKind::Forming => apply_forming(stream, update.bar, last_confirmed),
            BarUpdateKind::Confirmed => apply_confirmed(stream, update.bar, last_confirmed),
        }
    }
}

fn append_confirmed(
    confirmed: &mut AppendHistory<Bar>,
    bar: Bar,
    last_confirmed: Option<i64>,
) -> Result<(), RequestFeedError> {
    if let Some(last) = last_confirmed {
        if bar.time == last {
            return Err(RequestFeedError::DuplicateBars { time: bar.time });
        }
        if bar.time < last {
            return Err(RequestFeedError::Stale {
                time: bar.time,
                last_confirmed: last,
            });
        }
    }
    confirmed.push(bar);
    Ok(())
}

fn apply_forming(
    stream: &mut RequestStream,
    bar: Bar,
    last_confirmed: Option<i64>,
) -> Result<(), RequestFeedError> {
    if let Some(last) = last_confirmed
        && bar.time <= last
    {
        return Err(RequestFeedError::Stale {
            time: bar.time,
            last_confirmed: last,
        });
    }
    if let Some(forming) = stream.forming
        && forming.time != bar.time
    {
        return Err(RequestFeedError::FormingMismatch {
            expected: forming.time,
            time: bar.time,
        });
    }
    stream.forming = Some(bar);
    Ok(())
}

fn apply_confirmed(
    stream: &mut RequestStream,
    bar: Bar,
    last_confirmed: Option<i64>,
) -> Result<(), RequestFeedError> {
    if let Some(forming) = stream.forming {
        if forming.time != bar.time {
            return Err(RequestFeedError::FormingMismatch {
                expected: forming.time,
                time: bar.time,
            });
        }
        stream.forming = None;
        return append_confirmed(&mut stream.confirmed, bar, last_confirmed);
    }
    append_confirmed(&mut stream.confirmed, bar, last_confirmed)
}

impl RequestFeedError {
    pub(crate) fn runtime_error(self) -> RuntimeError {
        RuntimeError {
            message: self.to_string(),
        }
    }
}

impl fmt::Display for RequestFeedError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateBars { time } => {
                write!(
                    formatter,
                    "E_REQUEST_FEED_TIME: duplicate requested bar time `{time}`"
                )
            }
            Self::FormingMismatch { expected, time } => write!(
                formatter,
                "E_REQUEST_FEED_FORMING: requested bar `{time}` does not match forming time `{expected}`"
            ),
            Self::FormingOpen { time } => write!(
                formatter,
                "E_REQUEST_FEED_FORMING: cannot append historical requested bar `{time}` while a forming request bar is open"
            ),
            Self::Stale {
                time,
                last_confirmed,
            } => write!(
                formatter,
                "E_REQUEST_FEED_TIME: requested bar `{time}` must be later than confirmed time `{last_confirmed}`"
            ),
        }
    }
}

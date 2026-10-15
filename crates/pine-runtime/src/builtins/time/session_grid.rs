//! Pine session strings define virtual bars independently of exchange bars.
//! Keep only session spans; minute/second bars are selected arithmetically.
use chrono::{Datelike, NaiveDate, Utc};

use super::{
    session::{TimeSession, TimeSessionTimezone},
    timeframe_bucket_bounds, timeframe_change_bucket, utc_datetime_from_millis,
};
use crate::{RequestTimeframe, RuntimeError};

#[derive(Clone, Copy)]
struct Span {
    open: i64,
    close: i64,
}

fn range_error() -> RuntimeError {
    RuntimeError {
        message: "time virtual-session timestamp is out of range".into(),
    }
}

struct Grid<'a, F> {
    session: &'a TimeSession,
    timezone: TimeSessionTimezone,
    group: &'a str,
    group_seconds: i64,
    stride: i64,
    aggregate: bool,
    nominal_calendar: bool,
    charge: F,
}

impl<F: FnMut() -> Result<(), RuntimeError>> Grid<'_, F> {
    fn raw_spans(&mut self, bucket: i64) -> Result<Vec<Span>, RuntimeError> {
        (self.charge)()?;
        let (open, close) = timeframe_bucket_bounds(bucket, self.group, self.group_seconds)
            .ok_or_else(range_error)?;
        let mut day = utc_datetime_from_millis(open)?.date_naive();
        let end = utc_datetime_from_millis(close)?.date_naive();
        let mut spans: Vec<Span> = Vec::new();
        while day < end {
            (self.charge)()?;
            if self.session.days[day.weekday().num_days_from_sunday() as usize + 1] {
                for period in &self.session.periods {
                    (self.charge)()?;
                    let midnight = local_midnight(day)?;
                    let mut local_open = midnight
                        .checked_add(period.start_minute * 60_000)
                        .ok_or_else(range_error)?;
                    let mut local_close = midnight
                        .checked_add(period.end_minute * 60_000)
                        .ok_or_else(range_error)?;
                    if period.start_minute == 0 && period.end_minute == 0 {
                        local_close = local_close
                            .checked_add(86_400_000)
                            .ok_or_else(range_error)?;
                    } else if period.start_minute >= period.end_minute {
                        local_open = local_open.checked_sub(86_400_000).ok_or_else(range_error)?;
                    }
                    let span = Span {
                        open: self.timezone.open_utc_millis(local_open)?,
                        close: self.timezone.close_utc_millis(local_close)?,
                    };
                    if span.open < span.close {
                        spans.push(span);
                    }
                }
            }
            day = day.succ_opt().ok_or_else(range_error)?;
        }
        spans.sort_by_key(|span| span.open);
        let mut merged: Vec<Span> = Vec::new();
        for span in spans {
            if let Some(last) = merged.last_mut().filter(|last| last.close > span.open) {
                last.close = last.close.max(span.close);
            } else {
                merged.push(span);
            }
        }
        Ok(merged)
    }

    fn spans(&mut self, bucket: i64) -> Result<Vec<Span>, RuntimeError> {
        let merged = self.raw_spans(bucket)?;
        if self.aggregate
            && let (Some(first), Some(last)) = (merged.first(), merged.last())
        {
            let mut close = last.close;
            // Native W/M queries on intraday charts expose the next virtual
            // period's opening; D/W/M charts expose the last session close.
            if self.nominal_calendar {
                let mut next = bucket;
                loop {
                    next = next.checked_add(1).ok_or_else(range_error)?;
                    if let Some(first) = self.raw_spans(next)?.first() {
                        close = first.open;
                        break;
                    }
                }
            }
            return Ok(vec![Span {
                open: first.open,
                close,
            }]);
        }
        Ok(merged)
    }

    fn count(&self, span: Span) -> i64 {
        if self.aggregate {
            1
        } else {
            (span.close - span.open - 1) / self.stride + 1
        }
    }

    fn select(
        &mut self,
        time: i64,
        offset: i64,
        whole_session: bool,
    ) -> Result<Option<(i64, i64)>, RuntimeError> {
        offset.checked_mul(self.stride).ok_or_else(range_error)?;
        let datetime = utc_datetime_from_millis(time)?;
        let local_time = time
            .checked_add(i64::from(self.timezone.offset_seconds_at(&datetime)) * 1000)
            .ok_or_else(range_error)?;
        let anchor = timeframe_change_bucket(local_time, self.group, self.group_seconds)
            .ok_or_else(range_error)?;
        let mut candidates = Vec::new();
        for delta in [-1, 0, 1] {
            let bucket = anchor.checked_add(delta).ok_or_else(range_error)?;
            for (index, span) in self.spans(bucket)?.into_iter().enumerate() {
                candidates.push((bucket, index, span));
            }
        }
        candidates.sort_by_key(|(_, _, span)| span.open);
        let mut contains = candidates
            .iter()
            .find(|(_, _, span)| span.open <= time && time < span.close)
            .copied();
        if contains.is_none() && offset == 0 {
            return Ok(None);
        }
        let shift_to_future =
            self.nominal_calendar && offset != 0 && !self.in_session(local_time, time)?;
        if shift_to_future {
            contains = None;
        }
        let direction = if shift_to_future || offset <= 0 {
            1
        } else {
            -1
        };
        let initial = contains.or_else(|| {
            if direction < 0 {
                candidates
                    .iter()
                    .rev()
                    .find(|(_, _, span)| span.close <= time)
                    .copied()
            } else {
                candidates
                    .iter()
                    .find(|(_, _, span)| span.open > time)
                    .copied()
            }
        });
        let (mut bucket, mut index, mut span) = if let Some(initial) = initial {
            initial
        } else {
            let mut bucket = anchor.checked_add(2 * direction).ok_or_else(range_error)?;
            loop {
                let spans = self.spans(bucket)?;
                if !spans.is_empty() {
                    let index = if direction < 0 { spans.len() - 1 } else { 0 };
                    break (bucket, index, spans[index]);
                }
                bucket = bucket.checked_add(direction).ok_or_else(range_error)?;
            }
        };
        // Outside a session, native queries anchor to the nearest bar in the
        // offset's direction, then apply the entire offset (not offset - 1).
        let start = if contains.is_some() {
            if self.aggregate {
                0
            } else {
                (time - span.open) / self.stride
            }
        } else if direction < 0 {
            self.count(span) - 1
        } else {
            0
        };
        let mut target = i128::from(start) - i128::from(offset);
        let mut spans = self.spans(bucket)?;
        loop {
            let count = i128::from(self.count(span));
            if target >= 0 && target < count {
                break;
            }
            let direction = if target < 0 { -1 } else { 1 };
            if direction > 0 {
                target -= count;
            }
            if direction < 0 && index > 0 {
                index -= 1;
            } else if direction > 0 && index + 1 < spans.len() {
                index += 1;
            } else {
                loop {
                    bucket = bucket.checked_add(direction).ok_or_else(range_error)?;
                    spans = self.spans(bucket)?;
                    if !spans.is_empty() {
                        break;
                    }
                }
                index = if direction < 0 { spans.len() - 1 } else { 0 };
            }
            span = spans[index];
            if direction < 0 {
                target += i128::from(self.count(span));
            }
        }
        if self.aggregate || whole_session {
            return Ok(Some((span.open, span.close)));
        }
        let selected_open = span
            .open
            .checked_add(
                i64::try_from(target)
                    .map_err(|_| range_error())?
                    .checked_mul(self.stride)
                    .ok_or_else(range_error)?,
            )
            .ok_or_else(range_error)?;
        let selected_close = selected_open
            .checked_add(self.stride)
            .ok_or_else(range_error)?
            .min(span.close);
        Ok(Some((selected_open, selected_close)))
    }

    fn in_session(&mut self, local_time: i64, time: i64) -> Result<bool, RuntimeError> {
        let mut daily = Grid {
            session: self.session,
            timezone: self.timezone,
            group: "D",
            group_seconds: 86_400,
            stride: self.stride,
            aggregate: false,
            nominal_calendar: false,
            charge: &mut self.charge,
        };
        let anchor = timeframe_change_bucket(local_time, "D", 86_400).ok_or_else(range_error)?;
        for delta in [-1, 0, 1] {
            if daily
                .raw_spans(anchor.checked_add(delta).ok_or_else(range_error)?)?
                .iter()
                .any(|span| span.open <= time && time < span.close)
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

fn local_midnight(day: NaiveDate) -> Result<i64, RuntimeError> {
    Ok(day
        .and_hms_opt(0, 0, 0)
        .ok_or_else(range_error)?
        .and_local_timezone(Utc)
        .single()
        .ok_or_else(range_error)?
        .timestamp_millis())
}

pub(super) fn virtual_session_time(
    time: i64,
    timeframe: &RequestTimeframe,
    chart_seconds: i64,
    session: &TimeSession,
    timezone: TimeSessionTimezone,
    offset: i64,
    charge: impl FnMut() -> Result<(), RuntimeError>,
) -> Result<Option<(i64, i64)>, RuntimeError> {
    let seconds = timeframe.seconds();
    let aggregate = seconds >= 86_400;
    let mut grid = Grid {
        session,
        timezone,
        group: if aggregate { timeframe.value() } else { "D" },
        group_seconds: if aggregate { seconds } else { 86_400 },
        stride: seconds.checked_mul(1000).ok_or_else(range_error)?,
        aggregate,
        nominal_calendar: chart_seconds < 86_400 && timeframe.value().ends_with(['W', 'M']),
        charge,
    };
    grid.select(time, offset, !aggregate && chart_seconds >= 86_400)
}

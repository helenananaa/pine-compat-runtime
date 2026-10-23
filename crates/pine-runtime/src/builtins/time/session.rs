use chrono::{DateTime, Duration, LocalResult, Offset, TimeZone, Utc};
use chrono_tz::Tz;

use crate::RuntimeError;

use super::{parse_fixed_timezone_offset, utc_datetime_from_millis};

pub(super) struct TimeSession {
    pub(super) periods: Vec<TimeSessionPeriod>,
    pub(super) days: [bool; 8],
}

#[derive(Clone, Copy)]
pub(super) enum DefaultSessionDays {
    Weekdays,
    All,
}

impl DefaultSessionDays {
    fn values(self) -> [bool; 8] {
        match self {
            Self::Weekdays => [false, false, true, true, true, true, true, false],
            Self::All => all_session_days(),
        }
    }
}

pub(super) struct TimeSessionPeriod {
    pub(super) start_minute: i64,
    pub(super) end_minute: i64,
}

#[derive(Clone, Copy)]
pub(super) enum TimeSessionTimezone {
    Fixed(i32),
    Iana(Tz),
}

pub(super) fn parse_time_session_timezone(timezone: &str) -> Option<TimeSessionTimezone> {
    if let Some(offset) = parse_fixed_timezone_offset(timezone) {
        return Some(TimeSessionTimezone::Fixed(offset));
    }
    Some(TimeSessionTimezone::Iana(timezone.trim().parse().ok()?))
}

impl TimeSessionTimezone {
    pub(super) fn offset_seconds_at(self, datetime: &DateTime<Utc>) -> i32 {
        match self {
            Self::Fixed(offset) => offset,
            Self::Iana(timezone) => timezone
                .offset_from_utc_datetime(&datetime.naive_utc())
                .fix()
                .local_minus_utc(),
        }
    }

    pub(super) fn close_utc_millis(self, local_close_time: i64) -> Result<i64, RuntimeError> {
        match self {
            Self::Fixed(offset) => {
                local_close_time
                    .checked_sub(i64::from(offset).checked_mul(1000).ok_or_else(|| {
                        RuntimeError {
                            message: "time session timestamp is out of range".to_owned(),
                        }
                    })?)
                    .ok_or_else(|| RuntimeError {
                        message: "time session timestamp is out of range".to_owned(),
                    })
            }
            Self::Iana(timezone) => {
                let local_close = utc_datetime_from_millis(local_close_time)?.naive_utc();
                resolve_iana_session_close(timezone, local_close).ok_or_else(|| RuntimeError {
                    message: "time session timestamp is out of range".to_owned(),
                })
            }
        }
    }

    pub(super) fn open_utc_millis(self, local_time: i64) -> Result<i64, RuntimeError> {
        match self {
            Self::Fixed(_) => self.close_utc_millis(local_time),
            Self::Iana(timezone) => {
                let local = utc_datetime_from_millis(local_time)?.naive_utc();
                for minutes in 0..=2 * 24 * 60 {
                    let shifted = local.checked_add_signed(Duration::minutes(minutes));
                    let Some(shifted) = shifted else { break };
                    let resolved = match timezone.from_local_datetime(&shifted) {
                        LocalResult::Single(value) => Some(value.timestamp_millis()),
                        LocalResult::Ambiguous(a, b) => {
                            Some(a.timestamp_millis().min(b.timestamp_millis()))
                        }
                        LocalResult::None => None,
                    };
                    if let Some(value) = resolved {
                        return Ok(value);
                    }
                }
                Err(RuntimeError {
                    message: "time session timestamp is out of range".into(),
                })
            }
        }
    }
}

pub(super) fn parse_time_session(
    session: &str,
    default_days: DefaultSessionDays,
) -> Option<TimeSession> {
    if session == "24x7" {
        return Some(TimeSession {
            periods: vec![TimeSessionPeriod {
                start_minute: 0,
                end_minute: 0,
            }],
            days: all_session_days(),
        });
    }

    let (periods, days) = match session.split_once(':') {
        Some((periods, days)) => (periods, parse_session_days(days)?),
        None => (session, default_days.values()),
    };
    if periods.is_empty() {
        return None;
    }
    let periods = periods
        .split(',')
        .map(parse_session_period)
        .collect::<Option<Vec<_>>>()?;
    if periods.is_empty() {
        return None;
    }

    Some(TimeSession { periods, days })
}

fn resolve_iana_session_close(timezone: Tz, local_close: chrono::NaiveDateTime) -> Option<i64> {
    const MAX_GAP_MINUTES: i64 = 2 * 24 * 60;

    // A repeated close uses the later instant so the whole repeated wall-clock
    // interval remains in-session. A close inside a forward gap advances to
    // the first valid local minute, matching minute-granularity session input.
    if let Some(timestamp) = resolved_local_timestamp(timezone.from_local_datetime(&local_close)) {
        return Some(timestamp);
    }

    for minutes in 1..=MAX_GAP_MINUTES {
        let shifted = local_close.checked_add_signed(Duration::minutes(minutes))?;
        if let Some(timestamp) = resolved_local_timestamp(timezone.from_local_datetime(&shifted)) {
            return Some(timestamp);
        }
    }
    None
}

fn resolved_local_timestamp(result: LocalResult<DateTime<Tz>>) -> Option<i64> {
    match result {
        LocalResult::Single(datetime) => Some(datetime.timestamp_millis()),
        LocalResult::Ambiguous(first, second) => {
            Some(first.timestamp_millis().max(second.timestamp_millis()))
        }
        LocalResult::None => None,
    }
}

fn all_session_days() -> [bool; 8] {
    [false, true, true, true, true, true, true, true]
}

fn parse_session_days(days: &str) -> Option<[bool; 8]> {
    if days.is_empty() {
        return None;
    }
    let mut parsed = [false; 8];
    for ch in days.chars() {
        let day = ch.to_digit(10)?;
        if !(1..=7).contains(&day) {
            return None;
        }
        parsed[day as usize] = true;
    }
    Some(parsed)
}

fn parse_session_period(period: &str) -> Option<TimeSessionPeriod> {
    let (start, end) = period.split_once('-')?;
    Some(TimeSessionPeriod {
        start_minute: parse_session_hhmm(start)?,
        end_minute: parse_session_hhmm(end)?,
    })
}

fn parse_session_hhmm(value: &str) -> Option<i64> {
    if value == "2400" {
        return Some(0);
    }
    if value.len() != 4 || !value.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    let hour = value[..2].parse::<i64>().ok()?;
    let minute = value[2..].parse::<i64>().ok()?;
    if !(0..=23).contains(&hour) || !(0..=59).contains(&minute) {
        return None;
    }
    Some(hour * 60 + minute)
}

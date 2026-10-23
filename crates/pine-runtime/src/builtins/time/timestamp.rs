use chrono::{DateTime, Duration, LocalResult, NaiveDate, NaiveDateTime, Offset, TimeZone, Utc};
use chrono_tz::Tz;

use super::is_supported_utc_timezone;

#[derive(Clone, Copy)]
pub(super) enum NumericTimestampTimezone {
    Fixed(i32),
    Iana(Tz),
}

pub(super) fn parse_numeric_timestamp_timezone(timezone: &str) -> Option<NumericTimestampTimezone> {
    if let Some(offset) = parse_fixed_timezone_offset(timezone) {
        return Some(NumericTimestampTimezone::Fixed(offset));
    }
    Some(NumericTimestampTimezone::Iana(
        timezone.trim().parse().ok()?,
    ))
}

pub(super) fn numeric_timestamp_millis(
    timezone: NumericTimestampTimezone,
    local_datetime: DateTime<Utc>,
) -> Option<i64> {
    match timezone {
        NumericTimestampTimezone::Fixed(offset) => local_datetime
            .checked_sub_signed(Duration::try_seconds(i64::from(offset))?)
            .map(|datetime| datetime.timestamp_millis()),
        NumericTimestampTimezone::Iana(timezone) => {
            resolve_iana_numeric_timestamp(timezone, local_datetime.naive_utc())
        }
    }
}

pub(super) fn numeric_timestamp_parts_millis(
    timezone: NumericTimestampTimezone,
    parts: [i64; 6],
) -> Option<i64> {
    let [year, month, day, hour, minute, second] = parts;
    match timezone {
        NumericTimestampTimezone::Fixed(offset) => {
            // Proleptic Gregorian days from the Unix epoch. Work in i128 so
            // normalizing overflowing month/day/time fields cannot overflow
            // before the final checked millisecond conversion.
            let months = i128::from(year) * 12 + i128::from(month) - 1;
            let year = months.div_euclid(12);
            let month = months.rem_euclid(12) + 1;
            let year = year - i128::from(month <= 2);
            let era = year.div_euclid(400);
            let year_of_era = year - era * 400;
            let march_month = month + if month > 2 { -3 } else { 9 };
            let day_of_year = (153 * march_month + 2) / 5;
            let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
            let days = era * 146_097 + day_of_era - 719_468 + i128::from(day) - 1;
            let seconds = days * 86_400
                + i128::from(hour) * 3_600
                + i128::from(minute) * 60
                + i128::from(second)
                - i128::from(offset);
            i64::try_from(seconds * 1_000).ok()
        }
        NumericTimestampTimezone::Iana(_) => numeric_timestamp_millis(
            timezone,
            super::normalize_timestamp_parts(year, month, day, hour, minute, second)?,
        ),
    }
}

fn resolve_iana_numeric_timestamp(timezone: Tz, local: chrono::NaiveDateTime) -> Option<i64> {
    const MAX_GAP_MINUTES: i64 = 2 * 24 * 60;

    // Use compatible local-time resolution: overlaps select the earlier
    // absolute instant, while gaps shift the input forward by the offset jump.
    match timezone.from_local_datetime(&local) {
        LocalResult::Single(datetime) => return Some(datetime.timestamp_millis()),
        LocalResult::Ambiguous(first, second) => {
            return Some(first.timestamp_millis().min(second.timestamp_millis()));
        }
        LocalResult::None => {}
    }

    let before_offset = nearest_iana_offset(timezone, local, -1, MAX_GAP_MINUTES)?;
    let after_offset = nearest_iana_offset(timezone, local, 1, MAX_GAP_MINUTES)?;
    let gap_seconds = after_offset.checked_sub(before_offset)?;
    if gap_seconds <= 0 {
        return None;
    }
    let shifted = local.checked_add_signed(Duration::seconds(i64::from(gap_seconds)))?;
    match timezone.from_local_datetime(&shifted) {
        LocalResult::Single(datetime) => Some(datetime.timestamp_millis()),
        LocalResult::Ambiguous(first, second) => {
            Some(first.timestamp_millis().min(second.timestamp_millis()))
        }
        LocalResult::None => None,
    }
}

fn nearest_iana_offset(
    timezone: Tz,
    local: chrono::NaiveDateTime,
    direction: i64,
    max_minutes: i64,
) -> Option<i32> {
    for minutes in 1..=max_minutes {
        let delta = Duration::minutes(direction.checked_mul(minutes)?);
        let candidate = local.checked_add_signed(delta)?;
        match timezone.from_local_datetime(&candidate) {
            LocalResult::Single(datetime) => {
                return Some(datetime.offset().fix().local_minus_utc());
            }
            LocalResult::Ambiguous(first, second) => {
                let first = first.offset().fix().local_minus_utc();
                let second = second.offset().fix().local_minus_utc();
                return Some(if direction < 0 {
                    first.max(second)
                } else {
                    first.min(second)
                });
            }
            LocalResult::None => {}
        }
    }
    None
}

pub(super) fn parse_timestamp_date_string(date_string: &str) -> Result<i64, String> {
    let value = date_string.trim();
    if value.is_empty() {
        return Err("timestamp unsupported dateString ``".to_owned());
    }
    let separated = (value.as_bytes().get(10) == Some(&b':')
        && parse_iso_date_token(&value[..10]).is_some())
    .then(|| format!("{} {}", &value[..10], &value[11..]));
    let value = separated.as_deref().unwrap_or(value);

    if value.as_bytes().get(10) == Some(&b'T') {
        for format in ["%Y-%m-%dT%H:%M:%S%:z", "%Y-%m-%dT%H:%M%:z"] {
            if let Ok(datetime) = DateTime::parse_from_str(value, format) {
                return Ok(datetime.timestamp_millis());
            }
        }
        for format in ["%Y-%m-%dT%H:%M:%S", "%Y-%m-%dT%H:%M"] {
            if let Ok(datetime) = NaiveDateTime::parse_from_str(value, format) {
                return Ok(datetime.and_utc().timestamp_millis());
            }
        }
        return Err(format!("timestamp invalid dateString `{date_string}`"));
    }

    let mut tokens: Vec<&str> = value.split_whitespace().collect();
    if tokens.len() > 1 && parse_date_string_timezone(tokens[0]).is_some() {
        let timezone = tokens.remove(0);
        tokens.push(timezone);
    }
    let compact = tokens
        .first()
        .is_some_and(|token| parse_compact_day_month(token).is_some());
    let (year, month, day, next_index) = parse_timestamp_date_tokens(&tokens, value)?;
    let (hour, minute, second, next_index) = match tokens.get(next_index) {
        Some(token) if token.contains(':') => {
            let (hour, minute, second) = parse_timestamp_time_token(token, value)?;
            (hour, minute, second, next_index + 1)
        }
        _ => (0, 0, 0, next_index),
    };
    let timezone = match tokens.get(next_index) {
        Some(timezone) => parse_date_string_timezone(timezone)
            .ok_or_else(|| format!("timestamp unsupported dateString `{value}`"))?,
        None => NumericTimestampTimezone::Fixed(0),
    };
    if tokens.len() > next_index + usize::from(tokens.get(next_index).is_some()) {
        return Err(format!("timestamp unsupported dateString `{date_string}`"));
    }
    // Native compact English dates retain seconds but ignore numeric/GMT
    // suffix offsets. Their named IANA suffix is rejected, unlike the
    // separate spaced-date profile. Inserting a space changes semantics.
    let timezone = if compact {
        match timezone {
            NumericTimestampTimezone::Fixed(_) => NumericTimestampTimezone::Fixed(0),
            NumericTimestampTimezone::Iana(_) => {
                return Err(format!("timestamp unsupported dateString `{date_string}`"));
            }
        }
    } else {
        timezone
    };

    let Some(date) = NaiveDate::from_ymd_opt(year, month, day) else {
        return Err(format!("timestamp invalid dateString `{date_string}`"));
    };
    let Some(datetime) = date.and_hms_opt(hour, minute, second) else {
        return Err(format!("timestamp invalid dateString `{date_string}`"));
    };
    numeric_timestamp_millis(timezone, Utc.from_utc_datetime(&datetime))
        .ok_or_else(|| format!("timestamp invalid dateString `{date_string}`"))
}

fn parse_date_string_timezone(timezone: &str) -> Option<NumericTimestampTimezone> {
    parse_numeric_timestamp_timezone(timezone).or_else(|| {
        // RFC-style four-digit dateString offsets can exceed the numeric
        // timezone argument's 23-hour range (e.g. the official -3000 form).
        (timezone.len() == 5)
            .then(|| parse_timestamp_numeric_offset_limit(timezone, 99))
            .flatten()
            .map(NumericTimestampTimezone::Fixed)
    })
}

fn parse_timestamp_date_tokens(
    tokens: &[&str],
    original: &str,
) -> Result<(i32, u32, u32, usize), String> {
    let Some(first) = tokens.first() else {
        return Err(format!("timestamp unsupported dateString `{original}`"));
    };
    if let Some((year, month, day)) = parse_iso_date_token(first) {
        return Ok((year, month, day, 1));
    }
    if let Some((day, month)) = parse_compact_day_month(first) {
        let year = tokens
            .get(1)
            .and_then(|year| year.parse::<i32>().ok())
            .ok_or_else(|| format!("timestamp unsupported dateString `{original}`"))?;
        return Ok((year, month, day, 2));
    }
    if tokens.len() < 3 {
        return Err(format!("timestamp unsupported dateString `{original}`"));
    }
    if let (Ok(year), Ok(month), Ok(day)) = (
        tokens[0].parse::<i32>(),
        tokens[1].parse::<u32>(),
        tokens[2].parse::<u32>(),
    ) {
        return Ok((year, month, day, 3));
    }
    if let Some(month) = parse_english_month(tokens[0]) {
        let day = tokens[1]
            .parse::<u32>()
            .map_err(|_| format!("timestamp unsupported dateString `{original}`"))?;
        let year = tokens[2]
            .parse::<i32>()
            .map_err(|_| format!("timestamp unsupported dateString `{original}`"))?;
        return Ok((year, month, day, 3));
    }
    let day = tokens[0]
        .parse::<u32>()
        .map_err(|_| format!("timestamp unsupported dateString `{original}`"))?;
    let month = parse_english_month(tokens[1])
        .ok_or_else(|| format!("timestamp unsupported dateString `{original}`"))?;
    let year = tokens[2]
        .parse::<i32>()
        .map_err(|_| format!("timestamp unsupported dateString `{original}`"))?;
    Ok((year, month, day, 3))
}

fn parse_iso_date_token(token: &str) -> Option<(i32, u32, u32)> {
    let mut parts = token.split('-');
    let year = parts.next()?.parse::<i32>().ok()?;
    let month = parts.next()?.parse::<u32>().ok()?;
    let day = parts.next()?.parse::<u32>().ok()?;
    parts.next().is_none().then_some((year, month, day))
}

fn parse_compact_day_month(token: &str) -> Option<(u32, u32)> {
    let split = token.find(|ch: char| !ch.is_ascii_digit())?;
    if split == 0 {
        return None;
    }
    Some((
        token[..split].parse().ok()?,
        parse_english_month(&token[split..])?,
    ))
}

fn parse_english_month(month: &str) -> Option<u32> {
    match month.to_ascii_lowercase().as_str() {
        "jan" | "january" => Some(1),
        "feb" | "february" => Some(2),
        "mar" | "march" => Some(3),
        "apr" | "april" => Some(4),
        "may" => Some(5),
        "jun" | "june" => Some(6),
        "jul" | "july" => Some(7),
        "aug" | "august" => Some(8),
        "sep" | "sept" | "september" => Some(9),
        "oct" | "october" => Some(10),
        "nov" | "november" => Some(11),
        "dec" | "december" => Some(12),
        _ => None,
    }
}

fn parse_timestamp_time_token(token: &str, original: &str) -> Result<(u32, u32, u32), String> {
    let parts: Vec<&str> = token.split(':').collect();
    if !(2..=3).contains(&parts.len()) {
        return Err(format!("timestamp unsupported dateString `{original}`"));
    }
    let hour = parts[0]
        .parse::<u32>()
        .map_err(|_| format!("timestamp unsupported dateString `{original}`"))?;
    let minute = parts[1]
        .parse::<u32>()
        .map_err(|_| format!("timestamp unsupported dateString `{original}`"))?;
    let second = parts
        .get(2)
        .map_or(Ok(0), |value| value.parse::<u32>())
        .map_err(|_| format!("timestamp unsupported dateString `{original}`"))?;
    Ok((hour, minute, second))
}

pub(crate) fn parse_fixed_timezone_offset(timezone: &str) -> Option<i32> {
    let token = timezone.trim();
    if is_supported_utc_timezone(token) {
        return Some(0);
    }
    let offset = token
        .strip_prefix("UTC")
        .or_else(|| token.strip_prefix("GMT"))
        .unwrap_or(token);
    if offset == "0" {
        return Some(0);
    }
    parse_timestamp_numeric_offset(offset)
}

fn parse_timestamp_numeric_offset(offset: &str) -> Option<i32> {
    parse_timestamp_numeric_offset_limit(offset, 23)
}

fn parse_timestamp_numeric_offset_limit(offset: &str, max_hour: i32) -> Option<i32> {
    let sign = match offset.as_bytes().first()? {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    let body = &offset[1..];
    if !body.is_ascii() {
        return None;
    }
    let (hour, minute) = if let Some((hour, minute)) = body.split_once(':') {
        (hour.parse::<i32>().ok()?, minute.parse::<i32>().ok()?)
    } else if body.len() <= 2 {
        (body.parse::<i32>().ok()?, 0)
    } else if body.len() == 4 {
        (
            body[..2].parse::<i32>().ok()?,
            body[2..].parse::<i32>().ok()?,
        )
    } else {
        return None;
    };
    if !(0..=max_hour).contains(&hour) || !(0..=59).contains(&minute) {
        return None;
    }
    Some(sign * (hour * 3600 + minute * 60))
}

pub(crate) fn format_fixed_timezone_offset(offset_seconds: i32) -> String {
    let sign = if offset_seconds < 0 { '-' } else { '+' };
    let offset_seconds = offset_seconds.abs();
    let hour = offset_seconds / 3600;
    let minute = (offset_seconds % 3600) / 60;
    format!("{sign}{hour:02}{minute:02}")
}

#[cfg(test)]
mod iso_tests {
    use super::parse_timestamp_date_string;

    #[test]
    fn compact_english_dates_match_native_offset_and_seconds_controls() {
        for (text, expected) in [
            ("15Aug 2022 14:00 +0530", 1_660_572_000_000),
            ("15Aug 2022 14:00 GMT+0530", 1_660_572_000_000),
            ("15 Aug 2022 14:00 +0530", 1_660_552_200_000),
            ("1Jan 2024 05:30:01 +0530", 1_704_087_001_000),
            ("15Aug 2022", 1_660_521_600_000),
            ("15Aug 2022 14:00:30 +0000", 1_660_572_030_000),
            ("15Aug 2022 14:00 -0400", 1_660_572_000_000),
            ("15August 2022 14:00 +0000", 1_660_572_000_000),
            ("15aug 2022 14:00", 1_660_572_000_000),
        ] {
            assert_eq!(parse_timestamp_date_string(text), Ok(expected), "{text}");
        }
        for text in [
            "15Aug 2022 14:00 Asia/Kolkata",
            "15Aug 2022 14:00 junk",
            "30Feb 2022 14:00",
            "15Aug",
            "15月 2022 14:00",
        ] {
            assert!(parse_timestamp_date_string(text).is_err(), "{text}");
        }
    }

    #[test]
    fn month_first_english_dates_preserve_seconds_and_offsets() {
        assert_eq!(
            parse_timestamp_date_string("Jan 19 2024 00:00:01"),
            Ok(1_705_622_401_000)
        );
        assert_eq!(
            parse_timestamp_date_string("January 19 2024 05:30:01 +0530"),
            Ok(1_705_622_401_000)
        );
        assert!(parse_timestamp_date_string("Feb 30 2024 00:00").is_err());
        assert!(parse_timestamp_date_string("Jan 19 2024 00:00 junk").is_err());
        assert_eq!(
            parse_timestamp_date_string("2021-01-13:05:00"),
            Ok(1_610_514_000_000)
        );
        assert_eq!(
            parse_timestamp_date_string("01 Jan 2023 00:00 -3000"),
            Ok(1_672_639_200_000)
        );
        assert!(parse_timestamp_date_string("01 Jan 2023 00:00 -3060").is_err());
        assert!(super::parse_numeric_timestamp_timezone("-3000").is_none());
    }

    #[test]
    fn fixed_timestamp_calendar_matches_chrono_normalization_and_large_year_oracle() {
        use super::{NumericTimestampTimezone, numeric_timestamp_parts_millis};
        for year in [-10000, -400, -1, 0, 1, 1600, 1900, 1970, 2000, 2100, 10000] {
            for month in [-13, 0, 1, 2, 3, 12, 14, 25] {
                for day in [-5, 0, 1, 29, 32, 60] {
                    let expected = crate::builtins::time::normalize_timestamp_parts(
                        year, month, day, -25, 61, -1,
                    )
                    .unwrap()
                    .timestamp_millis();
                    assert_eq!(
                        numeric_timestamp_parts_millis(
                            NumericTimestampTimezone::Fixed(0),
                            [year, month, day, -25, 61, -1]
                        ),
                        Some(expected)
                    );
                }
            }
        }
        assert_eq!(
            numeric_timestamp_parts_millis(
                NumericTimestampTimezone::Fixed(0),
                [999999, 12, 31, 23, 59, 0]
            ),
            Some(31_494_784_780_740_000)
        );
        assert_eq!(
            numeric_timestamp_parts_millis(
                NumericTimestampTimezone::Fixed(19800),
                [999999, 12, 31, 23, 59, 0]
            ),
            Some(31_494_784_760_940_000)
        );
        assert!(
            numeric_timestamp_parts_millis(
                NumericTimestampTimezone::Fixed(0),
                [i64::MAX, 1, 1, 0, 0, 0]
            )
            .is_none()
        );
    }

    #[test]
    fn iso_timestamp_offsets_and_default_utc_match() {
        for value in [
            "2021-01-01T00:00:00",
            "2021-01-01T00:00",
            "2021-01-01T00:00+00:00",
            "2021-01-01T08:00:00+08:00",
            "2021 01 01",
            "UTC 01 Jan 2021",
            "GMT+08:00 01 Jan 2021 08:00",
        ] {
            assert_eq!(
                parse_timestamp_date_string(value),
                Ok(1_609_459_200_000),
                "{value}"
            );
        }
        for value in [
            "2021-02-29T00:00:00",
            "2021-01-01T25:00:00",
            "2021-01-01T00:00:00junk",
            "2021-01-01T00:00:00Z",
        ] {
            assert!(parse_timestamp_date_string(value).is_err(), "{value}");
        }
    }
}

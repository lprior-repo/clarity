#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use super::helpers::{get_days_in_month, is_leap_year};
use super::{FormatError, Iso8601Error};

// ============================================================================
// ISO 8601 DATE/TIME FORMAT CONSTANTS
// ============================================================================

// ---- Date-related constants ----

/// Expected length of an ISO 8601 date string (YYYY-MM-DD).
const DATE_LENGTH: usize = 10;

/// Character position of the first hyphen in a date (between year and month).
const DATE_FIRST_HYPHEN_POSITION: usize = 4;

/// Character position of the second hyphen in a date (between month and day).
const DATE_SECOND_HYPHEN_POSITION: usize = 7;

// ---- Time-related constants ----

/// Minimum length for a time string with timezone (HH:MMZ or HH:MM+ZZ).
const MIN_TIME_LENGTH: usize = 8;

/// Maximum value for hours (0-23 in 24-hour format).
const MAX_HOUR: u8 = 23;

/// Maximum value for minutes (0-59).
const MAX_MINUTE: u8 = 59;

/// Maximum value for seconds (0-60, allowing for leap seconds).
const MAX_SECOND: u8 = 60;

/// Character position of the time separator (colon) between hours and minutes.
const TIME_HOUR_COLON_POSITION: usize = 2;

/// Character position of the second colon (between minutes and seconds) if present.
const TIME_SECOND_COLON_POSITION: usize = 5;

/// Character position where seconds end (and timezone or fractional seconds begin).
const TIME_SECONDS_END_POSITION: usize = 8;

// ---- Datetime-related constants ----

/// Character position of the date-time separator (T or space) in a datetime string.
const DATETIME_SEPARATOR_POSITION: usize = 10;

// ---- Timezone-related constants ----

/// Length of timezone with colon separator (e.g., +05:30).
const TIMEZONE_LENGTH_WITH_COLON: usize = 5;

/// Length of timezone without colon separator (e.g., +0530).
const TIMEZONE_LENGTH_WITHOUT_COLON: usize = 4;

/// Character position of the colon in a timezone with separator.
const TIMEZONE_COLON_POSITION: usize = 2;

/// Minimum valid timezone offset in minutes (-12:00).
const MIN_TIMEZONE_OFFSET_MINUTES: i32 = -12 * 60;

/// Maximum valid timezone offset in minutes (+14:00).
const MAX_TIMEZONE_OFFSET_MINUTES: i32 = 14 * 60;

pub fn validate_iso8601(input: &str) -> Result<(), FormatError> {
  if input.is_empty() {
    return Err(Iso8601Error::Empty.into());
  }

  let chars: Vec<char> = input.chars().collect();
  let has_date =
    chars.len() >= DATE_LENGTH && chars.get(DATE_FIRST_HYPHEN_POSITION).copied() == Some('-');
  let has_time = input.contains(':');

  if has_date && has_time {
    validate_iso8601_datetime(input)
  } else if has_date {
    validate_iso8601_date(input).map_err(Into::into)
  } else if has_time {
    validate_iso8601_time(input).map_err(Into::into)
  } else {
    Err(Iso8601Error::InvalidFormat.into())
  }
}

fn validate_iso8601_date(input: &str) -> Result<(), Iso8601Error> {
  let chars: Vec<char> = input.chars().collect();

  // Require exactly DATE_LENGTH characters for date (YYYY-MM-DD)
  if chars.len() != DATE_LENGTH {
    return Err(Iso8601Error::InvalidFormat);
  }

  let year_str: String = chars[0..DATE_FIRST_HYPHEN_POSITION].iter().collect();
  let year: i32 = parse_int(&year_str).map_err(|()| Iso8601Error::InvalidYear(year_str.clone()))?;

  if chars.get(DATE_FIRST_HYPHEN_POSITION).copied() != Some('-') {
    return Err(Iso8601Error::InvalidDateSeparator);
  }

  let month_str: String = chars[DATE_FIRST_HYPHEN_POSITION + 1..DATE_SECOND_HYPHEN_POSITION]
    .iter()
    .collect();
  let month: u8 = parse_int(&month_str).map_err(|()| Iso8601Error::InvalidMonth(0))?;
  if !(1..=12).contains(&month) {
    return Err(Iso8601Error::InvalidMonth(month));
  }

  if chars.get(DATE_SECOND_HYPHEN_POSITION).copied() != Some('-') {
    return Err(Iso8601Error::InvalidDateSeparator);
  }

  let day_str: String = chars[DATE_SECOND_HYPHEN_POSITION + 1..DATE_LENGTH]
    .iter()
    .collect();
  let day: u8 = parse_int(&day_str).map_err(|()| Iso8601Error::InvalidDay(0, 31))?;
  let max_days = get_days_in_month(month, is_leap_year(year));
  if day < 1 || day > max_days {
    return Err(Iso8601Error::InvalidDay(day, max_days));
  }

  Ok(())
}

fn validate_iso8601_time(input: &str) -> Result<(), Iso8601Error> {
  let chars: Vec<char> = input.chars().collect();

  if chars.len() < MIN_TIME_LENGTH {
    return Err(Iso8601Error::InvalidFormat);
  }

  let hour_str: String = chars[0..TIME_HOUR_COLON_POSITION].iter().collect();
  let hour: u8 = parse_int(&hour_str).map_err(|()| Iso8601Error::InvalidHour(0))?;
  if hour > MAX_HOUR {
    return Err(Iso8601Error::InvalidHour(hour));
  }
  if chars.get(TIME_HOUR_COLON_POSITION).copied() != Some(':') {
    return Err(Iso8601Error::InvalidTimeSeparator);
  }

  let minute_str: String = chars[TIME_HOUR_COLON_POSITION + 1..TIME_SECOND_COLON_POSITION + 1]
    .iter()
    .collect();
  let minute: u8 = parse_int(&minute_str).map_err(|()| Iso8601Error::InvalidMinute(0))?;
  if minute > MAX_MINUTE {
    return Err(Iso8601Error::InvalidMinute(minute));
  }

  if chars.len() > TIME_SECOND_COLON_POSITION
    && chars.get(TIME_SECOND_COLON_POSITION).copied() == Some(':')
  {
    let second_str: String = chars[TIME_SECOND_COLON_POSITION + 1..TIME_SECONDS_END_POSITION]
      .iter()
      .collect();
    let second: u8 = parse_int(&second_str).map_err(|()| Iso8601Error::InvalidSecond(0))?;
    if second > MAX_SECOND {
      return Err(Iso8601Error::InvalidSecond(second));
    }

    if chars.len() > TIME_SECONDS_END_POSITION {
      let remaining: String = chars[TIME_SECONDS_END_POSITION..].iter().collect();
      validate_iso8601_timezone(&remaining)?;
    }
  } else if chars.len() > TIME_SECOND_COLON_POSITION {
    let remaining: String = chars[TIME_SECOND_COLON_POSITION..].iter().collect();
    if !remaining.starts_with('Z') && !remaining.starts_with('+') && !remaining.starts_with('-') {
      return Err(Iso8601Error::InvalidFormat);
    }
    validate_iso8601_timezone(&remaining)?;
  }

  Ok(())
}

fn validate_iso8601_datetime(input: &str) -> Result<(), FormatError> {
  let chars: Vec<char> = input.chars().collect();

  let Some(sep_pos) = chars
    .iter()
    .enumerate()
    .find(|(index, ch)| {
      *index == DATETIME_SEPARATOR_POSITION && (**ch == 'T' || **ch == ' ')
    })
    .map(|(index, _)| index)
  else {
    return Err(Iso8601Error::InvalidDateTimeSeparator.into());
  };

  let date_str: String = chars[0..DATETIME_SEPARATOR_POSITION].iter().collect();
  let time_str: String = chars[sep_pos + 1..].iter().collect();

  validate_iso8601_date(&date_str)?;
  validate_iso8601_time(&time_str)?;
  Ok(())
}

fn validate_iso8601_timezone(input: &str) -> Result<(), Iso8601Error> {
  let chars: Vec<char> = input.chars().collect();

  if chars.is_empty() || input == "Z" {
    return Ok(());
  }

  let sign = match chars.first().copied() {
    Some('+') => 1,
    Some('-') => -1,
    _ => return Err(Iso8601Error::InvalidTimezone),
  };

  let tz_chars = &chars[1..];
  let (hour_str, minute_str) =
    if tz_chars.len() == TIMEZONE_LENGTH_WITH_COLON
      && tz_chars.get(TIMEZONE_COLON_POSITION).copied() == Some(':')
    {
      let hour: String = tz_chars[0..TIMEZONE_COLON_POSITION].iter().collect();
      let minute: String = tz_chars[TIMEZONE_COLON_POSITION + 1..TIMEZONE_LENGTH_WITH_COLON]
        .iter()
        .collect();
      (hour, minute)
    } else if tz_chars.len() == TIMEZONE_LENGTH_WITHOUT_COLON {
      let hour: String = tz_chars[0..TIMEZONE_COLON_POSITION].iter().collect();
      let minute: String = tz_chars[TIMEZONE_COLON_POSITION..TIMEZONE_LENGTH_WITHOUT_COLON]
        .iter()
        .collect();
      (hour, minute)
    } else {
      return Err(Iso8601Error::InvalidTimezone);
    };

  let hour: u8 = parse_int(&hour_str).map_err(|()| Iso8601Error::InvalidTimezoneHour(0))?;
  if hour > MAX_HOUR {
    return Err(Iso8601Error::InvalidTimezoneHour(hour));
  }

  let minute: u8 = parse_int(&minute_str).map_err(|()| Iso8601Error::InvalidTimezoneMinute(0))?;
  if minute > MAX_MINUTE {
    return Err(Iso8601Error::InvalidTimezoneMinute(minute));
  }

  let total_minutes = (i32::from(hour) * 60 + i32::from(minute)) * sign;
  if !(MIN_TIMEZONE_OFFSET_MINUTES..=MAX_TIMEZONE_OFFSET_MINUTES).contains(&total_minutes) {
    return Err(Iso8601Error::InvalidTimezone);
  }

  Ok(())
}

fn parse_int<T: std::str::FromStr>(value: &str) -> Result<T, ()> {
  value.parse().map_err(|_| ())
}

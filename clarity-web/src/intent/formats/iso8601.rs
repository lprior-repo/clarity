#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use super::helpers::{get_days_in_month, is_leap_year};
use super::{FormatError, Iso8601Error};

pub fn validate_iso8601(input: &str) -> Result<(), FormatError> {
  if input.is_empty() {
    return Err(Iso8601Error::Empty.into());
  }

  let chars: Vec<char> = input.chars().collect();
  let has_date = chars.len() >= 10 && chars.get(4).copied() == Some('-');
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

  // Require exactly 10 characters for date (YYYY-MM-DD)
  if chars.len() != 10 {
    return Err(Iso8601Error::InvalidFormat);
  }

  let year_str: String = chars[0..4].iter().collect();
  let year: i32 = parse_int(&year_str).map_err(|()| Iso8601Error::InvalidYear(year_str.clone()))?;

  if chars.get(4).copied() != Some('-') {
    return Err(Iso8601Error::InvalidDateSeparator);
  }

  let month_str: String = chars[5..7].iter().collect();
  let month: u8 = parse_int(&month_str).map_err(|()| Iso8601Error::InvalidMonth(0))?;
  if !(1..=12).contains(&month) {
    return Err(Iso8601Error::InvalidMonth(month));
  }

  if chars.get(7).copied() != Some('-') {
    return Err(Iso8601Error::InvalidDateSeparator);
  }

  let day_str: String = chars[8..10].iter().collect();
  let day: u8 = parse_int(&day_str).map_err(|()| Iso8601Error::InvalidDay(0, 31))?;
  let max_days = get_days_in_month(month, is_leap_year(year));
  if day < 1 || day > max_days {
    return Err(Iso8601Error::InvalidDay(day, max_days));
  }

  Ok(())
}

fn validate_iso8601_time(input: &str) -> Result<(), Iso8601Error> {
  let chars: Vec<char> = input.chars().collect();

  if chars.len() < 8 {
    return Err(Iso8601Error::InvalidFormat);
  }

  let hour_str: String = chars[0..2].iter().collect();
  let hour: u8 = parse_int(&hour_str).map_err(|()| Iso8601Error::InvalidHour(0))?;
  if hour > 23 {
    return Err(Iso8601Error::InvalidHour(hour));
  }
  if chars.get(2).copied() != Some(':') {
    return Err(Iso8601Error::InvalidTimeSeparator);
  }

  let minute_str: String = chars[3..5].iter().collect();
  let minute: u8 = parse_int(&minute_str).map_err(|()| Iso8601Error::InvalidMinute(0))?;
  if minute > 59 {
    return Err(Iso8601Error::InvalidMinute(minute));
  }

  if chars.len() > 5 && chars.get(5).copied() == Some(':') {
    let second_str: String = chars[6..8].iter().collect();
    let second: u8 = parse_int(&second_str).map_err(|()| Iso8601Error::InvalidSecond(0))?;
    if second > 60 {
      return Err(Iso8601Error::InvalidSecond(second));
    }

    if chars.len() > 8 {
      let remaining: String = chars[8..].iter().collect();
      validate_iso8601_timezone(&remaining)?;
    }
  } else if chars.len() > 5 {
    let remaining: String = chars[5..].iter().collect();
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
    .find(|(index, ch)| *index == 10 && (**ch == 'T' || **ch == ' '))
    .map(|(index, _)| index)
  else {
    return Err(Iso8601Error::InvalidDateTimeSeparator.into());
  };

  let date_str: String = chars[0..10].iter().collect();
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
  let (hour_str, minute_str) = if tz_chars.len() == 5 && tz_chars.get(2).copied() == Some(':') {
    let hour: String = tz_chars[0..2].iter().collect();
    let minute: String = tz_chars[3..5].iter().collect();
    (hour, minute)
  } else if tz_chars.len() == 4 {
    let hour: String = tz_chars[0..2].iter().collect();
    let minute: String = tz_chars[2..4].iter().collect();
    (hour, minute)
  } else {
    return Err(Iso8601Error::InvalidTimezone);
  };

  let hour: u8 = parse_int(&hour_str).map_err(|()| Iso8601Error::InvalidTimezoneHour(0))?;
  if hour > 23 {
    return Err(Iso8601Error::InvalidTimezoneHour(hour));
  }

  let minute: u8 = parse_int(&minute_str).map_err(|()| Iso8601Error::InvalidTimezoneMinute(0))?;
  if minute > 59 {
    return Err(Iso8601Error::InvalidTimezoneMinute(minute));
  }

  let total_minutes = (i32::from(hour) * 60 + i32::from(minute)) * sign;
  if !(-12 * 60..=14 * 60).contains(&total_minutes) {
    return Err(Iso8601Error::InvalidTimezone);
  }

  Ok(())
}

fn parse_int<T: std::str::FromStr>(value: &str) -> Result<T, ()> {
  value.parse().map_err(|_| ())
}

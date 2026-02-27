use super::helpers::{get_days_in_month, is_leap_year};
use super::{FormatError, Iso8601Error};

pub fn validate_iso8601(input: &str) -> Result<(), FormatError> {
  if input.is_empty() {
    return Err(Iso8601Error::Empty.into());
  }

  let has_date = input.len() >= 10 && input.chars().nth(4) == Some('-');
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
  if input.len() < 10 {
    return Err(Iso8601Error::InvalidFormat);
  }

  let year_str = &input[0..4];
  let year: i32 = parse_int(year_str).map_err(|()| Iso8601Error::InvalidYear(year_str.into()))?;

  if input.chars().nth(4) != Some('-') {
    return Err(Iso8601Error::InvalidDateSeparator);
  }

  let month_str = &input[5..7];
  let month: u8 = parse_int(month_str).map_err(|()| Iso8601Error::InvalidMonth(0))?;
  if !(1..=12).contains(&month) {
    return Err(Iso8601Error::InvalidMonth(month));
  }

  if input.chars().nth(7) != Some('-') {
    return Err(Iso8601Error::InvalidDateSeparator);
  }

  let day_str = &input[8..10];
  let day: u8 = parse_int(day_str).map_err(|()| Iso8601Error::InvalidDay(0, 31))?;
  let max_days = get_days_in_month(month, is_leap_year(year));
  if day < 1 || day > max_days {
    return Err(Iso8601Error::InvalidDay(day, max_days));
  }

  Ok(())
}

fn validate_iso8601_time(input: &str) -> Result<(), Iso8601Error> {
  if input.len() < 8 {
    return Err(Iso8601Error::InvalidFormat);
  }

  let hour: u8 = parse_int(&input[0..2]).map_err(|()| Iso8601Error::InvalidHour(0))?;
  if hour > 23 {
    return Err(Iso8601Error::InvalidHour(hour));
  }
  if input.chars().nth(2) != Some(':') {
    return Err(Iso8601Error::InvalidTimeSeparator);
  }

  let minute: u8 = parse_int(&input[3..5]).map_err(|()| Iso8601Error::InvalidMinute(0))?;
  if minute > 59 {
    return Err(Iso8601Error::InvalidMinute(minute));
  }

  if input.len() > 5 && input.chars().nth(5) == Some(':') {
    let second: u8 = parse_int(&input[6..8]).map_err(|()| Iso8601Error::InvalidSecond(0))?;
    if second > 60 {
      return Err(Iso8601Error::InvalidSecond(second));
    }

    if input.len() > 8 {
      validate_iso8601_timezone(&input[8..])?;
    }
  } else if input.len() > 5 {
    let remaining = &input[5..];
    if !remaining.starts_with('Z') && !remaining.starts_with('+') && !remaining.starts_with('-') {
      return Err(Iso8601Error::InvalidFormat);
    }
    validate_iso8601_timezone(remaining)?;
  }

  Ok(())
}

fn validate_iso8601_datetime(input: &str) -> Result<(), FormatError> {
  let Some(sep_pos) = input
    .chars()
    .enumerate()
    .find(|(index, ch)| *index == 10 && (*ch == 'T' || *ch == ' '))
    .map(|(index, _)| index)
  else {
    return Err(Iso8601Error::InvalidDateTimeSeparator.into());
  };

  validate_iso8601_date(&input[0..10])?;
  validate_iso8601_time(&input[sep_pos + 1..])?;
  Ok(())
}

fn validate_iso8601_timezone(input: &str) -> Result<(), Iso8601Error> {
  if input.is_empty() || input == "Z" {
    return Ok(());
  }

  let sign = match input.chars().next() {
    Some('+') => 1,
    Some('-') => -1,
    _ => return Err(Iso8601Error::InvalidTimezone),
  };

  let tz_content = &input[1..];
  let (hour_str, minute_str) = if tz_content.len() == 5 && tz_content.chars().nth(2) == Some(':') {
    (&tz_content[0..2], &tz_content[3..5])
  } else if tz_content.len() == 4 {
    (&tz_content[0..2], &tz_content[2..4])
  } else {
    return Err(Iso8601Error::InvalidTimezone);
  };

  let hour: u8 = parse_int(hour_str).map_err(|()| Iso8601Error::InvalidTimezoneHour(0))?;
  if hour > 23 {
    return Err(Iso8601Error::InvalidTimezoneHour(hour));
  }

  let minute: u8 = parse_int(minute_str).map_err(|()| Iso8601Error::InvalidTimezoneMinute(0))?;
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

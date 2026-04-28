//! Format Validators Module - WP06
//!
//! Provides validation functions for common data formats:
//! - Email addresses (RFC 5322)
//! - UUIDs (RFC 4122)
//! - URIs (RFC 3986)
//! - ISO 8601 date/time strings

#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use thiserror::Error;

// =============================================================================
// Error Types
// =============================================================================

/// Top-level format validation error
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum FormatError {
  #[error("email validation failed: {0}")]
  Email(#[from] EmailError),

  #[error("uuid validation failed: {0}")]
  Uuid(#[from] UuidError),

  #[error("uri validation failed: {0}")]
  Uri(#[from] UriError),

  #[error("iso8601 validation failed: {0}")]
  Iso8601(#[from] Iso8601Error),
}

/// Email validation error reasons
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum EmailError {
  #[error("email is empty")]
  Empty,

  #[error("missing '@' separator")]
  MissingAtSign,

  #[error("multiple '@' separators found")]
  MultipleAtSigns,

  #[error("local part is empty")]
  EmptyLocalPart,

  #[error("domain part is empty")]
  EmptyDomain,

  #[error("local part exceeds 64 characters")]
  LocalPartTooLong,

  #[error("domain part exceeds 255 characters")]
  DomainTooLong,

  #[error("local part contains invalid character: '{0}'")]
  InvalidLocalChar(char),

  #[error("domain contains invalid character: '{0}'")]
  InvalidDomainChar(char),

  #[error("consecutive dots not allowed")]
  ConsecutiveDots,

  #[error("local part cannot start with dot")]
  LocalStartsWithDot,

  #[error("local part cannot end with dot")]
  LocalEndsWithDot,

  #[error("domain cannot start with dot")]
  DomainStartsWithDot,

  #[error("domain cannot end with dot")]
  DomainEndsWithDot,

  #[error("domain label cannot start with hyphen")]
  DomainLabelStartsWithHyphen,

  #[error("domain label cannot end with hyphen")]
  DomainLabelEndsWithHyphen,

  #[error("domain label exceeds 63 characters")]
  DomainLabelTooLong,

  #[error("domain must have at least one dot (TLD required)")]
  MissingTld,
}

/// UUID validation error reasons
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum UuidError {
  #[error("uuid is empty")]
  Empty,

  #[error("uuid must be 36 characters, got {0}")]
  WrongLength(usize),

  #[error("uuid must have hyphens at positions 8, 13, 18, 23")]
  MissingHyphens,

  #[error("invalid character at position {0}: '{1}'")]
  InvalidChar(usize, char),

  #[error("invalid version: character at position 14 must be '1'-'5', got '{0}'")]
  InvalidVersion(char),

  #[error("invalid variant: character at position 19 must be '8', '9', 'a', 'b' (case-insensitive), got '{0}'")]
  InvalidVariant(char),
}

/// URI validation error reasons
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum UriError {
  #[error("uri is empty")]
  Empty,

  #[error("missing scheme separator '://'")]
  MissingSchemeSeparator,

  #[error("scheme is empty")]
  EmptyScheme,

  #[error("scheme must start with a letter")]
  SchemeMustStartWithLetter,

  #[error("scheme contains invalid character: '{0}'")]
  InvalidSchemeChar(char),

  #[error("authority (host) is required")]
  MissingAuthority,

  #[error("authority is empty")]
  EmptyAuthority,
}

/// ISO 8601 validation error reasons
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum Iso8601Error {
  #[error("date/time string is empty")]
  Empty,

  #[error("invalid format: expected YYYY-MM-DD, HH:MM:SS, or combined format")]
  InvalidFormat,

  #[error("invalid year: {0}")]
  InvalidYear(String),

  #[error("invalid month: {0} (must be 01-12)")]
  InvalidMonth(u8),

  #[error("invalid day: {0} (must be 01-{1})")]
  InvalidDay(u8, u8),

  #[error("invalid hour: {0} (must be 00-23)")]
  InvalidHour(u8),

  #[error("invalid minute: {0} (must be 00-59)")]
  InvalidMinute(u8),

  #[error("invalid second: {0} (must be 00-59 or 60 for leap second)")]
  InvalidSecond(u8),

  #[error("invalid date separator: expected '-'")]
  InvalidDateSeparator,

  #[error("invalid time separator: expected ':'")]
  InvalidTimeSeparator,

  #[error("invalid datetime separator: expected 'T' or space")]
  InvalidDateTimeSeparator,

  #[error("invalid timezone format")]
  InvalidTimezone,

  #[error("invalid timezone hour: {0} (must be 00-23)")]
  InvalidTimezoneHour(u8),

  #[error("invalid timezone minute: {0} (must be 00-59)")]
  InvalidTimezoneMinute(u8),

  #[error("invalid hexadecimal character: '{0}'")]
  InvalidHexChar(char),
}

// =============================================================================
// Email Validation (RFC 5322)
// =============================================================================

/// Validates an email address according to RFC 5322 rules.
///
/// # Validation Rules
/// - Must contain exactly one '@' separator
/// - Local part: max 64 chars, alphanumeric + special chars (see RFC 5322)
/// - Domain part: max 255 chars, labels separated by dots, each label max 63 chars
/// - No consecutive dots
/// - No leading/trailing dots in local or domain
/// - Domain labels cannot start/end with hyphen
/// - Domain must have at least one TLD (contain a dot)
///
/// # Errors
/// Returns `FormatError::Email` with specific `EmailError` variant on failure.
///
/// # Examples
/// ```
/// use clarity_web::intent::formats::{validate_email, FormatError};
///
/// assert!(validate_email("user@example.com").is_ok());
/// assert!(validate_email("invalid").is_err());
/// ```
pub fn validate_email(input: &str) -> Result<(), FormatError> {
  // Check empty
  if input.is_empty() {
    return Err(EmailError::Empty.into());
  }

  // Check for exactly one '@'
  let at_count = input.chars().filter(|&c| c == '@').count();
  match at_count {
    0 => return Err(EmailError::MissingAtSign.into()),
    1 => {}
    _ => return Err(EmailError::MultipleAtSigns.into()),
  }

  // Split into local and domain
  let (local, domain) = input.split_once('@').ok_or(EmailError::MissingAtSign)?;

  validate_email_local(local)?;
  validate_email_domain(domain)?;

  Ok(())
}

/// Allowed special characters in email local part
const ALLOWED_LOCAL_SPECIAL: &[char] = &[
  '!', '#', '$', '%', '&', '\'', '*', '+', '-', '/', '=', '?', '^', '_', '`', '{', '|', '}', '~',
  '.', '"',
];

/// Validates the local part of an email address.
fn validate_email_local(local: &str) -> Result<(), EmailError> {
  if local.is_empty() {
    return Err(EmailError::EmptyLocalPart);
  }

  if local.len() > 64 {
    return Err(EmailError::LocalPartTooLong);
  }

  if local.starts_with('.') {
    return Err(EmailError::LocalStartsWithDot);
  }

  if local.ends_with('.') {
    return Err(EmailError::LocalEndsWithDot);
  }

  // Check for consecutive dots
  if local.contains("..") {
    return Err(EmailError::ConsecutiveDots);
  }

  // Validate characters
  // Allowed: alphanumeric + !#$%&'*+-/=?^_`{|}~
  local
    .chars()
    .find(|c| {
      !(c.is_ascii_alphanumeric() || ALLOWED_LOCAL_SPECIAL.contains(c) || *c == '.' && !local.contains(".."))
    })
    .map_or(Ok(()), |c| Err(EmailError::InvalidLocalChar(c)))?;

  // Re-check consecutive dots after char validation
  let chars: Vec<char> = local.chars().collect();
  chars
    .windows(2)
    .find(|w| w[0] == '.' && w[1] == '.')
    .map_or(Ok(()), |_| Err(EmailError::ConsecutiveDots))?;

  Ok(())
}

/// Validates the domain part of an email address.
fn validate_email_domain(domain: &str) -> Result<(), EmailError> {
  if domain.is_empty() {
    return Err(EmailError::EmptyDomain);
  }

  if domain.len() > 255 {
    return Err(EmailError::DomainTooLong);
  }

  if domain.starts_with('.') {
    return Err(EmailError::DomainStartsWithDot);
  }

  if domain.ends_with('.') {
    return Err(EmailError::DomainEndsWithDot);
  }

  // Must have at least one dot (TLD required)
  if !domain.contains('.') {
    return Err(EmailError::MissingTld);
  }

  // Check for consecutive dots
  if domain.contains("..") {
    return Err(EmailError::ConsecutiveDots);
  }

  // Validate labels
  domain.split('.').try_for_each(validate_domain_label)?;

  Ok(())
}

/// Validates a single domain label.
fn validate_domain_label(label: &str) -> Result<(), EmailError> {
  if label.is_empty() {
    return Err(EmailError::ConsecutiveDots);
  }

  if label.len() > 63 {
    return Err(EmailError::DomainLabelTooLong);
  }

  if label.starts_with('-') {
    return Err(EmailError::DomainLabelStartsWithHyphen);
  }

  if label.ends_with('-') {
    return Err(EmailError::DomainLabelEndsWithHyphen);
  }

  // Validate characters: alphanumeric and hyphen only
  label
    .chars()
    .find(|c| !c.is_ascii_alphanumeric() && *c != '-')
    .map_or(Ok(()), |c| Err(EmailError::InvalidDomainChar(c)))?;

  Ok(())
}

// =============================================================================
// UUID Validation (RFC 4122)
// =============================================================================

/// Validates a UUID according to RFC 4122 rules.
///
/// # Format
/// - 8-4-4-4-12 format: `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`
/// - Total length: 36 characters (32 hex + 4 hyphens)
/// - Version: character at position 14 (index 14) must be '1'-'5'
/// - Variant: character at position 19 (index 19) must be '8', '9', 'a', 'b', 'A', 'B'
///
/// # Errors
/// Returns `FormatError::Uuid` with specific `UuidError` variant on failure.
///
/// # Examples
/// ```
/// use clarity_web::intent::formats::{validate_uuid, FormatError};
///
/// assert!(validate_uuid("550e8400-e29b-41d4-a716-446655440000").is_ok());
/// assert!(validate_uuid("invalid").is_err());
/// ```
pub fn validate_uuid(input: &str) -> Result<(), FormatError> {
  if input.is_empty() {
    return Err(UuidError::Empty.into());
  }

  if input.len() != 36 {
    return Err(UuidError::WrongLength(input.len()).into());
  }

  // Check hyphen positions: 8, 13, 18, 23
  let hyphen_positions = [8, 13, 18, 23];
  let chars: Vec<char> = input.chars().collect();

  hyphen_positions
    .iter()
    .find(|&&pos| chars.get(pos) != Some(&'-'))
    .map_or(Ok(()), |_| {
      Err(FormatError::Uuid(UuidError::MissingHyphens))
    })?;

  // Validate hex characters and positions
  chars
    .iter()
    .enumerate()
    .find(|(idx, c)| !hyphen_positions.contains(idx) && !is_valid_hex_char(**c))
    .map_or(Ok(()), |(idx, c)| {
      Err(FormatError::Uuid(UuidError::InvalidChar(idx, *c)))
    })?;

  // Validate version (position 14, index 14)
  let version_char = chars.get(14).ok_or(UuidError::WrongLength(input.len()))?;
  if !matches!(version_char, '1' | '2' | '3' | '4' | '5') {
    return Err(UuidError::InvalidVersion(*version_char).into());
  }

  // Validate variant (position 19, index 19)
  let variant_char = chars.get(19).ok_or(UuidError::WrongLength(input.len()))?;
  if !matches!(variant_char.to_ascii_lowercase(), '8' | '9' | 'a' | 'b') {
    return Err(UuidError::InvalidVariant(*variant_char).into());
  }

  Ok(())
}

/// Checks if a character is a valid hexadecimal digit.
const fn is_valid_hex_char(c: char) -> bool {
  c.is_ascii_hexdigit()
}

// =============================================================================
// URI Validation (RFC 3986)
// =============================================================================

/// Validates a URI according to RFC 3986 rules.
///
/// # Format
/// - Must have a scheme followed by '://'
/// - Scheme must start with a letter
/// - Scheme can contain letters, digits, '+', '-', '.'
/// - Authority (host) is required after '://'
///
/// # Errors
/// Returns `FormatError::Uri` with specific `UriError` variant on failure.
///
/// # Examples
/// ```
/// use clarity_web::intent::formats::{validate_uri, FormatError};
///
/// assert!(validate_uri("https://example.com").is_ok());
/// assert!(validate_uri("mailto:test@example.com").is_err()); // no authority
/// ```
pub fn validate_uri(input: &str) -> Result<(), FormatError> {
  if input.is_empty() {
    return Err(UriError::Empty.into());
  }

  // Find :// separator
  let Some(separator_idx) = input.find("://") else {
    return Err(UriError::MissingSchemeSeparator.into());
  };

  // Extract scheme
  let scheme = &input[..separator_idx];
  validate_uri_scheme(scheme)?;

  // Extract authority
  let authority_start = separator_idx + 3;
  if authority_start >= input.len() {
    return Err(UriError::MissingAuthority.into());
  }

  let authority_and_path = &input[authority_start..];

  // Find end of authority (first '/' or end of string)
  let authority_end = authority_and_path
    .find('/')
    .map_or(authority_and_path.len(), |pos| pos);
  let authority = &authority_and_path[..authority_end];

  if authority.is_empty() {
    return Err(UriError::EmptyAuthority.into());
  }

  Ok(())
}

/// Validates a URI scheme.
fn validate_uri_scheme(scheme: &str) -> Result<(), UriError> {
  if scheme.is_empty() {
    return Err(UriError::EmptyScheme);
  }

  let chars: Vec<char> = scheme.chars().collect();

  // First character must be a letter
  let first_char = chars.first().ok_or(UriError::EmptyScheme)?;
  if !first_char.is_ascii_alphabetic() {
    return Err(UriError::SchemeMustStartWithLetter);
  }

  // Validate remaining characters
  chars
    .iter()
    .skip(1)
    .find(|c| !c.is_ascii_alphanumeric() && !matches!(c, '+' | '-' | '.'))
    .map_or(Ok(()), |c| Err(UriError::InvalidSchemeChar(*c)))?;

  Ok(())
}

// =============================================================================
// ISO 8601 Validation
// =============================================================================

/// Validates an ISO 8601 date/time string.
///
/// # Supported Formats
/// - Date only: `YYYY-MM-DD`
/// - Time only: `HH:MM:SS` or `HH:MM:SSZ` or `HH:MM:SS+HH:MM`
/// - Combined: `YYYY-MM-DDTHH:MM:SS` with optional timezone
///
/// # Validation Rules
/// - Year: any valid integer
/// - Month: 01-12
/// - Day: 01 to max days in month (considering leap years)
/// - Hour: 00-23
/// - Minute: 00-59
/// - Second: 00-59 (or 60 for leap second)
/// - Timezone: Z, +HH:MM, -HH:MM, +HHMM, -HHMM
///
/// # Errors
/// Returns `FormatError::Iso8601` with specific `Iso8601Error` variant on failure.
///
/// # Examples
/// ```
/// use clarity_web::intent::formats::{validate_iso8601, FormatError};
///
/// assert!(validate_iso8601("2024-02-27").is_ok());
/// assert!(validate_iso8601("2024-02-30").is_err()); // Feb doesn't have 30 days
/// ```
pub fn validate_iso8601(input: &str) -> Result<(), FormatError> {
  if input.is_empty() {
    return Err(Iso8601Error::Empty.into());
  }

  // Determine format: date-only, time-only, or combined
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

/// Validates an ISO 8601 date (YYYY-MM-DD).
fn validate_iso8601_date(input: &str) -> Result<(), Iso8601Error> {
  if input.len() < 10 {
    return Err(Iso8601Error::InvalidFormat);
  }

  // Parse year
  let year_str = &input[0..4];
  let year: i32 =
    parse_int(year_str).map_err(|()| Iso8601Error::InvalidYear(year_str.to_string()))?;

  // Check separator
  if input.chars().nth(4) != Some('-') {
    return Err(Iso8601Error::InvalidDateSeparator);
  }

  // Parse month
  let month_str = &input[5..7];
  let month: u8 = parse_int(month_str).map_err(|()| Iso8601Error::InvalidMonth(0))?;
  if !(1..=12).contains(&month) {
    return Err(Iso8601Error::InvalidMonth(month));
  }

  // Check separator
  if input.chars().nth(7) != Some('-') {
    return Err(Iso8601Error::InvalidDateSeparator);
  }

  // Parse day
  let day_str = &input[8..10];
  let day: u8 = parse_int(day_str).map_err(|()| Iso8601Error::InvalidDay(0, 31))?;

  let is_leap = is_leap_year(year);
  let max_days = get_days_in_month(month, is_leap);

  if day < 1 || day > max_days {
    return Err(Iso8601Error::InvalidDay(day, max_days));
  }

  Ok(())
}

/// Validates an ISO 8601 time (HH:MM:SS with optional timezone).
fn validate_iso8601_time(input: &str) -> Result<(), Iso8601Error> {
  if input.len() < 8 {
    return Err(Iso8601Error::InvalidFormat);
  }

  // Parse hour
  let hour_str = &input[0..2];
  let hour: u8 = parse_int(hour_str).map_err(|()| Iso8601Error::InvalidHour(0))?;
  if hour > 23 {
    return Err(Iso8601Error::InvalidHour(hour));
  }

  // Check separator
  if input.chars().nth(2) != Some(':') {
    return Err(Iso8601Error::InvalidTimeSeparator);
  }

  // Parse minute
  let minute_str = &input[3..5];
  let minute: u8 = parse_int(minute_str).map_err(|()| Iso8601Error::InvalidMinute(0))?;
  if minute > 59 {
    return Err(Iso8601Error::InvalidMinute(minute));
  }

  // Check if seconds are present
  if input.len() > 5 && input.chars().nth(5) == Some(':') {
    // Parse second
    let second_str = &input[6..8];
    let second: u8 = parse_int(second_str).map_err(|()| Iso8601Error::InvalidSecond(0))?;
    if second > 60 {
      // 60 allowed for leap second
      return Err(Iso8601Error::InvalidSecond(second));
    }

    // Check for timezone
    if input.len() > 8 {
      let tz_part = &input[8..];
      validate_iso8601_timezone(tz_part)?;
    }
  } else if input.len() > 5 {
    // No seconds but has more chars - check for timezone without seconds
    let remaining = &input[5..];
    if !remaining.starts_with('Z') && !remaining.starts_with('+') && !remaining.starts_with('-') {
      return Err(Iso8601Error::InvalidFormat);
    }
    validate_iso8601_timezone(remaining)?;
  }

  Ok(())
}

/// Validates an ISO 8601 combined datetime (YYYY-MM-DDTHH:MM:SS with optional timezone).
fn validate_iso8601_datetime(input: &str) -> Result<(), FormatError> {
  // Find the separator (T or space)
  let Some(sep_pos) = input
    .chars()
    .enumerate()
    .find(|(i, c)| *i == 10 && (*c == 'T' || *c == ' '))
    .map(|(i, _)| i)
  else {
    return Err(Iso8601Error::InvalidDateTimeSeparator.into());
  };

  // Validate date part
  let date_part = &input[0..10];
  validate_iso8601_date(date_part)?;

  // Validate time part
  let time_part = &input[sep_pos + 1..];
  validate_iso8601_time(time_part)?;

  Ok(())
}

/// Validates an ISO 8601 timezone component.
fn validate_iso8601_timezone(input: &str) -> Result<(), Iso8601Error> {
  if input.is_empty() {
    return Ok(());
  }

  // Z (UTC)
  if input == "Z" {
    return Ok(());
  }

  // +HH:MM or -HH:MM or +HHMM or -HHMM
  let first_char = input.chars().next();
  let sign = match first_char {
    Some('+') => 1,
    Some('-') => -1,
    _ => return Err(Iso8601Error::InvalidTimezone),
  };

  let tz_content = &input[1..];

  // Check format: HH:MM or HHMM
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

  // Validate that offset is within -12:00 to +14:00
  let total_minutes = (i32::from(hour) * 60 + i32::from(minute)) * sign;
  if !(-12 * 60..=14 * 60).contains(&total_minutes) {
    return Err(Iso8601Error::InvalidTimezone);
  }

  Ok(())
}

/// Parses a string to an integer without panicking.
fn parse_int<T: std::str::FromStr>(s: &str) -> Result<T, ()> {
  s.parse().map_err(|_| ())
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Determines if a year is a leap year.
///
/// A year is a leap year if:
/// - It is divisible by 4, AND
/// - Either not divisible by 100, OR divisible by 400
///
/// # Examples
/// ```
/// use clarity_web::intent::formats::is_leap_year;
///
/// assert!(is_leap_year(2024));  // Divisible by 4, not by 100
/// assert!(!is_leap_year(1900)); // Divisible by 100 but not 400
/// assert!(is_leap_year(2000));  // Divisible by 400
/// ```
#[must_use]
pub const fn is_leap_year(year: i32) -> bool {
  (year % 4 == 0) && ((year % 100 != 0) || (year % 400 == 0))
}

/// Returns the number of days in a given month.
///
/// # Arguments
/// - `month`: Month number (1-12)
/// - `is_leap`: Whether the year is a leap year (affects February)
///
/// # Returns
/// Number of days in the month, or 0 if month is invalid.
///
/// # Examples
/// ```
/// use clarity_web::intent::formats::get_days_in_month;
///
/// assert_eq!(get_days_in_month(1, false), 31);  // January
/// assert_eq!(get_days_in_month(2, true), 29);   // February in leap year
/// assert_eq!(get_days_in_month(2, false), 28);  // February in non-leap year
/// assert_eq!(get_days_in_month(4, false), 30);  // April
/// ```
#[must_use]
pub const fn get_days_in_month(month: u8, is_leap: bool) -> u8 {
  match month {
    1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
    4 | 6 | 9 | 11 => 30,
    2 => {
      if is_leap {
        29
      } else {
        28
      }
    }
    _ => 0, // Invalid month
  }
}

/// Checks if a string contains only valid hexadecimal characters.
///
/// # Examples
/// ```
/// use clarity_web::intent::formats::is_valid_hex;
///
/// assert!(is_valid_hex("0123456789abcdefABCDEF"));
/// assert!(!is_valid_hex("ghij")); // 'g', 'h', 'i', 'j' are not hex
/// ```
#[must_use]
pub fn is_valid_hex(s: &str) -> bool {
  !s.is_empty() && s.chars().all(is_valid_hex_char)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(
  clippy::unwrap_used,
  clippy::expect_used,
  clippy::panic,
  clippy::float_cmp,
  clippy::needless_collect,
  clippy::unnecessary_debug_formatting,
  clippy::match_same_arms,
  clippy::option_if_let_else,
  clippy::suspicious_else_formatting,
  clippy::manual_let_else,
  clippy::match_wild_err_arm,
  clippy::match_like_matches_macro,
  clippy::needless_pass_by_value
)]
mod tests {
  use super::*;

  // -------------------------------------------------------------------------
  // Email Validation Tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_valid_emails() {
    let valid_emails = vec![
      "user@example.com",
      "user.name@example.com",
      "user+tag@example.com",
      "user@sub.example.com",
      "a@b.co",
      "test123@test-site.org",
      "UPPER@EXAMPLE.COM",
      "user!def@example.org",
      "user#def@example.org",
      "user$def@example.org",
      "user%def@example.org",
      "user&def@example.org",
      "user'def@example.org",
      "user*def@example.org",
      "user/def@example.org",
      "user=def@example.org",
      "user?def@example.org",
      "user^def@example.org",
      "user_def@example.org",
      "user`def@example.org",
      "user{def}@example.org",
      "user|def@example.org",
      "user}def@example.org",
      "user~def@example.org",
    ];

    for email in valid_emails {
      assert!(
        validate_email(email).is_ok(),
        "Expected '{email}' to be valid"
      );
    }
  }

  #[test]
  fn test_invalid_emails() {
    let invalid_emails = vec![
      ("", EmailError::Empty),
      ("no-at-sign", EmailError::MissingAtSign),
      ("two@@at.com", EmailError::MultipleAtSigns),
      ("@nodomain.com", EmailError::EmptyLocalPart),
      ("no-local@", EmailError::EmptyDomain),
      ("no-tld", EmailError::MissingAtSign),
      ("user@notld", EmailError::MissingTld),
      ("user@.com", EmailError::DomainStartsWithDot),
      ("user@domain.", EmailError::DomainEndsWithDot),
      ("user@-domain.com", EmailError::DomainLabelStartsWithHyphen),
      ("user@domain-.com", EmailError::DomainLabelEndsWithHyphen),
      (".user@domain.com", EmailError::LocalStartsWithDot),
      ("user.@domain.com", EmailError::LocalEndsWithDot),
      ("user..name@domain.com", EmailError::ConsecutiveDots),
      ("user@domain..com", EmailError::ConsecutiveDots),
      ("user@do main.com", EmailError::InvalidDomainChar(' ')),
    ];

    for (email, expected_error) in invalid_emails {
      let result = validate_email(email);
      if let Err(FormatError::Email(err)) = result {
        assert_eq!(err, expected_error, "For email '{email}'");
      } else {
        panic!("Expected EmailError::{expected_error:?} for '{email}', got {result:?}");
      }
    }
  }

  // -------------------------------------------------------------------------
  // UUID Validation Tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_valid_uuids() {
    let valid_uuids = vec![
      "550e8400-e29b-41d4-a716-446655440000", // v4
      "6ba7b810-9dad-11d1-80b4-00c04fd430c8", // v1
      "6ba7b811-9dad-21d4-a716-00c04fd430c8", // v2
      "6ba7b812-9dad-31d4-a716-00c04fd430c8", // v3
      "6ba7b814-9dad-41d4-a716-00c04fd430c8", // v4
      "6ba7b815-9dad-51d4-a716-00c04fd430c8", // v5
      "550E8400-E29B-41D4-A716-446655440000", // uppercase
    ];

    for uuid in valid_uuids {
      assert!(validate_uuid(uuid).is_ok(), "Expected '{uuid}' to be valid");
    }
  }

  #[test]
  fn test_invalid_uuids() {
    let invalid_uuids = vec![
      ("", UuidError::Empty),
      ("too-short", UuidError::WrongLength(9)),
      (
        "550e8400e29b41d4a716446655440000",
        UuidError::WrongLength(32),
      ), // No hyphens, wrong length
      (
        "550e8400-e29b-41d4-a71644665544000",
        UuidError::WrongLength(34),
      ), // Missing one char, wrong length
      (
        "550e8400-e29b-61d4-a716-446655440000",
        UuidError::InvalidVersion('6'),
      ), // v6 invalid
      (
        "550e8400-e29b-01d4-a716-446655440000",
        UuidError::InvalidVersion('0'),
      ), // v0 invalid
      (
        "550e8400-e29b-41d4-c716-446655440000",
        UuidError::InvalidVariant('c'),
      ), // c invalid variant
    ];

    for (uuid, expected_error) in invalid_uuids {
      let result = validate_uuid(uuid);
      if let Err(FormatError::Uuid(err)) = result {
        assert_eq!(err, expected_error, "For uuid '{uuid}'");
      } else {
        panic!("Expected UuidError::{expected_error:?} for '{uuid}', got {result:?}");
      }
    }
  }

  // -------------------------------------------------------------------------
  // URI Validation Tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_valid_uris() {
    let valid_uris = vec![
      "https://example.com",
      "http://localhost",
      "ftp://files.example.com",
      "wss://websocket.example.com:8080",
      "https://sub.domain.example.com/path",
      "https://example.com:443/path?query=value#fragment",
      "http+unix://socket",
      "custom.scheme://host",
      "a://b",
      "Z://example",
      "https://example.com/path/to/resource",
    ];

    for uri in valid_uris {
      assert!(validate_uri(uri).is_ok(), "Expected '{uri}' to be valid");
    }
  }

  #[test]
  fn test_invalid_uris() {
    let invalid_uris = vec![
      ("", UriError::Empty),
      ("no-scheme.com", UriError::MissingSchemeSeparator),
      ("://no-scheme.com", UriError::EmptyScheme),
      (
        "123://invalid-scheme.com",
        UriError::SchemeMustStartWithLetter,
      ),
      ("sch@eme://host.com", UriError::InvalidSchemeChar('@')),
      ("https://", UriError::MissingAuthority),
    ];

    for (uri, expected_error) in invalid_uris {
      let result = validate_uri(uri);
      if let Err(FormatError::Uri(err)) = result {
        assert_eq!(err, expected_error, "For uri '{uri}'");
      } else {
        panic!("Expected UriError::{expected_error:?} for '{uri}', got {result:?}");
      }
    }
  }

  // -------------------------------------------------------------------------
  // ISO 8601 Validation Tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_valid_iso8601_dates() {
    let valid_dates = vec![
      "2024-01-01",
      "2024-12-31",
      "2024-02-29", // Leap year
      "2000-02-29", // Century leap year
      "1900-02-28", // Non-leap century year
      "2023-02-28",
      "2024-06-15",
    ];

    for date in valid_dates {
      assert!(
        validate_iso8601(date).is_ok(),
        "Expected '{date}' to be valid"
      );
    }
  }

  #[test]
  fn test_invalid_iso8601_dates() {
    let invalid_dates = vec![
      ("", Iso8601Error::Empty),
      ("2024-13-01", Iso8601Error::InvalidMonth(13)),
      ("2024-00-01", Iso8601Error::InvalidMonth(0)),
      ("2024-01-32", Iso8601Error::InvalidDay(32, 31)),
      ("2024-01-00", Iso8601Error::InvalidDay(0, 31)),
      ("2023-02-29", Iso8601Error::InvalidDay(29, 28)), // Non-leap year
      ("1900-02-29", Iso8601Error::InvalidDay(29, 28)), // Non-leap century
    ];

    for (date, expected_error) in invalid_dates {
      let result = validate_iso8601(date);
      if let Err(FormatError::Iso8601(err)) = result {
        assert_eq!(err, expected_error, "For date '{date}'");
      } else {
        panic!("Expected Iso8601Error::{expected_error:?} for '{date}', got {result:?}");
      }
    }
  }

  #[test]
  fn test_valid_iso8601_times() {
    let valid_times = vec![
      "00:00:00",
      "23:59:59",
      "12:30:45",
      "00:00:00Z",
      "12:30:45+02:00",
      "12:30:45-05:00",
      "12:30:45+0200",
      "12:30:45-0500",
      "23:59:60", // Leap second
    ];

    for time in valid_times {
      assert!(
        validate_iso8601(time).is_ok(),
        "Expected '{time}' to be valid"
      );
    }
  }

  #[test]
  fn test_invalid_iso8601_times() {
    let invalid_times = vec![
      ("24:00:00", Iso8601Error::InvalidHour(24)),
      ("12:60:00", Iso8601Error::InvalidMinute(60)),
      ("12:00:61", Iso8601Error::InvalidSecond(61)),
    ];

    for (time, expected_error) in invalid_times {
      let result = validate_iso8601(time);
      if let Err(FormatError::Iso8601(err)) = result {
        assert_eq!(err, expected_error, "For time '{time}'");
      } else {
        panic!("Expected Iso8601Error::{expected_error:?} for '{time}', got {result:?}");
      }
    }
  }

  #[test]
  fn test_valid_iso8601_datetimes() {
    let valid_datetimes = vec![
      "2024-02-27T12:30:45",
      "2024-02-27T12:30:45Z",
      "2024-02-27T12:30:45+02:00",
      "2024-02-27T12:30:45-05:00",
      "2024-02-27 12:30:45", // Space separator also valid
    ];

    for dt in valid_datetimes {
      assert!(validate_iso8601(dt).is_ok(), "Expected '{dt}' to be valid");
    }
  }

  // -------------------------------------------------------------------------
  // Helper Function Tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_is_leap_year() {
    assert!(is_leap_year(2024)); // Divisible by 4
    assert!(is_leap_year(2000)); // Divisible by 400
    assert!(!is_leap_year(1900)); // Divisible by 100 but not 400
    assert!(!is_leap_year(2023)); // Not divisible by 4
    assert!(is_leap_year(2400)); // Divisible by 400
    assert!(!is_leap_year(2100)); // Divisible by 100 but not 400
  }

  #[test]
  fn test_get_days_in_month() {
    // Non-leap year
    assert_eq!(get_days_in_month(1, false), 31); // January
    assert_eq!(get_days_in_month(2, false), 28); // February
    assert_eq!(get_days_in_month(3, false), 31); // March
    assert_eq!(get_days_in_month(4, false), 30); // April
    assert_eq!(get_days_in_month(5, false), 31); // May
    assert_eq!(get_days_in_month(6, false), 30); // June
    assert_eq!(get_days_in_month(7, false), 31); // July
    assert_eq!(get_days_in_month(8, false), 31); // August
    assert_eq!(get_days_in_month(9, false), 30); // September
    assert_eq!(get_days_in_month(10, false), 31); // October
    assert_eq!(get_days_in_month(11, false), 30); // November
    assert_eq!(get_days_in_month(12, false), 31); // December

    // Leap year
    assert_eq!(get_days_in_month(2, true), 29);

    // Invalid month
    assert_eq!(get_days_in_month(0, false), 0);
    assert_eq!(get_days_in_month(13, false), 0);
  }

  #[test]
  fn test_is_valid_hex() {
    assert!(is_valid_hex("0123456789"));
    assert!(is_valid_hex("abcdef"));
    assert!(is_valid_hex("ABCDEF"));
    assert!(is_valid_hex("0123abcDEF"));

    assert!(!is_valid_hex(""));
    assert!(!is_valid_hex("ghij"));
    assert!(!is_valid_hex("xyz"));
    assert!(!is_valid_hex("0x123"));
  }
}

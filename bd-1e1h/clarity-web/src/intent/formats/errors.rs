use thiserror::Error;

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

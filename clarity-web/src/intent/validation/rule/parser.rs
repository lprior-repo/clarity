#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Rule expression parser
//! Parses human-friendly rule strings like "equals foo" or "integer >= 5"

use thiserror::Error;

/// Parsed rule expression
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum RuleExpr {
  // Equality
  Equals(String),
  EqualsVariable(String),
  EqualsInt(i32),
  EqualsFloat(f64),
  EqualsBool(bool),

  // Types
  IsString,
  IsInteger,
  IsNumber,
  IsBoolean,
  IsArray,
  IsObject,
  IsNull,

  // String patterns
  StringMatching(String),
  StringStartingWith(String),
  StringEndingWith(String),
  StringContaining(String),
  NonEmptyString,
  IsEmail,
  IsUuid,
  IsUri,
  IsJwt,
  IsIso8601,

  // Numbers
  IntegerGte(i32),
  IntegerGt(i32),
  IntegerLte(i32),
  IntegerLt(i32),
  IntegerBetween(i32, i32),
  NumberBetween(f64, f64),

  // Presence
  Present,
  Absent,
  NotNull,

  // Arrays
  NonEmptyArray,
  ArrayOfLength(u32),
  ArrayWithMinItems(u32),
  ArrayWithMaxItems(u32),
  ArrayWhereEach(Box<Self>),

  // Compound
  ValidJwt,
  ValidIso8601,
  OneOf(Vec<String>),

  // Contains reference
  ContainsVariable(String),

  // Unknown/raw for rules we can't parse yet
  Raw(String),
}

/// Errors that can occur when parsing a rule string
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RuleParseError {
  #[error("input is empty or whitespace only")]
  EmptyInput,

  #[error("invalid syntax: {0}")]
  InvalidSyntax(String),

  #[error("unknown rule type: {0}")]
  UnknownRuleType(String),

  #[error("invalid value: {0}")]
  InvalidValue(String),
}

/// Parse a rule string into a `RuleExpr`
///
/// # Errors
/// Returns `RuleParseError` if the input is invalid or cannot be parsed
pub fn parse(rule: &str) -> Result<RuleExpr, RuleParseError> {
  let rule = rule.trim();

  // Check for empty input
  if rule.is_empty() {
    return Err(RuleParseError::EmptyInput);
  }

  // Try parsers in order
  try_parse_equals(rule)
    .or_else(|| try_parse_type(rule))
    .or_else(|| try_parse_string_pattern(rule))
    .or_else(|| try_parse_number(rule))
    .or_else(|| try_parse_presence(rule))
    .or_else(|| try_parse_array(rule))
    .or_else(|| try_parse_compound(rule))
    .ok_or_else(|| RuleParseError::UnknownRuleType(rule.to_string()))
}

fn try_parse_equals(rule: &str) -> Option<RuleExpr> {
  let prefix = "equals ";
  rule.strip_prefix(prefix).map(parse_equals_value)
}

fn parse_equals_value(value: &str) -> RuleExpr {
  parse_equals_variable(value)
    .or_else(|| parse_equals_bool(value))
    .or_else(|| parse_equals_number(value))
    .map_or_else(|| RuleExpr::Equals(value.to_string()), |v| v)
}

fn parse_equals_variable(value: &str) -> Option<RuleExpr> {
  let trimmed = value.trim();
  trimmed
    .strip_prefix("${")
    .and_then(|s| s.strip_suffix('}'))
    .map(|var_name| RuleExpr::EqualsVariable(var_name.to_string()))
}

fn parse_equals_bool(value: &str) -> Option<RuleExpr> {
  match value.trim() {
    "true" => Some(RuleExpr::EqualsBool(true)),
    "false" => Some(RuleExpr::EqualsBool(false)),
    _ => None,
  }
}

fn parse_equals_number(value: &str) -> Option<RuleExpr> {
  let trimmed = value.trim();

  // Try integer first
  trimmed
    .parse::<i32>()
    .ok()
    .map(RuleExpr::EqualsInt)
    // Then try float
    .or_else(|| trimmed.parse::<f64>().ok().map(RuleExpr::EqualsFloat))
}

fn try_parse_type(rule: &str) -> Option<RuleExpr> {
  match rule {
    "string" => Some(RuleExpr::IsString),
    "integer" => Some(RuleExpr::IsInteger),
    "number" => Some(RuleExpr::IsNumber),
    "boolean" => Some(RuleExpr::IsBoolean),
    "array" => Some(RuleExpr::IsArray),
    "object" => Some(RuleExpr::IsObject),
    "null" => Some(RuleExpr::IsNull),
    _ => None,
  }
}

fn try_parse_string_pattern(rule: &str) -> Option<RuleExpr> {
  // Direct pattern matches
  match rule {
    "non-empty string" => Some(RuleExpr::NonEmptyString),
    "email" => Some(RuleExpr::IsEmail),
    "uuid" => Some(RuleExpr::IsUuid),
    "uri" => Some(RuleExpr::IsUri),
    "jwt" => Some(RuleExpr::IsJwt),
    "iso8601 datetime" => Some(RuleExpr::IsIso8601),
    _ => try_parse_prefix_string_pattern(rule),
  }
}

fn try_parse_prefix_string_pattern(rule: &str) -> Option<RuleExpr> {
  parse_prefix_pattern(rule, "string matching ", RuleExpr::StringMatching)
    .or_else(|| {
      parse_prefix_pattern(rule, "string starting with ", |s| {
        RuleExpr::StringStartingWith(s)
      })
    })
    .or_else(|| {
      parse_prefix_pattern(rule, "string ending with ", |s| {
        RuleExpr::StringEndingWith(s)
      })
    })
    .or_else(|| {
      parse_prefix_pattern(rule, "string containing ", |s| {
        RuleExpr::StringContaining(s)
      })
    })
    .or_else(|| parse_contains_variable(rule))
}

fn parse_prefix_pattern<F>(rule: &str, prefix: &str, constructor: F) -> Option<RuleExpr>
where
  F: Fn(String) -> RuleExpr,
{
  rule
    .strip_prefix(prefix)
    .map(|rest| constructor(rest.to_string()))
}

fn parse_contains_variable(rule: &str) -> Option<RuleExpr> {
  rule
    .strip_prefix("contains ${")
    .and_then(|s| s.strip_suffix('}'))
    .map(|var_name| RuleExpr::ContainsVariable(var_name.to_string()))
}

fn try_parse_number(rule: &str) -> Option<RuleExpr> {
  try_parse_integer_comparison(rule).or_else(|| parse_number_between(rule))
}

fn try_parse_integer_comparison(rule: &str) -> Option<RuleExpr> {
  parse_integer_gte(rule)
    .or_else(|| parse_integer_gt(rule))
    .or_else(|| parse_integer_lte(rule))
    .or_else(|| parse_integer_lt(rule))
}

fn parse_integer_gte(rule: &str) -> Option<RuleExpr> {
  parse_int_comparison(rule, "integer >= ", 11, RuleExpr::IntegerGte)
}

fn parse_integer_gt(rule: &str) -> Option<RuleExpr> {
  rule
    .strip_prefix("integer > ")
    .and_then(|rest| rest.trim().parse::<i32>().ok())
    .map(RuleExpr::IntegerGt)
}

fn parse_integer_lte(rule: &str) -> Option<RuleExpr> {
  parse_int_comparison(rule, "integer <= ", 11, RuleExpr::IntegerLte)
}

fn parse_integer_lt(rule: &str) -> Option<RuleExpr> {
  parse_int_comparison(rule, "integer < ", 10, RuleExpr::IntegerLt)
}

fn parse_int_comparison<F>(
  rule: &str,
  prefix: &str,
  _drop_len: usize,
  constructor: F,
) -> Option<RuleExpr>
where
  F: Fn(i32) -> RuleExpr,
{
  rule
    .strip_prefix(prefix)
    .and_then(|rest| rest.trim().parse::<i32>().ok())
    .map(constructor)
}

fn parse_number_between(rule: &str) -> Option<RuleExpr> {
  rule.strip_prefix("number between ").and_then(|rest| {
    let parts: Vec<&str> = rest.split(" and ").collect();
    if parts.len() == 2 {
      let low: f64 = parts[0].trim().parse().ok()?;
      let high: f64 = parts[1].trim().parse().ok()?;
      Some(RuleExpr::NumberBetween(low, high))
    } else {
      None
    }
  })
}

fn try_parse_presence(rule: &str) -> Option<RuleExpr> {
  match rule {
    "present" => Some(RuleExpr::Present),
    "absent" => Some(RuleExpr::Absent),
    "not null" => Some(RuleExpr::NotNull),
    _ => None,
  }
}

fn try_parse_array(rule: &str) -> Option<RuleExpr> {
  match rule {
    "non-empty array" => Some(RuleExpr::NonEmptyArray),
    _ => try_parse_prefix_array(rule),
  }
}

fn try_parse_prefix_array(rule: &str) -> Option<RuleExpr> {
  parse_array_of_length(rule)
    .or_else(|| parse_array_with_min(rule))
    .or_else(|| parse_array_with_max(rule))
    .or_else(|| parse_array_where_each(rule))
}

fn parse_array_of_length(rule: &str) -> Option<RuleExpr> {
  rule
    .strip_prefix("array of length ")
    .and_then(|rest| rest.trim().parse::<u32>().ok())
    .map(RuleExpr::ArrayOfLength)
}

fn parse_array_with_min(rule: &str) -> Option<RuleExpr> {
  parse_array_with_items(rule, "array with min ", 15, |n| {
    RuleExpr::ArrayWithMinItems(n)
  })
}

fn parse_array_with_max(rule: &str) -> Option<RuleExpr> {
  parse_array_with_items(rule, "array with max ", 15, |n| {
    RuleExpr::ArrayWithMaxItems(n)
  })
}

fn parse_array_with_items<F>(
  rule: &str,
  prefix: &str,
  _drop_len: usize,
  constructor: F,
) -> Option<RuleExpr>
where
  F: Fn(u32) -> RuleExpr,
{
  rule
    .strip_prefix(prefix)
    .and_then(|rest| rest.strip_suffix(" items").or(Some(rest)))
    .and_then(|num_str| num_str.trim().parse::<u32>().ok())
    .map(constructor)
}

fn parse_array_where_each(rule: &str) -> Option<RuleExpr> {
  rule.strip_prefix("array where each ").map(|inner| {
    let inner_rule = normalize_inner_rule(inner);
    let inner_expr = parse(&inner_rule).map_or_else(|_| RuleExpr::Raw(inner.to_string()), |v| v);
    RuleExpr::ArrayWhereEach(Box::new(inner_expr))
  })
}

fn normalize_inner_rule(inner: &str) -> String {
  let trimmed = inner.trim();
  trimmed.strip_prefix("is ").map_or_else(
    || {
      trimmed.strip_prefix("matches ").map_or_else(
        || trimmed.to_string(),
        |rest| format!("string matching {rest}"),
      )
    },
    ToString::to_string,
  )
}

fn try_parse_compound(rule: &str) -> Option<RuleExpr> {
  match rule {
    "valid JWT" => Some(RuleExpr::ValidJwt),
    "valid ISO8601 datetime" => Some(RuleExpr::ValidIso8601),
    _ => rule
      .strip_prefix("one of ")
      .and_then(|list_str| parse_string_list(list_str).ok())
      .map(RuleExpr::OneOf),
  }
}

/// Parse a list like `["a", "b", "c"]` or `[a, b, c]`
fn parse_string_list(s: &str) -> Result<Vec<String>, RuleParseError> {
  let s = s.trim();

  // Check for brackets
  let content = s
    .strip_prefix('[')
    .and_then(|s| s.strip_suffix(']'))
    .ok_or_else(|| RuleParseError::InvalidSyntax("missing brackets".to_string()))?;

  // Split by comma and process each item
  let items: Vec<String> = content
    .split(',')
    .map(|item| {
      let item = item.trim();
      // Remove quotes if present
      item
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .map_or(item, |s| s)
        .to_string()
    })
    .filter(|s| !s.is_empty())
    .collect();

  if items.is_empty() && !content.trim().is_empty() {
    Err(RuleParseError::InvalidValue("empty list".to_string()))
  } else {
    Ok(items)
  }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::float_cmp, clippy::needless_collect, clippy::unnecessary_debug_formatting, clippy::match_same_arms, clippy::option_if_let_else, clippy::suspicious_else_formatting, clippy::manual_let_else, clippy::match_wild_err_arm, clippy::match_like_matches_macro, clippy::needless_pass_by_value)]
mod tests {
  use super::*;

  #[test]
  fn parse_returns_rule_expr() {
    let result = parse("present");
    assert!(result.is_ok());
  }

  #[test]
  fn parse_trims_whitespace() {
    let result = parse("  equals foo  ");
    assert_eq!(result, Ok(RuleExpr::Equals("foo".to_string())));
  }
}

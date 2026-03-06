#![allow(
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
  clippy::match_like_matches_macro
)]
use crate::intent::security::{validate_regex_pattern, RegexVulnerability, SecurityError};

#[test]
fn test_validate_regex_pattern_valid() {
  assert!(validate_regex_pattern("^\\w+$").is_ok());
  assert!(validate_regex_pattern("[a-z]+").is_ok());
  assert!(validate_regex_pattern("test.*pattern").is_ok());
}

#[test]
fn test_validate_regex_pattern_empty() {
  assert!(matches!(
    validate_regex_pattern(""),
    Err(SecurityError::EmptyInput)
  ));
}

#[test]
fn test_validate_regex_pattern_null_byte() {
  assert!(matches!(
    validate_regex_pattern("pattern\0"),
    Err(SecurityError::NullByteDetected)
  ));
}

#[test]
fn test_validate_regex_pattern_exponential_plus() {
  assert!(matches!(
    validate_regex_pattern("(.+)+"),
    Err(SecurityError::ReDoSVulnerability {
      vulnerability: RegexVulnerability::ExponentialBacktracking
    })
  ));
}

#[test]
fn test_validate_regex_pattern_exponential_star() {
  assert!(matches!(
    validate_regex_pattern("(.*)*"),
    Err(SecurityError::ReDoSVulnerability {
      vulnerability: RegexVulnerability::ExponentialBacktracking
    })
  ));
}

#[test]
fn test_validate_regex_pattern_nested_quantifiers_general() {
  let result = validate_regex_pattern("(x*)*");
  assert!(result.is_err());
  assert!(matches!(
    result,
    Err(SecurityError::ReDoSVulnerability {
      vulnerability: RegexVulnerability::NestedQuantifiers
        | RegexVulnerability::ExponentialBacktracking
    })
  ));
}

#[test]
fn test_validate_regex_pattern_nested_star_general() {
  assert!(matches!(
    validate_regex_pattern("(a*)*"),
    Err(SecurityError::ReDoSVulnerability {
      vulnerability: RegexVulnerability::NestedQuantifiers
    })
  ));
}

#[test]
fn test_validate_regex_pattern_overlapping_wildcards() {
  assert!(matches!(
    validate_regex_pattern(".*.*"),
    Err(SecurityError::ReDoSVulnerability {
      vulnerability: RegexVulnerability::OverlappingWildcards
    })
  ));
}

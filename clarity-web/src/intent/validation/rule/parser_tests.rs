#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::intent::validation::rule::parser::{parse, RuleExpr, RuleParseError};

/// Helper to convert `RuleParseError` to string for test assertions
fn error_msg(err: &RuleParseError) -> String {
  err.to_string()
}

mod equality_tests {
  use super::*;

  #[test]
  fn parse_equals_string() {
    let result = parse("equals foo");
    assert_eq!(result, Ok(RuleExpr::Equals("foo".to_string())));
  }

  #[test]
  fn parse_equals_variable() {
    let result = parse("equals ${my_var}");
    assert_eq!(result, Ok(RuleExpr::EqualsVariable("my_var".to_string())));
  }

  #[test]
  fn parse_equals_int() {
    let result = parse("equals 42");
    assert_eq!(result, Ok(RuleExpr::EqualsInt(42)));
  }

  #[test]
  fn parse_equals_negative_int() {
    let result = parse("equals -10");
    assert_eq!(result, Ok(RuleExpr::EqualsInt(-10)));
  }

  #[test]
  fn parse_equals_float() {
    let result = parse("equals 3.14");
    assert_eq!(result, Ok(RuleExpr::EqualsFloat(3.14)));
  }

  #[test]
  fn parse_equals_true() {
    let result = parse("equals true");
    assert_eq!(result, Ok(RuleExpr::EqualsBool(true)));
  }

  #[test]
  fn parse_equals_false() {
    let result = parse("equals false");
    assert_eq!(result, Ok(RuleExpr::EqualsBool(false)));
  }

  #[test]
  fn parse_equals_with_whitespace() {
    let result = parse("  equals bar  ");
    assert_eq!(result, Ok(RuleExpr::Equals("bar".to_string())));
  }
}

mod type_tests {
  use super::*;

  #[test]
  fn parse_is_string() {
    let result = parse("string");
    assert_eq!(result, Ok(RuleExpr::IsString));
  }

  #[test]
  fn parse_is_integer() {
    let result = parse("integer");
    assert_eq!(result, Ok(RuleExpr::IsInteger));
  }

  #[test]
  fn parse_is_number() {
    let result = parse("number");
    assert_eq!(result, Ok(RuleExpr::IsNumber));
  }

  #[test]
  fn parse_is_boolean() {
    let result = parse("boolean");
    assert_eq!(result, Ok(RuleExpr::IsBoolean));
  }

  #[test]
  fn parse_is_array() {
    let result = parse("array");
    assert_eq!(result, Ok(RuleExpr::IsArray));
  }

  #[test]
  fn parse_is_object() {
    let result = parse("object");
    assert_eq!(result, Ok(RuleExpr::IsObject));
  }

  #[test]
  fn parse_is_null() {
    let result = parse("null");
    assert_eq!(result, Ok(RuleExpr::IsNull));
  }
}

mod string_pattern_tests {
  use super::*;

  #[test]
  fn parse_string_matching() {
    let result = parse("string matching ^\\w+$");
    assert_eq!(result, Ok(RuleExpr::StringMatching("^\\w+$".to_string())));
  }

  #[test]
  fn parse_string_starting_with() {
    let result = parse("string starting with foo");
    assert_eq!(result, Ok(RuleExpr::StringStartingWith("foo".to_string())));
  }

  #[test]
  fn parse_string_ending_with() {
    let result = parse("string ending with bar");
    assert_eq!(result, Ok(RuleExpr::StringEndingWith("bar".to_string())));
  }

  #[test]
  fn parse_string_containing() {
    let result = parse("string containing baz");
    assert_eq!(result, Ok(RuleExpr::StringContaining("baz".to_string())));
  }

  #[test]
  fn parse_non_empty_string() {
    let result = parse("non-empty string");
    assert_eq!(result, Ok(RuleExpr::NonEmptyString));
  }

  #[test]
  fn parse_email() {
    let result = parse("email");
    assert_eq!(result, Ok(RuleExpr::IsEmail));
  }

  #[test]
  fn parse_uuid() {
    let result = parse("uuid");
    assert_eq!(result, Ok(RuleExpr::IsUuid));
  }

  #[test]
  fn parse_uri() {
    let result = parse("uri");
    assert_eq!(result, Ok(RuleExpr::IsUri));
  }

  #[test]
  fn parse_jwt() {
    let result = parse("jwt");
    assert_eq!(result, Ok(RuleExpr::IsJwt));
  }

  #[test]
  fn parse_iso8601() {
    let result = parse("iso8601 datetime");
    assert_eq!(result, Ok(RuleExpr::IsIso8601));
  }
}

mod number_tests {
  use super::*;

  #[test]
  fn parse_integer_gte() {
    let result = parse("integer >= 5");
    assert_eq!(result, Ok(RuleExpr::IntegerGte(5)));
  }

  #[test]
  fn parse_integer_gt() {
    let result = parse("integer > 10");
    assert_eq!(result, Ok(RuleExpr::IntegerGt(10)));
  }

  #[test]
  fn parse_integer_lte() {
    let result = parse("integer <= 100");
    assert_eq!(result, Ok(RuleExpr::IntegerLte(100)));
  }

  #[test]
  fn parse_integer_lt() {
    let result = parse("integer < 0");
    assert_eq!(result, Ok(RuleExpr::IntegerLt(0)));
  }

  #[test]
  fn parse_number_between() {
    let result = parse("number between 1.5 and 10.5");
    assert_eq!(result, Ok(RuleExpr::NumberBetween(1.5, 10.5)));
  }
}

mod presence_tests {
  use super::*;

  #[test]
  fn parse_present() {
    let result = parse("present");
    assert_eq!(result, Ok(RuleExpr::Present));
  }

  #[test]
  fn parse_absent() {
    let result = parse("absent");
    assert_eq!(result, Ok(RuleExpr::Absent));
  }

  #[test]
  fn parse_not_null() {
    let result = parse("not null");
    assert_eq!(result, Ok(RuleExpr::NotNull));
  }
}

mod array_tests {
  use super::*;

  #[test]
  fn parse_non_empty_array() {
    let result = parse("non-empty array");
    assert_eq!(result, Ok(RuleExpr::NonEmptyArray));
  }

  #[test]
  fn parse_array_of_length() {
    let result = parse("array of length 5");
    assert_eq!(result, Ok(RuleExpr::ArrayOfLength(5)));
  }

  #[test]
  fn parse_array_with_min_items() {
    let result = parse("array with min 3 items");
    assert_eq!(result, Ok(RuleExpr::ArrayWithMinItems(3)));
  }

  #[test]
  fn parse_array_with_max_items() {
    let result = parse("array with max 10 items");
    assert_eq!(result, Ok(RuleExpr::ArrayWithMaxItems(10)));
  }

  #[test]
  fn parse_array_where_each() {
    let result = parse("array where each is string");
    assert!(matches!(
      result,
      Ok(RuleExpr::ArrayWhereEach(inner)) if *inner == RuleExpr::IsString
    ));
  }
}

mod compound_tests {
  use super::*;

  #[test]
  fn parse_valid_jwt() {
    let result = parse("valid JWT");
    assert_eq!(result, Ok(RuleExpr::ValidJwt));
  }

  #[test]
  fn parse_valid_iso8601() {
    let result = parse("valid ISO8601 datetime");
    assert_eq!(result, Ok(RuleExpr::ValidIso8601));
  }

  #[test]
  fn parse_one_of() {
    let result = parse("one of [\"a\", \"b\", \"c\"]");
    assert_eq!(
      result,
      Ok(RuleExpr::OneOf(vec![
        "a".to_string(),
        "b".to_string(),
        "c".to_string()
      ]))
    );
  }

  #[test]
  fn parse_one_of_without_quotes() {
    let result = parse("one of [foo, bar, baz]");
    assert_eq!(
      result,
      Ok(RuleExpr::OneOf(vec![
        "foo".to_string(),
        "bar".to_string(),
        "baz".to_string()
      ]))
    );
  }

  #[test]
  fn parse_contains_variable() {
    let result = parse("contains ${my_var}");
    assert_eq!(result, Ok(RuleExpr::ContainsVariable("my_var".to_string())));
  }
}

mod error_tests {
  use super::*;

  #[test]
  fn error_empty_input() {
    let result = parse("");
    assert!(matches!(result, Err(RuleParseError::EmptyInput)));
  }

  #[test]
  fn error_whitespace_only() {
    let result = parse("   ");
    assert!(matches!(result, Err(RuleParseError::EmptyInput)));
  }

  #[test]
  fn error_invalid_syntax() {
    let result = parse("invalid!!!");
    assert!(matches!(result, Err(RuleParseError::UnknownRuleType(_))));
  }

  #[test]
  fn error_unknown_rule_type() {
    let result = parse("unknown_rule value");
    assert!(matches!(result, Err(RuleParseError::UnknownRuleType(_))));
  }

  #[test]
  fn error_invalid_value_missing() {
    let result = parse("equals");
    assert!(matches!(result, Err(RuleParseError::UnknownRuleType(_))));
  }

  #[test]
  fn error_invalid_integer_comparison() {
    let result = parse("integer >= abc");
    assert!(matches!(result, Err(RuleParseError::UnknownRuleType(_))));
  }
}

mod edge_case_tests {
  use super::*;

  #[test]
  fn parse_with_leading_whitespace() {
    let result = parse("  present");
    assert_eq!(result, Ok(RuleExpr::Present));
  }

  #[test]
  fn parse_with_trailing_whitespace() {
    let result = parse("string   ");
    assert_eq!(result, Ok(RuleExpr::IsString));
  }

  #[test]
  fn parse_integer_zero() {
    let result = parse("equals 0");
    assert_eq!(result, Ok(RuleExpr::EqualsInt(0)));
  }

  #[test]
  fn parse_number_between_negative() {
    let result = parse("number between -5.5 and 5.5");
    assert_eq!(result, Ok(RuleExpr::NumberBetween(-5.5, 5.5)));
  }
}

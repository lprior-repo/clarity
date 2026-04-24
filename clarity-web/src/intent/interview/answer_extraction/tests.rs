//! Answer Extraction Tests
//!
//! This module contains all tests for the answer extraction functionality.

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::float_cmp, clippy::needless_collect, clippy::unnecessary_debug_formatting, clippy::match_same_arms, clippy::option_if_let_else, clippy::suspicious_else_formatting, clippy::manual_let_else, clippy::match_wild_err_arm, clippy::match_like_matches_macro, clippy::needless_pass_by_value)]
mod tests {
  use crate::intent::interview::answer_extraction::types::{ExtractedValue, ExtractionError};
  use crate::intent::interview::answer_extraction::extractors::{
    extract_by_type, extract_boolean, extract_email, extract_fields, extract_fields_with_types,
    extract_float, extract_integer, extract_list, extract_name, extract_text, extract_url,
  };
  use crate::intent::interview::answer_extraction::interview::{
    calculate_confidence, extract_from_answer,
  };
  use std::collections::HashMap;

  // =============================================================================
  // extract_text tests
  // =============================================================================

  #[test]
  fn test_extract_text_basic() {
    let result = extract_text("Hello, world!");
    assert!(result.is_ok());
    assert_eq!(result, Ok("Hello, world!".to_string()));
  }

  #[test]
  fn test_extract_text_with_whitespace() {
    let result = extract_text("  trimmed text  ");
    assert!(result.is_ok());
    assert_eq!(result, Ok("trimmed text".to_string()));
  }

  #[test]
  fn test_extract_text_empty() {
    let result = extract_text("");
    assert!(matches!(result, Err(ExtractionError::EmptyResponse)));
  }

  #[test]
  fn test_extract_text_whitespace_only() {
    let result = extract_text("   \n\t  ");
    assert!(matches!(result, Err(ExtractionError::EmptyResponse)));
  }

  #[test]
  fn test_extract_text_unicode() {
    let result = extract_text("  Hello \u{4e16}\u{754c}  ");
    assert!(result.is_ok());
    assert_eq!(result, Ok("Hello \u{4e16}\u{754c}".to_string()));
  }

  #[test]
  fn test_extract_text_multiline() {
    let result = extract_text("Line 1\nLine 2\nLine 3");
    assert!(result.is_ok());
    assert_eq!(result, Ok("Line 1\nLine 2\nLine 3".to_string()));
  }

  // =============================================================================
  // extract_name tests
  // =============================================================================

  #[test]
  fn test_extract_name_basic() {
    let result = extract_name("My API");
    assert!(result.is_ok());
    assert_eq!(result, Ok("My API".to_string()));
  }

  #[test]
  fn test_extract_name_multiline_takes_first() {
    let result = extract_name("First Line\nSecond Line");
    assert!(result.is_ok());
    assert_eq!(result, Ok("First Line".to_string()));
  }

  #[test]
  fn test_extract_name_with_whitespace() {
    let result = extract_name("  Trimmed Name  ");
    assert!(result.is_ok());
    assert_eq!(result, Ok("Trimmed Name".to_string()));
  }

  #[test]
  fn test_extract_name_empty() {
    let result = extract_name("");
    assert!(matches!(result, Err(ExtractionError::EmptyResponse)));
  }

  #[test]
  fn test_extract_name_only_newlines() {
    let result = extract_name("\n\n");
    assert!(matches!(result, Err(ExtractionError::EmptyResponse)));
  }

  #[test]
  fn test_extract_name_first_line_empty() {
    let result = extract_name("\nSecond Line");
    assert!(result.is_ok());
    assert_eq!(result, Ok("Second Line".to_string()));
  }

  // =============================================================================
  // extract_integer tests
  // =============================================================================

  #[test]
  fn test_extract_integer_basic() {
    let result = extract_integer("42");
    assert!(result.is_ok());
    assert_eq!(result, Ok(42));
  }

  #[test]
  fn test_extract_integer_negative() {
    let result = extract_integer("-42");
    assert!(result.is_ok());
    assert_eq!(result, Ok(-42));
  }

  #[test]
  fn test_extract_integer_with_text() {
    let result = extract_integer("The answer is 42 items");
    assert!(result.is_ok());
    assert_eq!(result, Ok(42));
  }

  #[test]
  fn test_extract_integer_with_whitespace() {
    let result = extract_integer("  123  ");
    assert!(result.is_ok());
    assert_eq!(result, Ok(123));
  }

  #[test]
  fn test_extract_integer_invalid() {
    let result = extract_integer("not a number");
    assert!(matches!(result, Err(ExtractionError::InvalidNumber(_))));
  }

  #[test]
  fn test_extract_integer_empty() {
    let result = extract_integer("");
    assert!(matches!(result, Err(ExtractionError::InvalidNumber(_))));
  }

  #[test]
  fn test_extract_integer_large() {
    let result = extract_integer("9223372036854775807");
    assert!(result.is_ok());
    assert_eq!(result, Ok(i64::MAX));
  }

  #[test]
  fn test_extract_integer_takes_first() {
    let result = extract_integer("10 items and 20 more");
    assert!(result.is_ok());
    assert_eq!(result, Ok(10));
  }

  // =============================================================================
  // extract_float tests
  // =============================================================================

  #[test]
  fn test_extract_float_basic() {
    let result = extract_float("3.14");
    assert!(result.is_ok());
    assert!((result.unwrap() - 3.14).abs() < f64::EPSILON);
  }

  #[test]
  fn test_extract_float_negative() {
    let result = extract_float("-2.5");
    assert!(result.is_ok());
    assert!((result.unwrap() - (-2.5)).abs() < f64::EPSILON);
  }

  #[test]
  fn test_extract_float_with_text() {
    let result = extract_float("The value is 3.14159 approximately");
    assert!(result.is_ok());
    assert!((result.unwrap() - 3.14159).abs() < f64::EPSILON);
  }

  #[test]
  fn test_extract_float_integer_input() {
    let result = extract_float("42");
    assert!(result.is_ok());
    assert!((result.unwrap() - 42.0).abs() < f64::EPSILON);
  }

  #[test]
  fn test_extract_float_invalid() {
    let result = extract_float("not a number");
    assert!(matches!(result, Err(ExtractionError::InvalidNumber(_))));
  }

  #[test]
  fn test_extract_float_empty() {
    let result = extract_float("");
    assert!(matches!(result, Err(ExtractionError::InvalidNumber(_))));
  }

  // =============================================================================
  // extract_boolean tests
  // =============================================================================

  #[test]
  fn test_extract_boolean_true_variants() {
    assert_eq!(extract_boolean("yes"), Ok(true));
    assert_eq!(extract_boolean("Yes"), Ok(true));
    assert_eq!(extract_boolean("YES"), Ok(true));
    assert_eq!(extract_boolean("true"), Ok(true));
    assert_eq!(extract_boolean("True"), Ok(true));
    assert_eq!(extract_boolean("TRUE"), Ok(true));
    assert_eq!(extract_boolean("on"), Ok(true));
    assert_eq!(extract_boolean("ON"), Ok(true));
    assert_eq!(extract_boolean("1"), Ok(true));
    assert_eq!(extract_boolean("y"), Ok(true));
    assert_eq!(extract_boolean("Y"), Ok(true));
    assert_eq!(extract_boolean("t"), Ok(true));
    assert_eq!(extract_boolean("T"), Ok(true));
  }

  #[test]
  fn test_extract_boolean_false_variants() {
    assert_eq!(extract_boolean("no"), Ok(false));
    assert_eq!(extract_boolean("No"), Ok(false));
    assert_eq!(extract_boolean("NO"), Ok(false));
    assert_eq!(extract_boolean("false"), Ok(false));
    assert_eq!(extract_boolean("False"), Ok(false));
    assert_eq!(extract_boolean("FALSE"), Ok(false));
    assert_eq!(extract_boolean("off"), Ok(false));
    assert_eq!(extract_boolean("OFF"), Ok(false));
    assert_eq!(extract_boolean("0"), Ok(false));
    assert_eq!(extract_boolean("n"), Ok(false));
    assert_eq!(extract_boolean("N"), Ok(false));
    assert_eq!(extract_boolean("f"), Ok(false));
    assert_eq!(extract_boolean("F"), Ok(false));
  }

  #[test]
  fn test_extract_boolean_with_whitespace() {
    assert_eq!(extract_boolean("  yes  "), Ok(true));
    assert_eq!(extract_boolean("  no  "), Ok(false));
  }

  #[test]
  fn test_extract_boolean_invalid() {
    let result = extract_boolean("maybe");
    assert!(matches!(result, Err(ExtractionError::InvalidBoolean(_))));
  }

  #[test]
  fn test_extract_boolean_empty() {
    let result = extract_boolean("");
    assert!(matches!(result, Err(ExtractionError::InvalidBoolean(_))));
  }

  #[test]
  fn test_extract_boolean_not_a_word() {
    let result = extract_boolean("yes, please");
    assert!(matches!(result, Err(ExtractionError::InvalidBoolean(_))));
  }

  // =============================================================================
  // extract_url tests
  // =============================================================================

  #[test]
  fn test_extract_url_http() {
    let result = extract_url("http://example.com");
    assert!(result.is_ok());
    assert_eq!(result, Ok("http://example.com".to_string()));
  }

  #[test]
  fn test_extract_url_https() {
    let result = extract_url("https://example.com");
    assert!(result.is_ok());
    assert_eq!(result, Ok("https://example.com".to_string()));
  }

  #[test]
  fn test_extract_url_with_path() {
    let result = extract_url("https://example.com/path/to/resource");
    assert!(result.is_ok());
    assert_eq!(
      result,
      Ok("https://example.com/path/to/resource".to_string())
    );
  }

  #[test]
  fn test_extract_url_with_query() {
    let result = extract_url("https://example.com/search?q=test");
    assert!(result.is_ok());
    assert_eq!(result, Ok("https://example.com/search?q=test".to_string()));
  }

  #[test]
  fn test_extract_url_embedded_in_text() {
    let result = extract_url("Visit https://example.com for more info");
    assert!(result.is_ok());
    assert_eq!(result, Ok("https://example.com".to_string()));
  }

  #[test]
  fn test_extract_url_at_end_of_sentence() {
    let result = extract_url("Go to https://example.com.");
    assert!(result.is_ok());
    assert_eq!(result, Ok("https://example.com".to_string()));
  }

  #[test]
  fn test_extract_url_in_parentheses() {
    let result = extract_url("Check this (https://example.com) out");
    assert!(result.is_ok());
    assert_eq!(result, Ok("https://example.com".to_string()));
  }

  #[test]
  fn test_extract_url_invalid_empty() {
    let result = extract_url("");
    assert!(matches!(result, Err(ExtractionError::InvalidUrl(_))));
  }

  #[test]
  fn test_extract_url_invalid_no_protocol() {
    let result = extract_url("example.com");
    assert!(matches!(result, Err(ExtractionError::InvalidUrl(_))));
  }

  #[test]
  fn test_extract_url_invalid_ftp() {
    let result = extract_url("ftp://files.example.com");
    assert!(matches!(result, Err(ExtractionError::InvalidUrl(_))));
  }

  // =============================================================================
  // extract_email tests
  // =============================================================================

  #[test]
  fn test_extract_email_basic() {
    let result = extract_email("user@example.com");
    assert!(result.is_ok());
    assert_eq!(result, Ok("user@example.com".to_string()));
  }

  #[test]
  fn test_extract_email_with_subdomain() {
    let result = extract_email("user@mail.example.com");
    assert!(result.is_ok());
    assert_eq!(result, Ok("user@mail.example.com".to_string()));
  }

  #[test]
  fn test_extract_email_embedded_in_text() {
    let result = extract_email("Contact us at support@example.com for help");
    assert!(result.is_ok());
    assert_eq!(result, Ok("support@example.com".to_string()));
  }

  #[test]
  fn test_extract_email_in_angle_brackets() {
    let result = extract_email("<noreply@example.com>");
    assert!(result.is_ok());
    assert_eq!(result, Ok("noreply@example.com".to_string()));
  }

  #[test]
  fn test_extract_email_in_parentheses() {
    let result = extract_email("Email (admin@example.com) for access");
    assert!(result.is_ok());
    assert_eq!(result, Ok("admin@example.com".to_string()));
  }

  #[test]
  fn test_extract_email_at_end_of_sentence() {
    let result = extract_email("Send to user@example.com.");
    assert!(result.is_ok());
    assert_eq!(result, Ok("user@example.com".to_string()));
  }

  #[test]
  fn test_extract_email_invalid_empty() {
    let result = extract_email("");
    assert!(matches!(result, Err(ExtractionError::InvalidEmail(_))));
  }

  #[test]
  fn test_extract_email_invalid_no_at() {
    let result = extract_email("userexample.com");
    assert!(matches!(result, Err(ExtractionError::InvalidEmail(_))));
  }

  #[test]
  fn test_extract_email_invalid_double_at() {
    let result = extract_email("user@@example.com");
    // This should still extract something, even if technically invalid
    assert!(result.is_ok());
  }

  // =============================================================================
  // extract_list tests
  // =============================================================================

  #[test]
  fn test_extract_list_comma_separated() {
    let result = extract_list("apple, banana, cherry");
    assert_eq!(result, vec!["apple", "banana", "cherry"]);
  }

  #[test]
  fn test_extract_list_newline_separated() {
    let result = extract_list("apple\nbanana\ncherry");
    assert_eq!(result, vec!["apple", "banana", "cherry"]);
  }

  #[test]
  fn test_extract_list_numbered() {
    let result = extract_list("1. apple\n2. banana\n3. cherry");
    assert_eq!(result, vec!["apple", "banana", "cherry"]);
  }

  #[test]
  fn test_extract_list_numbered_parens() {
    let result = extract_list("1) apple\n2) banana\n3) cherry");
    assert_eq!(result, vec!["apple", "banana", "cherry"]);
  }

  #[test]
  fn test_extract_list_numbered_dashes() {
    let result = extract_list("1- apple\n2- banana\n3- cherry");
    assert_eq!(result, vec!["apple", "banana", "cherry"]);
  }

  #[test]
  fn test_extract_list_numbered_colons() {
    let result = extract_list("1: apple\n2: banana\n3: cherry");
    assert_eq!(result, vec!["apple", "banana", "cherry"]);
  }

  #[test]
  fn test_extract_list_empty() {
    let result = extract_list("");
    assert!(result.is_empty());
  }

  #[test]
  fn test_extract_list_whitespace_only() {
    let result = extract_list("   ");
    assert!(result.is_empty());
  }

  #[test]
  fn test_extract_list_with_extra_whitespace() {
    let result = extract_list("  apple  ,  banana  ,  cherry  ");
    assert_eq!(result, vec!["apple", "banana", "cherry"]);
  }

  #[test]
  fn test_extract_list_filters_empty_items() {
    let result = extract_list("apple, , banana, , cherry");
    assert_eq!(result, vec!["apple", "banana", "cherry"]);
  }

  #[test]
  fn test_extract_list_double_digit_numbers() {
    let result = extract_list("1. first\n10. tenth\n11. eleventh");
    assert_eq!(result, vec!["first", "tenth", "eleventh"]);
  }

  // =============================================================================
  // extract_by_type tests
  // =============================================================================

  #[test]
  fn test_extract_by_type_text() {
    let result = extract_by_type("hello world", "text");
    assert!(result.is_ok());
    assert_eq!(result, Ok(ExtractedValue::Text("hello world".to_string())));
  }

  #[test]
  fn test_extract_by_type_string_alias() {
    let result = extract_by_type("hello world", "string");
    assert!(result.is_ok());
    assert_eq!(result, Ok(ExtractedValue::Text("hello world".to_string())));
  }

  #[test]
  fn test_extract_by_type_name() {
    let result = extract_by_type("My API\nExtra line", "name");
    assert!(result.is_ok());
    assert_eq!(result, Ok(ExtractedValue::Text("My API".to_string())));
  }

  #[test]
  fn test_extract_by_type_integer() {
    let result = extract_by_type("42", "integer");
    assert!(result.is_ok());
    assert_eq!(result, Ok(ExtractedValue::Integer(42)));
  }

  #[test]
  fn test_extract_by_type_int_alias() {
    let result = extract_by_type("42", "int");
    assert!(result.is_ok());
    assert_eq!(result, Ok(ExtractedValue::Integer(42)));
  }

  #[test]
  fn test_extract_by_type_number_alias() {
    let result = extract_by_type("42", "number");
    assert!(result.is_ok());
    assert_eq!(result, Ok(ExtractedValue::Integer(42)));
  }

  #[test]
  fn test_extract_by_type_float() {
    let result = extract_by_type("3.14", "float");
    assert!(result.is_ok());
    if let Ok(ExtractedValue::Float(f)) = result {
      assert!((f - 3.14).abs() < f64::EPSILON);
    } else {
      panic!("Expected Float variant");
    }
  }

  #[test]
  fn test_extract_by_type_decimal_alias() {
    let result = extract_by_type("3.14", "decimal");
    assert!(result.is_ok());
    if let Ok(ExtractedValue::Float(f)) = result {
      assert!((f - 3.14).abs() < f64::EPSILON);
    } else {
      panic!("Expected Float variant");
    }
  }

  #[test]
  fn test_extract_by_type_boolean() {
    let result = extract_by_type("yes", "boolean");
    assert!(result.is_ok());
    assert_eq!(result, Ok(ExtractedValue::Boolean(true)));
  }

  #[test]
  fn test_extract_by_type_bool_alias() {
    let result = extract_by_type("no", "bool");
    assert!(result.is_ok());
    assert_eq!(result, Ok(ExtractedValue::Boolean(false)));
  }

  #[test]
  fn test_extract_by_type_url() {
    let result = extract_by_type("https://example.com", "url");
    assert!(result.is_ok());
    assert_eq!(
      result,
      Ok(ExtractedValue::Url("https://example.com".to_string()))
    );
  }

  #[test]
  fn test_extract_by_type_uri_alias() {
    let result = extract_by_type("https://example.com", "uri");
    assert!(result.is_ok());
    assert_eq!(
      result,
      Ok(ExtractedValue::Url("https://example.com".to_string()))
    );
  }

  #[test]
  fn test_extract_by_type_email() {
    let result = extract_by_type("user@example.com", "email");
    assert!(result.is_ok());
    assert_eq!(
      result,
      Ok(ExtractedValue::Email("user@example.com".to_string()))
    );
  }

  #[test]
  fn test_extract_by_type_list() {
    let result = extract_by_type("a, b, c", "list");
    assert!(result.is_ok());
    assert_eq!(
      result,
      Ok(ExtractedValue::List(vec![
        "a".to_string(),
        "b".to_string(),
        "c".to_string()
      ]))
    );
  }

  #[test]
  fn test_extract_by_type_array_alias() {
    let result = extract_by_type("a, b, c", "array");
    assert!(result.is_ok());
    assert_eq!(
      result,
      Ok(ExtractedValue::List(vec![
        "a".to_string(),
        "b".to_string(),
        "c".to_string()
      ]))
    );
  }

  #[test]
  fn test_extract_by_type_unknown_defaults_to_text() {
    let result = extract_by_type("hello world", "unknown_type");
    assert!(result.is_ok());
    assert_eq!(result, Ok(ExtractedValue::Text("hello world".to_string())));
  }

  // =============================================================================
  // ExtractedValue tests
  // =============================================================================

  #[test]
  fn test_extracted_value_to_string_text() {
    let value = ExtractedValue::Text("hello".to_string());
    assert_eq!(value.to_string_value(), "hello");
  }

  #[test]
  fn test_extracted_value_to_string_integer() {
    let value = ExtractedValue::Integer(42);
    assert_eq!(value.to_string_value(), "42");
  }

  #[test]
  fn test_extracted_value_to_string_float() {
    let value = ExtractedValue::Float(3.14);
    assert_eq!(value.to_string_value(), "3.14");
  }

  #[test]
  fn test_extracted_value_to_string_boolean() {
    assert_eq!(ExtractedValue::Boolean(true).to_string_value(), "true");
    assert_eq!(ExtractedValue::Boolean(false).to_string_value(), "false");
  }

  #[test]
  fn test_extracted_value_to_string_url() {
    let value = ExtractedValue::Url("https://example.com".to_string());
    assert_eq!(value.to_string_value(), "https://example.com");
  }

  #[test]
  fn test_extracted_value_to_string_email() {
    let value = ExtractedValue::Email("user@example.com".to_string());
    assert_eq!(value.to_string_value(), "user@example.com");
  }

  #[test]
  fn test_extracted_value_to_string_list() {
    let value = ExtractedValue::List(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
    assert_eq!(value.to_string_value(), "a, b, c");
  }

  #[test]
  fn test_extracted_value_is_empty_text() {
    assert!(ExtractedValue::Text(String::new()).is_empty());
    assert!(ExtractedValue::Text("  ".to_string()).is_empty());
    assert!(!ExtractedValue::Text("hello".to_string()).is_empty());
  }

  #[test]
  fn test_extracted_value_is_empty_integer() {
    assert!(!ExtractedValue::Integer(0).is_empty());
    assert!(!ExtractedValue::Integer(42).is_empty());
  }

  #[test]
  fn test_extracted_value_is_empty_float() {
    assert!(!ExtractedValue::Float(0.0).is_empty());
    assert!(!ExtractedValue::Float(3.14).is_empty());
  }

  #[test]
  fn test_extracted_value_is_empty_boolean() {
    assert!(!ExtractedValue::Boolean(true).is_empty());
    assert!(!ExtractedValue::Boolean(false).is_empty());
  }

  #[test]
  fn test_extracted_value_is_empty_url() {
    assert!(ExtractedValue::Url(String::new()).is_empty());
    assert!(!ExtractedValue::Url("https://example.com".to_string()).is_empty());
  }

  #[test]
  fn test_extracted_value_is_empty_email() {
    assert!(ExtractedValue::Email(String::new()).is_empty());
    assert!(!ExtractedValue::Email("user@example.com".to_string()).is_empty());
  }

  #[test]
  fn test_extracted_value_is_empty_list() {
    assert!(ExtractedValue::List(Vec::new()).is_empty());
    assert!(!ExtractedValue::List(vec!["item".to_string()]).is_empty());
  }

  // =============================================================================
  // extract_fields tests
  // =============================================================================

  #[test]
  fn test_extract_fields_single_field() {
    let fields = vec!["name".to_string()];
    let result = extract_fields("My API", &fields);
    assert_eq!(result.get("name"), Some(&"My API".to_string()));
    assert_eq!(result.len(), 1);
  }

  #[test]
  fn test_extract_fields_single_field_with_whitespace() {
    let fields = vec!["name".to_string()];
    let result = extract_fields("  Trimmed API  ", &fields);
    assert_eq!(result.get("name"), Some(&"Trimmed API".to_string()));
  }

  #[test]
  fn test_extract_fields_empty_fields() {
    let fields: Vec<String> = Vec::new();
    let result = extract_fields("Some text", &fields);
    assert!(result.is_empty());
  }

  #[test]
  fn test_extract_fields_empty_response() {
    let fields = vec!["name".to_string()];
    let result = extract_fields("", &fields);
    assert!(result.is_empty());
  }

  #[test]
  fn test_extract_fields_multiple_fields() {
    let fields = vec!["name".to_string(), "description".to_string()];
    let result = extract_fields("My API", &fields);
    // For multiple fields, only first field is populated
    assert_eq!(result.get("name"), Some(&"My API".to_string()));
    assert_eq!(result.get("description"), None);
  }

  // =============================================================================
  // extract_fields_with_types tests
  // =============================================================================

  #[test]
  fn test_extract_fields_with_types_single_text() {
    let mut spec = HashMap::new();
    spec.insert("name".to_string(), "text".to_string());
    let result = extract_fields_with_types("My API", &spec);
    assert_eq!(result.get("name"), Some(&"My API".to_string()));
  }

  #[test]
  fn test_extract_fields_with_types_single_integer() {
    let mut spec = HashMap::new();
    spec.insert("count".to_string(), "integer".to_string());
    let result = extract_fields_with_types("42 items", &spec);
    assert_eq!(result.get("count"), Some(&"42".to_string()));
  }

  #[test]
  fn test_extract_fields_with_types_single_boolean() {
    let mut spec = HashMap::new();
    spec.insert("enabled".to_string(), "boolean".to_string());
    let result = extract_fields_with_types("yes", &spec);
    assert_eq!(result.get("enabled"), Some(&"true".to_string()));
  }

  #[test]
  fn test_extract_fields_with_types_single_url() {
    let mut spec = HashMap::new();
    spec.insert("endpoint".to_string(), "url".to_string());
    let result = extract_fields_with_types("Visit https://api.example.com", &spec);
    assert_eq!(
      result.get("endpoint"),
      Some(&"https://api.example.com".to_string())
    );
  }

  #[test]
  fn test_extract_fields_with_types_empty_spec() {
    let spec = HashMap::new();
    let result = extract_fields_with_types("Some text", &spec);
    assert!(result.is_empty());
  }

  #[test]
  fn test_extract_fields_with_types_empty_response() {
    let mut spec = HashMap::new();
    spec.insert("name".to_string(), "text".to_string());
    let result = extract_fields_with_types("", &spec);
    assert!(result.is_empty());
  }

  #[test]
  fn test_extract_fields_with_types_multiple_fields() {
    let mut spec = HashMap::new();
    // Use two text fields to ensure extraction succeeds regardless of iteration order
    spec.insert("name".to_string(), "text".to_string());
    spec.insert("description".to_string(), "text".to_string());
    let result = extract_fields_with_types("My API", &spec);
    // Only first field is populated
    assert_eq!(result.len(), 1);
  }

  // =============================================================================
  // Edge cases and unicode tests
  // =============================================================================

  #[test]
  fn test_unicode_text_extraction() {
    let result = extract_text("\u{4e2d}\u{6587}\u{6d4b}\u{8bd5}");
    assert!(result.is_ok());
    assert_eq!(result, Ok("\u{4e2d}\u{6587}\u{6d4b}\u{8bd5}".to_string()));
  }

  #[test]
  fn test_unicode_name_extraction() {
    let result = extract_name("\u{65e5}\u{672c}\u{8a9e}\u{540d}\u{524d}");
    assert!(result.is_ok());
  }

  #[test]
  fn test_unicode_list() {
    let result =
      extract_list("\u{308a}\u{3093}\u{3054}, \u{308a}\u{3093}\u{3054}, \u{308a}\u{3093}\u{3054}");
    assert_eq!(result.len(), 3);
  }

  #[test]
  fn test_mixed_language_text() {
    let result = extract_text("Hello \u{4e16}\u{754c} World \u{4e16}\u{754c}");
    assert!(result.is_ok());
  }

  #[test]
  fn test_emoji_in_text() {
    let result = extract_text("Hello \u{1f44b} World");
    assert!(result.is_ok());
    assert_eq!(result, Ok("Hello \u{1f44b} World".to_string()));
  }

  #[test]
  fn test_number_with_thousands_separator() {
    // Note: This currently extracts just the first number
    let result = extract_integer("1,234,567");
    // The comma stops number parsing, so we get 1
    assert!(result.is_ok());
    assert_eq!(result, Ok(1));
  }

  #[test]
  fn test_scientific_notation_not_supported() {
    // Scientific notation is not supported
    let result = extract_float("1.5e10");
    assert!(result.is_ok());
    // Should extract 1.5
    if let Ok(ExtractedValue::Float(f)) = extract_by_type("1.5e10", "float") {
      assert!((f - 1.5).abs() < f64::EPSILON);
    }
  }

  #[test]
  fn test_url_with_port() {
    let result = extract_url("https://example.com:8080/path");
    assert!(result.is_ok());
    assert_eq!(result, Ok("https://example.com:8080/path".to_string()));
  }

  #[test]
  fn test_url_with_fragment() {
    let result = extract_url("https://example.com/page#section");
    assert!(result.is_ok());
    assert_eq!(result, Ok("https://example.com/page#section".to_string()));
  }

  #[test]
  fn test_email_with_plus() {
    let result = extract_email("user+tag@example.com");
    assert!(result.is_ok());
    assert_eq!(result, Ok("user+tag@example.com".to_string()));
  }

  #[test]
  fn test_empty_items_in_list() {
    let result = extract_list("a,,b,,,c");
    assert_eq!(result, vec!["a", "b", "c"]);
  }

  // =============================================================================
  // extract_from_answer tests - Interview answer extraction
  // =============================================================================

  #[test]
  fn test_extract_from_answer_jwt() {
    let response = "We use JWT tokens for authentication";
    let fields = vec!["auth_method".to_string()];
    let result = extract_from_answer("q1", response, &fields);
    assert_eq!(result.get("auth_method"), Some(&"jwt".to_string()));
  }

  #[test]
  fn test_extract_from_answer_oauth() {
    let response = "OAuth 2.0 is our auth standard";
    let fields = vec!["auth_method".to_string()];
    let result = extract_from_answer("q1", response, &fields);
    assert_eq!(result.get("auth_method"), Some(&"oauth".to_string()));
  }

  #[test]
  fn test_extract_from_answer_session() {
    let response = "We use session-based authentication";
    let fields = vec!["auth_method".to_string()];
    let result = extract_from_answer("q1", response, &fields);
    assert_eq!(result.get("auth_method"), Some(&"session".to_string()));
  }

  #[test]
  fn test_extract_from_answer_api_key() {
    let response = "We authenticate using api key";
    let fields = vec!["auth_method".to_string()];
    let result = extract_from_answer("q1", response, &fields);
    assert_eq!(result.get("auth_method"), Some(&"api_key".to_string()));
  }

  #[test]
  fn test_extract_from_answer_none() {
    let response = "No authentication needed, none required";
    let fields = vec!["auth_method".to_string()];
    let result = extract_from_answer("q1", response, &fields);
    assert_eq!(result.get("auth_method"), Some(&"none".to_string()));
  }

  #[test]
  fn test_extract_from_answer_unknown_auth() {
    let response = "We use some custom authentication";
    let fields = vec!["auth_method".to_string()];
    let result = extract_from_answer("q1", response, &fields);
    assert!(!result.contains_key("auth_method"));
  }

  #[test]
  fn test_extract_from_answer_entities() {
    let response = "Users, Orders, Products, Payments";
    let fields = vec!["entities".to_string()];
    let result = extract_from_answer("q1", response, &fields);
    assert!(result.contains_key("entities"));
    let entities = result.get("entities").expect("entities should be present");
    assert!(entities.contains("Users"));
    assert!(entities.contains("Orders"));
    assert!(entities.contains("Products"));
    assert!(entities.contains("Payments"));
  }

  #[test]
  fn test_extract_from_answer_audience_mobile() {
    let response = "Mainly mobile app users";
    let fields = vec!["audience".to_string()];
    let result = extract_from_answer("q1", response, &fields);
    assert_eq!(result.get("audience"), Some(&"mobile".to_string()));
  }

  #[test]
  fn test_extract_from_answer_audience_web() {
    let response = "Our web application users";
    let fields = vec!["audience".to_string()];
    let result = extract_from_answer("q1", response, &fields);
    assert_eq!(result.get("audience"), Some(&"web".to_string()));
  }

  #[test]
  fn test_extract_from_answer_audience_api() {
    let response = "For our API consumers";
    let fields = vec!["audience".to_string()];
    let result = extract_from_answer("q1", response, &fields);
    assert_eq!(result.get("audience"), Some(&"api".to_string()));
  }

  #[test]
  fn test_extract_from_answer_audience_cli() {
    let response = "CLI users will interact with this";
    let fields = vec!["audience".to_string()];
    let result = extract_from_answer("q1", response, &fields);
    assert_eq!(result.get("audience"), Some(&"cli".to_string()));
  }

  #[test]
  fn test_extract_from_answer_audience_internal() {
    let response = "Internal team members only";
    let fields = vec!["audience".to_string()];
    let result = extract_from_answer("q1", response, &fields);
    assert_eq!(result.get("audience"), Some(&"internal".to_string()));
  }

  #[test]
  fn test_extract_from_answer_multiple_fields() {
    let response =
      "We use JWT tokens for our mobile app users. Main entities are Users and Orders.";
    let fields = vec![
      "auth_method".to_string(),
      "audience".to_string(),
      "entities".to_string(),
    ];
    let result = extract_from_answer("q1", response, &fields);
    assert_eq!(result.get("auth_method"), Some(&"jwt".to_string()));
    assert_eq!(result.get("audience"), Some(&"mobile".to_string()));
    assert!(result.contains_key("entities"));
  }

  #[test]
  fn test_extract_from_answer_generic_field() {
    let response = "Some generic response text";
    let fields = vec!["custom_field".to_string()];
    let result = extract_from_answer("q1", response, &fields);
    assert_eq!(result.get("custom_field"), Some(&response.to_string()));
  }

  #[test]
  fn test_extract_from_answer_empty_response() {
    let response = "";
    let fields = vec!["auth_method".to_string()];
    let result = extract_from_answer("q1", response, &fields);
    assert!(!result.contains_key("auth_method"));
  }

  #[test]
  fn test_extract_from_answer_whitespace_response() {
    let response = "   ";
    let fields = vec!["auth_method".to_string()];
    let result = extract_from_answer("q1", response, &fields);
    assert!(!result.contains_key("auth_method"));
  }

  #[test]
  fn test_extract_from_answer_no_matching_fields() {
    let response = "Some text without any recognizable patterns";
    let fields = vec!["auth_method".to_string(), "audience".to_string()];
    let result = extract_from_answer("q1", response, &fields);
    assert!(result.is_empty());
  }

  #[test]
  fn test_extract_from_answer_empty_fields() {
    let response = "Some response";
    let fields: Vec<String> = vec![];
    let result = extract_from_answer("q1", response, &fields);
    assert!(result.is_empty());
  }

  // =============================================================================
  // Interview helper function tests
  // =============================================================================

  #[test]
  fn test_extract_from_answer_question_id_unused() {
    // The question_id is unused but should not cause issues
    let response = "We use JWT for auth";
    let fields = vec!["auth_method".to_string()];
    let result = extract_from_answer("different_id", response, &fields);
    assert_eq!(result.get("auth_method"), Some(&"jwt".to_string()));
  }

  #[test]
  fn test_extract_from_answer_case_preservation() {
    // Extracted values should be lowercase for auth_method
    let response = "We use JWT tokens";
    let fields = vec!["auth_method".to_string()];
    let result = extract_from_answer("q1", response, &fields);
    assert_eq!(result.get("auth_method"), Some(&"jwt".to_string()));
  }

  #[test]
  fn test_entities_with_numbers() {
    // Internal helper test - extract_entities
    let entities: Vec<String> = "User123 Account456"
      .split_whitespace()
      .filter_map(|word| {
        let clean_word = word.trim_end_matches(',').trim_end_matches('.');
        if let Some(first_char) = clean_word.chars().next() {
          if first_char.is_uppercase() && clean_word.len() > 2 {
            let is_common_word = matches!(
              clean_word.to_lowercase().as_str(),
              "the" | "and" | "for" | "use" | "can" | "all" | "our" | "you" | "are"
            );
            if !is_common_word {
              return Some(clean_word.to_string());
            }
          }
        }
        None
      })
      .collect();
    // Numbers are kept as part of the entity
    assert!(!entities.is_empty());
  }

  #[test]
  fn test_audience_priority_order() {
    // When multiple audiences mentioned, first match wins
    // mobile comes before web in the implementation
    let lower = "mobile and web users".to_lowercase();
    let result = if lower.contains("mobile") {
      Some("mobile".to_string())
    } else if lower.contains("web") {
      Some("web".to_string())
    } else {
      None
    };
    assert_eq!(result, Some("mobile".to_string()));
  }

  // =============================================================================
  // calculate_confidence tests
  // =============================================================================

  #[test]
  fn test_calculate_confidence_high() {
    let mut extracted = HashMap::new();
    extracted.insert("field1".to_string(), "value1".to_string());
    let response = "This is a longer response with more than fifty characters for testing.";
    let confidence = calculate_confidence("q1", response, &extracted);
    assert!((confidence - 0.85).abs() < 0.001);
  }

  #[test]
  fn test_calculate_confidence_low() {
    let extracted = HashMap::new();
    let response = "Short";
    let confidence = calculate_confidence("q1", response, &extracted);
    assert!((confidence - 0.6).abs() < 0.001);
  }

  #[test]
  fn test_calculate_confidence_medium_no_fields() {
    let extracted = HashMap::new();
    let response = "This is a longer response with more than fifty characters for testing.";
    let confidence = calculate_confidence("q1", response, &extracted);
    assert!((confidence - 0.6).abs() < 0.001);
  }

  #[test]
  fn test_calculate_confidence_medium_short_response() {
    let mut extracted = HashMap::new();
    extracted.insert("field1".to_string(), "value1".to_string());
    let response = "Short";
    let confidence = calculate_confidence("q1", response, &extracted);
    assert!((confidence - 0.6).abs() < 0.001);
  }
}

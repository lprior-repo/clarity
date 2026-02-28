//! Answer Extraction Logic
//!
//! Extracts structured data from free-text interview responses.
//! This is where the "AI adapts" - the extraction logic learns from patterns.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use std::collections::HashMap;

/// Extract fields from answer text.
///
/// Given a response and a list of fields to extract, this function attempts
/// to extract structured data based on pattern matching for known field types.
///
/// # Arguments
/// * `_question_id` - The ID of the question (unused but kept for API compatibility)
/// * `response` - The free-text response from the user
/// * `extract_fields` - List of field names to attempt to extract
///
/// # Returns
/// A HashMap containing the successfully extracted field-value pairs.
/// Fields that couldn't be extracted are simply not included in the result.
#[must_use]
pub fn extract_from_answer(
  _question_id: &str,
  response: &str,
  extract_fields: &[String],
) -> HashMap<String, String> {
  let mut result = HashMap::new();

  for field in extract_fields {
    if let Some(value) = simple_extract(field, response) {
      result.insert(field.clone(), value);
    }
  }

  result
}

/// Simple extraction patterns - can be extended to use NLP/LLM.
///
/// Returns `Some(value)` if extraction successful, `None` otherwise.
fn simple_extract(field: &str, text: &str) -> Option<String> {
  match field {
    // Auth-related extractions
    "auth_method" => extract_auth_method(text),

    // Entity/data model extractions
    "entities" => extract_entities(text),

    // Audience extraction
    "audience" => extract_audience(text),

    // Generic field - just return the trimmed text if it's substantial
    _ => {
      let trimmed = text.trim();
      if trimmed.is_empty() {
        None
      } else {
        Some(trimmed.to_string())
      }
    }
  }
}

/// Extract authentication method from text.
///
/// Recognizes: jwt, oauth, session, api_key, none
fn extract_auth_method(text: &str) -> Option<String> {
  let lower = text.to_lowercase();

  if lower.contains("jwt") {
    Some("jwt".to_string())
  } else if lower.contains("oauth") {
    Some("oauth".to_string())
  } else if lower.contains("session") {
    Some("session".to_string())
  } else if lower.contains("api key") || lower.contains("api_key") {
    Some("api_key".to_string())
  } else if lower.contains("none") {
    Some("none".to_string())
  } else {
    None
  }
}

/// Extract entity names from text.
///
/// Looks for capitalized words which are likely entity names.
/// Filters out common words and short words.
fn extract_entities(text: &str) -> Option<String> {
  let words: Vec<&str> = text.split_whitespace().collect();

  let entities: Vec<String> = words
    .into_iter()
    .filter_map(|word| {
      // Remove trailing punctuation
      let clean_word = word.trim_end_matches(',').trim_end_matches('.');

      // Check if it starts with uppercase and is long enough
      if let Some(first_char) = clean_word.chars().next() {
        if first_char.is_uppercase() && clean_word.len() > 2 {
          // Filter out common non-entity words
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

  if entities.is_empty() {
    None
  } else {
    Some(entities.join(", "))
  }
}

/// Extract audience type from text.
///
/// Recognizes: mobile, web, api, cli, internal
fn extract_audience(text: &str) -> Option<String> {
  let lower = text.to_lowercase();

  if lower.contains("mobile") {
    Some("mobile".to_string())
  } else if lower.contains("web") {
    Some("web".to_string())
  } else if lower.contains("api") {
    Some("api".to_string())
  } else if lower.contains("cli") {
    Some("cli".to_string())
  } else if lower.contains("internal") {
    Some("internal".to_string())
  } else {
    None
  }
}

/// Calculate confidence in answer extraction (0-1).
///
/// Longer responses with more extracted fields result in higher confidence.
#[must_use]
pub fn calculate_confidence(
  _question_id: &str,
  response: &str,
  extracted: &HashMap<String, String>,
) -> f64 {
  let response_length = response.trim().len();
  let field_count = extracted.len();

  // Longer responses with more fields = higher confidence
  if response_length > 50 && field_count > 0 {
    0.85
  } else {
    0.6
  }
}

#[cfg(test)]
mod tests {
  use super::*;

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
  fn test_extract_from_answer_entities() {
    let response = "Users, Orders, Products, Payments";
    let fields = vec!["entities".to_string()];
    let result = extract_from_answer("q1", response, &fields);
    assert!(result.contains_key("entities"));
    let entities = result.get("entities").unwrap();
    assert!(entities.contains("Users"));
    assert!(entities.contains("Orders"));
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
  fn test_extract_from_answer_multiple_fields() {
    let response = "We use JWT tokens for our mobile app users. Main entities are Users and Orders.";
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

  #[test]
  fn test_extract_auth_method_session() {
    let response = "We use session-based authentication";
    let result = extract_auth_method(response);
    assert_eq!(result, Some("session".to_string()));
  }

  #[test]
  fn test_extract_auth_method_api_key() {
    let response = "We authenticate using api key";
    let result = extract_auth_method(response);
    assert_eq!(result, Some("api_key".to_string()));
  }

  #[test]
  fn test_extract_auth_method_none() {
    let response = "No authentication needed, none required";
    let result = extract_auth_method(response);
    assert_eq!(result, Some("none".to_string()));
  }

  #[test]
  fn test_extract_auth_method_unknown() {
    let response = "We use some custom authentication";
    let result = extract_auth_method(response);
    assert!(result.is_none());
  }

  #[test]
  fn test_extract_audience_api() {
    let response = "For our API consumers";
    let result = extract_audience(response);
    assert_eq!(result, Some("api".to_string()));
  }

  #[test]
  fn test_extract_audience_cli() {
    let response = "CLI users will interact with this";
    let result = extract_audience(response);
    assert_eq!(result, Some("cli".to_string()));
  }

  #[test]
  fn test_extract_audience_internal() {
    let response = "Internal team members only";
    let result = extract_audience(response);
    assert_eq!(result, Some("internal".to_string()));
  }

  #[test]
  fn test_extract_audience_unknown() {
    let response = "Some random users";
    let result = extract_audience(response);
    assert!(result.is_none());
  }

  #[test]
  fn test_extract_entities_filters_common_words() {
    let response = "The And For Use Can All Our You Are";
    let result = extract_entities(response);
    assert!(result.is_none());
  }

  #[test]
  fn test_extract_entities_handles_punctuation() {
    let response = "Users, Orders. Products!";
    let fields = vec!["entities".to_string()];
    let result = extract_from_answer("q1", response, &fields);
    assert!(result.contains_key("entities"));
    let entities = result.get("entities").unwrap();
    assert!(entities.contains("Users"));
    assert!(entities.contains("Orders"));
    assert!(entities.contains("Products"));
  }

  #[test]
  fn test_extract_entities_short_words_filtered() {
    let response = "A B Cd Efg"; // Cd is 2 chars, should be filtered
    let result = extract_entities(response);
    // Only "Efg" should be extracted (3 chars, starts with uppercase)
    assert_eq!(result, Some("Efg".to_string()));
  }
}

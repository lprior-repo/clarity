//! Interview-Specific Answer Extraction
//!
//! This module provides extraction functions specifically designed for
//! interview responses, including domain-specific field extractors like
//! auth_method, entities, and audience detection.

use std::collections::HashMap;

/// Extracts fields from answer text for interview responses.
///
/// This function is the main entry point for extracting structured data from
/// interview answers. It uses pattern matching to identify known field types
/// and extracts appropriate values.
///
/// # Arguments
///
/// * `_question_id` - The ID of the question (unused but kept for API compatibility)
/// * `response` - The free-text response from the user
/// * `extract_fields` - List of field names to attempt to extract
///
/// # Returns
///
/// A `HashMap` containing the successfully extracted field-value pairs.
/// Fields that couldn't be extracted are simply not included in the result.
///
/// # Supported Field Types
///
/// | Field Name | Extraction Method |
/// |------------|-------------------|
/// | `auth_method` | Recognizes jwt, oauth, session, `api_key`, none |
/// | `entities` | Extracts capitalized words (entity names) |
/// | `audience` | Recognizes mobile, web, api, cli, internal |
/// | (any other) | Returns trimmed text if non-empty |
///
/// # Example
///
/// ```ignore
/// use clarity_web::intent::interview::answer_extraction::interview::extract_from_answer;
///
/// let response = "We use JWT tokens for our mobile app. Main entities are Users and Orders.";
/// let fields = vec!["auth_method".to_string(), "audience".to_string(), "entities".to_string()];
/// let result = extract_from_answer("q1", response, &fields);
///
/// assert_eq!(result.get("auth_method"), Some(&"jwt".to_string()));
/// assert_eq!(result.get("audience"), Some(&"mobile".to_string()));
/// assert!(result.contains_key("entities"));
/// ```
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

/// Dispatches to the appropriate extraction function based on field name.
///
/// This internal function maps field names to their extraction handlers.
/// For unknown field names, it returns the trimmed text as a fallback.
///
/// # Returns
///
/// - `Some(value)` if extraction was successful
/// - `None` if extraction failed or produced no meaningful result
fn simple_extract(field: &str, text: &str) -> Option<String> {
  match field {
    // Authentication method detection
    "auth_method" => extract_auth_method(text),

    // Entity/data model name extraction
    "entities" => extract_entities(text),

    // Target audience detection
    "audience" => extract_audience(text),

    // Generic fallback: return trimmed text if non-empty
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

/// Extracts authentication method from text.
///
/// Performs case-insensitive substring matching to identify common
/// authentication methods mentioned in the response.
///
/// # Recognized Values
///
/// | Pattern | Extracted Value |
/// |---------|-----------------|
/// | "jwt" | "jwt" |
/// | "oauth" | "oauth" |
/// | "session" | "session" |
/// | "api key" or "`api_key`" | "`api_key`" |
/// | "none" | "none" |
///
/// # Priority
///
/// Patterns are checked in order; the first match is returned.
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

/// Extracts entity names from text.
///
/// Identifies potential entity names by looking for capitalized words.
/// This heuristic works well for domain entities like "Users", "Orders", etc.
///
/// # Algorithm
///
/// 1. Split text into words
/// 2. For each word, strip trailing punctuation
/// 3. Check if word starts with uppercase and is > 2 characters
/// 4. Filter out common non-entity words (the, and, for, etc.)
/// 5. Join remaining words with ", "
///
/// # Returns
///
/// - `Some(entities)` with comma-separated entity names if any found
/// - `None` if no entity-like words are found
fn extract_entities(text: &str) -> Option<String> {
  let entities: Vec<String> = text
    .split_whitespace()
    .filter_map(|word| {
      // Strip trailing punctuation that might follow entity names
      let clean_word = word.trim_end_matches(',').trim_end_matches('.');

      // Entity candidates must start with uppercase and be meaningful length
      if let Some(first_char) = clean_word.chars().next() {
        if first_char.is_uppercase() && clean_word.len() > 2 {
          // Exclude common words that happen to be capitalized
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

/// Extracts audience type from text.
///
/// Performs case-insensitive substring matching to identify the target
/// audience for the API or application.
///
/// # Recognized Values
///
/// | Pattern | Extracted Value |
/// |---------|-----------------|
/// | "mobile" | "mobile" |
/// | "web" | "web" |
/// | "api" | "api" |
/// | "cli" | "cli" |
/// | "internal" | "internal" |
///
/// # Priority
///
/// Patterns are checked in order; the first match is returned.
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

/// Calculates confidence score for answer extraction quality.
///
/// This function provides a heuristic confidence score (0.0 to 1.0) indicating
/// how reliable the extraction results are likely to be. Higher scores indicate
/// more confident extractions.
///
/// # Scoring Logic
///
/// | Condition | Confidence |
/// |-----------|------------|
/// | Response > 50 chars AND fields extracted | 0.85 |
/// | Otherwise | 0.60 |
///
/// # Arguments
///
/// * `_question_id` - The ID of the question (reserved for future use)
/// * `response` - The original response text
/// * `extracted` - The `HashMap` of extracted field-value pairs
///
/// # Example
///
/// ```ignore
/// use std::collections::HashMap;
/// use clarity_web::intent::interview::answer_extraction::interview::calculate_confidence;
///
/// let mut extracted = HashMap::new();
/// extracted.insert("auth_method".to_string(), "jwt".to_string());
/// let response = "This is a longer response with more than fifty characters for testing.";
///
/// let confidence = calculate_confidence("q1", response, &extracted);
/// assert!((confidence - 0.85).abs() < 0.001);
/// ```
///
/// # Future Improvements
///
/// The confidence calculation could be enhanced to consider:
/// - Number of fields successfully extracted vs. requested
/// - Quality of individual extractions (e.g., valid URL, valid email)
/// - Response coherence and structure
#[must_use]
pub fn calculate_confidence<S: std::hash::BuildHasher>(
  _question_id: &str,
  response: &str,
  extracted: &HashMap<String, String, S>,
) -> f64 {
  let response_length = response.trim().len();
  let _field_count = extracted.len();

  // Heuristic: longer responses with successful extractions indicate
  // more thoughtful answers and thus higher confidence
  if response_length > 50 && !extracted.is_empty() {
    0.85
  } else {
    0.6
  }
}

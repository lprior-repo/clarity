#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

//! Question types for surveys and forms
//!
//! Provides a comprehensive set of question types with validation and serialization support.
//! All question construction returns Result<T, QuestionTypeError> - no unwraps, no panics.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Error types for question construction and validation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestionTypeError {
  /// A required field is missing
  #[serde(rename = "missing_field")]
  MissingField { field: String },

  /// Validation failed with a specific reason
  #[serde(rename = "validation")]
  Validation { reason: String },

  /// Serialization/deserialization error
  #[serde(rename = "serialization")]
  Serialization { reason: String },
}

impl fmt::Display for QuestionTypeError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::MissingField { field } => write!(f, "Missing required field: {field}"),
      Self::Validation { reason } => write!(f, "Validation failed: {reason}"),
      Self::Serialization { reason } => write!(f, "Serialization error: {reason}"),
    }
  }
}

impl std::error::Error for QuestionTypeError {}

/// Question types supported by the system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum QuestionType {
  /// Simple text input
  Text {
    prompt: String,
    default: Option<String>,
  },

  /// Multiple choice question
  MultipleChoice {
    prompt: String,
    options: Vec<String>,
    default: Option<usize>,
  },

  /// Boolean (yes/no) question
  Boolean {
    prompt: String,
    default: Option<bool>,
  },

  /// Numeric range question
  NumericRange {
    prompt: String,
    min: i64,
    max: i64,
    default: Option<i64>,
  },

  /// Date question
  Date {
    prompt: String,
    default: Option<String>, // ISO 8601 date string
  },

  /// Long text (multi-line) question
  LongText {
    prompt: String,
    default: Option<String>,
    max_length: usize,
  },

  /// Rating scale question
  Rating { prompt: String, min: i64, max: i64 },

  /// Code/snippet question
  Code {
    prompt: String,
    language: String,
    default: Option<String>,
  },

  /// File upload question
  FileUpload {
    prompt: String,
    allowed_types: Vec<String>, // e.g., ["pdf", "docx"]
    required: bool,
  },

  /// Ranking/ordering question
  Ranking {
    prompt: String,
    options: Vec<String>,
  },
}

impl QuestionType {
  /// Create a text question
  ///
  /// # Errors
  /// Returns `QuestionTypeError::MissingField` if prompt is empty or whitespace-only
  pub fn text(prompt: &str, default: Option<String>) -> Result<Self, QuestionTypeError> {
    let trimmed = prompt.trim();
    if trimmed.is_empty() {
      return Err(QuestionTypeError::MissingField {
        field: "prompt".to_string(),
      });
    }

    Ok(Self::Text {
      prompt: trimmed.to_string(),
      default,
    })
  }

  /// Create a multiple choice question
  ///
  /// # Errors
  /// - Returns `QuestionTypeError::MissingField` if prompt is empty
  /// - Returns `QuestionTypeError::Validation` if options are empty or contain duplicates
  /// - Returns `QuestionTypeError::Validation` if default index is out of bounds
  pub fn multiple_choice(
    prompt: &str,
    options: Vec<String>,
    default: Option<usize>,
  ) -> Result<Self, QuestionTypeError> {
    let trimmed = prompt.trim();
    if trimmed.is_empty() {
      return Err(QuestionTypeError::MissingField {
        field: "prompt".to_string(),
      });
    }

    if options.is_empty() {
      return Err(QuestionTypeError::Validation {
        reason: "options cannot be empty".to_string(),
      });
    }

    // Check for duplicates
    let unique_options: std::collections::HashSet<_> = options.iter().collect();
    if unique_options.len() != options.len() {
      return Err(QuestionTypeError::Validation {
        reason: "options must be unique (found duplicates)".to_string(),
      });
    }

    // Validate default index
    if let Some(idx) = default {
      if idx >= options.len() {
        return Err(QuestionTypeError::Validation {
          reason: format!(
            "default index {} out of bounds (valid range: 0-{})",
            idx,
            options.len() - 1
          ),
        });
      }
    }

    Ok(Self::MultipleChoice {
      prompt: trimmed.to_string(),
      options,
      default,
    })
  }

  /// Create a boolean question
  ///
  /// # Errors
  /// Returns `QuestionTypeError::MissingField` if prompt is empty
  pub fn boolean(prompt: &str, default: Option<bool>) -> Result<Self, QuestionTypeError> {
    let trimmed = prompt.trim();
    if trimmed.is_empty() {
      return Err(QuestionTypeError::MissingField {
        field: "prompt".to_string(),
      });
    }

    Ok(Self::Boolean {
      prompt: trimmed.to_string(),
      default,
    })
  }

  /// Create a numeric range question
  ///
  /// # Errors
  /// - Returns `QuestionTypeError::MissingField` if prompt is empty
  /// - Returns `QuestionTypeError::Validation` if min > max or default is out of range
  pub fn numeric_range(
    prompt: &str,
    min: i64,
    max: i64,
    default: Option<i64>,
  ) -> Result<Self, QuestionTypeError> {
    let trimmed = prompt.trim();
    if trimmed.is_empty() {
      return Err(QuestionTypeError::MissingField {
        field: "prompt".to_string(),
      });
    }

    if min > max {
      return Err(QuestionTypeError::Validation {
        reason: format!("min ({min}) cannot be greater than max ({max})"),
      });
    }

    // Validate default is within range
    if let Some(val) = default {
      if val < min || val > max {
        return Err(QuestionTypeError::Validation {
          reason: format!("default value {val} is outside valid range [{min}, {max}]"),
        });
      }
    }

    Ok(Self::NumericRange {
      prompt: trimmed.to_string(),
      min,
      max,
      default,
    })
  }

  /// Create a date question
  ///
  /// # Errors
  /// Returns `QuestionTypeError::MissingField` if prompt is empty
  pub fn date(prompt: &str, default: Option<String>) -> Result<Self, QuestionTypeError> {
    let trimmed = prompt.trim();
    if trimmed.is_empty() {
      return Err(QuestionTypeError::MissingField {
        field: "prompt".to_string(),
      });
    }

    Ok(Self::Date {
      prompt: trimmed.to_string(),
      default,
    })
  }

  /// Create a long text question
  ///
  /// # Errors
  /// Returns `QuestionTypeError::MissingField` if prompt is empty
  pub fn long_text(
    prompt: &str,
    default: Option<String>,
    max_length: usize,
  ) -> Result<Self, QuestionTypeError> {
    let trimmed = prompt.trim();
    if trimmed.is_empty() {
      return Err(QuestionTypeError::MissingField {
        field: "prompt".to_string(),
      });
    }

    Ok(Self::LongText {
      prompt: trimmed.to_string(),
      default,
      max_length,
    })
  }

  /// Create a rating scale question
  ///
  /// # Errors
  /// - Returns `QuestionTypeError::MissingField` if prompt is empty
  /// - Returns `QuestionTypeError::Validation` if min >= max
  pub fn rating(prompt: &str, min: i64, max: i64) -> Result<Self, QuestionTypeError> {
    let trimmed = prompt.trim();
    if trimmed.is_empty() {
      return Err(QuestionTypeError::MissingField {
        field: "prompt".to_string(),
      });
    }

    if min >= max {
      return Err(QuestionTypeError::Validation {
        reason: format!("min ({min}) must be less than max ({max})"),
      });
    }

    Ok(Self::Rating {
      prompt: trimmed.to_string(),
      min,
      max,
    })
  }

  /// Create a code question
  ///
  /// # Errors
  /// Returns `QuestionTypeError::MissingField` if prompt is empty
  pub fn code(
    prompt: &str,
    language: &str,
    default: Option<String>,
  ) -> Result<Self, QuestionTypeError> {
    let trimmed = prompt.trim();
    if trimmed.is_empty() {
      return Err(QuestionTypeError::MissingField {
        field: "prompt".to_string(),
      });
    }

    Ok(Self::Code {
      prompt: trimmed.to_string(),
      language: language.to_string(),
      default,
    })
  }

  /// Create a file upload question
  ///
  /// # Errors
  /// Returns `QuestionTypeError::MissingField` if prompt is empty
  pub fn file_upload(
    prompt: &str,
    allowed_types: Vec<String>,
    required: bool,
  ) -> Result<Self, QuestionTypeError> {
    let trimmed = prompt.trim();
    if trimmed.is_empty() {
      return Err(QuestionTypeError::MissingField {
        field: "prompt".to_string(),
      });
    }

    Ok(Self::FileUpload {
      prompt: trimmed.to_string(),
      allowed_types,
      required,
    })
  }

  /// Create a ranking question
  ///
  /// # Errors
  /// - Returns `QuestionTypeError::MissingField` if prompt is empty
  /// - Returns `QuestionTypeError::Validation` if options are empty or contain duplicates
  pub fn ranking(prompt: &str, options: Vec<String>) -> Result<Self, QuestionTypeError> {
    let trimmed = prompt.trim();
    if trimmed.is_empty() {
      return Err(QuestionTypeError::MissingField {
        field: "prompt".to_string(),
      });
    }

    if options.is_empty() {
      return Err(QuestionTypeError::Validation {
        reason: "options cannot be empty".to_string(),
      });
    }

    // Check for duplicates
    let unique_options: std::collections::HashSet<_> = options.iter().collect();
    if unique_options.len() != options.len() {
      return Err(QuestionTypeError::Validation {
        reason: "options must be unique (found duplicates)".to_string(),
      });
    }

    Ok(Self::Ranking {
      prompt: trimmed.to_string(),
      options,
    })
  }

  /// Get the prompt text for this question
  #[must_use]
  pub fn prompt(&self) -> &str {
    match self {
      Self::Text { prompt, .. } => prompt,
      Self::MultipleChoice { prompt, .. } => prompt,
      Self::Boolean { prompt, .. } => prompt,
      Self::NumericRange { prompt, .. } => prompt,
      Self::Date { prompt, .. } => prompt,
      Self::LongText { prompt, .. } => prompt,
      Self::Rating { prompt, .. } => prompt,
      Self::Code { prompt, .. } => prompt,
      Self::FileUpload { prompt, .. } => prompt,
      Self::Ranking { prompt, .. } => prompt,
    }
  }

  /// Validate the question structure
  ///
  /// # Errors
  /// Returns `QuestionTypeError::Validation` if the question is invalid
  pub fn validate(&self) -> Result<(), QuestionTypeError> {
    match self {
      Self::MultipleChoice {
        options, default, ..
      } => {
        if options.is_empty() {
          return Err(QuestionTypeError::Validation {
            reason: "options cannot be empty".to_string(),
          });
        }
        if let Some(idx) = default {
          if *idx >= options.len() {
            return Err(QuestionTypeError::Validation {
              reason: format!("default index {idx} out of bounds"),
            });
          }
        }
        Ok(())
      }
      Self::NumericRange {
        min, max, default, ..
      } => {
        if min > max {
          return Err(QuestionTypeError::Validation {
            reason: format!("min ({min}) cannot be greater than max ({max})"),
          });
        }
        if let Some(val) = default {
          if *val < *min || *val > *max {
            return Err(QuestionTypeError::Validation {
              reason: format!("default {val} outside range [{min}, {max}]"),
            });
          }
        }
        Ok(())
      }
      Self::Rating { min, max, .. } => {
        if min >= max {
          return Err(QuestionTypeError::Validation {
            reason: format!("min ({min}) must be less than max ({max})"),
          });
        }
        Ok(())
      }
      Self::Ranking { options, .. } => {
        if options.is_empty() {
          return Err(QuestionTypeError::Validation {
            reason: "options cannot be empty".to_string(),
          });
        }
        Ok(())
      }
      _ => Ok(()),
    }
  }

  /// Validate an answer for this question
  ///
  /// # Errors
  /// Returns `QuestionTypeError::Validation` if the answer is invalid
  pub fn validate_answer(&self, answer: &str) -> Result<(), QuestionTypeError> {
    match self {
      Self::LongText { max_length, .. } => {
        if answer.len() > *max_length {
          return Err(QuestionTypeError::Validation {
            reason: format!(
              "answer length {} exceeds maximum length {}",
              answer.len(),
              max_length
            ),
          });
        }
        Ok(())
      }
      _ => Ok(()), // Other types don't validate answers in this basic implementation
    }
  }

  /// Display a human-readable prompt with type indicator
  #[must_use]
  pub fn display_prompt(&self) -> String {
    let type_indicator = match self {
      Self::Text { .. } => "Text",
      Self::MultipleChoice { .. } => "Multiple Choice",
      Self::Boolean { .. } => "Boolean",
      Self::NumericRange { .. } => "Numeric Range",
      Self::Date { .. } => "Date",
      Self::LongText { .. } => "Long Text",
      Self::Rating { .. } => "Rating",
      Self::Code { .. } => "Code",
      Self::FileUpload { .. } => "File Upload",
      Self::Ranking { .. } => "Ranking",
    };

    format!("[{}] {}", type_indicator, self.prompt())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  // Basic smoke tests - comprehensive tests are in tests/question_types_test.rs
  #[test]
  fn test_text_question_creation() {
    let result = QuestionType::text("Test prompt", None);
    assert!(result.is_ok());
  }

  #[test]
  fn test_empty_prompt_rejected() {
    let result = QuestionType::text("", None);
    assert!(result.is_err());
    assert!(matches!(
      result,
      Err(QuestionTypeError::MissingField { field })
      if field == "prompt"
    ));
  }

  #[test]
  fn test_whitespace_prompt_rejected() {
    let result = QuestionType::text("   ", None);
    assert!(result.is_err());
  }

  #[test]
  fn test_multiple_choice_basic() {
    let result = QuestionType::multiple_choice("Choose", vec!["A".to_string()], Some(0));
    assert!(result.is_ok());
  }

  #[test]
  fn test_empty_options_rejected() {
    let result = QuestionType::multiple_choice("Choose", vec![], None);
    assert!(result.is_err());
  }

  #[test]
  fn test_duplicate_options_rejected() {
    let result =
      QuestionType::multiple_choice("Choose", vec!["A".to_string(), "A".to_string()], None);
    assert!(result.is_err());
  }

  #[test]
  fn test_out_of_bounds_default_rejected() {
    let result = QuestionType::multiple_choice("Choose", vec!["A".to_string()], Some(5));
    assert!(result.is_err());
  }

  #[test]
  fn test_numeric_range_basic() {
    let result = QuestionType::numeric_range("Rate", 1, 5, Some(3));
    assert!(result.is_ok());
  }

  #[test]
  fn test_invalid_range_rejected() {
    let result = QuestionType::numeric_range("Rate", 10, 1, Some(5));
    assert!(result.is_err());
  }

  #[test]
  fn test_negative_range_supported() {
    let result = QuestionType::numeric_range("Temp", -10, 10, Some(0));
    assert!(result.is_ok());
  }

  #[test]
  fn test_rating_basic() {
    let result = QuestionType::rating("Rate", 1, 5);
    assert!(result.is_ok());
  }

  #[test]
  fn test_rating_min_equals_max_rejected() {
    let result = QuestionType::rating("Rate", 5, 5);
    assert!(result.is_err());
  }

  #[test]
  fn test_prompt_returns_correct_text() {
    let q = QuestionType::text("Test prompt", None).unwrap();
    assert_eq!(q.prompt(), "Test prompt");
  }

  #[test]
  fn test_display_prompt_includes_type() {
    let q = QuestionType::text("Test", None).unwrap();
    let display = q.display_prompt();
    assert!(display.contains("[Text]"));
    assert!(display.contains("Test"));
  }

  #[test]
  fn test_validate_passes_for_valid_question() {
    let q = QuestionType::text("Test", None).unwrap();
    assert!(q.validate().is_ok());
  }

  #[test]
  fn test_validate_answer_enforces_max_length() {
    let q = QuestionType::long_text("Test", None, 10).unwrap();
    let result = q.validate_answer("This is way too long");
    assert!(result.is_err());
  }

  #[test]
  fn test_equality_same_data() {
    let q1 = QuestionType::text("Same", None).unwrap();
    let q2 = QuestionType::text("Same", None).unwrap();
    assert_eq!(q1, q2);
  }

  #[test]
  fn test_inequality_different_data() {
    let q1 = QuestionType::text("One", None).unwrap();
    let q2 = QuestionType::text("Two", None).unwrap();
    assert_ne!(q1, q2);
  }
}

//! Output formatter module for Clarity
//!
//! This module provides a trait-based formatter system that supports multiple
//! output formats (JSON, Markdown, Plain Text) for domain objects like Interviews.
//!
//! # Design Principles
//!
//! - **Trait-based**: All formatters implement the `OutputFormatter` trait
//! - **Zero panic**: No unwrap/expect/panic in implementation
//! - **Result types**: All formatting operations return `Result<String, FormatError>`
//! - **Immutable**: Formatters don't modify input data
//! - **Thread-safe**: Formatters can be shared across threads
//!
//! # Example
//!
//! ```rust
//! use clarity_core::formatter::{OutputFormat, OutputFormatter};
//! use clarity_core::interview::Interview;
//!
//! let interview = get_interview()?;
//! let formatter = OutputFormat::Json.formatter();
//! let json_output = formatter.format(&interview)?;
//! # Ok::<(), clarity_core::formatter::FormatError>(())
//! ```

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::interview::Interview;
use std::fmt;
use std::fmt::Write;

/// Error types for formatting operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatError {
  /// Serialization failed for the target format
  SerializationFailed(String),

  /// Data contains invalid UTF-8 sequences
  InvalidUtf8,

  /// Required field is missing from data
  MissingField(String),

  /// Data contains circular references
  CircularReference,

  /// Requested format is not supported
  UnsupportedFormat(String),

  /// IO error during formatting
  IoError(String),
}

impl fmt::Display for FormatError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::SerializationFailed(msg) => write!(f, "Serialization failed: {msg}"),
      Self::InvalidUtf8 => write!(f, "Data contains invalid UTF-8 sequences"),
      Self::MissingField(field) => write!(f, "Missing required field: {field}"),
      Self::CircularReference => write!(f, "Data contains circular references"),
      Self::UnsupportedFormat(format) => write!(f, "Unsupported format: {format}"),
      Self::IoError(msg) => write!(f, "IO error: {msg}"),
    }
  }
}

impl std::error::Error for FormatError {}

/// Generic output formatter trait
///
/// All formatters implement this trait for consistent interface.
pub trait OutputFormatter<T> {
  /// Format the input data as a String
  ///
  /// # Errors
  /// Returns `FormatError` if formatting fails
  fn format(&self, data: &T) -> Result<String, FormatError>;

  /// Get the format name (e.g., "json", "markdown", "text")
  #[must_use]
  fn format_name(&self) -> &str;

  /// Get the MIME type for this format
  #[must_use]
  fn mime_type(&self) -> &str;
}

/// Format enum for runtime format selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
  /// JSON format
  Json,
  /// Markdown format
  Markdown,
  /// Plain text format
  PlainText,
}

impl OutputFormat {
  /// Parse format from string
  ///
  /// # Errors
  /// Returns `FormatError::UnsupportedFormat` if the format string is not recognized
  pub fn from_str(s: &str) -> Result<Self, FormatError> {
    match s.to_lowercase().as_str() {
      "json" => Ok(Self::Json),
      "markdown" | "md" => Ok(Self::Markdown),
      "text" | "txt" => Ok(Self::PlainText),
      _ => Err(FormatError::UnsupportedFormat(s.to_string())),
    }
  }

  /// Get formatter for this format
  #[must_use]
  pub fn formatter(&self) -> Box<dyn OutputFormatter<Interview>> {
    match self {
      Self::Json => Box::new(JsonFormatter::new()),
      Self::Markdown => Box::new(MarkdownFormatter::new()),
      Self::PlainText => Box::new(PlainTextFormatter::new()),
    }
  }
}

/// JSON formatter implementation
///
/// Formats data as JSON with optional pretty-printing.
#[derive(Debug, Clone)]
pub struct JsonFormatter {
  pretty: bool,
}

impl JsonFormatter {
  /// Create a new JSON formatter (compact mode by default)
  #[must_use]
  pub const fn new() -> Self {
    Self { pretty: false }
  }

  /// Create a formatter with pretty-printing enabled
  #[must_use]
  pub const fn with_pretty(pretty: bool) -> Self {
    Self { pretty }
  }

  /// Create a formatter that produces pretty-printed output
  #[must_use]
  pub const fn pretty() -> Self {
    Self { pretty: true }
  }

  /// Create a formatter that produces compact output
  #[must_use]
  pub const fn compact() -> Self {
    Self { pretty: false }
  }
}

impl Default for JsonFormatter {
  fn default() -> Self {
    Self::new()
  }
}

impl OutputFormatter<Interview> for JsonFormatter {
  fn format(&self, data: &Interview) -> Result<String, FormatError> {
    use crate::interview::AnswerValue;

    // Build JSON manually
    let mut json = String::new();
    write!(json, "{{").map_err(|e| FormatError::IoError(e.to_string()))?;
    write!(json, "\"id\":\"{}\",", data.id).map_err(|e| FormatError::IoError(e.to_string()))?;
    write!(json, "\"spec_name\":\"{}\",", data.spec_name).map_err(|e| FormatError::IoError(e.to_string()))?;
    write!(json, "\"state\":\"{}\",", data.state).map_err(|e| FormatError::IoError(e.to_string()))?;
    let title_json = serde_json::to_string(&data.title)
      .map_err(|e| FormatError::SerializationFailed(e.to_string()))?;
    write!(json, "\"title\":{},", title_json)
      .map_err(|e| FormatError::IoError(e.to_string()))?;
    let desc_json = serde_json::to_string(&data.description)
      .map_err(|e| FormatError::SerializationFailed(e.to_string()))?;
    write!(json, "\"description\":{},", desc_json)
      .map_err(|e| FormatError::IoError(e.to_string()))?;

    // Questions
    write!(json, "\"questions\":[").map_err(|e| FormatError::IoError(e.to_string()))?;
    for (i, q) in data.questions.iter().enumerate() {
      if i > 0 {
        write!(json, ",").map_err(|e| FormatError::IoError(e.to_string()))?;
      }
      write!(json, "{{").map_err(|e| FormatError::IoError(e.to_string()))?;
      let text_json = serde_json::to_string(&q.text)
        .map_err(|e| FormatError::SerializationFailed(e.to_string()))?;
      write!(json, "\"text\":{},", text_json)
        .map_err(|e| FormatError::IoError(e.to_string()))?;
      let help_json = serde_json::to_string(&q.help_text)
        .map_err(|e| FormatError::SerializationFailed(e.to_string()))?;
      write!(json, "\"help_text\":{},", help_json)
        .map_err(|e| FormatError::IoError(e.to_string()))?;
      write!(json, "\"required\":{},", q.required).map_err(|e| FormatError::IoError(e.to_string()))?;
      write!(json, "\"question_type\":\"{:?}\"", q.question_type).map_err(|e| FormatError::IoError(e.to_string()))?;
      write!(json, "}}").map_err(|e| FormatError::IoError(e.to_string()))?;
    }
    write!(json, "],").map_err(|e| FormatError::IoError(e.to_string()))?;

    // Answers
    write!(json, "\"answers\":[").map_err(|e| FormatError::IoError(e.to_string()))?;
    for (i, a) in data.answers.iter().enumerate() {
      if i > 0 {
        write!(json, ",").map_err(|e| FormatError::IoError(e.to_string()))?;
      }
      write!(json, "{{\"question_index\":{},", a.question_index).map_err(|e| FormatError::IoError(e.to_string()))?;
      match &a.value {
        AnswerValue::Text(s) => write!(json, "\"value\":\"{}\"}}", s.replace('"', "\\\"")),
        AnswerValue::Boolean(b) => write!(json, "\"value\":{}}}", b),
        AnswerValue::MultipleChoice(idx) => write!(json, "\"value\":{}}}", idx),
        AnswerValue::Numeric(n) => write!(json, "\"value\":{}}}", n),
      }.map_err(|e| FormatError::IoError(e.to_string()))?;
    }
    write!(json, "],").map_err(|e| FormatError::IoError(e.to_string()))?;

    write!(json, "\"created_at\":{},", data.created_at.as_secs()).map_err(|e| FormatError::IoError(e.to_string()))?;
    write!(json, "\"updated_at\":{}", data.updated_at.as_secs()).map_err(|e| FormatError::IoError(e.to_string()))?;
    write!(json, "}}").map_err(|e| FormatError::IoError(e.to_string()))?;

    // Pretty print if needed
    if self.pretty {
      let parsed: serde_json::Value = serde_json::from_str(&json)
        .map_err(|e| FormatError::SerializationFailed(e.to_string()))?;
      serde_json::to_string_pretty(&parsed)
        .map_err(|e| FormatError::SerializationFailed(e.to_string()))
    } else {
      Ok(json)
    }
  }

  fn format_name(&self) -> &str {
    "json"
  }

  fn mime_type(&self) -> &str {
    "application/json"
  }
}

/// Markdown formatter implementation
///
/// Formats interviews as human-readable Markdown documents.
#[derive(Debug, Clone)]
pub struct MarkdownFormatter;

impl MarkdownFormatter {
  /// Create a new Markdown formatter
  #[must_use]
  pub const fn new() -> Self {
    Self
  }
}

impl Default for MarkdownFormatter {
  fn default() -> Self {
    Self::new()
  }
}

impl OutputFormatter<Interview> for MarkdownFormatter {
  fn format(&self, interview: &Interview) -> Result<String, FormatError> {
    let mut output = String::new();

    // Title
    let title = interview.title.as_deref().map_or("Untitled Interview", |t| t);
    writeln!(output, "# {}", title).map_err(|e| FormatError::IoError(e.to_string()))?;

    // Metadata
    writeln!(output, "\n**ID**: {}", interview.id)
      .map_err(|e| FormatError::IoError(e.to_string()))?;
    writeln!(output, "**Spec**: {}", interview.spec_name)
      .map_err(|e| FormatError::IoError(e.to_string()))?;
    writeln!(output, "**Status**: {}", interview.state)
      .map_err(|e| FormatError::IoError(e.to_string()))?;

    // Description
    if let Some(desc) = &interview.description {
      writeln!(output, "\n## Description\n\n{}", desc)
        .map_err(|e| FormatError::IoError(e.to_string()))?;
    }

    // Questions
    writeln!(output, "\n## Questions\n").map_err(|e| FormatError::IoError(e.to_string()))?;

    for (i, question) in interview.questions.iter().enumerate() {
      writeln!(output, "{}. {}", i + 1, question.text)
        .map_err(|e| FormatError::IoError(e.to_string()))?;

      if let Some(help) = &question.help_text {
        writeln!(output, "   - *Help: {}*", help)
          .map_err(|e| FormatError::IoError(e.to_string()))?;
      }

      if question.required {
        writeln!(output, "   - **Required**").map_err(|e| FormatError::IoError(e.to_string()))?;
      }
    }

    Ok(output)
  }

  fn format_name(&self) -> &str {
    "markdown"
  }

  fn mime_type(&self) -> &str {
    "text/markdown"
  }
}

/// Plain text formatter implementation
///
/// Formats interviews as simple, readable plain text.
#[derive(Debug, Clone)]
pub struct PlainTextFormatter;

impl PlainTextFormatter {
  /// Create a new plain text formatter
  #[must_use]
  pub const fn new() -> Self {
    Self
  }
}

impl Default for PlainTextFormatter {
  fn default() -> Self {
    Self::new()
  }
}

impl OutputFormatter<Interview> for PlainTextFormatter {
  fn format(&self, interview: &Interview) -> Result<String, FormatError> {
    let mut output = String::new();

    let title = interview.title.as_deref().map_or("Untitled Interview", |t| t);
    writeln!(output, "Interview: {}", title).map_err(|e| FormatError::IoError(e.to_string()))?;
    writeln!(output, "ID: {}", interview.id).map_err(|e| FormatError::IoError(e.to_string()))?;
    writeln!(output, "Spec: {}", interview.spec_name)
      .map_err(|e| FormatError::IoError(e.to_string()))?;
    writeln!(output, "Status: {}", interview.state)
      .map_err(|e| FormatError::IoError(e.to_string()))?;

    if let Some(desc) = &interview.description {
      writeln!(output, "Description: {}", desc).map_err(|e| FormatError::IoError(e.to_string()))?;
    }

    writeln!(output, "\nQuestions:").map_err(|e| FormatError::IoError(e.to_string()))?;

    for (i, question) in interview.questions.iter().enumerate() {
      writeln!(output, "  {}. {}", i + 1, question.text)
        .map_err(|e| FormatError::IoError(e.to_string()))?;

      if let Some(help) = &question.help_text {
        writeln!(output, "     Help: {}", help)
          .map_err(|e: fmt::Error| FormatError::IoError(e.to_string()))?;
      }

      if question.required {
        writeln!(output, "     Required: Yes")
          .map_err(|e: fmt::Error| FormatError::IoError(e.to_string()))?;
      }
    }

    Ok(output)
  }

  fn format_name(&self) -> &str {
    "text"
  }

  fn mime_type(&self) -> &str {
    "text/plain"
  }
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::expect_used)]
  #![allow(clippy::panic)]
  use super::*;
  use crate::interview::{InterviewBuilder, Question, QuestionType, Timestamp};

  /// Helper function to create a test interview
  fn create_test_interview() -> Interview {
    InterviewBuilder::new()
      .id("550e8400-e29b-41d4-a716-446655440000".to_string())
      .spec_name("test-spec".to_string())
      .title("Requirements Gathering".to_string())
      .description("Gather system requirements from stakeholders".to_string())
      .add_question(Question {
        text: "What are the main features?".to_string(),
        help_text: Some("List the top 3-5 features".to_string()),
        required: true,
        question_type: QuestionType::Text,
      })
      .add_question(Question {
        text: "Is performance critical?".to_string(),
        help_text: None,
        required: true,
        question_type: QuestionType::Boolean,
      })
      .add_question(Question {
        text: "What is the target platform?".to_string(),
        help_text: Some("e.g., Web, Mobile, Desktop".to_string()),
        required: false,
        question_type: QuestionType::MultipleChoice,
      })
      .build()
      .expect("valid interview")
  }

  #[test]
  fn test_format_interview_as_json_produces_valid_json() {
    let interview = create_test_interview();
    let formatter = JsonFormatter::new();

    let result = formatter.format(&interview);

    assert!(result.is_ok());
    let json_str = result.unwrap();

    // Verify it's valid JSON
    let parsed: serde_json::Value =
      serde_json::from_str(&json_str).expect("Output should be valid JSON");

    // Verify fields
    assert_eq!(parsed["title"], interview.title.unwrap_or_default());
    assert_eq!(parsed["spec_name"], interview.spec_name);
  }

  #[test]
  fn test_format_interview_as_markdown_includes_headers_and_content() {
    let interview = create_test_interview();
    let formatter = MarkdownFormatter::new();

    let result = formatter.format(&interview);

    assert!(result.is_ok());
    let md_str = result.unwrap();

    assert!(md_str.starts_with("# Requirements Gathering"));
    assert!(md_str.contains("## Description"));
    assert!(md_str.contains("## Questions"));
    assert!(md_str.contains("1. What are the main features?"));
    assert!(md_str.contains("2. Is performance critical?"));
    assert!(md_str.contains("3. What is the target platform?"));
  }

  #[test]
  fn test_format_interview_as_plain_text_removes_markdown_syntax() {
    let interview = create_test_interview();
    let formatter = PlainTextFormatter::new();

    let result = formatter.format(&interview);

    assert!(result.is_ok());
    let text = result.unwrap();

    assert!(!text.contains('#'));
    assert!(!text.contains('*'));
    assert!(text.contains("Interview: Requirements Gathering"));
    assert!(text.contains("ID:"));
    assert!(text.contains("Spec:"));
    assert!(text.contains("Status:"));
  }

  #[test]
  fn test_format_empty_interview_produces_valid_output() {
    let interview = InterviewBuilder::new()
      .id("550e8400-e29b-41d4-a716-446655440000".to_string())
      .spec_name("test-spec".to_string())
      .title("".to_string())
      .build()
      .expect("valid interview");

    let formatter = JsonFormatter::new();
    let result = formatter.format(&interview);

    assert!(result.is_ok());

    // Verify it's valid JSON even with empty fields
    let parsed: serde_json::Value =
      serde_json::from_str(&result.unwrap()).expect("Empty interview should produce valid JSON");
    assert_eq!(parsed["title"], "");
    assert_eq!(
      parsed["questions"]
        .as_array()
        .expect("questions should be array")
        .len(),
      0
    );
  }

  #[test]
  fn test_format_interview_with_special_characters_escaped_properly() {
    let interview = InterviewBuilder::new()
      .id("550e8400-e29b-41d4-a716-446655440000".to_string())
      .spec_name("test-spec".to_string())
      .title("Test \"Quotes\" and \\backslashes\\ and émojis 🎉".to_string())
      .build()
      .expect("valid interview");

    let formatter = JsonFormatter::new();
    let result = formatter.format(&interview);

    assert!(result.is_ok());
    let json_str = result.unwrap();

    // Verify it parses correctly
    let parsed: serde_json::Value =
      serde_json::from_str(&json_str).expect("Special characters should be properly escaped");

    assert_eq!(parsed["title"], interview.title.unwrap_or_default());
  }

  #[test]
  fn test_json_formatter_pretty_prints_by_default_false() {
    let interview = create_test_interview();
    let formatter = JsonFormatter::new(); // compact by default

    let result = formatter.format(&interview);

    assert!(result.is_ok());
    let json_str = result.unwrap();

    // Compact JSON should not have newlines (except in strings)
    let newline_count = json_str.matches('\n').count();
    assert_eq!(newline_count, 0, "Default JSON should be compact");
  }

  #[test]
  fn test_json_formatter_pretty_mode_includes_indentation() {
    let interview = create_test_interview();
    let formatter = JsonFormatter::pretty();

    let result = formatter.format(&interview);

    assert!(result.is_ok());
    let json_str = result.unwrap();

    // Pretty printed JSON contains newlines
    assert!(json_str.contains('\n'));

    // Count indentation (should be consistent)
    let lines: Vec<&str> = json_str.lines().collect();
    let indented_lines = lines
      .iter()
      .filter(|line| line.starts_with("  ") || line.starts_with("    "))
      .count();

    assert!(indented_lines > 0, "Pretty JSON should be indented");
  }

  #[test]
  fn test_json_formatter_compact_mode_removes_whitespace() {
    let interview = create_test_interview();
    let formatter = JsonFormatter::compact();

    let result = formatter.format(&interview);

    assert!(result.is_ok());
    let json_str = result.unwrap();

    // Compact JSON should not have newlines (except in strings)
    let newline_count = json_str.matches('\n').count();
    assert_eq!(newline_count, 0, "Compact JSON should not contain newlines");
  }

  #[test]
  fn test_parse_format_from_invalid_string_returns_error() {
    let result = OutputFormat::from_str("xml");

    assert!(result.is_err());
    match result {
      Err(FormatError::UnsupportedFormat(format)) => {
        assert_eq!(format, "xml");
      }
      _ => panic!("Expected UnsupportedFormat error"),
    }
  }

  #[test]
  fn test_parse_format_from_valid_strings() {
    assert_eq!(OutputFormat::from_str("json").unwrap(), OutputFormat::Json);
    assert_eq!(OutputFormat::from_str("JSON").unwrap(), OutputFormat::Json);
    assert_eq!(
      OutputFormat::from_str("markdown").unwrap(),
      OutputFormat::Markdown
    );
    assert_eq!(
      OutputFormat::from_str("md").unwrap(),
      OutputFormat::Markdown
    );
    assert_eq!(
      OutputFormat::from_str("text").unwrap(),
      OutputFormat::PlainText
    );
    assert_eq!(
      OutputFormat::from_str("txt").unwrap(),
      OutputFormat::PlainText
    );
  }

  #[test]
  fn test_formatter_returns_correct_metadata() {
    let json_formatter = JsonFormatter::new();
    assert_eq!(json_formatter.format_name(), "json");
    assert_eq!(json_formatter.mime_type(), "application/json");

    let md_formatter = MarkdownFormatter::new();
    assert_eq!(md_formatter.format_name(), "markdown");
    assert_eq!(md_formatter.mime_type(), "text/markdown");

    let text_formatter = PlainTextFormatter::new();
    assert_eq!(text_formatter.format_name(), "text");
    assert_eq!(text_formatter.mime_type(), "text/plain");
  }

  #[test]
  fn test_format_large_interview_completes_in_reasonable_time() {
    let mut builder = InterviewBuilder::new()
      .id("550e8400-e29b-41d4-a716-446655440000".to_string())
      .spec_name("test-spec".to_string());

    for i in 0..100 {
      builder = builder.add_question(Question {
        text: format!("Question {}", i),
        help_text: Some(format!("Help text for question {}", i)),
        required: i % 2 == 0,
        question_type: QuestionType::Text,
      });
    }

    let interview = builder.build().expect("valid interview");

    let formatter = JsonFormatter::new();
    let start = std::time::Instant::now();
    let result = formatter.format(&interview);
    let duration = start.elapsed();

    assert!(result.is_ok());
    assert!(
      duration.as_secs() < 1,
      "Formatting should complete in less than 1 second"
    );

    let json_str = result.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
    assert_eq!(
      parsed["questions"]
        .as_array()
        .expect("questions array")
        .len(),
      100
    );
  }

  #[test]
  fn test_output_format_formatter_returns_correct_formatter() {
    let json_formatter = OutputFormat::Json.formatter();
    assert_eq!(json_formatter.format_name(), "json");

    let md_formatter = OutputFormat::Markdown.formatter();
    assert_eq!(md_formatter.format_name(), "markdown");

    let text_formatter = OutputFormat::PlainText.formatter();
    assert_eq!(text_formatter.format_name(), "text");
  }

  #[test]
  fn test_format_interview_with_no_title() {
    let interview = InterviewBuilder::new()
      .id("550e8400-e29b-41d4-a716-446655440000".to_string())
      .spec_name("test-spec".to_string())
      .build()
      .expect("valid interview");

    let formatter = MarkdownFormatter::new();
    let result = formatter.format(&interview);

    assert!(result.is_ok());
    let md_str = result.unwrap();
    assert!(md_str.starts_with("# Untitled Interview"));
  }

  #[test]
  fn test_format_interview_with_no_description() {
    let interview = InterviewBuilder::new()
      .id("550e8400-e29b-41d4-a716-446655440000".to_string())
      .spec_name("test-spec".to_string())
      .title("Test Interview".to_string())
      .build()
      .expect("valid interview");

    let formatter = MarkdownFormatter::new();
    let result = formatter.format(&interview);

    assert!(result.is_ok());
    let md_str = result.unwrap();
    // Should not have Description section
    assert!(!md_str.contains("## Description"));
  }

  #[test]
  fn test_format_error_display() {
    let err = FormatError::SerializationFailed("test error".to_string());
    assert_eq!(format!("{err}"), "Serialization failed: test error");

    let err = FormatError::UnsupportedFormat("xml".to_string());
    assert_eq!(format!("{err}"), "Unsupported format: xml");

    let err = FormatError::MissingField("title".to_string());
    assert_eq!(format!("{err}"), "Missing required field: title");
  }
}

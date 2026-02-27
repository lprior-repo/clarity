#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Error Type System for Intent Module (WP04)
//!
//! Provides structured error handling with contextual information and field name
//! suggestions for JSON parsing errors. Uses Levenshtein distance for typo detection.
//!
//! ## Key Types
//!
//! - [`IntentError`]: Top-level error enum with all error variants
//! - [`ContextualError`]: Rich error with context, source location, and suggestions
//! - [`ValidationError`]: Structured validation failure details
//! - [`FieldFailure`]: Individual field-level failure with suggestions
//! - [`Suggestion`]: Typo correction with edit distance

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

// =============================================================================
// Core Error Types
// =============================================================================

/// Top-level error enum for all intent module operations
#[derive(Debug, Error, Clone, PartialEq, Serialize, Deserialize)]
pub enum IntentError {
    /// JSON parsing failed
    #[error("JSON parse error: {0}")]
    JsonParse(String),

    /// Required field is missing
    #[error("missing required field: {0}")]
    MissingField(String),

    /// Field has invalid type
    #[error("invalid type for field '{field}': expected {expected}, got {actual}")]
    InvalidType {
        /// Field name
        field: String,
        /// Expected type
        expected: String,
        /// Actual type found
        actual: String,
    },

    /// Field has invalid value
    #[error("invalid value for field '{field}': {reason}")]
    InvalidValue {
        /// Field name
        field: String,
        /// Reason for invalidity
        reason: String,
    },

    /// Unknown field encountered
    #[error("unknown field: {0}")]
    UnknownField(String),

    /// Validation failed
    #[error("validation failed: {0}")]
    ValidationFailed(String),

    /// IO error occurred
    #[error("IO error: {0}")]
    Io(String),

    /// File not found
    #[error("file not found: {0}")]
    FileNotFound(String),

    /// Invalid path
    #[error("invalid path: {0}")]
    InvalidPath(String),

    /// Circular dependency detected
    #[error("circular dependency: {0}")]
    CircularDependency(String),

    /// Constraint violation
    #[error("constraint violation: {0}")]
    ConstraintViolation(String),

    /// Configuration error
    #[error("configuration error: {0}")]
    Configuration(String),

    /// Internal error (should not happen)
    #[error("internal error: {0}")]
    Internal(String),
}

/// Contextual error with rich information for debugging and user feedback
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextualError {
    /// The underlying error
    pub error: IntentError,
    /// Human-readable error message
    pub message: String,
    /// Source file path (if applicable)
    pub source_file: Option<String>,
    /// Line number in source (if applicable)
    pub line: Option<usize>,
    /// Column number in source (if applicable)
    pub column: Option<usize>,
    /// JSON path to the error location (e.g., "spec.beads[0].name")
    pub json_path: Option<String>,
    /// Suggestions for fixing the error
    pub suggestions: Vec<Suggestion>,
    /// Additional context as key-value pairs
    pub context: Vec<(String, String)>,
}

impl ContextualError {
    /// Create a new contextual error
    ///
    /// # Errors
    ///
    /// Returns `Err` if the message is empty
    pub fn new(error: IntentError, message: impl Into<String>) -> Result<Self, IntentError> {
        let msg = message.into();
        if msg.is_empty() {
            return Err(IntentError::Internal("error message cannot be empty".into()));
        }
        Ok(Self {
            error,
            message: msg,
            source_file: None,
            line: None,
            column: None,
            json_path: None,
            suggestions: Vec::new(),
            context: Vec::new(),
        })
    }

    /// Add source file information
    #[must_use]
    pub fn with_source_file(mut self, path: impl Into<String>) -> Self {
        self.source_file = Some(path.into());
        self
    }

    /// Add line number
    #[must_use]
    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    /// Add column number
    #[must_use]
    pub fn with_column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }

    /// Add JSON path
    #[must_use]
    pub fn with_json_path(mut self, path: impl Into<String>) -> Self {
        self.json_path = Some(path.into());
        self
    }

    /// Add a suggestion
    #[must_use]
    pub fn with_suggestion(mut self, suggestion: Suggestion) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    /// Add multiple suggestions
    #[must_use]
    pub fn with_suggestions(mut self, suggestions: Vec<Suggestion>) -> Self {
        self.suggestions.extend(suggestions);
        self
    }

    /// Add context key-value pair
    #[must_use]
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.push((key.into(), value.into()));
        self
    }

    /// Check if this error has location information
    #[must_use]
    pub fn has_location(&self) -> bool {
        self.source_file.is_some() || self.line.is_some() || self.json_path.is_some()
    }

    /// Get a formatted location string
    #[must_use]
    pub fn location_string(&self) -> Option<String> {
        match (&self.source_file, &self.line, &self.column) {
            (Some(file), Some(line), Some(col)) => {
                Some(format!("{file}:{line}:{col}"))
            }
            (Some(file), Some(line), None) => Some(format!("{file}:{line}")),
            (Some(file), None, None) => Some(file.clone()),
            (None, Some(line), Some(col)) => Some(format!("line {line}, column {col}")),
            (None, Some(line), None) => Some(format!("line {line}")),
            _ => None,
        }
    }
}

impl fmt::Display for ContextualError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format_error(self))
    }
}

impl std::error::Error for ContextualError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

/// Structured validation error with multiple field failures
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationError {
    /// Overall validation message
    pub message: String,
    /// Individual field failures
    pub field_failures: Vec<FieldFailure>,
    /// Total number of errors (may exceed field_failures.len() if truncated)
    pub total_errors: usize,
}

impl ValidationError {
    /// Create a new validation error
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            field_failures: Vec::new(),
            total_errors: 0,
        }
    }

    /// Create a validation error with field failures
    #[must_use]
    pub fn with_failures(message: impl Into<String>, failures: Vec<FieldFailure>) -> Self {
        let total = failures.len();
        Self {
            message: message.into(),
            field_failures: failures,
            total_errors: total,
        }
    }

    /// Add a field failure
    #[must_use]
    pub fn add_failure(mut self, failure: FieldFailure) -> Self {
        self.field_failures.push(failure);
        self.total_errors += 1;
        self
    }

    /// Check if validation has any failures
    #[must_use]
    pub fn has_failures(&self) -> bool {
        !self.field_failures.is_empty()
    }

    /// Get failures for a specific field
    #[must_use]
    pub fn failures_for_field(&self, field: &str) -> Vec<&FieldFailure> {
        self.field_failures
            .iter()
            .filter(|f| f.field == field)
            .collect()
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Validation Error: {}", self.message)?;
        for failure in &self.field_failures {
            writeln!(f, "  - {failure}")?;
        }
        if self.total_errors > self.field_failures.len() {
            writeln!(
                f,
                "  ... and {} more errors",
                self.total_errors - self.field_failures.len()
            )?;
        }
        Ok(())
    }
}

/// Individual field-level validation failure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldFailure {
    /// Field name that failed validation
    pub field: String,
    /// Error code (e.g., "required", "invalid_type", "out_of_range")
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// The actual value that caused the failure (as string)
    pub actual_value: Option<String>,
    /// Expected value or format
    pub expected: Option<String>,
    /// Suggestions for fixing (e.g., similar field names)
    pub suggestions: Vec<Suggestion>,
}

impl FieldFailure {
    /// Create a new field failure
    #[must_use]
    pub fn new(field: impl Into<String>, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            code: code.into(),
            message: message.into(),
            actual_value: None,
            expected: None,
            suggestions: Vec::new(),
        }
    }

    /// Create a "required field missing" failure
    #[must_use]
    pub fn required(field: impl Into<String>) -> Self {
        Self::new(field, "required", "This field is required")
    }

    /// Create an "invalid type" failure
    #[must_use]
    pub fn invalid_type(
        field: impl Into<String>,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) -> Self {
        let expected_str = expected.into();
        let actual_str = actual.into();
        Self::new(field, "invalid_type", format!("Expected {expected_str}, got {actual_str}"))
            .with_expected(expected_str)
            .with_actual(actual_str)
    }

    /// Create an "unknown field" failure with suggestions
    #[must_use]
    pub fn unknown_field(field: impl Into<String>, suggestions: Vec<Suggestion>) -> Self {
        let field_str = field.into();
        Self::new(field_str.clone(), "unknown_field", format!("Unknown field '{field_str}'"))
            .with_suggestions(suggestions)
    }

    /// Add actual value
    #[must_use]
    pub fn with_actual(mut self, value: impl Into<String>) -> Self {
        self.actual_value = Some(value.into());
        self
    }

    /// Add expected value
    #[must_use]
    pub fn with_expected(mut self, value: impl Into<String>) -> Self {
        self.expected = Some(value.into());
        self
    }

    /// Add suggestions
    #[must_use]
    pub fn with_suggestions(mut self, suggestions: Vec<Suggestion>) -> Self {
        self.suggestions = suggestions;
        self
    }
}

impl fmt::Display for FieldFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Field '{}': {}", self.field, self.message)?;
        if !self.suggestions.is_empty() {
            let suggestions_str = self
                .suggestions
                .iter()
                .map(|s| s.text.as_str())
                .join(", ");
            write!(f, " (did you mean: {suggestions_str}?)")?;
        }
        Ok(())
    }
}

/// A suggestion for fixing an error, with edit distance
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Suggestion {
    /// The suggested text
    pub text: String,
    /// Levenshtein distance from the original
    pub distance: usize,
}

impl Suggestion {
    /// Create a new suggestion
    #[must_use]
    pub fn new(text: impl Into<String>, distance: usize) -> Self {
        Self {
            text: text.into(),
            distance,
        }
    }
}

impl fmt::Display for Suggestion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (distance: {})", self.text, self.distance)
    }
}

// =============================================================================
// Levenshtein Distance
// =============================================================================

/// Compute the Levenshtein edit distance between two strings.
///
/// Uses the Wagner-Fischer algorithm with Unicode support via chars.
/// Time complexity: O(a.len() * b.len())
/// Space complexity: O(min(a.len(), b.len()))
///
/// # Examples
///
/// ```
/// use clarity_web::intent::errors::levenshtein;
///
/// assert_eq!(levenshtein("kitten", "sitting"), 3);
/// assert_eq!(levenshtein("", "hello"), 5);
/// assert_eq!(levenshtein("same", "same"), 0);
/// ```
#[must_use]
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    let a_len = a_chars.len();
    let b_len = b_chars.len();

    // Handle empty string cases
    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    // Optimize space by using two rows instead of full matrix
    // We iterate over the shorter string for the inner loop
    let (longer, shorter) = if a_len > b_len {
        (&a_chars, &b_chars)
    } else {
        (&b_chars, &a_chars)
    };

    let shorter_len = shorter.len();

    // Previous row of distances
    let mut prev_row: Vec<usize> = (0..=shorter_len).collect();
    // Current row being computed
    let mut curr_row: Vec<usize> = vec![0; shorter_len + 1];

    for (i, long_char) in longer.iter().enumerate() {
        // First element of current row is i + 1
        curr_row[0] = i + 1;

        for (j, short_char) in shorter.iter().enumerate() {
            let cost = if long_char == short_char { 0 } else { 1 };

            curr_row[j + 1] = (prev_row[j + 1] + 1) // deletion
                .min(curr_row[j] + 1) // insertion
                .min(prev_row[j] + cost); // substitution
        }

        // Swap rows
        std::mem::swap(&mut prev_row, &mut curr_row);
    }

    // After the last swap, prev_row contains our result
    prev_row[shorter_len]
}

// =============================================================================
// Field Suggestions
// =============================================================================

/// Suggest similar field names based on Levenshtein distance.
///
/// Returns up to 3 suggestions sorted by distance, only including
/// suggestions with distance <= 2.
///
/// # Arguments
///
/// * `target` - The unknown/misspelled field name
/// * `available` - List of valid field names
///
/// # Returns
///
/// A vector of suggestions, max 3, sorted by distance (ascending)
///
/// # Examples
///
/// ```
/// use clarity_web::intent::errors::suggest_field_names;
///
/// let available = vec![
///     "name".to_string(),
///     "version".to_string(),
///     "description".to_string(),
/// ];
///
/// let suggestions = suggest_field_names("nam", &available);
/// assert_eq!(suggestions.len(), 1);
/// assert_eq!(suggestions[0].text, "name");
/// assert_eq!(suggestions[0].distance, 1);
/// ```
#[must_use]
pub fn suggest_field_names(target: &str, available: &[String]) -> Vec<Suggestion> {
    const MAX_SUGGESTIONS: usize = 3;
    const MAX_DISTANCE: usize = 2;

    available
        .iter()
        .filter_map(|field| {
            let distance = levenshtein(target, field);
            if distance <= MAX_DISTANCE {
                Some(Suggestion::new(field.clone(), distance))
            } else {
                None
            }
        })
        .sorted()
        .take(MAX_SUGGESTIONS)
        .collect()
}

// =============================================================================
// JSON Field Extraction
// =============================================================================

/// Extract available field names from a JSON value.
///
/// For objects, returns the keys.
/// For arrays of objects, returns the union of all keys.
/// For other types, returns an empty vector.
///
/// # Examples
///
/// ```
/// use serde_json::json;
/// use clarity_web::intent::errors::extract_available_fields;
///
/// let obj = json!({
///     "name": "test",
///     "version": "1.0"
/// });
///
/// let fields = extract_available_fields(&obj);
/// assert!(fields.contains(&"name".to_string()));
/// assert!(fields.contains(&"version".to_string()));
/// ```
#[must_use]
pub fn extract_available_fields(json: &serde_json::Value) -> Vec<String> {
    match json {
        serde_json::Value::Object(map) => map.keys().cloned().sorted().collect(),
        serde_json::Value::Array(arr) => arr
            .iter()
            .filter_map(|item| match item {
                serde_json::Value::Object(inner_map) => Some(inner_map.keys().cloned().collect::<Vec<_>>()),
                _ => None,
            })
            .flatten()
            .unique()
            .sorted()
            .collect(),
        _ => Vec::new(),
    }
}

// =============================================================================
// Error Formatting
// =============================================================================

/// Format a contextual error for CLI output.
///
/// Produces a human-readable error message with:
/// - Error type and message
/// - Location information (file, line, JSON path)
/// - Suggestions (if any)
/// - Additional context
///
/// # Examples
///
/// ```
/// use clarity_web::intent::errors::{ContextualError, IntentError, format_error};
///
/// let error = ContextualError::new(
///     IntentError::MissingField("name".into()),
///     "The 'name' field is required"
/// ).unwrap().with_json_path("spec.beads[0]");
///
/// let formatted = format_error(&error);
/// assert!(formatted.contains("name"));
/// assert!(formatted.contains("spec.beads[0]"));
/// ```
#[must_use]
pub fn format_error(error: &ContextualError) -> String {
    let mut output = String::new();

    // Error header with type
    let error_type = match &error.error {
        IntentError::JsonParse(_) => "JSON Parse Error",
        IntentError::MissingField(_) => "Missing Field",
        IntentError::InvalidType { .. } => "Type Error",
        IntentError::InvalidValue { .. } => "Value Error",
        IntentError::UnknownField(_) => "Unknown Field",
        IntentError::ValidationFailed(_) => "Validation Error",
        IntentError::Io(_) => "IO Error",
        IntentError::FileNotFound(_) => "File Not Found",
        IntentError::InvalidPath(_) => "Invalid Path",
        IntentError::CircularDependency(_) => "Circular Dependency",
        IntentError::ConstraintViolation(_) => "Constraint Violation",
        IntentError::Configuration(_) => "Configuration Error",
        IntentError::Internal(_) => "Internal Error",
    };

    output.push_str(&format!("Error: {error_type}\n"));
    output.push_str(&format!("  Message: {}\n", error.message));

    // Location information
    if let Some(location) = error.location_string() {
        output.push_str(&format!("  Location: {location}\n"));
    }

    if let Some(ref json_path) = error.json_path {
        output.push_str(&format!("  JSON Path: {json_path}\n"));
    }

    // Suggestions
    if !error.suggestions.is_empty() {
        output.push_str("  Suggestions:\n");
        for suggestion in &error.suggestions {
            output.push_str(&format!("    - {} (edit distance: {})\n", suggestion.text, suggestion.distance));
        }
    }

    // Additional context
    if !error.context.is_empty() {
        output.push_str("  Context:\n");
        for (key, value) in &error.context {
            output.push_str(&format!("    {key}: {value}\n"));
        }
    }

    output.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // =========================================================================
    // Levenshtein Distance Tests
    // =========================================================================

    #[test]
    fn levenshtein_empty_strings() {
        assert_eq!(levenshtein("", ""), 0);
        assert_eq!(levenshtein("", "hello"), 5);
        assert_eq!(levenshtein("hello", ""), 5);
    }

    #[test]
    fn levenshtein_identical_strings() {
        assert_eq!(levenshtein("hello", "hello"), 0);
        assert_eq!(levenshtein("world", "world"), 0);
    }

    #[test]
    fn levenshtein_single_operations() {
        // Single insertion
        assert_eq!(levenshtein("cat", "cats"), 1);
        // Single deletion
        assert_eq!(levenshtein("cats", "cat"), 1);
        // Single substitution
        assert_eq!(levenshtein("cat", "bat"), 1);
    }

    #[test]
    fn levenshtein_multiple_operations() {
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("saturday", "sunday"), 3);
    }

    #[test]
    fn levenshtein_unicode_support() {
        // Unicode characters
        assert_eq!(levenshtein("cafe", "cafe\u{0301}"), 1); // combining accent
        assert_eq!(levenshtein("hello", "hola"), 3);
        // Emoji
        assert_eq!(levenshtein("\u{2764}\u{fe0f}\u{200d}\u{1f525}", "\u{2764}\u{fe0f}"), 2);
    }

    #[test]
    fn levenshtein_case_sensitivity() {
        assert_eq!(levenshtein("Hello", "hello"), 1);
        assert_eq!(levenshtein("HELLO", "hello"), 5);
    }

    // =========================================================================
    // Field Suggestion Tests
    // =========================================================================

    #[test]
    fn suggest_field_names_exact_match() {
        let available = vec![
            "name".to_string(),
            "version".to_string(),
            "description".to_string(),
        ];
        let suggestions = suggest_field_names("name", &available);
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].text, "name");
        assert_eq!(suggestions[0].distance, 0);
    }

    #[test]
    fn suggest_field_names_typo() {
        let available = vec![
            "name".to_string(),
            "version".to_string(),
            "description".to_string(),
        ];
        let suggestions = suggest_field_names("nam", &available);
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].text, "name");
        assert_eq!(suggestions[0].distance, 1);
    }

    #[test]
    fn suggest_field_names_max_suggestions() {
        let available = vec![
            "name".to_string(),
            "nave".to_string(),
            "nate".to_string(),
            "late".to_string(),
        ];
        let suggestions = suggest_field_names("nam", &available);
        // Should return at most 3
        assert!(suggestions.len() <= 3);
    }

    #[test]
    fn suggest_field_names_distance_threshold() {
        let available = vec![
            "a".to_string(),
            "abcdefghijk".to_string(), // distance > 2 from "xy"
        ];
        let suggestions = suggest_field_names("xy", &available);
        // Only "a" should match (distance 2)
        assert!(suggestions.iter().all(|s| s.distance <= 2));
    }

    #[test]
    fn suggest_field_names_sorted_by_distance() {
        let available = vec![
            "abcd".to_string(), // distance 2 from "ab"
            "abc".to_string(),  // distance 1 from "ab"
            "ab".to_string(),   // distance 0 from "ab"
        ];
        let suggestions = suggest_field_names("ab", &available);
        assert!(suggestions.len() >= 2);
        // Verify sorted by distance
        for i in 1..suggestions.len() {
            assert!(suggestions[i - 1].distance <= suggestions[i].distance);
        }
    }

    #[test]
    fn suggest_field_names_no_match() {
        let available = vec![
            "completely".to_string(),
            "different".to_string(),
        ];
        let suggestions = suggest_field_names("xyz123", &available);
        assert!(suggestions.is_empty());
    }

    // =========================================================================
    // JSON Field Extraction Tests
    // =========================================================================

    #[test]
    fn extract_fields_from_object() {
        let json = json!({
            "name": "test",
            "version": "1.0",
            "active": true
        });
        let fields = extract_available_fields(&json);
        assert!(fields.contains(&"name".to_string()));
        assert!(fields.contains(&"version".to_string()));
        assert!(fields.contains(&"active".to_string()));
        assert_eq!(fields.len(), 3);
    }

    #[test]
    fn extract_fields_from_array_of_objects() {
        let json = json!([
            {"name": "a", "value": 1},
            {"name": "b", "other": 2}
        ]);
        let fields = extract_available_fields(&json);
        // Should contain union of keys
        assert!(fields.contains(&"name".to_string()));
        assert!(fields.contains(&"value".to_string()));
        assert!(fields.contains(&"other".to_string()));
    }

    #[test]
    fn extract_fields_from_non_object() {
        assert!(extract_available_fields(&json!("string")).is_empty());
        assert!(extract_available_fields(&json!(42)).is_empty());
        assert!(extract_available_fields(&json!(true)).is_empty());
        assert!(extract_available_fields(&json!(null)).is_empty());
    }

    #[test]
    fn extract_fields_sorted() {
        let json = json!({
            "z": 1,
            "a": 2,
            "m": 3
        });
        let fields = extract_available_fields(&json);
        assert_eq!(fields, vec!["a", "m", "z"]);
    }

    // =========================================================================
    // Contextual Error Tests
    // =========================================================================

    #[test]
    fn contextual_error_new() {
        let error = ContextualError::new(
            IntentError::MissingField("name".into()),
            "Name is required",
        );
        assert!(error.is_ok());
        let ctx = error.map_err(|_| ()).map_err(|_| "").ok();
        if let Some(ctx) = ctx {
            assert_eq!(ctx.message, "Name is required");
        }
    }

    #[test]
    fn contextual_error_empty_message_rejected() {
        let error = ContextualError::new(
            IntentError::MissingField("name".into()),
            "",
        );
        assert!(error.is_err());
    }

    #[test]
    fn contextual_error_builders() {
        let error = ContextualError::new(
            IntentError::InvalidType {
                field: "count".into(),
                expected: "number".into(),
                actual: "string".into(),
            },
            "Type mismatch",
        )
        .map_err(|_| ())
        .map_err(|_| "")
        .ok()
        .and_then(|e| {
            Some(
                e.with_source_file("test.json")
                    .with_line(42)
                    .with_column(10)
                    .with_json_path("items[0].count"),
            )
        });

        if let Some(error) = error {
            assert_eq!(error.source_file, Some("test.json".to_string()));
            assert_eq!(error.line, Some(42));
            assert_eq!(error.column, Some(10));
            assert_eq!(error.json_path, Some("items[0].count".to_string()));
            assert!(error.has_location());
            assert_eq!(error.location_string(), Some("test.json:42:10".to_string()));
        }
    }

    #[test]
    fn contextual_error_with_suggestions() {
        let error = ContextualError::new(
            IntentError::UnknownField("nmae".into()),
            "Unknown field",
        )
        .map_err(|_| ())
        .map_err(|_| "")
        .ok()
        .and_then(|e| {
            Some(
                e.with_suggestion(Suggestion::new("name", 2)),
            )
        });

        if let Some(error) = error {
            assert_eq!(error.suggestions.len(), 1);
            assert_eq!(error.suggestions[0].text, "name");
        }
    }

    // =========================================================================
    // Validation Error Tests
    // =========================================================================

    #[test]
    fn validation_error_new() {
        let error = ValidationError::new("Validation failed");
        assert_eq!(error.message, "Validation failed");
        assert!(error.field_failures.is_empty());
        assert!(!error.has_failures());
    }

    #[test]
    fn validation_error_with_failures() {
        let failures = vec![
            FieldFailure::required("name"),
            FieldFailure::invalid_type("count", "number", "string"),
        ];
        let error = ValidationError::with_failures("Multiple errors", failures);
        assert!(error.has_failures());
        assert_eq!(error.total_errors, 2);
    }

    #[test]
    fn validation_error_add_failure() {
        let error = ValidationError::new("Validation")
            .add_failure(FieldFailure::required("name"))
            .add_failure(FieldFailure::required("version"));
        assert_eq!(error.total_errors, 2);
    }

    #[test]
    fn field_failure_required() {
        let failure = FieldFailure::required("name");
        assert_eq!(failure.field, "name");
        assert_eq!(failure.code, "required");
        assert!(failure.actual_value.is_none());
    }

    #[test]
    fn field_failure_unknown_with_suggestions() {
        let failure = FieldFailure::unknown_field(
            "nmae",
            vec![
                Suggestion::new("name", 2),
                Suggestion::new("mane", 2),
            ],
        );
        assert_eq!(failure.code, "unknown_field");
        assert_eq!(failure.suggestions.len(), 2);
    }

    // =========================================================================
    // Format Error Tests
    // =========================================================================

    #[test]
    fn format_error_basic() {
        let error = ContextualError::new(
            IntentError::MissingField("name".into()),
            "Name field is required",
        )
        .map_err(|_| ())
        .map_err(|_| "")
        .ok();

        if let Some(error) = error {
            let formatted = format_error(&error);
            assert!(formatted.contains("Missing Field"));
            assert!(formatted.contains("Name field is required"));
        }
    }

    #[test]
    fn format_error_with_location() {
        let error = ContextualError::new(
            IntentError::JsonParse("Unexpected token".into()),
            "Failed to parse JSON",
        )
        .map_err(|_| ())
        .map_err(|_| "")
        .ok()
        .and_then(|e| {
            Some(
                e.with_source_file("spec.json")
                    .with_line(10)
                    .with_json_path("$.beads[0].name"),
            )
        });

        if let Some(error) = error {
            let formatted = format_error(&error);
            // Location is formatted as "file:line" when both are present
            assert!(formatted.contains("spec.json:10"));
            assert!(formatted.contains("$.beads[0].name"));
        }
    }

    #[test]
    fn format_error_with_suggestions() {
        let error = ContextualError::new(
            IntentError::UnknownField("nmae".into()),
            "Unknown field",
        )
        .map_err(|_| ())
        .map_err(|_| "")
        .ok()
        .and_then(|e| {
            Some(
                e.with_suggestions(vec![
                    Suggestion::new("name", 2),
                ]),
            )
        });

        if let Some(error) = error {
            let formatted = format_error(&error);
            assert!(formatted.contains("Suggestions"));
            assert!(formatted.contains("name"));
        }
    }

    // =========================================================================
    // IntentError Display Tests
    // =========================================================================

    #[test]
    fn intent_error_display() {
        assert_eq!(
            IntentError::MissingField("name".into()).to_string(),
            "missing required field: name"
        );

        let type_error = IntentError::InvalidType {
            field: "count".into(),
            expected: "number".into(),
            actual: "string".into(),
        };
        assert!(type_error.to_string().contains("count"));
        assert!(type_error.to_string().contains("number"));
        assert!(type_error.to_string().contains("string"));
    }
}

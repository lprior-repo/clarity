//! JSON Parser for Spec parsing (WP17)
//!
//! This module provides JSON parsing functionality for Spec structures,
//! with comprehensive error handling and validation.
//!
//! ## Design Principles
//!
//! - **Zero panics**: All fallible operations return `Result<T, E>`
//! - **Zero unwrap/expect**: No panics in production code
//! - **Graceful error handling**: Malformed JSON produces helpful error messages

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use serde_json::Value;
use thiserror::Error;

use super::types::Spec;

/// Error type for parsing operations
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// JSON syntax or structure error
    #[error("JSON error: {0}")]
    JsonError(String),

    /// Required field is missing from the JSON
    #[error("missing required field: {0}")]
    MissingField(String),

    /// Field has incorrect type
    #[error("invalid type for field '{field}': expected {expected}, got {actual}")]
    InvalidType {
        /// Field name
        field: String,
        /// Expected type
        expected: String,
        /// Actual type found
        actual: String,
    },

    /// Required field has empty value
    #[error("empty value for required field: {0}")]
    EmptyField(String),
}

/// Parse a JSON string into a Spec struct
///
/// This function parses the JSON string and validates that required fields
/// (name, version) are present. Optional fields will use their defaults.
///
/// # Errors
///
/// Returns `ParseError` if:
/// - JSON is malformed
/// - Required field `name` is missing or empty
///
/// # Example
///
/// ```rust,ignore
/// use clarity_web::intent::parser::parse_spec;
///
/// let json = r#"{"name": "my-spec", "description": "A spec"}"#;
/// let spec = parse_spec(json)?;
/// assert_eq!(spec.name, "my-spec");
/// ```
pub fn parse_spec(json: &str) -> Result<Spec, ParseError> {
    // Sanitize input string
    let sanitized = sanitize_string(json);

    // Parse JSON
    let value: Value = serde_json::from_str(&sanitized).map_err(|e| {
        ParseError::JsonError(format!(
            "Failed to parse JSON at line {}, column {}: {}",
            e.line(),
            e.column(),
            e
        ))
    })?;

    parse_spec_from_value(&value)
}

/// Parse from an already-parsed JSON value
///
/// This function validates the JSON structure and constructs a Spec.
///
/// # Errors
///
/// Returns `ParseError` if:
/// - Value is not an object
/// - Required field `name` is missing or empty
/// - Field types are incorrect
pub fn parse_spec_from_value(value: &Value) -> Result<Spec, ParseError> {
    // Ensure we have an object
    let obj = value.as_object().ok_or_else(|| ParseError::InvalidType {
        field: "root".to_string(),
        expected: "object".to_string(),
        actual: json_type_name(value),
    })?;

    // Extract and validate name (required)
    let name = extract_string_field(obj, "name")?;

    // Check for empty or whitespace-only name
    if name.trim().is_empty() {
        return Err(ParseError::EmptyField("name".to_string()));
    }

    // Parse the rest using serde (it handles defaults for optional fields)
    let spec: Spec = serde_json::from_value(value.clone()).map_err(|e| {
        ParseError::JsonError(format!("Failed to deserialize Spec: {e}"))
    })?;

    Ok(Spec {
        name,
        ..spec
    })
}

/// Sanitize a string by removing null bytes and trimming whitespace
///
/// Null bytes can cause issues with JSON parsing and should be removed.
/// Leading/trailing whitespace is also trimmed.
#[must_use]
pub fn sanitize_string(s: &str) -> String {
    s.chars()
        .filter(|&c| c != '\0')
        .collect::<String>()
        .trim()
        .to_string()
}

/// Validate a Spec for semantic correctness
///
/// Checks that:
/// - `name` is non-empty
/// - `features` list is non-empty (for a meaningful spec)
///
/// # Errors
///
/// Returns `ParseError` if validation fails.
pub fn validate_spec(spec: &Spec) -> Result<(), ParseError> {
    // Check name is non-empty
    if spec.name.trim().is_empty() {
        return Err(ParseError::EmptyField("name".to_string()));
    }

    // Check features is non-empty
    if spec.features.is_empty() {
        return Err(ParseError::EmptyField("features".to_string()));
    }

    Ok(())
}

/// Extract a string field from a JSON object
///
/// # Errors
///
/// Returns `ParseError` if the field is missing or not a string.
fn extract_string_field(obj: &serde_json::Map<String, Value>, field: &str) -> Result<String, ParseError> {
    obj.get(field).map_or_else(
        || Err(ParseError::MissingField(field.to_string())),
        |value| {
            value.as_str().map(str::to_string).ok_or_else(|| ParseError::InvalidType {
                field: field.to_string(),
                expected: "string".to_string(),
                actual: json_type_name(value),
            })
        },
    )
}

/// Get a human-readable type name for a JSON value
fn json_type_name(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(_) => "boolean".to_string(),
        Value::Number(_) => "number".to_string(),
        Value::String(_) => "string".to_string(),
        Value::Array(_) => "array".to_string(),
        Value::Object(_) => "object".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_spec_minimal() {
        let json = r#"{"name": "test-spec"}"#;
        let result = parse_spec(json);

        assert!(result.is_ok());
        let spec = result.expect("spec should parse");
        assert_eq!(spec.name, "test-spec");
        assert!(spec.description.is_empty());
        assert!(spec.features.is_empty());
    }

    #[test]
    fn test_parse_spec_with_description() {
        let json = r#"{"name": "test-spec", "description": "A test specification"}"#;
        let result = parse_spec(json);

        assert!(result.is_ok());
        let spec = result.expect("spec should parse");
        assert_eq!(spec.name, "test-spec");
        assert_eq!(spec.description, "A test specification");
    }

    #[test]
    fn test_parse_spec_with_features() {
        let json = r#"{
            "name": "test-spec",
            "features": [
                {
                    "name": "auth",
                    "description": "Authentication",
                    "behaviors": [
                        {"name": "login", "description": "User login"}
                    ]
                }
            ]
        }"#;
        let result = parse_spec(json);

        assert!(result.is_ok());
        let spec = result.expect("spec should parse");
        assert_eq!(spec.features.len(), 1);
        assert_eq!(spec.features[0].name, "auth");
    }

    #[test]
    fn test_parse_spec_missing_name() {
        let json = r#"{"description": "No name"}"#;
        let result = parse_spec(json);

        assert!(result.is_err());
        let err = result.expect_err("should fail");
        assert!(matches!(err, ParseError::MissingField(f) if f == "name"));
    }

    #[test]
    fn test_parse_spec_empty_name() {
        let json = r#"{"name": ""}"#;
        let result = parse_spec(json);

        assert!(result.is_err());
        let err = result.expect_err("should fail");
        assert!(matches!(err, ParseError::EmptyField(f) if f == "name"));
    }

    #[test]
    fn test_parse_spec_whitespace_name() {
        let json = r#"{"name": "   "}"#;
        let result = parse_spec(json);

        assert!(result.is_err());
        let err = result.expect_err("should fail");
        assert!(matches!(err, ParseError::EmptyField(f) if f == "name"));
    }

    #[test]
    fn test_parse_spec_invalid_name_type() {
        let json = r#"{"name": 123}"#;
        let result = parse_spec(json);

        assert!(result.is_err());
        let err = result.expect_err("should fail");
        assert!(matches!(err, ParseError::InvalidType { field, expected: _, .. } if field == "name"));
    }

    #[test]
    fn test_parse_spec_malformed_json() {
        let json = r#"{"name": "test"#;  // Missing closing quote and brace
        let result = parse_spec(json);

        assert!(result.is_err());
        let err = result.expect_err("should fail");
        assert!(matches!(err, ParseError::JsonError(_)));
    }

    #[test]
    fn test_parse_spec_not_an_object() {
        let json = r#"["not", "an", "object"]"#;
        let result = parse_spec(json);

        assert!(result.is_err());
        let err = result.expect_err("should fail");
        assert!(matches!(err, ParseError::InvalidType { field, expected: _, .. } if field == "root"));
    }

    #[test]
    fn test_parse_spec_with_null_bytes() {
        let json = "{\"name\": \"test\0spec\"}";
        let result = parse_spec(json);

        // Should parse after sanitization, but name will be "testspec"
        assert!(result.is_ok());
        let spec = result.expect("spec should parse");
        assert_eq!(spec.name, "testspec");
    }

    #[test]
    fn test_parse_spec_with_whitespace() {
        let json = r#"

           {"name": "test-spec"}

        "#;
        let result = parse_spec(json);

        assert!(result.is_ok());
        let spec = result.expect("spec should parse");
        assert_eq!(spec.name, "test-spec");
    }

    #[test]
    fn test_parse_spec_full() {
        let json = r#"{
            "name": "full-spec",
            "description": "A complete specification",
            "features": [
                {
                    "name": "auth",
                    "description": "Authentication",
                    "behaviors": [
                        {
                            "name": "login",
                            "description": "User login",
                            "verification": {
                                "verification_type": "unit_test",
                                "description": "Test login"
                            }
                        }
                    ]
                }
            ],
            "invariants": [
                {"name": "unique_email", "description": "Emails must be unique"}
            ],
            "anti_patterns": [
                {"name": "plain_text_password", "description": "Don't store plain text passwords"}
            ]
        }"#;
        let result = parse_spec(json);

        assert!(result.is_ok());
        let spec = result.expect("spec should parse");
        assert_eq!(spec.name, "full-spec");
        assert_eq!(spec.description, "A complete specification");
        assert_eq!(spec.features.len(), 1);
        assert_eq!(spec.invariants.len(), 1);
        assert_eq!(spec.anti_patterns.len(), 1);
    }

    #[test]
    fn test_sanitize_string_removes_null_bytes() {
        let input = "hello\0world";
        let result = sanitize_string(input);
        assert_eq!(result, "helloworld");
    }

    #[test]
    fn test_sanitize_string_trims_whitespace() {
        let input = "  hello world  ";
        let result = sanitize_string(input);
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_sanitize_string_combined() {
        let input = "  hello\0world  ";
        let result = sanitize_string(input);
        assert_eq!(result, "helloworld");
    }

    #[test]
    fn test_sanitize_string_empty() {
        let input = "";
        let result = sanitize_string(input);
        assert_eq!(result, "");
    }

    #[test]
    fn test_sanitize_string_only_nulls_and_whitespace() {
        let input = " \0 \0 ";
        let result = sanitize_string(input);
        assert_eq!(result, "");
    }

    #[test]
    fn test_validate_spec_valid() {
        let json = r#"{
            "name": "test-spec",
            "features": [{"name": "auth", "behaviors": [{"name": "login"}]}]
        }"#;
        let spec = parse_spec(json).expect("spec should parse");
        let result = validate_spec(&spec);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_spec_empty_features() {
        let json = r#"{"name": "test-spec", "features": []}"#;
        let spec = parse_spec(json).expect("spec should parse");
        let result = validate_spec(&spec);

        assert!(result.is_err());
        let err = result.expect_err("should fail validation");
        assert!(matches!(err, ParseError::EmptyField(f) if f == "features"));
    }

    #[test]
    fn test_validate_spec_no_features_field() {
        let json = r#"{"name": "test-spec"}"#;
        let spec = parse_spec(json).expect("spec should parse");
        let result = validate_spec(&spec);

        assert!(result.is_err());
        let err = result.expect_err("should fail validation");
        assert!(matches!(err, ParseError::EmptyField(f) if f == "features"));
    }

    #[test]
    fn test_parse_spec_from_value_valid() {
        let value = serde_json::json!({
            "name": "test-spec",
            "description": "Test"
        });
        let result = parse_spec_from_value(&value);

        assert!(result.is_ok());
        let spec = result.expect("spec should parse");
        assert_eq!(spec.name, "test-spec");
    }

    #[test]
    fn test_parse_spec_from_value_not_object() {
        let value = serde_json::json!("not an object");
        let result = parse_spec_from_value(&value);

        assert!(result.is_err());
        let err = result.expect_err("should fail");
        assert!(matches!(err, ParseError::InvalidType { field, expected: _, .. } if field == "root"));
    }

    #[test]
    fn test_parse_spec_from_value_missing_name() {
        let value = serde_json::json!({"description": "No name"});
        let result = parse_spec_from_value(&value);

        assert!(result.is_err());
        let err = result.expect_err("should fail");
        assert!(matches!(err, ParseError::MissingField(f) if f == "name"));
    }

    #[test]
    fn test_json_type_name() {
        assert_eq!(json_type_name(&Value::Null), "null");
        assert_eq!(json_type_name(&Value::Bool(true)), "boolean");
        assert_eq!(json_type_name(&serde_json::json!(42)), "number");
        assert_eq!(json_type_name(&serde_json::json!("string")), "string");
        assert_eq!(json_type_name(&serde_json::json!([])), "array");
        assert_eq!(json_type_name(&serde_json::json!({})), "object");
    }

    #[test]
    fn test_parse_error_display() {
        let err = ParseError::JsonError("test error".to_string());
        assert!(format!("{err}").contains("test error"));

        let err = ParseError::MissingField("name".to_string());
        assert!(format!("{err}").contains("name"));

        let err = ParseError::InvalidType {
            field: "test".to_string(),
            expected: "string".to_string(),
            actual: "number".to_string(),
        };
        let msg = format!("{err}");
        assert!(msg.contains("test"));
        assert!(msg.contains("string"));
        assert!(msg.contains("number"));

        let err = ParseError::EmptyField("name".to_string());
        assert!(format!("{err}").contains("name"));
    }

    #[test]
    fn test_parse_spec_with_ai_hints() {
        let json = r#"{
            "name": "test-spec",
            "features": [{"name": "auth", "behaviors": [{"name": "login"}]}],
            "ai_hints": {
                "preferred_libraries": ["serde", "tokio"],
                "style_hints": ["Use functional patterns"]
            }
        }"#;
        let result = parse_spec(json);

        assert!(result.is_ok());
        let spec = result.expect("spec should parse");
        assert_eq!(spec.ai_hints.preferred_libraries, vec!["serde", "tokio"]);
        assert_eq!(spec.ai_hints.style_hints, vec!["Use functional patterns"]);
    }

    #[test]
    fn test_parse_spec_with_dependencies() {
        let json = r#"{
            "name": "test-spec",
            "features": [
                {"name": "auth", "behaviors": [{"name": "login"}]},
                {"name": "users", "depends_on": ["auth"], "behaviors": [{"name": "create"}]}
            ]
        }"#;
        let result = parse_spec(json);

        assert!(result.is_ok());
        let spec = result.expect("spec should parse");
        assert_eq!(spec.features[1].depends_on, vec!["auth"]);
    }

    #[test]
    fn test_parse_spec_unicode() {
        let json = r#"{"name": "test-αβγ-日本語", "description": "Unicode: émojis 🎉"}"#;
        let result = parse_spec(json);

        assert!(result.is_ok());
        let spec = result.expect("spec should parse");
        assert_eq!(spec.name, "test-αβγ-日本語");
        assert_eq!(spec.description, "Unicode: émojis 🎉");
    }

    #[test]
    fn test_parse_spec_escaped_characters() {
        let json = r#"{"name": "test", "description": "Line1\nLine2\tTabbed"}"#;
        let result = parse_spec(json);

        assert!(result.is_ok());
        let spec = result.expect("spec should parse");
        assert!(spec.description.contains('\n'));
        assert!(spec.description.contains('\t'));
    }
}

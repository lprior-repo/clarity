use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonValue {
  String(String),
  Number(f64),
  Boolean(bool),
  Array(Vec<Self>),
  Object(Vec<(String, Self)>),
  Null,
}

impl JsonValue {
  pub fn string(s: impl Into<String>) -> Self {
    Self::String(s.into())
  }

  #[must_use]
  pub const fn number(n: f64) -> Self {
    Self::Number(n)
  }

  #[must_use]
  pub const fn boolean(b: bool) -> Self {
    Self::Boolean(b)
  }

  pub fn array(v: impl Into<Vec<Self>>) -> Self {
    Self::Array(v.into())
  }

  pub fn object(v: impl Into<Vec<(String, Self)>>) -> Self {
    Self::Object(v.into())
  }

  #[must_use]
  pub const fn null() -> Self {
    Self::Null
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
  pub field: String,
  pub message: String,
  pub next_actions: Vec<String>,
}

impl ErrorDetail {
  pub fn new(
    field: impl Into<String>,
    message: impl Into<String>,
    next_actions: impl Into<Vec<String>>,
  ) -> Self {
    Self {
      field: field.into(),
      message: message.into(),
      next_actions: next_actions.into(),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
  pub status: String,
  pub message: Option<String>,
  pub data: Option<JsonValue>,
  pub errors: Option<Vec<ErrorDetail>>,
  pub next_actions: Option<Vec<String>>,
  pub timestamp: String,
}

impl ApiResponse {
  pub fn success(message: impl Into<String>, data: Option<JsonValue>) -> Self {
    Self {
      status: "success".to_string(),
      message: Some(message.into()),
      data,
      errors: None,
      next_actions: None,
      timestamp: chrono::Utc::now().to_rfc3339(),
    }
  }

  pub fn error(message: impl Into<String>, errors: Vec<ErrorDetail>) -> Self {
    let next_actions = Self::extract_next_actions(&errors);
    Self {
      status: "error".to_string(),
      message: Some(message.into()),
      data: None,
      errors: Some(errors),
      next_actions: Some(next_actions),
      timestamp: chrono::Utc::now().to_rfc3339(),
    }
  }

  fn extract_next_actions(errors: &[ErrorDetail]) -> Vec<String> {
    errors
      .iter()
      .flat_map(|error| error.next_actions.clone())
      .collect()
  }
}

#[derive(Debug)]
pub struct JsonFormatterError {
  pub message: String,
}

impl fmt::Display for JsonFormatterError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.message)
  }
}

impl std::error::Error for JsonFormatterError {}

pub struct JsonFormatter {
  pretty: bool,
}

impl Default for JsonFormatter {
  fn default() -> Self {
    Self::new()
  }
}

impl JsonFormatter {
  #[must_use]
  pub const fn new() -> Self {
    Self { pretty: false }
  }

  #[must_use]
  pub const fn with_pretty(pretty: bool) -> Self {
    Self { pretty }
  }

  /// Formats a successful response
  ///
  /// # Errors
  /// Returns `JsonFormatterError::Serialization` if the response cannot be serialized
  pub fn format_success(&self, message: impl Into<String>) -> Result<String, JsonFormatterError> {
    let api_response = ApiResponse::success(message, None);
    self.serialize(api_response)
  }

  /// Formats an API response with status, message, and data
  ///
  /// # Errors
  /// Returns `JsonFormatterError::Serialization` if the response cannot be serialized
  pub fn format_response(
    &self,
    status: impl Into<String>,
    message: impl Into<String>,
    data: JsonValue,
  ) -> Result<String, JsonFormatterError> {
    let api_response = ApiResponse {
      status: status.into(),
      message: Some(message.into()),
      data: Some(data),
      errors: None,
      next_actions: None,
      timestamp: chrono::Utc::now().to_rfc3339(),
    };
    self.serialize(api_response)
  }

  /// Formats an error response with message and error details
  ///
  /// # Errors
  /// Returns `JsonFormatterError::Serialization` if the response cannot be serialized
  pub fn format_error(
    &self,
    message: impl Into<String>,
    errors: Vec<ErrorDetail>,
  ) -> Result<String, JsonFormatterError> {
    let api_response = ApiResponse::error(message, errors);
    self.serialize(api_response)
  }

  fn serialize<T>(&self, value: T) -> Result<String, JsonFormatterError>
  where
    T: Serialize,
  {
    let json_value = serde_json::to_value(&value).map_err(|e| JsonFormatterError {
      message: format!("Failed to serialize: {e}"),
    })?;

    if self.pretty {
      serde_json::to_string_pretty(&json_value).map_err(|e| JsonFormatterError {
        message: format!("Failed to format JSON: {e}"),
      })
    } else {
      serde_json::to_string(&json_value).map_err(|e| JsonFormatterError {
        message: format!("Failed to format JSON: {e}"),
      })
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  #[allow(clippy::unwrap_used)]
  #[allow(clippy::expect_used)]
  #[allow(clippy::float_cmp)]
  #[allow(clippy::uninlined_format_args)]
  #[allow(clippy::single_char_pattern)]
  #[test]
  fn test_json_formatter_basic() {
    let formatter = JsonFormatter::new();
    let result = formatter.format_success("Operation completed successfully");
    assert!(result.is_ok());
    let json_str = result.unwrap();
    assert!(json_str.contains("\"status\":\"success\""));
    assert!(json_str.contains("\"message\":\"Operation completed successfully\""));
    assert!(json_str.contains("\"timestamp\""));
  }

  #[test]
  #[allow(clippy::unwrap_used)]
  fn test_json_formatter_with_data() {
    let formatter = JsonFormatter::new();
    let data = JsonValue::object(vec![(
      "key".to_string(),
      JsonValue::string("value".to_string()),
    )]);
    let result = formatter.format_response("success", "Operation completed", data);
    assert!(result.is_ok());
    let json_str = result.unwrap();
    assert!(json_str.contains("\"status\":\"success\""));
    assert!(json_str.contains("\"data\""));
  }

  #[test]
  #[allow(clippy::unwrap_used)]
  fn test_json_formatter_error_with_next_actions() {
    let formatter = JsonFormatter::new();
    let errors = vec![ErrorDetail::new(
      "field1",
      "Value must be valid",
      vec!["Check field format".to_string()],
    )];
    let result = formatter.format_error("validation_failed", errors);
    assert!(result.is_ok());
    let json_str = result.unwrap();
    assert!(json_str.contains("\"status\":\"error\""));
    assert!(json_str.contains("\"next_actions\""));
  }

  #[test]
  #[allow(clippy::unwrap_used)]
  fn test_json_formatter_error_without_next_actions() {
    let formatter = JsonFormatter::new();
    let errors = vec![ErrorDetail::new("field1", "Value must be valid", vec![])];
    let result = formatter.format_error("validation_failed", errors);
    assert!(result.is_ok());
    let json_str = result.unwrap();
    assert!(json_str.contains("\"status\":\"error\""));
    assert!(json_str.contains("\"next_actions\":[]"));
  }

  #[test]
  #[allow(clippy::unwrap_used)]
  fn test_json_formatter_complex_nested_structure() {
    let formatter = JsonFormatter::new();
    let nested_data = JsonValue::object(vec![(
      "errors".to_string(),
      JsonValue::array(vec![JsonValue::object(vec![
        ("field".to_string(), JsonValue::string("field1".to_string())),
        (
          "message".to_string(),
          JsonValue::string("Invalid value".to_string()),
        ),
      ])]),
    )]);
    let result = formatter.format_response("error", "Validation failed", nested_data);
    assert!(result.is_ok());
    let json_str = result.unwrap();
    assert!(json_str.contains("\"status\":\"error\""));
    assert!(json_str.contains("\"errors\""));
  }

  #[test]
  #[allow(clippy::unwrap_used)]
  fn test_json_formatter_empty_errors() {
    let formatter = JsonFormatter::new();
    let result = formatter.format_error("error", vec![]);
    assert!(result.is_ok());
    let json_str = result.unwrap();
    assert!(json_str.contains("\"status\":\"error\""));
    assert!(json_str.contains("\"errors\":[]"));
  }

  #[test]
  fn test_json_formatter_pretty_formatting() {
    let formatter = JsonFormatter::with_pretty(true);
    let result = formatter.format_success("Test message");

    let validation = result.and_then(|json_str| {
      if json_str.contains('\n') && json_str.contains("\"status\":") {
        Ok(())
      } else {
        Err(JsonFormatterError {
          message: "Pretty formatting not applied".to_string(),
        })
      }
    });

    assert!(validation.is_ok(), "Pretty formatting should be applied");
  }

  #[test]
  fn test_json_value_constructors() {
    // Test all JsonValue constructors work correctly
    let str_val = JsonValue::string("test");
    let num_val = JsonValue::number(42.0);
    let bool_val = JsonValue::boolean(true);
    let null_val = JsonValue::null();
    let arr_val = JsonValue::array(vec![JsonValue::string("item")]);
    let obj_val = JsonValue::object(vec![("key".to_string(), JsonValue::string("value"))]);

    // Verify variants using pattern matching
    let checks: Vec<(&str, bool)> = vec![
      ("String variant", matches!(str_val, JsonValue::String(_))),
      ("Number variant", matches!(num_val, JsonValue::Number(_))),
      ("Boolean variant", matches!(bool_val, JsonValue::Boolean(_))),
      ("Null variant", matches!(null_val, JsonValue::Null)),
      ("Array variant", matches!(arr_val, JsonValue::Array(_))),
      ("Object variant", matches!(obj_val, JsonValue::Object(_))),
    ];

    assert!(
      checks.iter().all(|(_, passed)| *passed),
      "All JsonValue constructors should create correct variants"
    );
  }

  #[test]
  fn test_api_response_success() {
    let response = ApiResponse::success("Test message", None);

    assert_eq!(response.status, "success");
    assert_eq!(response.message, Some("Test message".to_string()));
    assert!(response.data.is_none());
    assert!(response.errors.is_none());
  }

  #[test]
  fn test_api_response_error() {
    let errors = vec![ErrorDetail::new(
      "field",
      "error",
      vec!["fix it".to_string()],
    )];
    let response = ApiResponse::error("Validation failed", errors.clone());

    assert_eq!(response.status, "error");
    assert_eq!(response.message, Some("Validation failed".to_string()));
    assert!(response.errors.is_some());

    // Check next_actions using is_some and length assertion
    assert!(response.next_actions.is_some());
    let next_actions_count = response.next_actions.as_ref().map_or(0, Vec::len);
    assert_eq!(next_actions_count, 1);
  }

  #[test]
  fn test_error_detail_new() {
    let detail = ErrorDetail::new("field1", "Invalid value", vec!["Check format".to_string()]);

    assert_eq!(detail.field, "field1");
    assert_eq!(detail.message, "Invalid value");
    assert_eq!(detail.next_actions.len(), 1);
    assert_eq!(
      detail.next_actions.get(0),
      Some(&"Check format".to_string())
    );
  }
}

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
    Null,
}

impl JsonValue {
    pub fn string(s: impl Into<String>) -> Self {
        Self::String(s.into())
    }

    pub fn number(n: f64) -> Self {
        Self::Number(n)
    }

    pub fn boolean(b: bool) -> Self {
        Self::Boolean(b)
    }

    pub fn array(v: impl Into<Vec<JsonValue>>) -> Self {
        Self::Array(v.into())
    }

    pub fn object(v: impl Into<Vec<(String, JsonValue)>>) -> Self {
        Self::Object(v.into())
    }

    pub fn null() -> Self {
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

impl JsonFormatter {
    pub fn new() -> Self {
        Self { pretty: false }
    }

    pub fn with_pretty(pretty: bool) -> Self {
        Self { pretty }
    }

    pub fn format_success(&self, message: impl Into<String>) -> Result<String, JsonFormatterError> {
        let api_response = ApiResponse::success(message, None);
        self.serialize(api_response)
    }

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
            message: format!("Failed to serialize: {}", e),
        })?;

        if self.pretty {
            serde_json::to_string_pretty(&json_value).map_err(|e| JsonFormatterError {
                message: format!("Failed to format JSON: {}", e),
            })
        } else {
            serde_json::to_string(&json_value).map_err(|e| JsonFormatterError {
                message: format!("Failed to format JSON: {}", e),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(result.is_ok());
        let json_str = result.unwrap();
        assert!(json_str.contains("\n"));
        assert!(json_str.contains("\"status\":"));
    }
}

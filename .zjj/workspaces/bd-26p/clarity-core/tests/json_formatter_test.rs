#[cfg(test)]
mod tests {
    use clarity_core::json_formatter::ErrorDetail;
    use clarity_core::json_formatter::JsonFormatter;
    use clarity_core::json_formatter::JsonValue;

    #[test]
    fn test_json_formatter_basic() {
        let formatter = JsonFormatter::new();
        let result = formatter.format_success("Operation completed successfully");
        assert!(result.is_ok());
        let json_str = result.unwrap();
        assert!(json_str.contains("\"status\":\"success\""));
        assert!(json_str.contains("\"message\":\"Operation completed successfully\""));
    }

    #[test]
    fn test_json_formatter_with_data() {
        let formatter = JsonFormatter::new();
        let data = JsonValue::Object(vec![(
            "key".to_string(),
            JsonValue::String("value".to_string()),
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
        let errors = vec![ErrorDetail {
            field: "field1".to_string(),
            message: "Value must be valid".to_string(),
            next_actions: vec!["Check field format".to_string()],
        }];
        let result = formatter.format_error("validation_failed", errors);
        assert!(result.is_ok());
        let json_str = result.unwrap();
        assert!(json_str.contains("\"status\":\"error\""));
        assert!(json_str.contains("\"next_actions\""));
    }

    #[test]
    fn test_json_formatter_error_without_next_actions() {
        let formatter = JsonFormatter::new();
        let errors = vec![ErrorDetail {
            field: "field1".to_string(),
            message: "Value must be valid".to_string(),
            next_actions: vec![],
        }];
        let result = formatter.format_error("validation_failed", errors);
        assert!(result.is_ok());
        let json_str = result.unwrap();
        assert!(json_str.contains("\"status\":\"error\""));
        assert!(json_str.contains("\"next_actions\":[]"));
    }

    #[test]
    fn test_json_formatter_complex_nested_structure() {
        let formatter = JsonFormatter::new();
        let nested_data = JsonValue::Object(vec![(
            "errors".to_string(),
            JsonValue::Array(vec![JsonValue::Object(vec![
                ("field".to_string(), JsonValue::String("field1".to_string())),
                (
                    "message".to_string(),
                    JsonValue::String("Invalid value".to_string()),
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

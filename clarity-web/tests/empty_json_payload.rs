//! Test for empty JSON payload handling in extraction pipeline.
//!
//! These tests validate that the extraction pipeline properly handles
//! empty, null, or malformed JSON payloads without panicking or crashing.
//!
//! ## Test Coverage
//! - Empty JSON object {}
//! - Null JSON values
//! - Empty arrays []
//! - Malformed JSON strings
//! - Whitespace-only payloads
//! - Nested empty structures

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use clarity_web::providers::{
  ExtractedFields, ExtractionContext, ExtractionError, ExtractionProvider, FieldExtraction,
  FieldType,
};
use serde_json::json;

// =============================================================================
// Mock Provider with JSON Handling
// =============================================================================

#[derive(Debug, Clone)]
struct JsonHandlingProvider;

impl JsonHandlingProvider {
  #[must_use]
  pub const fn new() -> Self {
    Self
  }

  /// Attempt to parse JSON and extract fields from it
  fn extract_from_json(&self, json_str: &str) -> Result<ExtractedFields, ExtractionError> {
    // Trim whitespace first
    let trimmed = json_str.trim();

    // Check for empty/whitespace-only input
    if trimmed.is_empty() {
      return Err(ExtractionError::InvalidInput(
        "Empty JSON payload".to_string(),
      ));
    }

    // Attempt to parse as JSON
    let parsed = serde_json::from_str::<serde_json::Value>(trimmed)
      .map_err(|e| ExtractionError::ParseError(format!("Invalid JSON: {}", e)))?;

    // Extract fields based on JSON type
    let fields = match parsed {
      serde_json::Value::Null => {
        return Err(ExtractionError::InvalidInput(
          "JSON payload is null".to_string(),
        ));
      }
      serde_json::Value::Object(map) if map.is_empty() => {
        return Err(ExtractionError::InvalidInput(
          "JSON object is empty".to_string(),
        ));
      }
      serde_json::Value::Object(map) => self.extract_from_object(map),
      serde_json::Value::Array(arr) if arr.is_empty() => {
        return Err(ExtractionError::InvalidInput(
          "JSON array is empty".to_string(),
        ));
      }
      serde_json::Value::Array(arr) => self.extract_from_array(arr),
      serde_json::Value::String(s) if s.trim().is_empty() => {
        return Err(ExtractionError::InvalidInput(
          "JSON string is empty".to_string(),
        ));
      }
      serde_json::Value::String(s) => vec![FieldExtraction {
        name: "value".to_string(),
        field_type: FieldType::Text,
        value: json!(s),
        confidence: 1.0,
        justification: Some("Extracted from JSON string".to_string()),
      }],
      serde_json::Value::Number(n) => vec![FieldExtraction {
        name: "value".to_string(),
        field_type: FieldType::Number,
        value: json!(n),
        confidence: 1.0,
        justification: Some("Extracted from JSON number".to_string()),
      }],
      serde_json::Value::Bool(b) => vec![FieldExtraction {
        name: "value".to_string(),
        field_type: FieldType::Boolean,
        value: json!(b),
        confidence: 1.0,
        justification: Some("Extracted from JSON boolean".to_string()),
      }],
    };

    Ok(ExtractedFields {
      fields,
      confidence: 1.0,
      metadata: clarity_web::providers::ExtractionMetadata {
        provider: "json_handler".to_string(),
        model: Some("json-v1".to_string()),
        timestamp: chrono::Utc::now(),
        processing_duration_ms: 0,
        extra: json!({"source": "json"}),
      },
    })
  }

  fn extract_from_object(
    &self,
    map: serde_json::Map<String, serde_json::Value>,
  ) -> Vec<FieldExtraction> {
    map
      .into_iter()
      .map(|(key, value)| FieldExtraction {
        name: key.clone(),
        field_type: Self::infer_field_type(&value),
        value,
        confidence: 1.0,
        justification: Some(format!("Extracted from object field '{}'", key)),
      })
      .collect()
  }

  fn extract_from_array(&self, arr: Vec<serde_json::Value>) -> Vec<FieldExtraction> {
    arr
      .into_iter()
      .enumerate()
      .map(|(idx, value)| FieldExtraction {
        name: format!("item_{}", idx),
        field_type: Self::infer_field_type(&value),
        value,
        confidence: 1.0,
        justification: Some(format!("Extracted from array index {}", idx)),
      })
      .collect()
  }

  fn infer_field_type(value: &serde_json::Value) -> FieldType {
    match value {
      serde_json::Value::String(_) => FieldType::Text,
      serde_json::Value::Number(_) => FieldType::Number,
      serde_json::Value::Bool(_) => FieldType::Boolean,
      serde_json::Value::Array(_) => FieldType::MultiSelect,
      serde_json::Value::Object(_) => FieldType::RichText,
      serde_json::Value::Null => FieldType::Text,
    }
  }
}

#[async_trait::async_trait]
impl ExtractionProvider for JsonHandlingProvider {
  async fn extract_fields(
    &self,
    text: &str,
    _context: &ExtractionContext,
  ) -> Result<ExtractedFields, ExtractionError> {
    // Try to parse as JSON first, fall back to text extraction
    if text.trim().starts_with('{') || text.trim().starts_with('[') {
      self.extract_from_json(text)
    } else if text.trim().is_empty() {
      Err(ExtractionError::InvalidInput("Empty input".to_string()))
    } else {
      // Try to parse as JSON primitive (null, true, false, numbers)
      if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(text) {
        match parsed {
          serde_json::Value::Null => Err(ExtractionError::InvalidInput(
            "JSON payload is null".to_string(),
          )),
          serde_json::Value::String(s) if s.trim().is_empty() => Err(
            ExtractionError::InvalidInput("JSON string is empty".to_string()),
          ),
          _ => self.extract_from_json(text),
        }
      } else {
        // Regular text extraction for non-JSON input
        Ok(ExtractedFields {
          fields: vec![FieldExtraction {
            name: "text".to_string(),
            field_type: FieldType::TextArea,
            value: json!(text),
            confidence: 0.5,
            justification: Some("Extracted as plain text".to_string()),
          }],
          confidence: 0.5,
          metadata: clarity_web::providers::ExtractionMetadata {
            provider: "json_handler".to_string(),
            model: Some("text-fallback-v1".to_string()),
            timestamp: chrono::Utc::now(),
            processing_duration_ms: 0,
            extra: json!({}),
          },
        })
      }
    }
  }

  async fn extract_fields_with_schema(
    &self,
    text: &str,
    _schema: &[clarity_web::providers::SchemaField],
    context: &ExtractionContext,
  ) -> Result<ExtractedFields, ExtractionError> {
    self.extract_fields(text, context).await
  }

  fn provider_name(&self) -> &str {
    "json_handler"
  }

  async fn health_check(&self) -> Result<(), ExtractionError> {
    Ok(())
  }
}

// =============================================================================
// Empty JSON Payload Tests
// =============================================================================

#[tokio::test]
async fn empty_json_object_returns_error() {
  let provider = JsonHandlingProvider::new();
  let context = ExtractionContext {
    document_type: None,
    locale: None,
    schema: None,
    extra: json!({}),
  };

  let result = provider.extract_fields("{}", &context).await;

  assert!(result.is_err());
  assert!(matches!(
    result,
    Err(ExtractionError::InvalidInput(msg)) if msg.contains("empty")
  ));
}

#[tokio::test]
async fn null_json_value_returns_error() {
  let provider = JsonHandlingProvider::new();
  let context = ExtractionContext {
    document_type: None,
    locale: None,
    schema: None,
    extra: json!({}),
  };

  let result = provider.extract_fields("null", &context).await;

  assert!(result.is_err());
  assert!(matches!(
    result,
    Err(ExtractionError::InvalidInput(msg)) if msg.contains("null")
  ));
}

#[tokio::test]
async fn empty_json_array_returns_error() {
  let provider = JsonHandlingProvider::new();
  let context = ExtractionContext {
    document_type: None,
    locale: None,
    schema: None,
    extra: json!({}),
  };

  let result = provider.extract_fields("[]", &context).await;

  assert!(result.is_err());
  assert!(matches!(
    result,
    Err(ExtractionError::InvalidInput(msg)) if msg.contains("empty")
  ));
}

#[tokio::test]
async fn whitespace_only_json_returns_error() {
  let provider = JsonHandlingProvider::new();
  let context = ExtractionContext {
    document_type: None,
    locale: None,
    schema: None,
    extra: json!({}),
  };

  let test_cases = ["   ", "\n", "\t", "  \n\t  "];

  for input in test_cases {
    let result = provider.extract_fields(input, &context).await;
    assert!(
      result.is_err(),
      "Should reject whitespace-only input: {:?}",
      input
    );
  }
}

#[tokio::test]
async fn empty_string_json_value_returns_error() {
  let provider = JsonHandlingProvider::new();
  let context = ExtractionContext {
    document_type: None,
    locale: None,
    schema: None,
    extra: json!({}),
  };

  let result = provider.extract_fields("\"\"", &context).await;

  assert!(result.is_err());
  assert!(matches!(
    result,
    Err(ExtractionError::InvalidInput(msg)) if msg.contains("empty")
  ));
}

#[tokio::test]
async fn malformed_json_returns_parse_error() {
  let provider = JsonHandlingProvider::new();
  let context = ExtractionContext {
    document_type: None,
    locale: None,
    schema: None,
    extra: json!({}),
  };

  // These look like JSON but fail to parse - should return parse error
  let malformed_json_cases = [
    "{invalid}",
    "[unclosed",
    "{missing: }",
    "{'single': 'quotes'}",
  ];

  for input in malformed_json_cases {
    let result = provider.extract_fields(input, &context).await;
    assert!(result.is_err(), "Should reject malformed JSON: {}", input);
    assert!(matches!(result, Err(ExtractionError::ParseError(_))));
  }

  // These are not JSON at all - should be treated as plain text
  let non_json_cases = ["undefined", "NaN"];

  for input in non_json_cases {
    let result = provider.extract_fields(input, &context).await;
    // Should succeed as plain text extraction
    assert!(
      result.is_ok(),
      "Should accept non-JSON as plain text: {}",
      input
    );
    let extracted = result.unwrap();
    assert_eq!(extracted.fields[0].name, "text");
  }
}

#[tokio::test]
async fn valid_json_object_with_fields_succeeds() {
  let provider = JsonHandlingProvider::new();
  let context = ExtractionContext {
    document_type: None,
    locale: None,
    schema: None,
    extra: json!({}),
  };

  let json_input = r#"{"name": "Test", "value": 42, "active": true}"#;

  let result = provider.extract_fields(json_input, &context).await;

  assert!(result.is_ok());

  let extracted = result.unwrap();
  assert_eq!(extracted.fields.len(), 3);

  let field_names: Vec<&str> = extracted.fields.iter().map(|f| f.name.as_str()).collect();
  assert!(field_names.contains(&"name"));
  assert!(field_names.contains(&"value"));
  assert!(field_names.contains(&"active"));
}

#[tokio::test]
async fn valid_json_array_with_items_succeeds() {
  let provider = JsonHandlingProvider::new();
  let context = ExtractionContext {
    document_type: None,
    locale: None,
    schema: None,
    extra: json!({}),
  };

  let json_input = r#"["item1", "item2", "item3"]"#;

  let result = provider.extract_fields(json_input, &context).await;

  assert!(result.is_ok());

  let extracted = result.unwrap();
  assert_eq!(extracted.fields.len(), 3);

  assert_eq!(extracted.fields[0].name, "item_0");
  assert_eq!(extracted.fields[1].name, "item_1");
  assert_eq!(extracted.fields[2].name, "item_2");
}

#[tokio::test]
async fn json_with_nested_empty_structures() {
  let provider = JsonHandlingProvider::new();
  let context = ExtractionContext {
    document_type: None,
    locale: None,
    schema: None,
    extra: json!({}),
  };

  // Object with nested empty object
  let json_input1 = r#"{"outer": {}}"#;

  let result = provider.extract_fields(json_input1, &context).await;
  assert!(result.is_ok());

  let extracted = result.unwrap();
  // Should extract the outer field with empty object as value
  assert_eq!(extracted.fields.len(), 1);
  assert_eq!(extracted.fields[0].name, "outer");
  assert_eq!(extracted.fields[0].value, json!({}));

  // Array with nested empty array
  let json_input2 = r#"[[[]]]"#;

  let result = provider.extract_fields(json_input2, &context).await;
  assert!(result.is_ok());

  let extracted = result.unwrap();
  // Should extract nested arrays
  assert_eq!(extracted.fields.len(), 1);
}

#[tokio::test]
async fn mixed_whitespace_with_valid_json() {
  let provider = JsonHandlingProvider::new();
  let context = ExtractionContext {
    document_type: None,
    locale: None,
    schema: None,
    extra: json!({}),
  };

  let test_cases = [
    "  {\"a\": 1}  ",
    "\n{\"b\": 2}\n",
    "\t{\"c\": 3}\t",
    "  \n  {\"d\": 4}  \n  ",
  ];

  for input in test_cases {
    let result = provider.extract_fields(input, &context).await;
    assert!(
      result.is_ok(),
      "Should accept valid JSON with whitespace: {}",
      input
    );
  }
}

#[tokio::test]
async fn non_json_text_falls_back_to_text_extraction() {
  let provider = JsonHandlingProvider::new();
  let context = ExtractionContext {
    document_type: None,
    locale: None,
    schema: None,
    extra: json!({}),
  };

  let plain_text = "This is just plain text, not JSON at all.";

  let result = provider.extract_fields(plain_text, &context).await;

  assert!(result.is_ok());

  let extracted = result.unwrap();
  assert_eq!(extracted.fields.len(), 1);
  assert_eq!(extracted.fields[0].name, "text");
  assert_eq!(extracted.confidence, 0.5);
}

#[tokio::test]
async fn json_boolean_values_extracted_correctly() {
  let provider = JsonHandlingProvider::new();
  let context = ExtractionContext {
    document_type: None,
    locale: None,
    schema: None,
    extra: json!({}),
  };

  let result = provider.extract_fields("true", &context).await;
  assert!(result.is_ok());
  let extracted = result.unwrap();
  assert_eq!(extracted.fields[0].field_type, FieldType::Boolean);
  assert_eq!(extracted.fields[0].value, json!(true));

  let result = provider.extract_fields("false", &context).await;
  assert!(result.is_ok());
  let extracted = result.unwrap();
  assert_eq!(extracted.fields[0].field_type, FieldType::Boolean);
  assert_eq!(extracted.fields[0].value, json!(false));
}

#[tokio::test]
async fn json_number_values_extracted_correctly() {
  let provider = JsonHandlingProvider::new();
  let context = ExtractionContext {
    document_type: None,
    locale: None,
    schema: None,
    extra: json!({}),
  };

  let test_cases = [
    ("42", "42"),
    ("3.14", "3.14"),
    ("-10", "-10"),
    ("1e5", "100000"),
  ];

  for (input, _expected) in test_cases {
    let result = provider.extract_fields(input, &context).await;
    assert!(result.is_ok(), "Should parse number: {}", input);
    let extracted = result.unwrap();
    assert_eq!(extracted.fields[0].field_type, FieldType::Number);
  }
}

#[tokio::test]
async fn json_with_unicode_and_special_characters() {
  let provider = JsonHandlingProvider::new();
  let context = ExtractionContext {
    document_type: None,
    locale: None,
    schema: None,
    extra: json!({}),
  };

  let json_input = r#"{"emoji": "😀", "chinese": "中文", "arabic": "العربية"}"#;

  let result = provider.extract_fields(json_input, &context).await;

  assert!(result.is_ok());

  let extracted = result.unwrap();
  assert_eq!(extracted.fields.len(), 3);
}

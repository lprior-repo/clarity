#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Integration tests for `OpenCode` provider

use clarity_web::providers::{
  ExtractionContext, ExtractionError, ExtractionProvider, OpenCodeProvider,
};

fn create_provider() -> OpenCodeProvider {
  OpenCodeProvider::new(
    "https://api.opencode.ai/v1".to_string(),
    "test-session".to_string(),
  )
  .expect("provider should be created")
}

#[test]
fn test_opencode_provider_creation() {
  let provider = OpenCodeProvider::new(
    "https://api.opencode.ai/v1".to_string(),
    "test-session-123".to_string(),
  )
  .expect("provider should be created");

  assert_eq!(provider.session_id(), "test-session-123");
  assert_eq!(provider.provider_name(), "opencode");
}

#[test]
fn test_opencode_empty_text_validation() {
  let provider = create_provider();

  let rt = tokio::runtime::Runtime::new().expect("runtime should be created");

  // Test empty text input - should fail without making network call
  let context = ExtractionContext {
    document_type: None,
    locale: None,
    schema: None,
    extra: serde_json::json!({}),
  };

  let result = rt.block_on(provider.extract_fields("", &context));

  assert!(matches!(
    result,
    Err(ExtractionError::InvalidInput(msg)) if msg == "Input text cannot be empty"
  ));
}

#[test]
fn test_opencode_empty_schema_validation() {
  let provider = create_provider();
  let rt = tokio::runtime::Runtime::new().expect("runtime should be created");

  let context = ExtractionContext {
    document_type: None,
    locale: None,
    schema: None,
    extra: serde_json::json!({}),
  };

  let result = rt.block_on(provider.extract_fields_with_schema("some text", &[], &context));

  assert!(matches!(
    result,
    Err(ExtractionError::InvalidInput(msg)) if msg == "Schema cannot be empty"
  ));
}

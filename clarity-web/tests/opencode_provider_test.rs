#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Integration tests for `OpenCode` provider

use clarity_web::providers::{
  ExtractionContext, ExtractionError, ExtractionProvider, OpenCodeProvider,
};

#[test]
fn test_opencode_provider_creation() {
  let provider_result = OpenCodeProvider::new(
    "https://api.opencode.com".to_string(),
    "test-session-123".to_string(),
  );

  assert!(provider_result.is_ok());

  if let Ok(provider) = provider_result {
    assert_eq!(provider.session_id(), "test-session-123");
    assert_eq!(provider.provider_name(), "opencode");
  }
}

#[test]
fn test_opencode_empty_text_validation() {
  let provider_result = OpenCodeProvider::new(
    "https://api.opencode.com".to_string(),
    "test-session".to_string(),
  );

  assert!(provider_result.is_ok());
  let provider = if let Ok(provider) = provider_result {
    provider
  } else {
    return;
  };

  // Create a runtime to test async function
  let runtime_result = tokio::runtime::Runtime::new();
  assert!(runtime_result.is_ok());
  let rt = if let Ok(runtime) = runtime_result {
    runtime
  } else {
    return;
  };

  // Test empty text input - should fail without making network call
  let context = ExtractionContext {
    document_type: None,
    locale: None,
    schema: None,
    extra: serde_json::json!({}),
  };

  let result = rt.block_on(provider.extract_fields("", &context));

  assert!(result.is_err());
  if let Err(ExtractionError::InvalidInput(msg)) = result {
    assert_eq!(msg, "Input text cannot be empty");
  } else {
    assert!(false, "Expected InvalidInput error");
  }
}

#[test]
fn test_opencode_empty_schema_validation() {
  let provider_result = OpenCodeProvider::new(
    "https://api.opencode.com".to_string(),
    "test-session".to_string(),
  );

  assert!(provider_result.is_ok());
  let provider = if let Ok(provider) = provider_result {
    provider
  } else {
    return;
  };

  let runtime_result = tokio::runtime::Runtime::new();
  assert!(runtime_result.is_ok());
  let rt = if let Ok(runtime) = runtime_result {
    runtime
  } else {
    return;
  };

  let context = ExtractionContext {
    document_type: None,
    locale: None,
    schema: None,
    extra: serde_json::json!({}),
  };

  let result = rt.block_on(provider.extract_fields_with_schema("some text", &[], &context));

  assert!(result.is_err());
  if let Err(ExtractionError::InvalidInput(msg)) = result {
    assert_eq!(msg, "Schema cannot be empty");
  } else {
    assert!(false, "Expected InvalidInput error");
  }
}

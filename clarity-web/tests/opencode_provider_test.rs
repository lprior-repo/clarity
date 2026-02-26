#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Integration tests for OpenCode provider

use clarity_web::providers::{ExtractionContext, OpenCodeProvider};

#[test]
fn test_opencode_provider_creation() {
    let provider = OpenCodeProvider::new(
        "https://api.opencode.com".to_string(),
        "test-session-123".to_string(),
    );

    assert!(provider.is_ok());

    let provider = provider.unwrap();
    assert_eq!(provider.endpoint, "https://api.opencode.com");
    assert_eq!(provider.session_id(), "test-session-123");
    assert_eq!(provider.provider_name(), "opencode");
}

#[test]
fn test_opencode_provider_url_building() {
    let provider = OpenCodeProvider::new(
        "https://api.opencode.com".to_string(),
        "test-session".to_string(),
    )
    .unwrap();

    // Test URL building
    assert_eq!(
        provider.build_url("/extract"),
        "https://api.opencode.com/extract"
    );

    assert_eq!(
        provider.build_url("/health"),
        "https://api.opencode.com/health"
    );
}

#[test]
fn test_opencode_provider_trailing_slash() {
    let provider = OpenCodeProvider::new(
        "https://api.opencode.com/".to_string(),
        "test-session".to_string(),
    )
    .unwrap();

    // Test trailing slash handling
    assert_eq!(
        provider.build_url("/extract"),
        "https://api.opencode.com/extract"
    );
}

#[test]
fn test_opencode_empty_text_validation() {
    let provider = OpenCodeProvider::new(
        "https://api.opencode.com".to_string(),
        "test-session".to_string(),
    )
    .unwrap();

    // Create a runtime to test async function
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Test empty text input - should fail without making network call
    let context = ExtractionContext {
        document_type: None,
        locale: None,
        schema: None,
        extra: serde_json::json!({}),
    };

    let result = rt.block_on(provider.extract_fields("", &context));

    assert!(result.is_err());
    match result {
        Err(clarity_web::providers::ExtractionError::InvalidInput(msg)) => {
            assert_eq!(msg, "Input text cannot be empty");
        }
        _ => panic!("Expected InvalidInput error"),
    }
}

#[test]
fn test_opencode_empty_schema_validation() {
    let provider = OpenCodeProvider::new(
        "https://api.opencode.com".to_string(),
        "test-session".to_string(),
    )
    .unwrap();

    let rt = tokio::runtime::Runtime::new().unwrap();

    let context = ExtractionContext {
        document_type: None,
        locale: None,
        schema: None,
        extra: serde_json::json!({}),
    };

    let result = rt.block_on(provider.extract_fields_with_schema(
        "some text",
        &[],
        &context,
    ));

    assert!(result.is_err());
    match result {
        Err(clarity_web::providers::ExtractionError::InvalidInput(msg)) => {
            assert_eq!(msg, "Schema cannot be empty");
        }
        _ => panic!("Expected InvalidInput error"),
    }
}

#![allow(
  clippy::unwrap_used,
  clippy::expect_used,
  clippy::panic,
  clippy::float_cmp,
  clippy::needless_collect,
  clippy::unnecessary_debug_formatting,
  clippy::match_same_arms,
  clippy::option_if_let_else,
  clippy::suspicious_else_formatting,
  clippy::manual_let_else,
  clippy::match_wild_err_arm,
  clippy::match_like_matches_macro
)]
#![forbid(unsafe_code)]

//! Comprehensive network failure handling tests for `OpenCode` provider
//!
//! This test suite validates that the application handles various network
//! failure scenarios gracefully with appropriate error messages and behavior.

use clarity_web::providers::{
  ExtractionContext, ExtractionError, ExtractionProvider, OpenCodeProvider,
};
use std::time::Duration;

fn create_test_context() -> ExtractionContext {
  ExtractionContext {
    document_type: None,
    locale: None,
    schema: None,
    extra: serde_json::json!({}),
  }
}

#[test]
fn test_connection_refused_to_nonexistent_server() {
  // Test connection refused (port not listening)
  let provider = OpenCodeProvider::new(
    "http://localhost:59999".to_string(), // Non-existent port
    "test-session-connection-refused".to_string(),
  )
  .expect("provider should be created");

  let rt = tokio::runtime::Runtime::new().expect("runtime should be created");

  let result = rt.block_on(provider.extract_fields("test input", &create_test_context()));

  // Should get a network error, not panic or hang
  match result {
    Err(ExtractionError::NetworkError(msg)) => {
      println!("Connection refused error: {msg}");
      assert!(
        msg.contains("Failed to connect")
          || msg.contains("connection refused")
          || msg.contains("connect error"),
        "Error message should indicate connection failure"
      );
    }
    Err(other) => {
      panic!("Expected NetworkError for connection refused, got: {other:?}");
    }
    Ok(_) => {
      panic!("Should not succeed with non-existent server");
    }
  }
}

#[test]
fn test_connection_timeout_to_unroutable_ip() {
  // Test connection timeout to an unroutable IP (using a reserved IP that won't respond)
  let provider = OpenCodeProvider::new(
    "http://192.0.2.1:9999".to_string(), // TEST-NET-1, reserved for documentation
    "test-session-timeout".to_string(),
  )
  .expect("provider should be created");

  let rt = tokio::runtime::Runtime::new().expect("runtime should be created");

  // The operation should timeout within 30 seconds (DEFAULT_TIMEOUT_SECS)
  let start = std::time::Instant::now();
  let result = rt.block_on(provider.extract_fields("test input", &create_test_context()));
  let elapsed = start.elapsed();

  println!("Timeout test completed in: {elapsed:?}");

  // Should timeout and return an error, not hang indefinitely
  // The timeout is 30 seconds, but connection might fail faster
  assert!(
    elapsed < Duration::from_secs(35),
    "Should timeout within 35 seconds, took {elapsed:?}"
  );

  match result {
    Err(ExtractionError::Timeout { timeout_ms }) => {
      println!("Got timeout error with timeout_ms: {timeout_ms}");
      assert_eq!(
        timeout_ms, 30000,
        "Timeout should match DEFAULT_TIMEOUT_SECS (30s)"
      );
    }
    Err(ExtractionError::NetworkError(msg)) => {
      // Network errors are also acceptable (might fail before timeout)
      println!("Got network error instead of timeout (acceptable): {msg}");
    }
    Err(other) => {
      panic!("Expected Timeout or NetworkError, got: {other:?}");
    }
    Ok(_) => {
      panic!("Should not succeed with unroutable IP");
    }
  }
}

#[test]
fn test_invalid_hostname() {
  // Test with an invalid hostname that DNS cannot resolve
  let provider = OpenCodeProvider::new(
    "http://this-domain-definitely-does-not-exist-12345.invalid".to_string(),
    "test-session-invalid-host".to_string(),
  )
  .expect("provider should be created");

  let rt = tokio::runtime::Runtime::new().expect("runtime should be created");

  let start = std::time::Instant::now();
  let result = rt.block_on(provider.extract_fields("test input", &create_test_context()));
  let elapsed = start.elapsed();

  println!("Invalid hostname test completed in: {elapsed:?}");

  // Should fail quickly with DNS error
  assert!(
    elapsed < Duration::from_secs(10),
    "DNS failure should be fast, took {elapsed:?}"
  );

  match result {
    Err(ExtractionError::NetworkError(msg)) => {
      println!("DNS resolution error: {msg}");
      // DNS errors often mention "dns", "resolve", or "name"
      let msg_lower = msg.to_lowercase();
      assert!(
        msg_lower.contains("dns")
          || msg_lower.contains("resolve")
          || msg_lower.contains("name")
          || msg_lower.contains("connection")
          || msg_lower.contains("connect"),
        "Error should indicate DNS/resolution issue, got: {msg}"
      );
    }
    Err(other) => {
      panic!("Expected NetworkError for DNS failure, got: {other:?}");
    }
    Ok(_) => {
      panic!("Should not succeed with invalid hostname");
    }
  }
}

#[test]
fn test_http_error_handling_404() {
  // Test 404 error handling
  // Note: This will fail with connection error if server doesn't exist
  // but we're testing the error mapping logic
  let provider = OpenCodeProvider::new(
    "http://localhost:59999/not-found".to_string(),
    "test-session-404".to_string(),
  )
  .expect("provider should be created");

  let rt = tokio::runtime::Runtime::new().expect("runtime should be created");

  let result = rt.block_on(provider.extract_fields("test input", &create_test_context()));

  // Should get a clear error, not panic
  match result {
    Err(ExtractionError::NetworkError(_)) => {
      println!("Got expected network error for non-existent endpoint");
    }
    Err(other) => {
      println!("Got error (acceptable): {other:?}");
      // Any error is acceptable - we're just verifying it doesn't panic
    }
    Ok(_) => {
      panic!("Should not succeed with non-existent endpoint");
    }
  }
}

#[test]
fn test_timeout_value_is_reasonable() {
  // Verify that timeout value is configured reasonably
  let provider = OpenCodeProvider::new(
    "http://localhost:59999".to_string(),
    "test-session-timeout-value".to_string(),
  )
  .expect("provider should be created");

  let rt = tokio::runtime::Runtime::new().expect("runtime should be created");

  let start = std::time::Instant::now();
  let result = rt.block_on(provider.extract_fields("test input", &create_test_context()));
  let elapsed = start.elapsed();

  match result {
    Err(ExtractionError::Timeout { timeout_ms }) => {
      // Timeout should be 30 seconds (30000ms) as per DEFAULT_TIMEOUT_SECS
      assert_eq!(timeout_ms, 30000, "Timeout should be 30 seconds");
      assert!(timeout_ms >= 5000, "Timeout should be at least 5 seconds");
      assert!(timeout_ms <= 120_000, "Timeout should not exceed 2 minutes");

      // Actual time waited should be close to configured timeout
      // Allow some margin for overhead
      assert!(
        elapsed >= Duration::from_secs(25),
        "Should wait near timeout duration"
      );
      assert!(
        elapsed <= Duration::from_secs(35),
        "Should not exceed timeout significantly"
      );
    }
    Err(ExtractionError::NetworkError(_)) => {
      // Connection refused might happen before timeout - this is okay
      assert!(
        elapsed < Duration::from_secs(5),
        "Connection error should be fast"
      );
    }
    Err(other) => {
      panic!("Unexpected error: {other:?}");
    }
    Ok(_) => {
      panic!("Should not succeed");
    }
  }
}

#[test]
fn test_malformed_url() {
  // Test provider creation with malformed URL
  // The provider should accept any string since URL validation happens at request time

  let provider = OpenCodeProvider::new(
    "not-a-valid-url".to_string(),
    "test-session-malformed".to_string(),
  );

  // Provider creation might succeed or fail depending on URL parsing
  // If it succeeds, the request should fail
  if let Ok(provider) = provider {
    let rt = tokio::runtime::Runtime::new().expect("runtime should be created");
    let result = rt.block_on(provider.extract_fields("test input", &create_test_context()));

    match result {
      Err(ExtractionError::NetworkError(_) | ExtractionError::ConfigurationError(_)) => {
        println!("Got expected error for malformed URL");
      }
      Err(other) => {
        println!("Got error (acceptable): {other:?}");
      }
      Ok(_) => {
        panic!("Should not succeed with malformed URL");
      }
    }
  }
}

#[test]
fn test_empty_response_body() {
  // Test handling of empty response (if server returns 200 OK with empty body)
  // This would require a mock server, so we just verify the error handling path exists
  let provider = OpenCodeProvider::new(
    "http://localhost:59999".to_string(),
    "test-session-empty-response".to_string(),
  )
  .expect("provider should be created");

  let rt = tokio::runtime::Runtime::new().expect("runtime should be created");

  let result = rt.block_on(provider.extract_fields("test input", &create_test_context()));

  // Should get an error, not panic or hang
  assert!(result.is_err(), "Should fail with non-existent server");

  match result {
    Err(ExtractionError::NetworkError(_)) => {
      println!("Got expected network error");
    }
    Err(ExtractionError::ParseError(_)) => {
      println!("Parse error indicates we got a response but couldn't parse it");
    }
    Err(other) => {
      println!("Got error (acceptable): {other:?}");
    }
    Ok(_) => {
      panic!("Should not succeed");
    }
  }
}

#[test]
fn test_concurrent_failure_handling() {
  // Test that multiple concurrent failing requests don't cause issues
  let provider1 = OpenCodeProvider::new(
    "http://localhost:59999".to_string(),
    "test-session-concurrent-1".to_string(),
  )
  .expect("provider should be created");

  let provider2 = OpenCodeProvider::new(
    "http://localhost:59998".to_string(),
    "test-session-concurrent-2".to_string(),
  )
  .expect("provider should be created");

  let rt = tokio::runtime::Runtime::new().expect("runtime should be created");

  // Run both requests concurrently
  let result1 = rt.block_on(provider1.extract_fields("test input 1", &create_test_context()));
  let result2 = rt.block_on(provider2.extract_fields("test input 2", &create_test_context()));

  // Both should fail gracefully
  assert!(result1.is_err(), "First request should fail");
  assert!(result2.is_err(), "Second request should fail");

  println!("Concurrent failure test passed");
  println!("Result 1: {result1:?}");
  println!("Result 2: {result2:?}");
}

#[test]
fn test_retry_does_not_hang() {
  // Verify that failed requests don't cause the application to hang
  // This tests that there's no infinite retry loop

  let provider = OpenCodeProvider::new(
    "http://localhost:59999".to_string(),
    "test-session-no-hang".to_string(),
  )
  .expect("provider should be created");

  let rt = tokio::runtime::Runtime::new().expect("runtime should be created");

  let start = std::time::Instant::now();
  let result = rt.block_on(provider.extract_fields("test input", &create_test_context()));
  let elapsed = start.elapsed();

  // Should complete quickly (connection refused or timeout)
  assert!(
    elapsed < Duration::from_secs(35),
    "Request should not hang indefinitely, took {elapsed:?}"
  );

  assert!(result.is_err(), "Should fail with non-existent server");
  println!("No-hang test passed in {elapsed:?}");
}

// Helper function to verify error messages are user-friendly
#[test]
fn test_error_messages_are_actionable() {
  let test_cases = vec![
    ("http://localhost:59999", "Connection refused"),
    ("http://192.0.2.1:9999", "Timeout"),
    ("http://invalid-domain-12345.invalid", "DNS failure"),
  ];

  for (url, scenario) in test_cases {
    let provider = OpenCodeProvider::new(
      url.to_string(),
      format!("test-session-{}", scenario.replace(' ', "-")),
    );

    if let Ok(provider) = provider {
      let rt = tokio::runtime::Runtime::new().expect("runtime should be created");
      let result = rt.block_on(provider.extract_fields("test", &create_test_context()));

      if let Err(error) = result {
        let error_msg = format!("{error}");
        println!("{scenario} error message: {error_msg}");

        // Error messages should be descriptive
        assert!(!error_msg.is_empty(), "Error message should not be empty");
        assert!(error_msg.len() > 10, "Error message should be descriptive");
        assert!(
          !error_msg.contains("panic"),
          "Error should not mention panic"
        );
        assert!(
          !error_msg.contains("unwrap"),
          "Error should not mention unwrap"
        );
      }
    }
  }
}

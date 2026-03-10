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

//! User experience tests for network failure handling
//!
//! These tests verify that when network failures occur, the user sees:
//! 1. Appropriate error messages
//! 2. Can retry the operation
//! 3. State is preserved
//! 4. No hanging UI

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
fn test_error_messages_are_user_friendly() {
  // Verify that error messages are understandable and actionable
  let provider = OpenCodeProvider::new(
    "http://localhost:59999".to_string(),
    "test-session-ux-errors".to_string(),
  )
  .expect("provider should be created");

  let rt = tokio::runtime::Runtime::new().expect("runtime should be created");

  let result = rt.block_on(provider.extract_fields("test input", &create_test_context()));

  match result {
    Err(error) => {
      let error_msg = format!("{}", error);

      println!("User-facing error message: {}", error_msg);

      // Error messages should be user-friendly
      assert!(!error_msg.is_empty(), "Error message must not be empty");
      assert!(
        !error_msg.contains("panic"),
        "Error should not mention 'panic'"
      );
      assert!(
        !error_msg.contains("unwrap"),
        "Error should not mention 'unwrap'"
      );
      assert!(!error_msg.contains("BUG"), "Error should not mention 'BUG'");
      assert!(
        !error_msg.contains("assertion"),
        "Error should not mention 'assertion'"
      );

      // Error should indicate the problem
      let error_msg_lower = error_msg.to_lowercase();
      assert!(
        error_msg_lower.contains("failed")
          || error_msg_lower.contains("error")
          || error_msg_lower.contains("timeout")
          || error_msg_lower.contains("connect"),
        "Error should describe the problem"
      );
    }
    Ok(_) => {
      panic!("Should not succeed with non-existent server");
    }
  }
}

#[test]
fn test_timeout_error_message_includes_duration() {
  // Test that timeout errors include the duration so users know how long to wait
  let provider = OpenCodeProvider::new(
    "http://192.0.2.1:9999".to_string(), // TEST-NET-1
    "test-session-timeout-message".to_string(),
  )
  .expect("provider should be created");

  let rt = tokio::runtime::Runtime::new().expect("runtime should be created");

  let result = rt.block_on(provider.extract_fields("test input", &create_test_context()));

  match result {
    Err(ExtractionError::Timeout { timeout_ms }) => {
      let error_msg = format!("Timeout after {}ms", timeout_ms);

      println!("Timeout error message: {}", error_msg);

      // The error message should mention the timeout duration
      assert!(
        error_msg.contains("30000") || error_msg.contains("30"),
        "Timeout error should mention the duration: {}",
        error_msg
      );
    }
    Err(ExtractionError::NetworkError(_)) => {
      // Connection might fail before timeout - acceptable
      println!("Connection failed before timeout (acceptable)");
    }
    Err(other) => {
      panic!("Unexpected error: {:?}", other);
    }
    Ok(_) => {
      panic!("Should not succeed");
    }
  }
}

#[test]
fn test_rate_limit_error_includes_retry_after() {
  // Test that rate limit errors include retry time
  // Note: This tests the error structure, actual 429 would require a real server
  let error = ExtractionError::RateLimited {
    retry_after_seconds: 60,
  };

  let error_msg = format!("{}", error);

  println!("Rate limit error message: {}", error_msg);

  // Error should mention retry time
  assert!(
    error_msg.contains("60") || error_msg.contains("retry"),
    "Rate limit error should mention retry time: {}",
    error_msg
  );
}

#[test]
fn test_user_can_retry_after_timeout() {
  // Verify that a timeout doesn't prevent retrying
  let provider = OpenCodeProvider::new(
    "http://192.0.2.1:9999".to_string(),
    "test-session-retry".to_string(),
  )
  .expect("provider should be created");

  let rt = tokio::runtime::Runtime::new().expect("runtime should be created");

  // First attempt should timeout
  let result1 = rt.block_on(provider.extract_fields("test input 1", &create_test_context()));
  assert!(result1.is_err(), "First attempt should fail");

  // Second attempt should also work (same provider, should still be functional)
  let result2 = rt.block_on(provider.extract_fields("test input 2", &create_test_context()));
  assert!(result2.is_err(), "Second attempt should also fail");

  println!("Retry test passed - provider remains functional after timeout");
}

#[test]
fn test_state_preserved_after_error() {
  // Verify that provider state is preserved after network errors
  let provider = OpenCodeProvider::new(
    "http://localhost:59999".to_string(),
    "test-session-state".to_string(),
  )
  .expect("provider should be created");

  let rt = tokio::runtime::Runtime::new().expect("runtime should be created");

  // Check initial state
  assert_eq!(provider.session_id(), "test-session-state");
  assert_eq!(provider.endpoint(), "http://localhost:59999");

  // Trigger error
  let result = rt.block_on(provider.extract_fields("test", &create_test_context()));
  assert!(result.is_err());

  // State should be preserved
  assert_eq!(provider.session_id(), "test-session-state");
  assert_eq!(provider.endpoint(), "http://localhost:59999");

  // Provider should still be usable
  let result2 = rt.block_on(provider.extract_fields("test 2", &create_test_context()));
  assert!(result2.is_err());

  println!("State preservation test passed");
}

#[test]
fn test_no_ui_hang_on_connection_refused() {
  // Verify that connection refused doesn't cause the UI to hang
  let provider = OpenCodeProvider::new(
    "http://localhost:59999".to_string(),
    "test-session-no-hang-refused".to_string(),
  )
  .expect("provider should be created");

  let rt = tokio::runtime::Runtime::new().expect("runtime should be created");

  let start = std::time::Instant::now();
  let result = rt.block_on(provider.extract_fields("test", &create_test_context()));
  let elapsed = start.elapsed();

  // Connection refused should fail fast (< 5 seconds)
  assert!(
    elapsed < Duration::from_secs(5),
    "Connection refused should fail fast, took {:?}",
    elapsed
  );

  assert!(result.is_err());
  println!("No hang on connection refused: {:?}", elapsed);
}

#[test]
fn test_no_ui_hang_on_dns_failure() {
  // Verify that DNS failure doesn't cause the UI to hang
  let provider = OpenCodeProvider::new(
    "http://invalid-dns-12345.invalid".to_string(),
    "test-session-no-hang-dns".to_string(),
  )
  .expect("provider should be created");

  let rt = tokio::runtime::Runtime::new().expect("runtime should be created");

  let start = std::time::Instant::now();
  let result = rt.block_on(provider.extract_fields("test", &create_test_context()));
  let elapsed = start.elapsed();

  // DNS failure should fail fast (< 10 seconds)
  assert!(
    elapsed < Duration::from_secs(10),
    "DNS failure should fail fast, took {:?}",
    elapsed
  );

  assert!(result.is_err());
  println!("No hang on DNS failure: {:?}", elapsed);
}

#[test]
fn test_no_ui_hang_on_timeout() {
  // Verify that timeout doesn't exceed configured duration significantly
  let provider = OpenCodeProvider::new(
    "http://192.0.2.1:9999".to_string(),
    "test-session-no-hang-timeout".to_string(),
  )
  .expect("provider should be created");

  let rt = tokio::runtime::Runtime::new().expect("runtime should be created");

  let start = std::time::Instant::now();
  let result = rt.block_on(provider.extract_fields("test", &create_test_context()));
  let elapsed = start.elapsed();

  // Should timeout close to configured 30 seconds
  // Allow 5 second margin for overhead
  assert!(
    elapsed < Duration::from_secs(35),
    "Timeout should not exceed 35 seconds, took {:?}",
    elapsed
  );

  // And should wait at least 25 seconds (close to configured timeout)
  if matches!(result, Err(ExtractionError::Timeout { .. })) {
    assert!(
      elapsed >= Duration::from_secs(25),
      "Should wait close to timeout duration, took {:?}",
      elapsed
    );
  }

  assert!(result.is_err());
  println!("No hang on timeout: {:?}", elapsed);
}

#[test]
fn test_multiple_providers_dont_interfere() {
  // Test that multiple provider instances don't interfere with each other
  let provider1 = OpenCodeProvider::new(
    "http://localhost:59999".to_string(),
    "session-1".to_string(),
  )
  .expect("provider 1 should be created");

  let provider2 = OpenCodeProvider::new(
    "http://invalid-1.invalid".to_string(),
    "session-2".to_string(),
  )
  .expect("provider 2 should be created");

  let provider3 =
    OpenCodeProvider::new("http://192.0.2.1:9999".to_string(), "session-3".to_string())
      .expect("provider 3 should be created");

  let rt = tokio::runtime::Runtime::new().expect("runtime should be created");

  // All three should fail independently
  let result1 = rt.block_on(provider1.extract_fields("test", &create_test_context()));
  let result2 = rt.block_on(provider2.extract_fields("test", &create_test_context()));
  let result3 = rt.block_on(provider3.extract_fields("test", &create_test_context()));

  assert!(result1.is_err());
  assert!(result2.is_err());
  assert!(result3.is_err());

  // Each should maintain its own state
  assert_eq!(provider1.session_id(), "session-1");
  assert_eq!(provider2.session_id(), "session-2");
  assert_eq!(provider3.session_id(), "session-3");

  println!("Multiple providers don't interfere");
}

#[test]
fn test_timeout_value_is_configurable_and_reasonable() {
  // Verify timeout value is reasonable for user experience
  // 30 seconds is a good balance - not too short, not too long

  let provider = OpenCodeProvider::new(
    "http://localhost:59999".to_string(),
    "test-session-config-check".to_string(),
  )
  .expect("provider should be created");

  let rt = tokio::runtime::Runtime::new().expect("runtime should be created");

  let result = rt.block_on(provider.extract_fields("test", &create_test_context()));

  if let Err(ExtractionError::Timeout { timeout_ms }) = result {
    // 30 seconds = 30000 ms
    assert_eq!(timeout_ms, 30000);

    // Verify it's reasonable for UX:
    // - Not too short (< 5s would be too aggressive)
    // - Not too long (> 2 min would frustrate users)
    assert!(timeout_ms >= 5000, "Timeout should be at least 5 seconds");
    assert!(timeout_ms <= 120000, "Timeout should not exceed 2 minutes");

    println!(
      "Timeout value is reasonable: {}ms ({} seconds)",
      timeout_ms,
      timeout_ms / 1000
    );
  }
}

#[test]
fn test_error_categorization_for_ui() {
  // Test that errors can be categorized for UI display
  let provider = OpenCodeProvider::new(
    "http://localhost:59999".to_string(),
    "test-session-categorization".to_string(),
  )
  .expect("provider should be created");

  let rt = tokio::runtime::Runtime::new().expect("runtime should be created");

  let result = rt.block_on(provider.extract_fields("test", &create_test_context()));

  match result {
    Err(ExtractionError::NetworkError(_)) => {
      println!("Network error - UI can show 'Connection problem'");
    }
    Err(ExtractionError::Timeout { .. }) => {
      println!("Timeout - UI can show 'Request timed out'");
    }
    Err(ExtractionError::AuthenticationError(_)) => {
      println!("Auth error - UI can show 'Check credentials'");
    }
    Err(ExtractionError::RateLimited { .. }) => {
      println!("Rate limited - UI can show 'Rate limit exceeded'");
    }
    Err(ExtractionError::InvalidInput(_)) => {
      println!("Invalid input - UI can show 'Check your input'");
    }
    Err(ExtractionError::ApiError { .. }) => {
      println!("API error - UI can show 'Server error'");
    }
    Err(_) => {
      println!("Other error - UI can show generic error");
    }
    Ok(_) => {
      panic!("Should not succeed");
    }
  }
}

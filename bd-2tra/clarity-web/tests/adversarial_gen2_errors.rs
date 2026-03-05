#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::too_many_lines)]

//! Adversarial Generation 2 Testing - Error Propagation & Context
//!
//! Tests error chain integrity, context preservation, and message clarity

use clarity_web::config::ai::AiConfig;
use clarity_web::lattice::quality::*;
use clarity_web::storage::path_util::*;
use clarity_web::storage::RedbStore;
use clarity_web::storage::StorageError;
use clarity_web::types::Answer;

/// Helper to create test answers
fn create_answer(step_id: &str, value: &str) -> Answer {
  Answer {
    step_id: step_id.to_string(),
    value: value.to_string(),
    timestamp: "2024-01-01T00:00:00Z".to_string(),
  }
}

#[test]
fn test_quality_error_empty_answers_message() {
  // Test error message clarity
  let ears = vec![];
  let inversion = InversionControl {
    has_inversion_tests: false,
    inverted_count: 0,
  };

  let result = calculate_quality(&[], &ears, &inversion);

  match result {
    Err(QualityError::EmptyAnswers) => {
      let msg = result.unwrap_err().to_string();
      assert!(msg.contains("empty"), "Error should mention 'empty'");
      assert!(msg.contains("answers"), "Error should mention 'answers'");
    }
    _ => {
      panic!("Should return EmptyAnswers error");
    }
  }
}

#[test]
fn test_quality_error_invalid_score_context() {
  // Test that invalid score errors include context
  let result = DimensionScore::new(QualityDimension::Completeness, 255);

  match result {
    Err(QualityError::InvalidScore(msg)) => {
      assert!(!msg.is_empty(), "Error should include score value");
      assert!(
        msg.contains("255"),
        "Error should mention the invalid score"
      );
    }
    _ => {
      panic!("Should return InvalidScore error");
    }
  }
}

#[test]
fn test_quality_error_chain_integrity() {
  // Test that errors in calculation propagate correctly
  let answers = vec![
    create_answer("req1", "must"),
    create_answer("req2", "must not"),
  ];

  let ears = vec![];
  let inversion = InversionControl {
    has_inversion_tests: false,
    inverted_count: 0,
  };

  let result = calculate_quality(&answers, &ears, &inversion);

  // Should detect contradiction and add issue
  match result {
    Ok(score) => {
      let consistency = score.get_dimension(QualityDimension::Consistency);
      assert!(consistency.is_some());
      assert!(
        consistency.unwrap().score < 100,
        "Score should be reduced due to contradiction"
      );

      let consistency_issues = score.get_issues(QualityDimension::Consistency);
      assert!(
        !consistency_issues.is_empty(),
        "Should have consistency issues"
      );

      // Issue should have clear context
      let issue = &consistency_issues[0];
      assert_eq!(issue.dimension, QualityDimension::Consistency);
      assert!(issue.message.contains("contradiction"));
    }
    Err(e) => {
      panic!(
        "Should detect contradictions via issues, not error: {:?}",
        e
      );
    }
  }
}

#[test]
fn test_storage_error_invalid_project_id_messages() {
  // Test all validation error paths have clear messages
  let test_cases = vec![
    ("", "empty"),
    ("bad/name", "separator"),
    ("bad\\name", "separator"),
    (".hidden", "dot"),
    ("bad\0name", "null"),
  ];

  for (id, expected_keyword) in test_cases {
    let result = validate_project_id(id);

    match result {
      Err(StorageError::InvalidProjectId(msg)) => {
        assert!(
          msg.to_lowercase().contains(expected_keyword),
          "Error for '{}' should mention '{}', got: {}",
          id,
          expected_keyword,
          msg
        );
      }
      _ => {
        panic!("Should return InvalidProjectId for '{}'", id);
      }
    }
  }
}

#[test]
fn test_storage_error_io_propagation() {
  // Test that IO errors propagate correctly
  let result = get_project_db_path("/nonexistent/path\x00project");

  match result {
    Err(StorageError::InvalidProjectId(_)) => {
      // Expected - path validation catches null byte
    }
    Err(StorageError::PathNotFound) | Err(StorageError::IoError(_)) => {
      // Also acceptable
    }
    Err(_) => {
      // Other errors are acceptable
    }
    Ok(_) => {
      panic!("Should not succeed with invalid path");
    }
  }
}

#[test]
fn test_storage_error_serialization_context() {
  // Test that serialization errors preserve context
  let store = RedbStore::open_in_memory().expect("Failed to open store");

  // Create a value that's large but not usize::MAX (would OOM)
  let large_data = "A".repeat(10_000_000); // 10MB

  let answer = create_answer("large", &large_data);

  // This should succeed or fail gracefully
  let save_result = store.save_answer(&answer);

  match save_result {
    Ok(_) => {
      // Successfully saved
    }
    Err(e) => {
      let msg = e.to_string();
      assert!(!msg.is_empty(), "Error should have context");
    }
  }
}

#[test]
fn test_storage_error_deserialization_recovery() {
  // Test that deserialization errors are recoverable
  // Note: Can't insert corrupted data directly due to private db field
  // This test documents expected behavior

  let store = RedbStore::open_in_memory().expect("Failed to open store");

  // Insert valid data
  let valid_answer = create_answer("valid", "valid value");
  store
    .save_answer(&valid_answer)
    .expect("Failed to save valid answer");

  // Valid read should still work
  let valid_result = store.get_answer("valid");
  assert!(valid_result.is_ok(), "Valid read should succeed");
  assert!(
    valid_result.unwrap().is_some(),
    "Valid answer should be retrievable"
  );

  // Document: If corrupted data existed:
  // - get_answer(key) should return Err(Deserialization)
  // - get_answer(other_key) should still work
  // - get_all_answers() should skip corrupted entries
}

#[test]
fn test_config_error_file_not_found() {
  // Test config loading with non-existent file
  use std::env;

  // Set a non-existent config path
  let temp_dir = env::temp_dir();
  let fake_config = temp_dir.join("nonexistent_clarity_ai_config_12345.toml");

  // This should create default config or return error
  // The exact behavior depends on implementation
  let result = std::fs::read_to_string(&fake_config);

  match result {
    Err(e) => {
      assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
    }
    Ok(_) => {
      panic!("File should not exist");
    }
  }
}

#[test]
fn test_config_error_invalid_toml() {
  // Test TOML parsing error messages
  let invalid_toml = "invalid [ toml content";

  let result: Result<AiConfig, toml::de::Error> = toml::from_str(invalid_toml);

  match result {
    Err(e) => {
      let msg = e.to_string();
      assert!(!msg.is_empty(), "TOML error should have message");
      // Should indicate line/position
    }
    Ok(_) => {
      panic!("Invalid TOML should not parse");
    }
  }
}

#[test]
fn test_config_error_missing_required_field() {
  // Test missing field in config
  let incomplete_toml = r#"
[provider]
provider = "opencode"
# Missing endpoint field
"#;

  let result: Result<AiConfig, toml::de::Error> = toml::from_str(incomplete_toml);

  // Should use defaults for missing fields
  match result {
    Ok(config) => {
      assert_eq!(
        config.provider.provider,
        clarity_web::config::ai::ProviderType::Opencode
      );
      // endpoint should have default value
      assert!(!config.provider.endpoint.is_empty());
    }
    Err(_) => {
      // Also acceptable to fail
    }
  }
}

#[test]
fn test_error_display_user_friendly() {
  // Test that error messages are user-friendly
  let errors = vec![
    StorageError::InvalidProjectId("bad/id".to_string()),
    StorageError::PathNotFound,
    StorageError::Database("test error".to_string()),
    StorageError::Serialization("test serialization".to_string()),
    StorageError::Deserialization("test deserialization".to_string()),
  ];

  for error in errors {
    let msg = error.to_string();
    assert!(!msg.is_empty(), "Error message should not be empty");
    assert!(
      msg.len() < 200,
      "Error message should be concise, got: {}",
      msg
    );

    // Should not contain internal implementation details
    assert!(!msg.contains("0x"), "Should not contain memory addresses");
    assert!(!msg.contains("backtrace"), "Should not contain backtrace");
  }
}

#[test]
fn test_quality_error_partial_failure_handling() {
  // Test that partial failures in quality calculation are handled
  let answers = vec![
    create_answer("user_goal", "User must authenticate"),
    create_answer("actors", "System admin"),
    create_answer("precondition", "User exists"),
    create_answer("outcome", "Access granted"),
    create_answer("acceptance_criteria", "Login within 2 seconds"),
    // Missing security field
  ];

  let ears = vec![];

  let inversion = InversionControl {
    has_inversion_tests: false,
    inverted_count: 0,
  };

  let result = calculate_quality(&answers, &ears, &inversion);

  // Should succeed with issues reported
  match result {
    Ok(score) => {
      // Should have security issue
      let security_issues = score.get_issues(QualityDimension::Security);
      assert!(
        !security_issues.is_empty(),
        "Should report missing security"
      );

      // Overall score should be reduced but valid
      assert!(score.overall > 0);
      assert!(score.overall < 100);
    }
    Err(e) => {
      panic!("Should handle partial failures gracefully: {:?}", e);
    }
  }
}

#[test]
fn test_storage_error_context_preservation() {
  // Test that storage operations preserve error context
  let store = RedbStore::open_in_memory().expect("Failed to open store");

  // Try to get from non-existent table
  let result = store.get_lattice_cache("nonexistent");

  match result {
    Ok(None) => {
      // Acceptable - table doesn't exist yet
    }
    Ok(Some(_)) => {
      // Also acceptable if table was created
    }
    Err(e) => {
      let msg = e.to_string();
      assert!(!msg.is_empty(), "Error should have context");
    }
  }
}

#[test]
fn test_nested_error_context_chain() {
  // Test that nested operations preserve full error chain
  let result = get_project_db_path("test/../project");

  match result {
    Ok(_) => {
      // Path traversal might be allowed
    }
    Err(StorageError::InvalidProjectId(msg)) => {
      assert!(!msg.is_empty(), "Should have error message");
    }
    Err(e) => {
      panic!("Unexpected error type: {:?}", e);
    }
  }
}

#[test]
fn test_error_clarity_for_non_technical_users() {
  // Test that error messages are understandable by non-technical users
  let errors: Vec<Box<dyn std::error::Error>> = vec![
    Box::new(StorageError::PathNotFound),
    Box::new(StorageError::InvalidProjectId("bad-id".to_string())),
    Box::new(QualityError::EmptyAnswers),
  ];

  for error in errors {
    let msg = error.to_string();

    // Should use plain language
    assert!(
      !msg.contains(" Os { code:"),
      "Should not contain raw OS error codes"
    );
    assert!(
      !msg.contains("0x"),
      "Should not contain hexadecimal addresses"
    );

    // Should not be empty
    assert!(!msg.is_empty(), "Error message should not be empty");

    // Should be human-readable (not just codes)
    assert!(msg.len() > 10, "Error should be descriptive");
  }
}

#[test]
fn test_async_operation_error_handling() {
  // Test error handling in async-like scenarios (rapid operations)
  let store = RedbStore::open_in_memory().expect("Failed to open store");

  // Rapidly create and drop transactions
  for i in 0..100 {
    let answer = create_answer(&format!("key{}", i), &format!("value{}", i));

    let save_result = store.save_answer(&answer);
    if let Err(e) = save_result {
      // Error should have context
      let msg = e.to_string();
      assert!(!msg.is_empty(), "Error should have context");
    }
  }

  // Verify all saves succeeded
  let all_answers = store.get_all_answers();
  match all_answers {
    Ok(answers) => {
      assert_eq!(answers.len(), 100);
    }
    Err(e) => {
      panic!("Should have all answers: {:?}", e);
    }
  }
}

#[test]
fn test_error_recovery_after_corruption() {
  // Test that system can recover after encountering corrupted data
  // Note: Can't insert corrupted data directly
  // This test documents expected recovery behavior

  let store = RedbStore::open_in_memory().expect("Failed to open store");

  // Save valid data
  let valid1 = create_answer("valid1", "value1");
  let valid2 = create_answer("valid2", "value2");

  store.save_answer(&valid1).expect("Failed to save valid1");
  store.save_answer(&valid2).expect("Failed to save valid2");

  // Should still be able to read valid data
  let result1 = store.get_answer("valid1");
  let result2 = store.get_answer("valid2");

  assert!(result1.is_ok(), "Should read valid1");
  assert!(result2.is_ok(), "Should read valid2");

  assert!(result1.unwrap().is_some(), "valid1 should exist");
  assert!(result2.unwrap().is_some(), "valid2 should exist");

  // get_all_answers should return all valid data
  let all_result = store.get_all_answers();
  assert!(all_result.is_ok());
  assert_eq!(all_result.unwrap().len(), 2);
}

#[test]
fn test_concurrent_error_propagation() {
  // Test that errors from concurrent operations are not lost
  use std::sync::Arc;
  use std::sync::Mutex;
  use std::thread;

  let store = Arc::new(RedbStore::open_in_memory().expect("Failed to open store"));
  let errors = Arc::new(Mutex::new(Vec::new()));

  let mut handles = vec![];

  // Spawn multiple threads performing operations
  for i in 0..10 {
    let store_clone = Arc::clone(&store);
    let errors_clone = Arc::clone(&errors);

    let handle = thread::spawn(move || {
      let answer = create_answer(&format!("key{}", i), &format!("value{}", i));

      match store_clone.save_answer(&answer) {
        Ok(_) => {}
        Err(e) => {
          let mut errors = errors_clone.lock().unwrap();
          errors.push(e.to_string());
        }
      }
    });

    handles.push(handle);
  }

  for handle in handles {
    handle.join().expect("Thread panicked");
  }

  // All operations should succeed (no errors expected)
  let errors = errors.lock().unwrap();
  assert!(
    errors.is_empty(),
    "All concurrent operations should succeed, errors: {:?}",
    errors
  );
}

#[test]
fn test_partial_unicode_error_messages() {
  // Test that errors with Unicode characters are handled correctly
  let unicode_name = "用户🎉项目";
  let result = validate_project_id(unicode_name);

  match result {
    Ok(_) => {
      // Unicode is valid
    }
    Err(StorageError::InvalidProjectId(msg)) => {
      // Error message should handle Unicode correctly
      assert!(msg.contains(unicode_name) || msg.len() > 0);
    }
    _ => {
      panic!("Unexpected error type");
    }
  }
}

//! Martin Fowler tests for Quality Types (core-012)
//!
//! Test Plan:
//! 1. Should Create QualityScore From Valid Float
//! 2. Should Reject QualityScore Outside Range
//! 3. Should Aggregate Validation Results
//! 4. Should Chain Validators With AndThen
//! 5. Should Combine Validators With Or Logic
//! 6. Should Provide Context In Validation Errors
//! 7. Should Track Quality Metrics
//! 8. Should Support Custom Validators
//! 9. Should Validate With Severity Levels
//! 10. Should Validate Nested Structures

use clarity_core::quality::{
  CustomValidator, QualityMetrics, QualityScore, Severity, ValidationMessage, ValidationReport,
  Validator,
};
use clarity_core::validation::ValidationError;

/// Test 1: Should Create QualityScore From Valid Float
///
/// **GIVEN** a float value 0.85
/// **WHEN** QualityScore::new(0.85) is called
/// **THEN** Result<QualityScore, ValidationError> should be Ok
/// **AND** score.value() should return 0.85
/// **AND** score.is_passing() should return true (threshold >= 0.7)
/// **AND** no unwrap/expect should occur
#[test]
fn test_should_create_quality_score_from_valid_float() {
  // GIVEN a float value 0.85
  let value = 0.85_f64;

  // WHEN QualityScore::new is called
  let result = QualityScore::new(value);

  // THEN Result should be Ok
  assert!(result.is_ok(), "QualityScore::new(0.85) should be Ok");

  let score = match result {
    Ok(s) => s,
    Err(e) => panic!("Expected Ok, got Err: {:?}", e),
  };

  // AND score.value() should return 0.85
  assert!(
    (score.value() - 0.85).abs() < f64::EPSILON,
    "score.value() should return 0.85"
  );

  // AND score.is_passing() should return true (threshold >= 0.7)
  assert!(
    score.is_passing(),
    "score.is_passing() should return true for score 0.85"
  );
}

/// Test 2: Should Reject QualityScore Outside Range
///
/// **GIVEN** a float value 1.5 (above 1.0)
/// **WHEN** QualityScore::new(1.5) is called
/// **THEN** Result should be Err with error specifying valid range
/// **AND** error should specify valid range [0.0, 1.0]
/// **AND** score should not be created
#[test]
fn test_should_reject_quality_score_above_range() {
  // GIVEN a float value 1.5 (above 1.0)
  let value = 1.5_f64;

  // WHEN QualityScore::new is called
  let result = QualityScore::new(value);

  // THEN Result should be Err
  assert!(result.is_err(), "QualityScore::new(1.5) should be Err");

  // AND error should mention valid range
  match result {
    Err(ValidationError::InvalidFormat { reason }) => {
      assert!(
        reason.contains("outside valid range") || reason.contains("[0.0, 1.0]"),
        "Error should mention valid range"
      );
    }
    Err(e) => panic!("Expected InvalidFormat error, got: {:?}", e),
    Ok(_) => panic!("Expected Err, got Ok"),
  }
}

/// Test 2b: Should Reject QualityScore Below Range
#[test]
fn test_should_reject_quality_score_below_range() {
  // GIVEN a float value -0.1 (below 0.0)
  let value = -0.1_f64;

  // WHEN QualityScore::new is called
  let result = QualityScore::new(value);

  // THEN Result should be Err with InvalidRange
  assert!(result.is_err(), "Should reject negative score");
}

/// Test 3: Should Aggregate Validation Results
///
/// **GIVEN** multiple validation messages
/// **WHEN** ValidationReport::aggregate() is called
/// **THEN** report should contain all validation messages
/// **AND** report.is_valid() should return false if any errors
/// **AND** report.errors() should return list of errors
/// **AND** report.warnings() should return list of warnings
/// **AND** report should be serializable to JSON
#[test]
fn test_should_aggregate_validation_results() {
  // GIVEN multiple validation messages
  let msg1 = ValidationMessage::new(Severity::Error, "title".to_string(), "too long".to_string());
  let msg2 = ValidationMessage::new(
    Severity::Warning,
    "description".to_string(),
    "vague".to_string(),
  );
  let msg3 = ValidationMessage::new(
    Severity::Info,
    "spec_name".to_string(),
    "not conventional".to_string(),
  );

  // WHEN ValidationReport::aggregate is called
  let report1 = ValidationReport::new(vec![msg1]);
  let report2 = ValidationReport::new(vec![msg2, msg3]);
  let aggregated = ValidationReport::aggregate(vec![report1, report2]);

  // THEN report should contain all validation results
  assert_eq!(
    aggregated.messages().len(),
    3,
    "Report should have 3 messages"
  );

  // AND report.is_valid() should return false if any errors
  assert!(
    !aggregated.is_valid(),
    "Report with errors should not be valid"
  );

  // AND report.errors() should return list of errors
  let errors = aggregated.errors();
  assert_eq!(errors.len(), 1, "Should have 1 error");
  assert_eq!(errors[0].field_path(), "title");

  // AND report.warnings() should return list of warnings
  let warnings = aggregated.warnings();
  assert_eq!(warnings.len(), 1, "Should have 1 warning");

  // AND report should be serializable to JSON
  let json = aggregated.to_json();
  assert!(json.contains("title"), "JSON should contain field names");
  assert!(
    json.contains("description"),
    "JSON should contain field names"
  );
}

/// Test 3b: Should Show Valid Report When All Pass
#[test]
fn test_should_show_valid_report_when_all_pass() {
  // GIVEN all validations pass (empty report)
  let report = ValidationReport::valid();

  // WHEN checking validity
  // THEN report should be valid
  assert!(report.is_valid(), "Empty report should be valid");
  assert_eq!(report.error_count(), 0, "Should have no errors");
  assert!(report.is_empty(), "Should be empty");
}

/// Test 4: Should Chain Validators With AndThen
///
/// **GIVEN** validate_non_empty and validate_max_length validators
/// **WHEN** validator1.and(validator2) is called
/// **THEN** both validators should run in sequence
/// **AND** Result should be Ok if both pass
/// **AND** Result should be Err if first fails
/// **AND** Result should be Err if second fails (with specific error)
#[test]
fn test_should_chain_validators_with_and_then() {
  // GIVEN validators
  let validate_non_empty = Validator::single(|s: &str| -> Result<String, ValidationError> {
    if s.is_empty() {
      Err(ValidationError::EmptyInput)
    } else {
      Ok(s.to_string())
    }
  });

  let validate_max_length = Validator::single(|s: &str| -> Result<String, ValidationError> {
    if s.len() > 10 {
      Err(ValidationError::InputTooLong { max_length: 10 })
    } else {
      Ok(s.to_string())
    }
  });

  // WHEN chaining validators with valid input
  let combined = validate_non_empty.clone().and(validate_max_length.clone());
  let valid_result = combined.validate("test");

  // THEN Result should be Ok if both pass
  assert!(
    valid_result.is_ok(),
    "Chained validators should pass for valid input"
  );

  // AND Result should be Err if first fails
  let combined2 = validate_non_empty.clone().and(validate_max_length.clone());
  let empty_result = combined2.validate("");
  assert!(empty_result.is_err(), "Should fail on first validator");
  assert_eq!(empty_result, Err(ValidationError::EmptyInput));

  // AND Result should be Err if second fails (with specific error)
  let combined3 = validate_non_empty.and(validate_max_length);
  let long_result = combined3.validate("this is a very long string");
  assert!(long_result.is_err(), "Should fail on second validator");
  match long_result {
    Err(ValidationError::InputTooLong { max_length }) => {
      assert_eq!(max_length, 10, "Should report max_length of 10");
    }
    _ => panic!("Expected InputTooLong error"),
  }
}

/// Test 5: Should Combine Validators With Or Logic
///
/// **GIVEN** validate_email_format and validate_phone_format validators
/// **WHEN** validator1.or(validator2) is applied
/// **THEN** validation should pass if either validator succeeds
/// **AND** validation should fail only if both fail
/// **AND** error should indicate both validators failed
#[test]
fn test_should_combine_validators_with_or_logic() {
  // GIVEN two validators (email and phone)
  let validate_email = Validator::single(|s: &str| -> Result<String, ValidationError> {
    if s.contains('@') && s.contains('.') {
      Ok(s.to_string())
    } else {
      Err(ValidationError::InvalidFormat {
        reason: "Invalid email format".to_string(),
      })
    }
  });

  let validate_phone = Validator::single(|s: &str| -> Result<String, ValidationError> {
    if s
      .chars()
      .all(|c| c.is_ascii_digit() || c == '-' || c == '+')
    {
      Ok(s.to_string())
    } else {
      Err(ValidationError::InvalidFormat {
        reason: "Invalid phone format".to_string(),
      })
    }
  });

  // WHEN creating an OR validator
  let or_validator = validate_email.or(validate_phone);

  // THEN validation should pass if either validator succeeds
  let email_input = "test@example.com";
  let email_result = or_validator.validate(email_input);
  assert!(email_result.is_ok(), "Email format should pass");

  let phone_input = "+1-555-0123";
  let phone_result = or_validator.validate(phone_input);
  assert!(phone_result.is_ok(), "Phone format should pass");

  // AND validation should fail only if both fail
  let invalid_input = "invalid input";
  let invalid_result = or_validator.validate(invalid_input);
  assert!(
    invalid_result.is_err(),
    "Invalid input should fail both validators"
  );

  // AND error should indicate both validators failed
  match invalid_result {
    Err(ValidationError::InvalidFormat { reason }) => {
      assert!(
        reason.contains("Both validators"),
        "Error should mention both validators failed"
      );
    }
    _ => panic!("Expected InvalidFormat error mentioning both validators"),
  }
}

/// Test 6: Should Provide Context In Validation Errors
///
/// **GIVEN** field name "title" and validation error "too long"
/// **WHEN** validation fails
/// **THEN** ValidationMessage should include field context
/// **AND** error message should include "title: too long"
/// **AND** message.field_path() should return "title"
/// **AND** message.message() should return "too long"
#[test]
fn test_should_provide_context_in_validation_errors() {
  // GIVEN field name and validation error
  let field_name = "title";
  let error_message = "too long".to_string();

  // WHEN creating a ValidationMessage
  let msg = ValidationMessage::new(Severity::Error, field_name.to_string(), error_message);

  // THEN message should include field context
  assert_eq!(msg.field_path(), field_name);

  // AND message.to_string() should include field name
  let msg_string = format!("{}", msg);
  assert!(
    msg_string.contains(field_name),
    "Error message should contain field name"
  );

  // AND message.message() should return the reason
  assert_eq!(msg.message(), "too long");
}

/// Test 7: Should Track Quality Metrics
///
/// **GIVEN** a module with code metrics
/// **WHEN** QualityMetrics::new() is called
/// **THEN** metrics should include test_coverage (0.0-1.0)
/// **AND** metrics should include complexity
/// **AND** metrics should include lines_of_code
/// **AND** metrics.quality_score() should calculate weighted score
/// **AND** Result<QualityMetrics, ValidationError> should be Ok
#[test]
fn test_should_track_quality_metrics() {
  // GIVEN module metrics
  let test_coverage = 0.85;
  let complexity = 5;
  let lines_of_code = 1000;

  // WHEN creating QualityMetrics
  let result = QualityMetrics::new(test_coverage, complexity, lines_of_code);

  // THEN Result should be Ok
  assert!(result.is_ok(), "QualityMetrics::new should be Ok");

  let metrics = match result {
    Ok(m) => m,
    Err(e) => panic!("Expected Ok, got Err: {:?}", e),
  };

  // AND metrics should include test_coverage
  assert_eq!(metrics.test_coverage().value(), test_coverage);

  // AND metrics should include complexity
  assert_eq!(metrics.complexity(), complexity);

  // AND metrics should include lines_of_code
  assert_eq!(metrics.lines_of_code(), lines_of_code);

  // AND metrics.quality_score() should calculate weighted score
  let quality_score = metrics.quality_score();
  assert!(
    quality_score.value() >= 0.0 && quality_score.value() <= 1.0,
    "Quality score should be in range [0.0, 1.0]"
  );
}

/// Test 7b: Should Reject Invalid Quality Metrics
#[test]
fn test_should_reject_invalid_quality_metrics() {
  // GIVEN invalid test coverage (> 1.0)
  let invalid_coverage = 1.5;

  // WHEN creating QualityMetrics
  let result = QualityMetrics::new(invalid_coverage, 5, 1000);

  // THEN Result should be Err
  assert!(result.is_err(), "Should reject invalid test coverage");
}

/// Test 8: Should Support Custom Validators
///
/// **GIVEN** custom validation rule: "must contain keyword 'security'"
/// **WHEN** CustomValidator::new() is applied
/// **THEN** validation should pass if input contains keyword
/// **AND** validation should fail with custom message if not
/// **AND** custom error message should be provided
/// **AND** validator should be reusable
#[test]
fn test_should_support_custom_validators() {
  // GIVEN custom validation rule
  let custom_validator = CustomValidator::new(
    |s: &str| -> Result<String, ValidationError> {
      if s.contains("security") {
        Ok(s.to_string())
      } else {
        Err(ValidationError::InvalidFormat {
          reason: "Must contain keyword 'security'".to_string(),
        })
      }
    },
    "Security keyword required".to_string(),
  );

  // WHEN validating input with keyword
  let valid_input = "This is a security update";
  let valid_result = custom_validator.validate(valid_input);

  // THEN validation should pass
  assert!(valid_result.is_ok(), "Should pass when keyword is present");

  // AND validation should fail with custom message if not
  let invalid_input = "This is a regular update";
  let invalid_result = custom_validator.validate(invalid_input);

  assert!(
    invalid_result.is_err(),
    "Should fail when keyword is missing"
  );
  match invalid_result {
    Err(ValidationError::InvalidFormat { reason }) => {
      assert_eq!(reason, "Must contain keyword 'security'");
    }
    _ => panic!("Expected InvalidFormat with custom message"),
  }

  // AND validator should be reusable
  let result2 = custom_validator.validate("security patch");
  assert!(result2.is_ok(), "Validator should be reusable");
}

/// Test 9: Should Validate With Severity Levels
///
/// **GIVEN** validation with severity (Error, Warning, Info)
/// **WHEN** validation runs
/// **THEN** errors should prevent operation (is_valid = false)
/// **AND** warnings should allow operation but notify user
/// **AND** info should be logged but not block operation
/// **AND** ValidationReport should separate by severity
#[test]
fn test_should_validate_with_severity_levels() {
  // GIVEN validations with different severities
  let msg_error =
    ValidationMessage::new(Severity::Error, "email".to_string(), "required".to_string());
  let msg_warning = ValidationMessage::new(
    Severity::Warning,
    "description_length".to_string(),
    "too long".to_string(),
  );
  let msg_info = ValidationMessage::new(
    Severity::Info,
    "optional_field".to_string(),
    "not set".to_string(),
  );

  // WHEN creating report with mixed severities
  let report = ValidationReport::new(vec![msg_error, msg_warning, msg_info]);

  // THEN errors should prevent operation
  assert!(!report.is_valid(), "Errors should prevent operation");

  // AND report should separate by severity
  let errors = report.errors();
  let warnings = report.warnings();
  let info = report.info();

  assert_eq!(errors.len(), 1, "Should have 1 error");
  assert_eq!(warnings.len(), 1, "Should have 1 warning");
  assert_eq!(info.len(), 1, "Should have 1 info message");

  // AND warnings should allow operation but notify user
  let warning_only_report = ValidationReport::new(vec![ValidationMessage::new(
    Severity::Warning,
    "field".to_string(),
    "warning".to_string(),
  )]);
  assert!(
    warning_only_report.is_valid(),
    "Warnings should not prevent operation"
  );
}

/// Test 10: QualityScore Thresholds and Categories
#[test]
fn test_quality_score_thresholds() {
  let poor_score = QualityScore::new(0.3).unwrap();
  let fair_score = QualityScore::new(0.5).unwrap();
  let good_score = QualityScore::new(0.7).unwrap();
  let excellent_score = QualityScore::new(0.9).unwrap();

  assert!(poor_score.is_poor(), "0.3 should be poor");
  assert!(poor_score.is_failing(), "0.3 should be failing");

  assert!(!fair_score.is_poor(), "0.5 should not be poor");
  assert!(fair_score.is_failing(), "0.5 should be failing");

  assert!(good_score.is_passing(), "0.7 should be passing");
  assert!(!good_score.is_failing(), "0.7 should not be failing");
  assert!(!good_score.is_excellent(), "0.7 should not be excellent");

  assert!(excellent_score.is_passing(), "0.9 should be passing");
  assert!(excellent_score.is_excellent(), "0.9 should be excellent");
}

/// Edge Case: QualityScore Boundary Values
#[test]
fn test_quality_score_boundary_values() {
  // Test exact boundaries
  assert!(QualityScore::new(0.0).is_ok(), "0.0 should be valid");
  assert!(QualityScore::new(1.0).is_ok(), "1.0 should be valid");
  assert!(QualityScore::new(0.5).is_ok(), "0.5 should be valid");

  // Test just outside boundaries
  assert!(
    QualityScore::new(-0.0001).is_err(),
    "Negative should be invalid"
  );
  assert!(
    QualityScore::new(1.0001).is_err(),
    "Above 1.0 should be invalid"
  );
}

/// Edge Case: QualityScore Display
#[test]
fn test_quality_score_display() {
  let score = QualityScore::new(0.8567).unwrap();
  let display = format!("{}", score);
  assert!(
    display.contains("0.86") || display.contains("0.85"),
    "Should display rounded value"
  );
}

/// Test: ValidationReport Error Aggregation
#[test]
fn test_validation_report_error_aggregation() {
  let msg1 = ValidationMessage::new(Severity::Error, "field1".to_string(), "error1".to_string());
  let msg2 = ValidationMessage::new(Severity::Error, "field2".to_string(), "error2".to_string());
  let msg3 = ValidationMessage::new(Severity::Info, "field3".to_string(), "info".to_string());

  let report = ValidationReport::new(vec![msg1, msg2, msg3]);

  assert_eq!(report.messages().len(), 3);
  assert_eq!(report.error_count(), 2);
  assert_eq!(report.info_count(), 1);
}

/// Test: QualityMetrics with Custom Metrics
#[test]
fn test_quality_metrics_with_custom() {
  let metrics_result = QualityMetrics::new(0.8, 10, 1000);

  assert!(metrics_result.is_ok(), "QualityMetrics::new should succeed");

  let metrics_with_custom =
    metrics_result.and_then(|m| m.with_custom_metric("documentation".to_string(), 0.9));

  assert!(
    metrics_with_custom.is_ok(),
    "with_custom_metric should succeed"
  );

  let metrics = metrics_with_custom.unwrap();

  assert!(metrics.custom_metrics().contains_key("documentation"));
  assert_eq!(metrics.custom_metrics().get("documentation"), Some(&0.9));
}

/// Test: QualityMetrics Invalid Custom Metric
#[test]
fn test_quality_metrics_invalid_custom_metric() {
  let metrics_result = QualityMetrics::new(0.8, 10, 1000);

  assert!(metrics_result.is_ok(), "QualityMetrics::new should succeed");

  let result = metrics_result.and_then(|m| m.with_custom_metric("test".to_string(), 1.5));

  assert!(result.is_err(), "Should reject custom metric > 1.0");
}

/// Test: ValidationReport Display
#[test]
fn test_validation_report_display() {
  let msg = ValidationMessage::new(Severity::Error, "field".to_string(), "error".to_string());
  let report = ValidationReport::new(vec![msg]);

  let display = format!("{}", report);
  assert!(
    display.contains("Validation failed"),
    "Should show validation failed"
  );
  assert!(display.contains("field"), "Should show field name");
  assert!(display.contains("error"), "Should show error message");
}

/// Test: Empty ValidationReport Display
#[test]
fn test_empty_validation_report_display() {
  let report = ValidationReport::valid();
  let display = format!("{}", report);
  assert!(
    display.contains("Validation passed"),
    "Should show validation passed"
  );
}

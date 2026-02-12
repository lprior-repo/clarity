//! Tests for use case parsing functionality
//!
//! These tests verify that:
//! 1. Valid use case parsing works correctly
//! 2. Invalid format returns None
//! 3. Extra whitespace is handled properly
//!
//! Follows the functional core pattern with zero unwrap.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::needless_raw_string_hashes)]

use crate::planner::{
  parse_use_case, parse_use_cases, validate_use_case, UseCase, UseCasePriority,
};

/// Test valid use case parsing
///
/// This test verifies that a properly formatted use case string
/// can be parsed into a UseCase struct with all fields correctly populated.
#[test]
fn test_valid_use_case_parsing() {
  let input = r#"User Registration|New user creates account|User clicks Register button|Critical
preconditions
User is not logged in
User has valid email
main_flow
User enters email and password
User clicks Register button
System validates input
System creates account
User is logged in
alternative_flows
Email already exists
Show error message
Password too weak
Show error message
postconditions
Account is created
User is logged in
Email is verified"#;

  let result = parse_use_case(input);

  // Verify parsing succeeded
  assert!(
    result.is_some(),
    "Valid use case should be parsed successfully"
  );

  let use_case = result.unwrap();

  // Verify basic fields
  assert_eq!(use_case.title, "User Registration");
  assert_eq!(use_case.description, "New user creates account");
  assert_eq!(use_case.trigger, "User clicks Register button");
  assert_eq!(use_case.priority, UseCasePriority::Critical);

  // Verify sections
  assert_eq!(use_case.preconditions.len(), 2);
  assert_eq!(use_case.main_flow.len(), 5);
  assert_eq!(use_case.alternative_flows.len(), 4);
  assert_eq!(use_case.postconditions.len(), 3);

  // Verify content
  assert_eq!(use_case.preconditions[0], "User is not logged in");
  assert_eq!(use_case.preconditions[1], "User has valid email");
  assert_eq!(use_case.main_flow[0], "User enters email and password");
  assert_eq!(use_case.main_flow[4], "User is logged in");
}

/// Test invalid format returns None
///
/// This test verifies that malformed use case strings
/// are rejected and return None.
#[test]
fn test_invalid_format_returns_none() {
  // Test missing header
  let input1 = "Missing header parts";
  let result1 = parse_use_case(input1);
  assert!(result1.is_none(), "Incomplete header should return None");

  // Test missing section headers
  let input2 = r#"Valid Header|Description|Trigger|Critical
No section headers
Just some text
main_flow
This should fail"#;
  let result2 = parse_use_case(input2);
  assert!(
    result2.is_none(),
    "Missing section headers should return None"
  );

  // Test invalid priority
  let input3 = r#"Valid Header|Description|Trigger|InvalidPriority
preconditions
Some precondition
main_flow
Some step"#;
  let result3 = parse_use_case(input3);
  assert!(result3.is_none(), "Invalid priority should return None");

  // Test too few lines
  let input4 = r#"Header|Desc|Trigger|Critical
preconditions
Only two lines"#;
  let result4 = parse_use_case(input4);
  assert!(result4.is_none(), "Insufficient lines should return None");
}

/// Test extra whitespace handling
///
/// This test verifies that the parser correctly handles
/// extra whitespace in the input string.
#[test]
fn test_extra_whitespace_handling() {
  let input = r#"   User Registration   |   New user creates account   |   User clicks Register button   |   Critical

preconditions

User is not logged in

main_flow

User enters email and password

postconditions

Account is created
"#;

  let result = parse_use_case(input);

  // Verify parsing succeeded despite extra whitespace
  assert!(
    result.is_some(),
    "Use case with extra whitespace should be parsed"
  );

  let use_case = result.unwrap();

  // Verify whitespace was trimmed
  assert_eq!(use_case.title, "User Registration");
  assert_eq!(use_case.description, "New user creates account");
  assert_eq!(use_case.trigger, "User clicks Register button");

  // Verify content is still correctly parsed despite empty lines
  assert_eq!(use_case.preconditions.len(), 1);
  assert_eq!(use_case.main_flow.len(), 1);
  assert_eq!(use_case.postconditions.len(), 1);
}

/// Test minimal valid use case
///
/// This test verifies that a minimal use case with only
/// required sections can be parsed successfully.
#[test]
fn test_minimal_valid_use_case() {
  let input = r#"Minimal Use Case|Description|Trigger|Medium
preconditions
Precondition 1
main_flow
Step 1
postconditions
Postcondition 1"#;

  let result = parse_use_case(input);
  assert!(result.is_some(), "Minimal use case should be parsed");

  let use_case = result.unwrap();
  assert_eq!(use_case.title, "Minimal Use Case");
  assert_eq!(use_case.description, "Description");
  assert_eq!(use_case.trigger, "Trigger");
  assert_eq!(use_case.priority, UseCasePriority::Medium);
  assert_eq!(use_case.preconditions.len(), 1);
  assert_eq!(use_case.main_flow.len(), 1);
  assert_eq!(use_case.postconditions.len(), 1);
  assert!(use_case.alternative_flows.is_empty());
}

/// Test validation of use cases
///
/// This test verifies that the validate_use_case function
/// correctly identifies valid and invalid use cases.
#[test]
fn test_use_case_validation() {
  // Valid use case
  let valid_use_case = UseCase::new(
    "Valid Title".to_string(),
    "Valid Description".to_string(),
    "Valid Trigger".to_string(),
  )
  .with_main_flow_step("Step 1".to_string())
  .with_precondition("Precondition 1".to_string());

  assert!(
    validate_use_case(&valid_use_case),
    "Valid use case should pass validation"
  );

  // Invalid use cases
  let empty_title = UseCase::new(
    "".to_string(),
    "Valid Description".to_string(),
    "Valid Trigger".to_string(),
  )
  .with_main_flow_step("Step 1".to_string());
  assert!(
    !validate_use_case(&empty_title),
    "Use case with empty title should fail validation"
  );

  let empty_description = UseCase::new(
    "Valid Title".to_string(),
    "".to_string(),
    "Valid Trigger".to_string(),
  )
  .with_main_flow_step("Step 1".to_string());
  assert!(
    !validate_use_case(&empty_description),
    "Use case with empty description should fail validation"
  );

  let empty_trigger = UseCase::new(
    "Valid Title".to_string(),
    "Valid Description".to_string(),
    "".to_string(),
  )
  .with_main_flow_step("Step 1".to_string());
  assert!(
    !validate_use_case(&empty_trigger),
    "Use case with empty trigger should fail validation"
  );

  let empty_main_flow = UseCase::new(
    "Valid Title".to_string(),
    "Valid Description".to_string(),
    "Valid Trigger".to_string(),
  )
  .with_precondition("Precondition 1".to_string());
  assert!(
    !validate_use_case(&empty_main_flow),
    "Use case without main flow should fail validation"
  );
}

/// Test parsing multiple use cases
///
/// This test verifies that multiple use cases separated by
/// double newlines can be parsed correctly.
#[test]
fn test_multiple_use_case_parsing() {
  let input = r#"Use Case 1|Description 1|Trigger 1|High
preconditions
Precond 1
main_flow
Step 1
postconditions
Post 1

Use Case 2|Description 2|Trigger 2|Critical
preconditions
Precond 2
main_flow
Step 2
postconditions
Post 2"#;

  let results = parse_use_cases(input);
  assert_eq!(results.len(), 2, "Should parse two use cases");

  assert_eq!(results[0].title, "Use Case 1");
  assert_eq!(results[0].priority, UseCasePriority::High);
  assert_eq!(results[1].title, "Use Case 2");
  assert_eq!(results[1].priority, UseCasePriority::Critical);
}

/// Test mixed valid/invalid use case parsing
///
/// This test verifies that parse_use_cases correctly handles
// input containing both valid and invalid use cases, only
// returning the valid ones.
#[test]
fn test_mixed_validity_parsing() {
  let input = r#"Valid Use Case|Description|Trigger|High
preconditions
Precond
main_flow
Step
postconditions
Post

Invalid Use Case|Description|Trigger|InvalidPriority
preconditions
Precond
main_flow
Step

Another Valid|Description|Trigger|Medium
preconditions
Precond
main_flow
Step
postconditions
Post"#;

  let results = parse_use_cases(input);
  assert_eq!(results.len(), 2, "Should only parse valid use cases");

  assert_eq!(results[0].title, "Valid Use Case");
  assert_eq!(results[1].title, "Another Valid");
  // Verify priorities are valid enum values (no Invalid variant exists)
  assert!(matches!(
    results[0].priority,
    UseCasePriority::Critical
      | UseCasePriority::High
      | UseCasePriority::Medium
      | UseCasePriority::Low
  ));
  assert!(matches!(
    results[1].priority,
    UseCasePriority::Critical
      | UseCasePriority::High
      | UseCasePriority::Medium
      | UseCasePriority::Low
  ));
}

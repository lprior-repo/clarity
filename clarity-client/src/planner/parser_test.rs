//! Standalone test for use case parser
//!
//! This file contains tests for the use case parsing functionality.
//! Tests valid parsing, invalid formats, and whitespace handling.

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_valid_use_case() {
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
    assert!(result.is_some());

    let use_case = result.unwrap();
    assert_eq!(use_case.title, "User Registration");
    assert_eq!(use_case.description, "New user creates account");
    assert_eq!(use_case.trigger, "User clicks Register button");
    assert_eq!(use_case.priority, UseCasePriority::Critical);
    assert_eq!(use_case.main_flow.len(), 5);
    assert_eq!(use_case.alternative_flows.len(), 2);
    assert_eq!(use_case.postconditions.len(), 3);
  }

  #[test]
  fn test_parse_use_case_with_extra_whitespace() {
    let input = r#"   User Registration   |   New user creates account   |   User clicks Register button   |   Critical

preconditions

User is not logged in

main_flow

User enters email and password

postconditions

Account is created
"#;

    let result = parse_use_case(input);
    assert!(result.is_some());

    let use_case = result.unwrap();
    assert_eq!(use_case.title, "User Registration");
    assert_eq!(use_case.description, "New user creates account");
    assert_eq!(use_case.trigger, "User clicks Register button");
    assert_eq!(use_case.main_flow.len(), 1);
    assert_eq!(use_case.preconditions.len(), 1);
  }

  #[test]
  fn test_parse_use_case_invalid_format() {
    let input = r#"Invalid format missing parts"#;
    let result = parse_use_case(input);
    assert!(result.is_none());
  }

  #[test]
  fn test_parse_use_case_missing_section_headers() {
    let input = r#"User Registration|New user creates account|User clicks Register button|Critical
Just some text without proper section headers
main_flow
User does something"#;
    let result = parse_use_case(input);
    assert!(result.is_none());
  }

  #[test]
  fn test_parse_use_case_invalid_priority() {
    let input = r#"User Registration|New user creates account|User clicks Register button|InvalidPriority
preconditions
User is not logged in
main_flow
User enters email and password"#;
    let result = parse_use_case(input);
    assert!(result.is_none());
  }

  #[test]
  fn test_parse_use_case_minimal() {
    let input = r#"Minimal Use Case|Description|Trigger|Medium
preconditions
Precondition 1
main_flow
Step 1
postconditions
Postcondition 1"#;

    let result = parse_use_case(input);
    assert!(result.is_some());

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

  #[test]
  fn test_parse_use_case_empty_lines() {
    let input = r#"Use Case|Description|Trigger|High
preconditions

main_flow

Step 1
postconditions


Postcondition 1"#;

    let result = parse_use_case(input);
    assert!(result.is_some());

    let use_case = result.unwrap();
    assert_eq!(use_case.main_flow.len(), 1);
    assert_eq!(use_case.postconditions.len(), 1);
  }

  #[test]
  fn test_parse_use_cases_multiple() {
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
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].title, "Use Case 1");
    assert_eq!(results[1].title, "Use Case 2");
  }

  #[test]
  fn test_parse_use_cases_mixed_validity() {
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
    assert_eq!(results.len(), 2); // Only valid ones
    assert_eq!(results[0].title, "Valid Use Case");
    assert_eq!(results[1].title, "Another Valid");
  }

  #[test]
  fn test_validate_use_case_valid() {
    let use_case = UseCase::new(
      "Valid".to_string(),
      "Description".to_string(),
      "Trigger".to_string(),
    )
    .with_main_flow_step("Step 1".to_string())
    .with_precondition("Precondition".to_string());

    assert!(validate_use_case(&use_case));
  }

  #[test]
  fn test_validate_use_case_empty_title() {
    let use_case = UseCase::new(
      "".to_string(),
      "Description".to_string(),
      "Trigger".to_string(),
    )
    .with_main_flow_step("Step 1".to_string());

    assert!(!validate_use_case(&use_case));
  }

  #[test]
  fn test_validate_use_case_empty_description() {
    let use_case = UseCase::new(
      "Title".to_string(),
      "".to_string(),
      "Trigger".to_string(),
    )
    .with_main_flow_step("Step 1".to_string());

    assert!(!validate_use_case(&use_case));
  }

  #[test]
  fn test_validate_use_case_empty_trigger() {
    let use_case = UseCase::new(
      "Title".to_string(),
      "Description".to_string(),
      "".to_string(),
    )
    .with_main_flow_step("Step 1".to_string());

    assert!(!validate_use_case(&use_case));
  }

  #[test]
  fn test_validate_use_case_empty_main_flow() {
    let use_case = UseCase::new(
      "Title".to_string(),
      "Description".to_string(),
      "Trigger".to_string(),
    )
    .with_precondition("Precondition".to_string());

    assert!(!validate_use_case(&use_case));
  }
}
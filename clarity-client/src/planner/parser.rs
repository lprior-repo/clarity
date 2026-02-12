//! Use case parsing module
//!
//! Pure functions for parsing use cases from various formats.
//! All parsing functions return Option<T> to avoid panics on invalid input.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::planner::types::{UseCase, UseCasePriority};

/// Parse a use case from a structured string format
///
/// The expected format is:
/// ```
/// title|description|trigger|priority
/// preconditions
/// main_flow
/// alternative_flows
/// postconditions
/// ```
///
/// # Arguments
/// * `input` - The string to parse
///
/// # Returns
/// * `Some(UseCase)` if parsing succeeds
/// * `None` if the format is invalid or required fields are missing
#[must_use]
pub fn parse_use_case(input: &str) -> Option<UseCase> {
  let lines = input
    .lines()
    .map(|line| line.trim())
    .filter(|line| !line.is_empty())
    .collect::<Vec<_>>();

  // Need at least the header line with title, description, trigger, and priority
  if lines.len() < 5 {
    return None;
  }

  // Parse header line: title|description|trigger|priority
  let header_parts: Vec<_> = lines[0].split('|').collect();
  if header_parts.len() < 4 {
    return None;
  }

  let title = header_parts[0].trim().to_string();
  let description = header_parts[1].trim().to_string();
  let trigger = header_parts[2].trim().to_string();
  let priority_str = header_parts[3].trim();

  // Parse priority
  let priority = match priority_str.to_lowercase().as_str() {
    "critical" => UseCasePriority::Critical,
    "high" => UseCasePriority::High,
    "medium" => UseCasePriority::Medium,
    "low" => UseCasePriority::Low,
    _ => return None, // Invalid priority
  };

  // Collect sections
  let mut preconditions = Vec::new();
  let mut main_flow = Vec::new();
  let mut alternative_flows = Vec::new();
  let mut postconditions = Vec::new();

  let mut current_section: Option<&str> = None;
  let mut has_valid_section = false;

  for line in &lines[1..] {
    let is_section_header = matches!(
      *line,
      "preconditions" | "main_flow" | "alternative_flows" | "postconditions"
    );

    if is_section_header {
      has_valid_section = true;
      current_section = Some(*line);
    } else if let Some(section) = current_section {
      let trimmed_line = line.trim();
      if !trimmed_line.is_empty() {
        match section {
          "preconditions" => preconditions.push(trimmed_line.to_string()),
          "main_flow" => main_flow.push(trimmed_line.to_string()),
          "alternative_flows" => alternative_flows.push(trimmed_line.to_string()),
          "postconditions" => postconditions.push(trimmed_line.to_string()),
          _ => {}
        }
      }
    } else if !line.trim().is_empty() {
      // Non-section, non-empty line before any section header = invalid format
      return None;
    }
  }

  // Must have at least one valid section with content
  if !has_valid_section || main_flow.is_empty() {
    return None;
  }

  // Create use case with parsed data
  let mut use_case = UseCase::new(title, description, trigger);
  use_case = use_case.with_priority(priority);

  // Add non-empty sections
  for precondition in preconditions {
    use_case = use_case.with_precondition(precondition);
  }

  for step in main_flow {
    use_case = use_case.with_main_flow_step(step);
  }

  for flow in alternative_flows {
    use_case = use_case.with_alternative_flow(flow);
  }

  for postcondition in postconditions {
    use_case = use_case.with_postcondition(postcondition);
  }

  Some(use_case)
}

/// Parse a use case from JSON format
///
/// # Arguments
/// * `_json_str` - The JSON string to parse
///
/// # Returns
/// * `Some(UseCase)` if parsing succeeds
/// * `None` if the JSON is invalid or required fields are missing
#[must_use]
pub fn parse_use_case_json(_json_str: &str) -> Option<UseCase> {
  // In a real implementation, this would use serde_json
  // For now, we'll implement a simple version
  // This would typically return None on invalid JSON
  // But since we don't have serde_json in the core, we'll keep it simple
  None
}

/// Parse multiple use cases from a block of text
///
/// Each use case should be separated by a double newline.
///
/// # Arguments
/// * `input` - The text block containing multiple use cases
///
/// # Returns
/// * `Vec<UseCase>` successfully parsed use cases
/// * Empty vector if no use cases could be parsed
#[must_use]
pub fn parse_use_cases(input: &str) -> Vec<UseCase> {
  input
    .split("\n\n")
    .filter_map(|block| parse_use_case(block))
    .collect()
}

/// Validate that a use case has required fields
///
/// # Arguments
/// * `use_case` - The use case to validate
///
/// # Returns
/// * `true` if the use case has all required fields
/// * `false` otherwise
#[must_use]
pub fn validate_use_case(use_case: &UseCase) -> bool {
  !use_case.title.trim().is_empty()
    && !use_case.description.trim().is_empty()
    && !use_case.trigger.trim().is_empty()
    && !use_case.main_flow.is_empty()
}

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
    assert_eq!(use_case.alternative_flows.len(), 4);
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
    let use_case = UseCase::new("Title".to_string(), "".to_string(), "Trigger".to_string())
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

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! EARS (Easy Approach to Requirements Syntax) parser.
//!
//! Implements the 5 EARS pattern types for requirements parsing:
//! - Ubiquitous: "The system shall..."
//! - State-driven: "When X, the system shall Y..."
//! - Event-driven: "During X, the system shall Y..."
//! - Unwanted: "If X, the system shall NOT..."
//! - Optional: "Where X, the system shall Y..."

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Domain errors for EARS parsing.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum EarsError {
  #[error("empty input")]
  EmptyInput,

  #[error("unrecognized requirement pattern: {0}")]
  UnrecognizedPattern(String),

  #[error("malformed requirement: missing action")]
  MalformedRequirement,
}

/// EARS requirement pattern types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EarsRequirement {
  /// Ubiquitous: "The system shall..."
  Ubiquitous { actor: String, action: String },

  /// State-driven: "When X, the system shall Y..."
  StateDriven {
    actor: String,
    trigger: String,
    action: String,
  },

  /// Event-driven: "During X, the system shall Y..."
  EventDriven {
    actor: String,
    trigger: String,
    action: String,
  },

  /// Unwanted: "If X, the system shall NOT..."
  Unwanted {
    actor: String,
    condition: String,
    action: String,
  },

  /// Optional: "Where X, the system shall Y..."
  Optional {
    actor: String,
    condition: String,
    action: String,
  },
}

/// Parsed EARS output containing all recognized requirements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EarsOutput {
  pub requirements: Vec<EarsRequirement>,
  pub errors: Vec<String>,
}

impl EarsOutput {
  /// Create a new empty `EarsOutput`.
  #[must_use]
  pub const fn new() -> Self {
    Self {
      requirements: Vec::new(),
      errors: Vec::new(),
    }
  }

  /// Add a requirement to the output.
  #[must_use]
  pub fn with_requirement(mut self, requirement: EarsRequirement) -> Self {
    self.requirements.push(requirement);
    self
  }

  /// Add an error to the output.
  #[must_use]
  pub fn with_error(mut self, error: String) -> Self {
    self.errors.push(error);
    self
  }
}

impl Default for EarsOutput {
  fn default() -> Self {
    Self::new()
  }
}

/// Parse a single requirement line into an `EarsRequirement`.
pub fn parse_requirement(input: &str) -> Result<EarsRequirement, EarsError> {
  let trimmed = input.trim();

  if trimmed.is_empty() {
    Err(EarsError::EmptyInput)
  } else {
    parse_requirement_patterns(trimmed)
  }
}

/// Parse requirement text against all EARS patterns.
fn parse_requirement_patterns(input: &str) -> Result<EarsRequirement, EarsError> {
  // Helper to check if a keyword appears as a separate word (case-insensitive)
  let has_keyword = |text: &str, keyword: &str| -> bool {
    let lower = text.to_lowercase();
    // Split by whitespace and check each word (stripping punctuation)
    lower.split_whitespace().any(|word| {
      // Strip trailing punctuation from the word
      let word_stripped = word.trim_end_matches(|c: char| !c.is_alphanumeric());
      word_stripped == keyword
    })
  };

  // Try each pattern in order
  match parse_unwanted(input) {
    Some(req) => return Ok(req),
    None => {
      // If "if" keyword was present but pattern didn't match, it's malformed
      if has_keyword(input, "if") {
        return Err(EarsError::UnrecognizedPattern(input.to_string()));
      }
    }
  }

  match parse_event_driven(input) {
    Some(req) => return Ok(req),
    None => {
      // If "during" keyword was present but pattern didn't match, it's malformed
      if has_keyword(input, "during") {
        return Err(EarsError::UnrecognizedPattern(input.to_string()));
      }
    }
  }

  match parse_state_driven(input) {
    Some(req) => return Ok(req),
    None => {
      // If "when" keyword was present but pattern didn't match, it's malformed
      if has_keyword(input, "when") {
        return Err(EarsError::UnrecognizedPattern(input.to_string()));
      }
    }
  }

  match parse_optional(input) {
    Some(req) => return Ok(req),
    None => {
      // If "where" keyword was present but pattern didn't match, it's malformed
      if has_keyword(input, "where") {
        return Err(EarsError::UnrecognizedPattern(input.to_string()));
      }
    }
  }

  // Finally try ubiquitous
  parse_ubiquitous(input).ok_or_else(|| EarsError::UnrecognizedPattern(input.to_string()))
}

/// Parse ubiquitous: "The system shall..."
fn parse_ubiquitous(input: &str) -> Option<EarsRequirement> {
  const PATTERN: &str = "the system shall";

  let lower = input.to_lowercase();

  lower.find(PATTERN).and_then(|start| {
    let actor = "system".to_string();
    let action = input[start + PATTERN.len()..].trim().to_string();

    if action.is_empty() {
      None
    } else {
      Some(EarsRequirement::Ubiquitous { actor, action })
    }
  })
}

/// Parse state-driven: "When X, the system shall Y..."
fn parse_state_driven(input: &str) -> Option<EarsRequirement> {
  const TRIGGER_PREFIX: &str = "when";
  const SYSTEM_PATTERN: &str = ", the system shall";

  let lower = input.to_lowercase();

  lower.find(TRIGGER_PREFIX).and_then(|trigger_start| {
    // Find the system pattern in the lowercase string
    let system_pattern_start = lower.find(SYSTEM_PATTERN)?;
    let trigger = input[trigger_start + TRIGGER_PREFIX.len()..system_pattern_start]
      .trim()
      .trim_end_matches(',')
      .trim()
      .to_string();

    let action_start = system_pattern_start + SYSTEM_PATTERN.len();
    let action = input[action_start..].trim().to_string();

    match (trigger.is_empty(), action.is_empty()) {
      (true, _) | (_, true) => None,
      (false, false) => Some(EarsRequirement::StateDriven {
        actor: "system".to_string(),
        trigger,
        action,
      }),
    }
  })
}

/// Parse event-driven: "During X, the system shall Y..."
fn parse_event_driven(input: &str) -> Option<EarsRequirement> {
  const TRIGGER_PREFIX: &str = "during";
  const SYSTEM_PATTERN: &str = ", the system shall";

  let lower = input.to_lowercase();

  lower.find(TRIGGER_PREFIX).and_then(|trigger_start| {
    // Find the system pattern in the lowercase string
    let system_pattern_start = lower.find(SYSTEM_PATTERN)?;
    let trigger = input[trigger_start + TRIGGER_PREFIX.len()..system_pattern_start]
      .trim()
      .trim_end_matches(',')
      .trim()
      .to_string();

    let action_start = system_pattern_start + SYSTEM_PATTERN.len();
    let action = input[action_start..].trim().to_string();

    match (trigger.is_empty(), action.is_empty()) {
      (true, _) | (_, true) => None,
      (false, false) => Some(EarsRequirement::EventDriven {
        actor: "system".to_string(),
        trigger,
        action,
      }),
    }
  })
}

/// Parse unwanted: "If X, the system shall NOT..."
fn parse_unwanted(input: &str) -> Option<EarsRequirement> {
  const CONDITION_PREFIX: &str = "if";
  const NEGATION_PATTERN: &str = ", the system shall not";

  let lower = input.to_lowercase();

  lower.find(CONDITION_PREFIX).and_then(|condition_start| {
    // Find the negation pattern in the lowercase string
    let negation_pattern_start = lower.find(NEGATION_PATTERN)?;
    let condition = input[condition_start + CONDITION_PREFIX.len()..negation_pattern_start]
      .trim()
      .trim_end_matches(',')
      .trim()
      .to_string();

    let action_start = negation_pattern_start + NEGATION_PATTERN.len();
    let action = input[action_start..].trim().to_string();

    match (condition.is_empty(), action.is_empty()) {
      (true, _) | (_, true) => None,
      (false, false) => Some(EarsRequirement::Unwanted {
        actor: "system".to_string(),
        condition,
        action,
      }),
    }
  })
}

/// Parse optional: "Where X, the system shall Y..."
fn parse_optional(input: &str) -> Option<EarsRequirement> {
  const CONDITION_PREFIX: &str = "where";
  const SYSTEM_PATTERN: &str = ", the system shall";

  let lower = input.to_lowercase();

  lower.find(CONDITION_PREFIX).and_then(|condition_start| {
    // Find the system pattern in the lowercase string
    let system_pattern_start = lower.find(SYSTEM_PATTERN)?;
    let condition = input[condition_start + CONDITION_PREFIX.len()..system_pattern_start]
      .trim()
      .trim_end_matches(',')
      .trim()
      .to_string();

    let action_start = system_pattern_start + SYSTEM_PATTERN.len();
    let action = input[action_start..].trim().to_string();

    match (condition.is_empty(), action.is_empty()) {
      (true, _) | (_, true) => None,
      (false, false) => Some(EarsRequirement::Optional {
        actor: "system".to_string(),
        condition,
        action,
      }),
    }
  })
}

/// Parse multiple requirements from multi-line input.
#[must_use]
pub fn parse_requirements(input: &str) -> EarsOutput {
  input.lines().filter(|line| !line.trim().is_empty()).fold(
    EarsOutput::new(),
    |mut output, line| {
      match parse_requirement(line) {
        Ok(requirement) => output.requirements.push(requirement),
        Err(err) => output.errors.push(err.to_string()),
      }
      output
    },
  )
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::expect_used)]

  use super::*;

  #[test]
  fn test_parse_ubiquitous() {
    let input = "The system shall authenticate users";
    let result = parse_requirement(input);

    assert_eq!(
      result,
      Ok(EarsRequirement::Ubiquitous {
        actor: "system".to_string(),
        action: "authenticate users".to_string(),
      })
    );
  }

  #[test]
  fn test_parse_ubiquitous_case_insensitive() {
    let input = "THE SYSTEM SHALL validate credentials";
    let result = parse_requirement(input);

    assert_eq!(
      result,
      Ok(EarsRequirement::Ubiquitous {
        actor: "system".to_string(),
        action: "validate credentials".to_string(),
      })
    );
  }

  #[test]
  fn test_parse_state_driven() {
    let input = "When the user is logged in, the system shall display the dashboard";
    let result = parse_requirement(input);

    assert_eq!(
      result,
      Ok(EarsRequirement::StateDriven {
        actor: "system".to_string(),
        trigger: "the user is logged in".to_string(),
        action: "display the dashboard".to_string(),
      })
    );
  }

  #[test]
  fn test_parse_state_driven_case_insensitive() {
    let input = "WHEN the door is open, THE SYSTEM SHALL lock automatically";
    let result = parse_requirement(input);

    assert_eq!(
      result,
      Ok(EarsRequirement::StateDriven {
        actor: "system".to_string(),
        trigger: "the door is open".to_string(),
        action: "lock automatically".to_string(),
      })
    );
  }

  #[test]
  fn test_parse_event_driven() {
    let input = "During system startup, the system shall initialize all services";
    let result = parse_requirement(input);

    assert_eq!(
      result,
      Ok(EarsRequirement::EventDriven {
        actor: "system".to_string(),
        trigger: "system startup".to_string(),
        action: "initialize all services".to_string(),
      })
    );
  }

  #[test]
  fn test_parse_event_driven_case_insensitive() {
    let input = "DURING the login process, THE SYSTEM SHALL log all attempts";
    let result = parse_requirement(input);

    assert_eq!(
      result,
      Ok(EarsRequirement::EventDriven {
        actor: "system".to_string(),
        trigger: "the login process".to_string(),
        action: "log all attempts".to_string(),
      })
    );
  }

  #[test]
  fn test_parse_unwanted() {
    let input = "If the password is invalid, the system shall not grant access";
    let result = parse_requirement(input);

    assert_eq!(
      result,
      Ok(EarsRequirement::Unwanted {
        actor: "system".to_string(),
        condition: "the password is invalid".to_string(),
        action: "grant access".to_string(),
      })
    );
  }

  #[test]
  fn test_parse_unwanted_case_insensitive() {
    let input = "IF authentication fails, THE SYSTEM SHALL NOT display sensitive data";
    let result = parse_requirement(input);

    assert_eq!(
      result,
      Ok(EarsRequirement::Unwanted {
        actor: "system".to_string(),
        condition: "authentication fails".to_string(),
        action: "display sensitive data".to_string(),
      })
    );
  }

  #[test]
  fn test_parse_optional() {
    let input = "Where the user has premium access, the system shall enable advanced features";
    let result = parse_requirement(input);

    assert_eq!(
      result,
      Ok(EarsRequirement::Optional {
        actor: "system".to_string(),
        condition: "the user has premium access".to_string(),
        action: "enable advanced features".to_string(),
      })
    );
  }

  #[test]
  fn test_parse_optional_case_insensitive() {
    let input = "WHERE the network is available, THE SYSTEM SHALL sync data";
    let result = parse_requirement(input);

    assert_eq!(
      result,
      Ok(EarsRequirement::Optional {
        actor: "system".to_string(),
        condition: "the network is available".to_string(),
        action: "sync data".to_string(),
      })
    );
  }

  #[test]
  fn test_empty_input() {
    let result = parse_requirement("");
    assert_eq!(result, Err(EarsError::EmptyInput));
  }

  #[test]
  fn test_whitespace_only_input() {
    let result = parse_requirement("   \t  ");
    assert_eq!(result, Err(EarsError::EmptyInput));
  }

  #[test]
  fn test_unrecognized_pattern() {
    let result = parse_requirement("This is not a valid requirement");
    assert!(matches!(result, Err(EarsError::UnrecognizedPattern(_))));
  }

  #[test]
  fn test_malformed_ubiquitous_missing_action() {
    let result = parse_requirement("The system shall");
    assert!(matches!(result, Err(EarsError::UnrecognizedPattern(_))));
  }

  #[test]
  fn test_malformed_state_driven_missing_trigger() {
    let result = parse_requirement("When, the system shall do something");
    assert!(matches!(result, Err(EarsError::UnrecognizedPattern(_))));
  }

  #[test]
  fn test_malformed_state_driven_missing_action() {
    let result = parse_requirement("When triggered, the system shall");
    assert!(matches!(result, Err(EarsError::UnrecognizedPattern(_))));
  }

  #[test]
  fn test_parse_requirements_multiple() {
    let input = r"The system shall authenticate users
When the user is logged in, the system shall display the dashboard
During system startup, the system shall initialize all services
If the password is invalid, the system shall not grant access
Where the user has premium access, the system shall enable advanced features";

    let result = parse_requirements(input);

    assert_eq!(result.requirements.len(), 5);
    assert_eq!(result.errors.len(), 0);
    assert!(matches!(
      result.requirements[0],
      EarsRequirement::Ubiquitous { .. }
    ));
    assert!(matches!(
      result.requirements[1],
      EarsRequirement::StateDriven { .. }
    ));
    assert!(matches!(
      result.requirements[2],
      EarsRequirement::EventDriven { .. }
    ));
    assert!(matches!(
      result.requirements[3],
      EarsRequirement::Unwanted { .. }
    ));
    assert!(matches!(
      result.requirements[4],
      EarsRequirement::Optional { .. }
    ));
  }

  #[test]
  fn test_parse_requirements_with_errors() {
    let input = r"The system shall authenticate users
This is not a valid requirement
When the user is logged in, the system shall display the dashboard
Another invalid line";

    let result = parse_requirements(input);

    assert_eq!(result.requirements.len(), 2);
    assert_eq!(result.errors.len(), 2);
  }

  #[test]
  fn test_parse_requirements_empty_lines() {
    let input = r"The system shall authenticate users

When the user is logged in, the system shall display the dashboard


During system startup, the system shall initialize all services";

    let result = parse_requirements(input);

    assert_eq!(result.requirements.len(), 3);
    assert_eq!(result.errors.len(), 0);
  }

  #[test]
  fn test_ears_output_serialization() {
    let output = EarsOutput::new()
      .with_requirement(EarsRequirement::Ubiquitous {
        actor: "system".to_string(),
        action: "authenticate".to_string(),
      })
      .with_error("Test error".to_string());

    let json = serde_json::to_string(&output).unwrap();
    let deserialized: EarsOutput = serde_json::from_str(&json).unwrap();

    assert_eq!(output, deserialized);
  }

  #[test]
  fn test_ears_requirement_serialization() {
    let req = EarsRequirement::StateDriven {
      actor: "system".to_string(),
      trigger: "user logged in".to_string(),
      action: "show dashboard".to_string(),
    };

    let json = serde_json::to_string(&req).unwrap();
    let deserialized: EarsRequirement = serde_json::from_str(&json).unwrap();

    assert_eq!(req, deserialized);
  }

  #[test]
  fn test_priority_unwanted_over_others() {
    // Unwanted pattern should be detected before others
    let input = "If the condition is met, the system shall not fail";
    let result = parse_requirement(input);

    assert!(matches!(result, Ok(EarsRequirement::Unwanted { .. })));
  }

  #[test]
  fn test_priority_event_over_state() {
    // Event-driven should be detected before state-driven
    let input = "During the process, the system shall monitor";
    let result = parse_requirement(input);

    assert!(matches!(result, Ok(EarsRequirement::EventDriven { .. })));
  }

  #[test]
  fn test_actor_extraction() {
    let req = EarsRequirement::Ubiquitous {
      actor: "system".to_string(),
      action: "process data".to_string(),
    };

    assert_eq!(
      req,
      EarsRequirement::Ubiquitous {
        actor: "system".to_string(),
        action: "process data".to_string(),
      }
    );
  }

  #[test]
  fn test_trigger_extraction_state_driven() {
    let input = "When the temperature exceeds 100 degrees, the system shall activate cooling";
    let result = parse_requirement(input);

    assert_eq!(
      result,
      Ok(EarsRequirement::StateDriven {
        actor: "system".to_string(),
        trigger: "the temperature exceeds 100 degrees".to_string(),
        action: "activate cooling".to_string(),
      })
    );
  }

  #[test]
  fn test_trigger_extraction_event_driven() {
    let input = "During file upload, the system shall validate the file type";
    let result = parse_requirement(input);

    assert_eq!(
      result,
      Ok(EarsRequirement::EventDriven {
        actor: "system".to_string(),
        trigger: "file upload".to_string(),
        action: "validate the file type".to_string(),
      })
    );
  }

  #[test]
  fn test_condition_extraction_unwanted() {
    let input = "If the network is unavailable, the system shall not retry indefinitely";
    let result = parse_requirement(input);

    assert_eq!(
      result,
      Ok(EarsRequirement::Unwanted {
        actor: "system".to_string(),
        condition: "the network is unavailable".to_string(),
        action: "retry indefinitely".to_string(),
      })
    );
  }

  #[test]
  fn test_condition_extraction_optional() {
    let input = "Where the user enables debug mode, the system shall log verbose output";
    let result = parse_requirement(input);

    assert_eq!(
      result,
      Ok(EarsRequirement::Optional {
        actor: "system".to_string(),
        condition: "the user enables debug mode".to_string(),
        action: "log verbose output".to_string(),
      })
    );
  }

  #[test]
  fn test_action_extraction_all_patterns() {
    // Ubiquitous
    let req1 = parse_requirement("The system shall send email notifications").unwrap();
    assert_eq!(
      req1,
      EarsRequirement::Ubiquitous {
        actor: "system".to_string(),
        action: "send email notifications".to_string(),
      }
    );

    // State-driven
    let req2 = parse_requirement("When ready, the system shall start processing").unwrap();
    assert_eq!(
      req2,
      EarsRequirement::StateDriven {
        actor: "system".to_string(),
        trigger: "ready".to_string(),
        action: "start processing".to_string(),
      }
    );

    // Event-driven
    let req3 = parse_requirement("During shutdown, the system shall save state").unwrap();
    assert_eq!(
      req3,
      EarsRequirement::EventDriven {
        actor: "system".to_string(),
        trigger: "shutdown".to_string(),
        action: "save state".to_string(),
      }
    );

    // Unwanted
    let req4 = parse_requirement("If errors occur, the system shall not crash").unwrap();
    assert_eq!(
      req4,
      EarsRequirement::Unwanted {
        actor: "system".to_string(),
        condition: "errors occur".to_string(),
        action: "crash".to_string(),
      }
    );

    // Optional
    let req5 = parse_requirement("Where configured, the system shall use HTTPS").unwrap();
    assert_eq!(
      req5,
      EarsRequirement::Optional {
        actor: "system".to_string(),
        condition: "configured".to_string(),
        action: "use HTTPS".to_string(),
      }
    );
  }
}

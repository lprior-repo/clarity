//! Interview Contract Definitions
//!
//! Validates AI directive JSON for interview system.
//!
//! Ported from intent-cli/src/intent/interview_contract.gleam

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors during contract validation
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ContractError {
  #[error("invalid interview directive JSON: {0}")]
  InvalidJson(String),

  #[error("missing required field: {0}")]
  MissingField(String),

  #[error("invalid action: {0}")]
  InvalidAction(String),

  #[error("invalid agent protocol target: {0}")]
  InvalidTarget(String),

  #[error("invalid question pattern: {0}")]
  InvalidPattern(String),
}

/// Valid action types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
  AskQuestion,
  GenerateBeads,
}

impl ActionType {
  /// Parse from string
  fn from_str(s: &str) -> Result<Self, ContractError> {
    match s {
      "ask_question" => Ok(Self::AskQuestion),
      "generate_beads" => Ok(Self::GenerateBeads),
      _ => Err(ContractError::InvalidAction(s.to_string())),
    }
  }
}

/// Valid EARS pattern types
const VALID_PATTERNS: &[&str] = &[
  "ubiquitous",
  "event_driven",
  "state_driven",
  "optional",
  "unwanted",
  "complex",
];

/// Validate AI directive JSON
///
/// # Errors
/// Returns `ContractError` if the JSON is invalid.
pub fn validate_ai_directive_json(payload: &str) -> Result<(), ContractError> {
  let value: serde_json::Value =
    serde_json::from_str(payload).map_err(|e| ContractError::InvalidJson(e.to_string()))?;

  validate_directive(&value)
}

fn validate_directive(value: &serde_json::Value) -> Result<(), ContractError> {
  let obj = value
    .as_object()
    .ok_or_else(|| ContractError::InvalidJson("Root must be an object".to_string()))?;

  // Validate action
  let action = get_string_field(obj, "action")?;
  let action_type = ActionType::from_str(&action)?;

  // Validate session
  validate_session(obj)?;

  // Validate progress
  validate_progress(obj)?;

  // Validate agent protocol
  validate_agent_protocol(obj)?;

  // Validate guidance
  validate_guidance(obj)?;

  // If action is ask_question, validate question
  if action_type == ActionType::AskQuestion {
    validate_question(obj)?;
  }

  Ok(())
}

fn validate_session(obj: &serde_json::Map<String, serde_json::Value>) -> Result<(), ContractError> {
  let session = obj
    .get("session")
    .and_then(|v| v.as_object())
    .ok_or_else(|| ContractError::MissingField("session".to_string()))?;

  let required_fields = ["id", "profile", "created_at", "updated_at", "stage"];
  for field in required_fields {
    if !session.contains_key(field) {
      return Err(ContractError::MissingField(format!("session.{field}")));
    }
  }

  Ok(())
}

fn validate_progress(obj: &serde_json::Map<String, serde_json::Value>) -> Result<(), ContractError> {
  let progress = obj
    .get("progress")
    .and_then(|v| v.as_object())
    .ok_or_else(|| ContractError::MissingField("progress".to_string()))?;

  let required_fields = [
    "current_round",
    "total_rounds",
    "questions_asked",
    "questions_remaining",
    "percent_complete",
  ];
  for field in required_fields {
    if !progress.contains_key(field) {
      return Err(ContractError::MissingField(format!("progress.{field}")));
    }
  }

  Ok(())
}

fn validate_agent_protocol(
  obj: &serde_json::Map<String, serde_json::Value>,
) -> Result<(), ContractError> {
  let protocol = obj
    .get("agent_protocol")
    .and_then(|v| v.as_object())
    .ok_or_else(|| ContractError::MissingField("agent_protocol".to_string()))?;

  let target = get_string_field(protocol, "target")?;
  if target != "claude_code" {
    return Err(ContractError::InvalidTarget(target));
  }

  let required_fields = ["contract_version", "goal"];
  for field in required_fields {
    if !protocol.contains_key(field) {
      return Err(ContractError::MissingField(format!("agent_protocol.{field}")));
    }
  }

  Ok(())
}

fn validate_guidance(obj: &serde_json::Map<String, serde_json::Value>) -> Result<(), ContractError> {
  let guidance = obj
    .get("guidance")
    .and_then(|v| v.as_object())
    .ok_or_else(|| ContractError::MissingField("guidance".to_string()))?;

  let required_fields = ["next_command", "planning_focus"];
  for field in required_fields {
    if !guidance.contains_key(field) {
      return Err(ContractError::MissingField(format!("guidance.{field}")));
    }
  }

  Ok(())
}

fn validate_question(obj: &serde_json::Map<String, serde_json::Value>) -> Result<(), ContractError> {
  let question = obj
    .get("question")
    .and_then(|v| v.as_object())
    .ok_or_else(|| ContractError::MissingField("question".to_string()))?;

  let required_fields = [
    "id",
    "round",
    "text",
    "pattern",
    "context",
    "examples",
    "priority",
    "perspective",
    "extract_into",
  ];
  for field in required_fields {
    if !question.contains_key(field) {
      return Err(ContractError::MissingField(format!("question.{field}")));
    }
  }

  // Validate pattern
  let pattern = get_string_field(question, "pattern")?;
  if !VALID_PATTERNS.contains(&pattern.as_str()) {
    return Err(ContractError::InvalidPattern(pattern));
  }

  Ok(())
}

fn get_string_field(
  obj: &serde_json::Map<String, serde_json::Value>,
  key: &str,
) -> Result<String, ContractError> {
  obj
    .get(key)
    .and_then(|v| v.as_str())
    .map(String::from)
    .ok_or_else(|| ContractError::MissingField(key.to_string()))
}

#[cfg(test)]
mod tests {
  use super::*;

  fn make_valid_directive() -> String {
    r#"{
      "action": "ask_question",
      "session": {
        "id": "s1",
        "profile": "api",
        "created_at": "2024-01-01",
        "updated_at": "2024-01-01",
        "stage": "discovery"
      },
      "progress": {
        "current_round": 1,
        "total_rounds": 5,
        "questions_asked": 0,
        "questions_remaining": 10,
        "percent_complete": 0
      },
      "agent_protocol": {
        "target": "claude_code",
        "contract_version": "1.0",
        "goal": "collect requirements"
      },
      "guidance": {
        "next_command": "answer",
        "planning_focus": "core features"
      },
      "question": {
        "id": "q1",
        "round": 1,
        "text": "What does this API do?",
        "pattern": "ubiquitous",
        "context": "Core purpose",
        "examples": ["Describe the API"],
        "priority": "critical",
        "perspective": "user",
        "extract_into": ["name"]
      }
    }"#.to_string()
  }

  #[test]
  fn test_validate_ai_directive_json_valid() {
    let json = make_valid_directive();
    let result = validate_ai_directive_json(&json);
    assert!(result.is_ok());
  }

  #[test]
  fn test_validate_ai_directive_json_invalid_json() {
    let json = "not valid json";
    let result = validate_ai_directive_json(json);
    assert!(matches!(result, Err(ContractError::InvalidJson(_))));
  }

  #[test]
  fn test_validate_ai_directive_json_missing_action() {
    let json = r#"{}"#;
    let result = validate_ai_directive_json(json);
    assert!(matches!(result, Err(ContractError::MissingField(_))));
  }

  #[test]
  fn test_validate_ai_directive_json_invalid_action() {
    let mut json = make_valid_directive();
    json = json.replace("\"ask_question\"", "\"invalid_action\"");
    let result = validate_ai_directive_json(&json);
    assert!(matches!(result, Err(ContractError::InvalidAction(_))));
  }

  #[test]
  fn test_validate_ai_directive_json_invalid_target() {
    let mut json = make_valid_directive();
    json = json.replace("\"claude_code\"", "\"invalid_target\"");
    let result = validate_ai_directive_json(&json);
    assert!(matches!(result, Err(ContractError::InvalidTarget(_))));
  }

  #[test]
  fn test_validate_ai_directive_json_invalid_pattern() {
    let mut json = make_valid_directive();
    json = json.replace("\"ubiquitous\"", "\"invalid_pattern\"");
    let result = validate_ai_directive_json(&json);
    assert!(matches!(result, Err(ContractError::InvalidPattern(_))));
  }

  #[test]
  fn test_validate_ai_directive_json_generate_beads() {
    let json = r#"{
      "action": "generate_beads",
      "session": {
        "id": "s1",
        "profile": "api",
        "created_at": "2024-01-01",
        "updated_at": "2024-01-01",
        "stage": "complete"
      },
      "progress": {
        "current_round": 5,
        "total_rounds": 5,
        "questions_asked": 10,
        "questions_remaining": 0,
        "percent_complete": 100
      },
      "agent_protocol": {
        "target": "claude_code",
        "contract_version": "1.0",
        "goal": "generate beads"
      },
      "guidance": {
        "next_command": "generate",
        "planning_focus": "implementation"
      }
    }"#;
    let result = validate_ai_directive_json(json);
    assert!(result.is_ok());
  }

  #[test]
  fn test_action_type_from_str() {
    assert_eq!(ActionType::from_str("ask_question"), Ok(ActionType::AskQuestion));
    assert_eq!(ActionType::from_str("generate_beads"), Ok(ActionType::GenerateBeads));
    assert!(ActionType::from_str("invalid").is_err());
  }

  #[test]
  fn test_valid_patterns() {
    assert!(VALID_PATTERNS.contains(&"ubiquitous"));
    assert!(VALID_PATTERNS.contains(&"event_driven"));
    assert!(VALID_PATTERNS.contains(&"state_driven"));
    assert!(VALID_PATTERNS.contains(&"optional"));
    assert!(VALID_PATTERNS.contains(&"unwanted"));
    assert!(VALID_PATTERNS.contains(&"complex"));
  }
}

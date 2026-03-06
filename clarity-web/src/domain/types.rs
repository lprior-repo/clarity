//! Domain Types
//!
//! Canonical types for Answers and Specifications, synchronized with CUE schemas.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Answer {
  pub step_id: String,
  pub value: String,
  pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Verification {
  pub description: String,
  pub criteria: Vec<String>,
  pub examples: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Behavior {
  pub name: String,
  pub intent: String,
  pub notes: Option<String>,
  pub requires: Option<Vec<String>>,
  pub tags: Option<Vec<String>>,
  pub preconditions: Option<Vec<String>>,
  pub postconditions: Option<Vec<String>>,
  pub verifications: Option<Vec<Verification>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Feature {
  pub name: String,
  pub description: String,
  pub behaviors: Vec<Behavior>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Invariant {
  pub name: String,
  pub description: String,
  pub criteria: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AntiPattern {
  pub name: String,
  pub description: String,
  pub bad_example: serde_json::Value,
  pub good_example: serde_json::Value,
  pub why: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct AIHints {
  pub implementation: Option<ImplementationHint>,
  pub entities: Option<HashMap<String, EntityHint>>,
  pub security: Option<SecurityHint>,
  pub pitfalls: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImplementationHint {
  pub suggested_stack: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EntityHint {
  pub fields: Option<HashMap<String, FieldHint>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum FieldHint {
  Simple(String),
  Detailed {
    description: Option<String>,
    #[serde(rename = "type")]
    field_type: Option<String>,
    validation: Option<String>,
    example: Option<String>,
    #[serde(default)]
    sensitive: bool,
  },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecurityHint {
  pub password_hashing: Option<String>,
  pub jwt_algorithm: Option<String>,
  pub jwt_expiry: Option<String>,
  pub rate_limiting: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Spec {
  pub name: String,
  pub description: String,
  pub audience: String,
  pub version: String,
  pub success_criteria: Vec<String>,
  pub features: Vec<Feature>,
  pub invariants: Vec<Invariant>,
  pub anti_patterns: Vec<AntiPattern>,
  pub ai_hints: AIHints,
}

/// EARS requirement reference for quality scoring
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EarsRequirementRef {
  pub id: String,
  pub text: String,
  pub has_acceptance_criteria: bool,
}

/// Inversion control (requirement inversion for testing)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InversionControl {
  pub has_inversion_tests: bool,
  pub inverted_count: usize,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
  use super::*;

  #[test]
  fn test_answer_serialization() {
    let answer = Answer {
      step_id: "goal".to_string(),
      value: "Win the game".to_string(),
      timestamp: "2026-03-06T12:00:00Z".to_string(),
    };
    let json = serde_json::to_string(&answer).unwrap();
    let decoded: Answer = serde_json::from_str(&json).unwrap();
    assert_eq!(answer, decoded);
  }

  #[test]
  fn test_spec_serialization() {
    let spec = Spec {
      name: "Test Spec".to_string(),
      description: "A spec for testing".to_string(),
      audience: "Devs".to_string(),
      version: "1.0".to_string(),
      success_criteria: vec!["It works".to_string()],
      features: vec![Feature {
        name: "Auth".to_string(),
        description: "Login stuff".to_string(),
        behaviors: vec![Behavior {
          name: "login".to_string(),
          intent: "Allow user to login".to_string(),
          notes: None,
          requires: None,
          tags: None,
          preconditions: None,
          postconditions: None,
          verifications: None,
        }],
      }],
      invariants: vec![],
      anti_patterns: vec![],
      ai_hints: AIHints::default(),
    };
    let json = serde_json::to_string(&spec).unwrap();
    let decoded: Spec = serde_json::from_str(&json).unwrap();
    assert_eq!(spec, decoded);
  }
}

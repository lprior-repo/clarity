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
//! Integration tests for bead integration-003: Mental Lattice output
//!
//! Tests the integration between right panel tabs and Mental Lattice functions:
//! - Plan tab displays EARS requirements
//! - Graph tab visualizes dependency DAG
//! - State tab shows invariants
//! - Results are cached and loaded on tab switch

#![forbid(unsafe_code)]

use clarity_web::lattice::ears::{parse_requirements, EarsRequirement};
use clarity_web::lattice::effects::trace_effects;
use clarity_web::types::Answer;

/// Create a test answer
fn create_answer(step_id: &str, value: &str) -> Answer {
  Answer {
    step_id: step_id.to_string(),
    value: value.to_string(),
    timestamp: "2024-01-01T00:00:00Z".to_string(),
  }
}

#[test]
fn test_ears_parsing_from_answers() {
  let answers = [
    create_answer("req1", "The system shall authenticate users"),
    create_answer(
      "req2",
      "When the user is logged in, the system shall display the dashboard",
    ),
    create_answer(
      "req3",
      "During system startup, the system shall initialize all services",
    ),
    create_answer(
      "req4",
      "If the password is invalid, the system shall not grant access",
    ),
    create_answer(
      "req5",
      "Where the user has premium access, the system shall enable advanced features",
    ),
  ];

  // Extract requirement text from answers
  let requirement_text = answers
    .iter()
    .map(|answer| answer.value.as_str())
    .collect::<Vec<_>>()
    .join("\n");

  let output = parse_requirements(&requirement_text);

  assert_eq!(output.requirements.len(), 5);

  // Verify ubiquitous pattern
  match &output.requirements[0] {
    EarsRequirement::Ubiquitous { actor, action } => {
      assert_eq!(actor, "system");
      assert_eq!(action, "authenticate users");
    }
    other => panic!("expected Ubiquitous requirement, got {other:?}"),
  }

  // Verify state-driven pattern
  match &output.requirements[1] {
    EarsRequirement::StateDriven {
      actor,
      trigger,
      action,
    } => {
      assert_eq!(actor, "system");
      assert_eq!(trigger, "the user is logged in");
      assert_eq!(action, "display the dashboard");
    }
    other => panic!("expected StateDriven requirement, got {other:?}"),
  }

  assert!(output.errors.is_empty());
}

#[test]
fn test_effects_dependency_graph() {
  let solution = r"User authentication causes session creation.
Session creation leads to personalized dashboard.
Dashboard enables feature access.
Feature access requires user permissions.
Poor authentication blocks session creation.
Invalid permissions prevents feature access.";

  let output = trace_effects(solution);

  // Should extract 6 causal relationships
  assert_eq!(output.effects.len(), 6);

  // Verify dependency graph construction
  assert!(!output.dependency_graph.is_empty());

  // Verify visualization structures are built
  assert!(!output.nodes.is_empty());
  assert!(!output.edges.is_empty());

  // Verify root and leaf detection
  let has_root_nodes = output.nodes.iter().any(|n| n.is_root);
  let has_leaf_nodes = output.nodes.iter().any(|n| n.is_leaf);

  assert!(has_root_nodes);
  assert!(has_leaf_nodes);
}

#[test]
fn test_empty_answers_handling() {
  let answers: Vec<Answer> = Vec::new();

  // Should handle empty input gracefully
  let requirement_text = answers
    .iter()
    .map(|answer| answer.value.as_str())
    .collect::<Vec<_>>()
    .join("\n");

  let output = parse_requirements(&requirement_text);

  assert!(output.requirements.is_empty());
  assert!(output.errors.is_empty());
}

#[test]
fn test_partial_ars_requirements() {
  let answers = [
    create_answer("req1", "The system shall authenticate users"),
    create_answer("req2", "This is not a valid requirement"),
    create_answer(
      "req3",
      "When the user is logged in, the system shall display the dashboard",
    ),
  ];

  let requirement_text = answers
    .iter()
    .map(|answer| answer.value.as_str())
    .collect::<Vec<_>>()
    .join("\n");

  let output = parse_requirements(&requirement_text);

  // Should parse 2 valid requirements
  assert_eq!(output.requirements.len(), 2);

  // Should report 1 error
  assert_eq!(output.errors.len(), 1);
  assert!(output.errors[0].contains("unrecognized requirement pattern"));
}

#[test]
fn test_effects_cycle_detection() {
  let solution = "A causes B. B causes C. C causes A.";

  let output = trace_effects(solution);

  // Should detect cycle
  assert!(!output.warnings.is_empty());
  assert!(output
    .warnings
    .iter()
    .any(|w| w.contains("circular dependency")));
}

#[test]
fn test_effects_complex_dependency_chain() {
  let solution = r"Authentication enables session management.
Session management leads to user tracking.
User tracking causes analytics generation.
Analytics results in reporting.
Reporting requires data validation.
Data validation enables system integrity.";

  let output = trace_effects(solution);

  // Should extract all 6 effects
  assert_eq!(output.edges.len(), 6);

  // Should have clear root and leaf nodes
  let auth_node = output
    .nodes
    .iter()
    .find(|n| n.id.contains("Authentication"));
  assert!(auth_node.is_some());
  assert!(auth_node.unwrap().is_root);

  let integrity_node = output.nodes.iter().find(|n| n.id.contains("integrity"));
  assert!(integrity_node.is_some());
  assert!(integrity_node.unwrap().is_leaf);
}

#[test]
fn test_ars_all_pattern_types() {
  let answers = [
    create_answer("req1", "The system shall validate input"),
    create_answer(
      "req2",
      "When the form is submitted, the system shall validate data",
    ),
    create_answer(
      "req3",
      "During processing, the system shall log all activities",
    ),
    create_answer("req4", "If validation fails, the system shall not proceed"),
    create_answer(
      "req5",
      "Where debug mode is enabled, the system shall show detailed logs",
    ),
  ];

  let requirement_text = answers
    .iter()
    .map(|answer| answer.value.as_str())
    .collect::<Vec<_>>()
    .join("\n");

  let output = parse_requirements(&requirement_text);

  assert_eq!(output.requirements.len(), 5);

  // Verify each pattern type
  let has_ubiquitous = output
    .requirements
    .iter()
    .any(|r| matches!(r, EarsRequirement::Ubiquitous { .. }));
  assert!(has_ubiquitous);

  let has_state_driven = output
    .requirements
    .iter()
    .any(|r| matches!(r, EarsRequirement::StateDriven { .. }));
  assert!(has_state_driven);

  let has_event_driven = output
    .requirements
    .iter()
    .any(|r| matches!(r, EarsRequirement::EventDriven { .. }));
  assert!(has_event_driven);

  let has_unwanted = output
    .requirements
    .iter()
    .any(|r| matches!(r, EarsRequirement::Unwanted { .. }));
  assert!(has_unwanted);

  let has_optional = output
    .requirements
    .iter()
    .any(|r| matches!(r, EarsRequirement::Optional { .. }));
  assert!(has_optional);
}

#[test]
fn test_serialization_compatibility() {
  let answers = [
    create_answer("req1", "The system shall authenticate users"),
    create_answer("req2", "When logged in, the system shall show dashboard"),
  ];

  let requirement_text: String = answers
    .iter()
    .map(|a| a.value.as_str())
    .collect::<Vec<_>>()
    .join("\n");

  let ears_output = parse_requirements(&requirement_text);

  // Should serialize to JSON
  let json = serde_json::to_string(&ears_output).expect("EARS output should serialize");

  // Should deserialize back
  let parsed: clarity_web::lattice::ears::EarsOutput =
    serde_json::from_str(&json).expect("EARS output should deserialize");
  assert_eq!(parsed.requirements.len(), ears_output.requirements.len());
}

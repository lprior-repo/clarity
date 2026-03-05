//! Spec Builder
//!
//! Converts interview session answers into valid CUE specifications.
//!
//! Ported from intent-cli/src/intent/spec_builder.gleam

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use itertools::Itertools;

use crate::intent::interview::types::{Answer, InterviewSession, Profile};
use crate::intent::types::{AIHints, Behavior, Feature, ImplementationHints, SecurityHints, Spec};
use crate::intent::util::contains_any_ignore_case;

/// Generated CUE code
#[derive(Debug, Clone)]
pub struct GeneratedCUE {
  pub package: String,
  pub imports: Vec<String>,
  pub body: String,
}

/// Build a CUE spec from a completed interview session
#[must_use]
pub fn build_spec_from_session(session: &InterviewSession) -> String {
  let features = extract_features_from_answers(&session.answers);
  let behaviors = extract_behaviors_from_answers(&session.answers, &session.profile);
  let constraints = extract_constraints_from_answers(&session.answers);
  let security = extract_security_requirements(&session.answers);
  let non_functional = extract_non_functional_requirements(&session.answers);

  let spec = GeneratedCUE {
    package: "package api".to_string(),
    imports: Vec::new(),
    body: build_spec_body(&features, &behaviors, &constraints, &security, &non_functional),
  };

  format!("{}\n\n{}", spec.package, spec.body)
}

/// Extract feature names/titles from answers
#[must_use]
pub fn extract_features_from_answers(answers: &[Answer]) -> Vec<String> {
  answers
    .iter()
    .filter(|answer| {
      contains_any_ignore_case(&answer.question_text, &["feature", "capability"])
    })
    .filter_map(|answer| {
      let trimmed = answer.response.trim();
      if trimmed.is_empty() {
        None
      } else {
        Some(trimmed.to_string())
      }
    })
    .collect()
}

/// Extract API behaviors (methods, paths, status codes)
#[must_use]
pub fn extract_behaviors_from_answers(answers: &[Answer], _profile: &Profile) -> String {
  let api_answers: Vec<&Answer> = answers
    .iter()
    .filter(|answer| {
      contains_any_ignore_case(&answer.question_text, &["endpoint", "path", "method"])
    })
    .collect();

  if api_answers.is_empty() {
    r#"// Define API behaviors here
behaviors: {
  // Add endpoint definitions
}"#
      .to_string()
  } else {
    let behavior_lines = api_answers
      .iter()
      .map(|answer| {
        format!(
          "  // {}\n  // {}",
          answer.question_text,
          answer.response.trim()
        )
      })
      .join("\n");

    format!(
      r#"// API behaviors from interview
behaviors: {{
{}
}}"#,
      behavior_lines
    )
  }
}

/// Extract constraints from answers
#[must_use]
pub fn extract_constraints_from_answers(answers: &[Answer]) -> Vec<String> {
  answers
    .iter()
    .filter(|answer| {
      contains_any_ignore_case(&answer.question_text, &["constraint", "limit", "requirement"])
    })
    .filter_map(|answer| {
      let trimmed = answer.response.trim();
      if trimmed.is_empty() {
        None
      } else {
        Some(trimmed.to_string())
      }
    })
    .collect()
}

/// Extract security requirements from answers
#[must_use]
pub fn extract_security_requirements(answers: &[Answer]) -> String {
  let security_answers: Vec<&Answer> = answers
    .iter()
    .filter(|answer| {
      contains_any_ignore_case(&answer.question_text, &["auth", "security", "permission"])
    })
    .collect();

  if security_answers.is_empty() {
    r#"security: {
  authentication: "todo"
  authorization: "todo"
}"#
      .to_string()
  } else {
    let security_lines = security_answers
      .iter()
      .map(|answer| {
        format!(
          "  // {}\n  requirement: \"{}\"",
          answer.question_text,
          answer.response.trim()
        )
      })
      .join("\n");

    format!(
      r#"security: {{
{}
}}"#,
      security_lines
    )
  }
}

/// Extract non-functional requirements (SLA, scale, monitoring)
#[must_use]
pub fn extract_non_functional_requirements(answers: &[Answer]) -> Vec<String> {
  answers
    .iter()
    .filter(|answer| {
      contains_any_ignore_case(
        &answer.question_text,
        &["sla", "scale", "performance", "monitoring", "latency"],
      )
    })
    .filter_map(|answer| {
      let trimmed = answer.response.trim();
      if trimmed.is_empty() {
        None
      } else {
        Some(trimmed.to_string())
      }
    })
    .collect()
}

/// Build the main body of the spec
fn build_spec_body(
  features: &[String],
  behaviors: &str,
  constraints: &[String],
  security: &str,
  non_functional: &[String],
) -> String {
  let features_section = if features.is_empty() {
    r#"// Features
features: {
  // Add feature definitions
}"#
      .to_string()
  } else {
    let feature_lines = features
      .iter()
      .map(|feature| format!("  \"{feature}\": true"))
      .join("\n");

    format!(
      r#"// Features extracted from interview
features: {{
{}
}}"#,
      feature_lines
    )
  };

  let constraints_section = if constraints.is_empty() {
    String::new()
  } else {
    let constraint_lines = constraints
      .iter()
      .map(|constraint| format!("  // {constraint}"))
      .join("\n");

    format!(
      "\n\n// Constraints and requirements\nconstraints: {{\n{constraint_lines}\n}}"
    )
  };

  let non_functional_section = if non_functional.is_empty() {
    String::new()
  } else {
    let nf_lines = non_functional
      .iter()
      .map(|requirement| format!("  // {requirement}"))
      .join("\n");

    format!(
      "\n\n// Non-functional requirements\nnonFunctional: {{\n{nf_lines}\n}}"
    )
  };

  format!(
    "{}\n\n{}\n\n{}{}{}",
    features_section, behaviors, security, constraints_section, non_functional_section
  )
}

/// Create a test spec with N behaviors - pure functional composition
#[must_use]
pub fn create_test_spec(behavior_count: usize) -> Spec {
  let behaviors: Vec<Behavior> = (1..=behavior_count)
    .filter_map(|i| Behavior::new(format!("b{i}")).ok())
    .collect();

  Spec {
    name: "test".to_string(),
    description: "test".to_string(),
    features: vec![Feature {
      name: "test-feature".to_string(),
      description: "test".to_string(),
      behaviors,
      depends_on: Vec::new(),
    }],
    invariants: Vec::new(),
    anti_patterns: Vec::new(),
    ai_hints: AIHints {
      implementation: ImplementationHints {
        suggested_stack: Vec::new(),
      },
      entities: std::collections::HashMap::new(),
      security: SecurityHints {
        password_hashing: String::new(),
        jwt_algorithm: String::new(),
        jwt_expiry: String::new(),
        rate_limiting: String::new(),
      },
      pitfalls: Vec::new(),
    },
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::intent::interview::types::Perspective;

  fn make_test_answer(question_text: &str, response: &str) -> Answer {
    Answer {
      question_id: "q1".to_string(),
      question_text: question_text.to_string(),
      perspective: Perspective::User,
      round: 1,
      response: response.to_string(),
      extracted: std::collections::HashMap::new(),
      confidence: 1.0,
      notes: String::new(),
      timestamp: String::new(),
    }
  }

  fn make_test_session(answers: Vec<Answer>) -> InterviewSession {
    InterviewSession {
      id: "test".to_string(),
      profile: Profile::Api,
      created_at: "2024-01-01".to_string(),
      updated_at: "2024-01-01".to_string(),
      completed_at: None,
      stage: crate::intent::interview::types::InterviewStage::Complete,
      rounds_completed: 1,
      answers,
      gaps: Vec::new(),
      conflicts: Vec::new(),
      raw_notes: String::new(),
      current_phase: 1,
      completed_phases: Vec::new(),
    }
  }

  #[test]
  fn test_extract_features_from_answers() {
    let answers = vec![
      make_test_answer("What features do you need?", "User authentication"),
      make_test_answer("What capability?", "Data export"),
      make_test_answer("Unrelated question", "Unrelated answer"),
    ];

    let features = extract_features_from_answers(&answers);
    assert_eq!(features.len(), 2);
    assert!(features.contains(&"User authentication".to_string()));
    assert!(features.contains(&"Data export".to_string()));
  }

  #[test]
  fn test_extract_features_from_answers_empty_response() {
    let answers = vec![
      make_test_answer("What features?", ""),
      make_test_answer("What features?", "   "),
    ];

    let features = extract_features_from_answers(&answers);
    assert!(features.is_empty());
  }

  #[test]
  fn test_extract_constraints_from_answers() {
    let answers = vec![
      make_test_answer("What constraints apply?", "Must be fast"),
      make_test_answer("Any limits?", "Max 100 users"),
      make_test_answer("Requirements?", "Must be secure"),
    ];

    let constraints = extract_constraints_from_answers(&answers);
    assert_eq!(constraints.len(), 3);
  }

  #[test]
  fn test_extract_security_requirements_empty() {
    let answers = vec![make_test_answer("What is your name?", "John")];

    let security = extract_security_requirements(&answers);
    assert!(security.contains("authentication: \"todo\""));
  }

  #[test]
  fn test_extract_security_requirements_with_answers() {
    let answers = vec![make_test_answer(
      "What authentication method?",
      "OAuth2",
    )];

    let security = extract_security_requirements(&answers);
    assert!(security.contains("requirement: \"OAuth2\""));
  }

  #[test]
  fn test_extract_non_functional_requirements() {
    let answers = vec![
      make_test_answer("What SLA requirements?", "99.9% uptime"),
      make_test_answer("Performance targets?", "Response under 100ms"),
      make_test_answer("Monitoring needs?", "Prometheus metrics"),
    ];

    let nf = extract_non_functional_requirements(&answers);
    assert_eq!(nf.len(), 3);
  }

  #[test]
  fn test_build_spec_from_session() {
    let answers = vec![
      make_test_answer("What features?", "Auth"),
      make_test_answer("What endpoint?", "/login"),
      make_test_answer("Security?", "JWT"),
    ];

    let session = make_test_session(answers);
    let spec = build_spec_from_session(&session);

    assert!(spec.contains("package api"));
    assert!(spec.contains("features:"));
    assert!(spec.contains("behaviors:"));
    assert!(spec.contains("security:"));
  }

  #[test]
  fn test_create_test_spec() {
    let spec = create_test_spec(5);
    assert_eq!(spec.name, "test");
    assert_eq!(spec.features.len(), 1);
    assert_eq!(spec.features[0].behaviors.len(), 5);
  }

  #[test]
  fn test_create_test_spec_zero_behaviors() {
    let spec = create_test_spec(0);
    assert_eq!(spec.features[0].behaviors.len(), 0);
  }

  #[test]
  fn test_extract_behaviors_from_answers_empty() {
    let answers = vec![make_test_answer("What is your name?", "John")];

    let behaviors = extract_behaviors_from_answers(&answers, &Profile::Api);
    assert!(behaviors.contains("// Define API behaviors here"));
  }

  #[test]
  fn test_extract_behaviors_from_answers_with_endpoints() {
    let answers = vec![
      make_test_answer("What endpoints?", "GET /users"),
      make_test_answer("What paths?", "/api/v1"),
    ];

    let behaviors = extract_behaviors_from_answers(&answers, &Profile::Api);
    assert!(behaviors.contains("// API behaviors from interview"));
  }
}

//! Vision Document Generator
//!
//! Generates comprehensive vision documents from specifications.
//!
//! Ported from intent-cli/src/intent/vision_document.gleam

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use itertools::Itertools;

use crate::intent::types::{Behavior, Feature, Spec};

/// Generate a vision document from a spec
#[must_use]
pub fn generate_vision_document(spec: &Spec) -> String {
  let title = format!("# Vision: {}\n\n", spec.name);
  let overview = generate_overview(spec);
  let features = generate_features(spec);
  let invariants = generate_invariants(spec);
  let anti_patterns = generate_anti_patterns(spec);
  let technical_considerations = generate_technical_considerations(spec);

  format!("{title}{overview}\n{features}\n{invariants}\n{anti_patterns}{technical_considerations}")
}

fn generate_overview(spec: &Spec) -> String {
  format!("## Overview\n\n## Description\n\n{}\n\n", spec.description)
}

fn generate_features(spec: &Spec) -> String {
  let features_content = spec
    .features
    .iter()
    .map(generate_feature_section)
    .join("\n\n");

  if features_content.is_empty() {
    String::new()
  } else {
    format!("## Features\n\n{features_content}")
  }
}

fn generate_feature_section(feature: &Feature) -> String {
  let header = format!("### {}\n\n", feature.name);
  let description = format!("{}\n\n", feature.description);
  let behaviors = generate_behaviors(&feature.behaviors);

  format!("{header}{description}{behaviors}")
}

fn generate_behaviors(behaviors: &[Behavior]) -> String {
  if behaviors.is_empty() {
    return String::new();
  }

  let behavior_list = behaviors.iter().map(generate_behavior_summary).join("\n");

  format!("#### Behaviors\n\n{behavior_list}\n")
}

fn generate_behavior_summary(behavior: &Behavior) -> String {
  let name = format!("**{}**", behavior.name);
  let desc = if behavior.description.is_empty() {
    String::new()
  } else {
    format!(": {}", behavior.description)
  };

  let preconditions = if behavior.preconditions.is_empty() {
    String::new()
  } else {
    let pre = behavior.preconditions.join(", ");
    format!("\n  - Preconditions: {pre}")
  };

  let postconditions = if behavior.postconditions.is_empty() {
    String::new()
  } else {
    let post = behavior.postconditions.join(", ");
    format!("\n  - Postconditions: {post}")
  };

  let verification = if behavior.verifications.is_empty() {
    String::new()
  } else {
    let verifs: Vec<String> = behavior
      .verifications
      .iter()
      .filter(|v| !v.description.is_empty())
      .map(|v| v.description.clone())
      .collect();
    if verifs.is_empty() {
      String::new()
    } else {
      let verif_str = verifs.join(", ");
      format!("\n  - Verification: {verif_str}")
    }
  };

  format!("- {name}{desc}{preconditions}{postconditions}{verification}")
}

fn generate_invariants(spec: &Spec) -> String {
  if spec.invariants.is_empty() {
    return String::new();
  }

  let invariant_items = spec
    .invariants
    .iter()
    .map(|invariant| {
      let name = format!("### {}", invariant.name);
      let description = invariant.description.clone();
      let criteria = if invariant.criteria.is_empty() {
        String::new()
      } else {
        let criteria_list = invariant
          .criteria
          .iter()
          .map(|c| format!("- {c}"))
          .join("\n");
        format!("\n\n**Criteria:**\n{criteria_list}")
      };

      format!("{name}\n\n{description}{criteria}")
    })
    .join("\n\n");

  format!("## Global Invariants\n\n{invariant_items}\n")
}

fn generate_anti_patterns(spec: &Spec) -> String {
  if spec.anti_patterns.is_empty() {
    return String::new();
  }

  let pattern_items = spec
    .anti_patterns
    .iter()
    .map(|pattern| {
      let name = format!("### {}", pattern.name);
      let description = pattern.description.clone();

      let why = if pattern.why_avoid.is_empty() {
        String::new()
      } else {
        format!("\n\n**Why Avoid:** {}", pattern.why_avoid)
      };

      let alternative = if pattern.alternative.is_empty() {
        String::new()
      } else {
        format!("\n\n**Alternative:** {}", pattern.alternative)
      };

      format!("{name}\n\n{description}{why}{alternative}")
    })
    .join("\n\n");

  format!("## Anti-Patterns\n\n{pattern_items}\n")
}

fn generate_technical_considerations(spec: &Spec) -> String {
  let hints = &spec.ai_hints;
  let architecture = section("Architecture", &hints.implementation.architecture);
  let performance = section("Performance Notes", &hints.implementation.performance_notes);
  let error_handling = section("Error Handling", &hints.implementation.error_handling);
  let entities = entities_section(spec);
  let security = security_section(spec);
  let libraries = libraries_section(spec);

  format!(
    "## Technical Considerations\n\n{architecture}{performance}{error_handling}{entities}{security}{libraries}"
  )
}

fn section(title: &str, content: &str) -> String {
  if content.is_empty() {
    String::new()
  } else {
    format!("### {title}\n\n{content}\n\n")
  }
}

fn entities_section(spec: &Spec) -> String {
  if spec.ai_hints.entities.is_empty() {
    return String::new();
  }

  let entity_list = spec
    .ai_hints
    .entities
    .iter()
    .map(|entity| {
      let fields = if entity.fields.is_empty() {
        String::new()
      } else {
        format!(
          "\n\n**Fields:**\n{}",
          entity
            .fields
            .iter()
            .map(|field| format!("- {field}"))
            .join("\n")
        )
      };

      let relationships = if entity.relationships.is_empty() {
        String::new()
      } else {
        format!(
          "\n\n**Relationships:**\n{}",
          entity
            .relationships
            .iter()
            .map(|relationship| format!("- {relationship}"))
            .join("\n")
        )
      };

      format!(
        "#### {}\n\n{}{}{}",
        entity.name, entity.description, fields, relationships
      )
    })
    .join("\n\n");

  format!("### Data Entities\n\n{entity_list}\n\n")
}

fn security_section(spec: &Spec) -> String {
  let security = &spec.ai_hints.security;
  let lines = [
    (!security.password_hashing.is_empty())
      .then(|| format!("- **Password Hashing:** {}", security.password_hashing)),
    (!security.jwt_algorithm.is_empty())
      .then(|| format!("- **JWT Algorithm:** {}", security.jwt_algorithm)),
    (!security.jwt_expiry.is_empty()).then(|| format!("- **JWT Expiry:** {}", security.jwt_expiry)),
    (!security.rate_limiting.is_empty())
      .then(|| format!("- **Rate Limiting:** {}", security.rate_limiting)),
  ]
  .into_iter()
  .flatten()
  .collect::<Vec<_>>();

  if lines.is_empty() {
    String::new()
  } else {
    format!("### Security Considerations\n\n{}\n\n", lines.join("\n"))
  }
}

fn libraries_section(spec: &Spec) -> String {
  if spec.ai_hints.preferred_libraries.is_empty() {
    return String::new();
  }

  let libraries = spec
    .ai_hints
    .preferred_libraries
    .iter()
    .map(|library| format!("- {library}"))
    .join("\n");

  format!("### Preferred Libraries\n\n{libraries}\n")
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::intent::types::{
    AIHints, AntiPattern, Behavior, Feature, ImplementationHints, Invariant, SecurityHints,
  };

  fn make_test_spec() -> Spec {
    Spec {
      name: "Test API".to_string(),
      description: "A test API for testing".to_string(),
      audience: "developers".to_string(),
      version: "1.0.0".to_string(),
      success_criteria: vec![],
      features: vec![Feature {
        name: "Authentication".to_string(),
        description: "User authentication feature".to_string(),
        behaviors: vec![Behavior {
          name: "login".to_string(),
          intent: "User can log in".to_string(),
          description: "User can log in".to_string(),
          notes: String::new(),
          requires: Vec::new(),
          tags: Vec::new(),
          verifications: Vec::new(),
          preconditions: vec!["User exists".to_string()],
          postconditions: vec!["Session created".to_string()],
        }],
        depends_on: Vec::new(),
      }],
      invariants: vec![Invariant {
        name: "Session validity".to_string(),
        description: "Sessions must be valid".to_string(),
        criteria: vec!["session.expired == false".to_string()],
      }],
      anti_patterns: vec![AntiPattern {
        name: "Plaintext passwords".to_string(),
        description: "Do not store passwords in plaintext".to_string(),
        bad_example: serde_json::Value::Null,
        good_example: serde_json::Value::Null,
        why_avoid: "Security risk".to_string(),
        alternative: "Use bcrypt or argon2".to_string(),
      }],
      ai_hints: AIHints {
        implementation: ImplementationHints {
          architecture: "Layered architecture".to_string(),
          performance_notes: "Cache frequently accessed data".to_string(),
          error_handling: "Use Result types".to_string(),
          suggested_stack: Vec::new(),
          key_components: Vec::new(),
        },
        entities: vec![],
        security: SecurityHints {
          password_hashing: "bcrypt".to_string(),
          jwt_algorithm: "RS256".to_string(),
          jwt_expiry: "24h".to_string(),
          rate_limiting: "100 req/min".to_string(),
        },
        preferred_libraries: vec!["Rust".to_string()],
        style_hints: Vec::new(),
        pitfalls: Vec::new(),
      },
    }
  }

  #[test]
  fn test_generate_vision_document_includes_title() {
    let spec = make_test_spec();
    let doc = generate_vision_document(&spec);
    assert!(doc.contains("# Vision: Test API"));
  }

  #[test]
  fn test_generate_vision_document_includes_description() {
    let spec = make_test_spec();
    let doc = generate_vision_document(&spec);
    assert!(doc.contains("## Description"));
    assert!(doc.contains("A test API for testing"));
  }

  #[test]
  fn test_generate_vision_document_includes_features() {
    let spec = make_test_spec();
    let doc = generate_vision_document(&spec);
    assert!(doc.contains("## Features"));
    assert!(doc.contains("### Authentication"));
  }

  #[test]
  fn test_generate_vision_document_includes_behaviors() {
    let spec = make_test_spec();
    let doc = generate_vision_document(&spec);
    assert!(doc.contains("#### Behaviors"));
    assert!(doc.contains("**login**"));
  }

  #[test]
  fn test_generate_vision_document_includes_invariants() {
    let spec = make_test_spec();
    let doc = generate_vision_document(&spec);
    assert!(doc.contains("## Global Invariants"));
    assert!(doc.contains("### Session validity"));
  }

  #[test]
  fn test_generate_vision_document_includes_anti_patterns() {
    let spec = make_test_spec();
    let doc = generate_vision_document(&spec);
    assert!(doc.contains("## Anti-Patterns"));
    assert!(doc.contains("### Plaintext passwords"));
  }

  #[test]
  fn test_generate_vision_document_includes_technical_considerations() {
    let spec = make_test_spec();
    let doc = generate_vision_document(&spec);
    assert!(doc.contains("## Technical Considerations"));
    assert!(doc.contains("### Architecture"));
    assert!(doc.contains("Layered architecture"));
  }

  #[test]
  fn test_generate_overview() {
    let spec = make_test_spec();
    let overview = generate_overview(&spec);
    assert!(overview.contains("## Overview"));
    assert!(overview.contains("## Description"));
  }

  #[test]
  fn test_generate_features_empty() {
    let mut spec = make_test_spec();
    spec.features = Vec::new();
    let features = generate_features(&spec);
    assert!(features.is_empty());
  }

  #[test]
  fn test_generate_invariants_empty() {
    let mut spec = make_test_spec();
    spec.invariants = Vec::new();
    let invariants = generate_invariants(&spec);
    assert!(invariants.is_empty());
  }

  #[test]
  fn test_generate_anti_patterns_empty() {
    let mut spec = make_test_spec();
    spec.anti_patterns = Vec::new();
    let anti_patterns = generate_anti_patterns(&spec);
    assert!(anti_patterns.is_empty());
  }
}

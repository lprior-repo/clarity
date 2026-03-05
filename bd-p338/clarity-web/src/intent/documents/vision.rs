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

  format!(
    "{}{}\n{}\n{}\n{}{}",
    title, overview, features, invariants, anti_patterns, technical_considerations
  )
}

fn generate_overview(spec: &Spec) -> String {
  format!(
    "## Overview\n\n## Description\n\n{}\n\n",
    spec.description
  )
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
  let desc = if behavior.intent.is_empty() {
    String::new()
  } else {
    format!(": {}", behavior.intent)
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
    behavior
      .verifications
      .iter()
      .filter(|v| !v.description.is_empty())
      .map(|v| format!("\n  - Verification: {}", v.description))
      .collect()
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
      let constraint = if invariant.constraint.is_empty() {
        String::new()
      } else {
        format!("\n\n**Constraint:** {}", invariant.constraint)
      };

      format!("{name}\n\n{description}{constraint}")
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

  let architecture = if hints.implementation.architecture.is_empty() {
    String::new()
  } else {
    format!(
      "### Architecture\n\n{}\n\n",
      hints.implementation.architecture
    )
  };

  let performance = if hints.implementation.performance_notes.is_empty() {
    String::new()
  } else {
    format!(
      "### Performance Notes\n\n{}\n\n",
      hints.implementation.performance_notes
    )
  };

  let error_handling = if hints.implementation.error_handling.is_empty() {
    String::new()
  } else {
    format!(
      "### Error Handling\n\n{}\n\n",
      hints.implementation.error_handling
    )
  };

  let entities = if hints.entities.is_empty() {
    String::new()
  } else {
    let entity_list = hints
      .entities
      .iter()
      .map(|entity| {
        let fields = if entity.fields.is_empty() {
          String::new()
        } else {
          let field_list = entity.fields.iter().map(|f| format!("- {f}")).join("\n");
          format!("\n\n**Fields:**\n{field_list}")
        };

        let relationships = if entity.relationships.is_empty() {
          String::new()
        } else {
          let rel_list = entity.relationships.iter().map(|r| format!("- {r}")).join("\n");
          format!("\n\n**Relationships:**\n{rel_list}")
        };

        format!("#### {}\n\n{}{}{}", entity.name, entity.description, fields, relationships)
      })
      .join("\n\n");

    format!("### Data Entities\n\n{entity_list}\n\n")
  };

  let security = if hints.security.authentication.is_empty()
    && hints.security.authorization.is_empty()
    && hints.security.concerns.is_empty()
  {
    String::new()
  } else {
    let auth = if hints.security.authentication.is_empty() {
      String::new()
    } else {
      format!("- **Authentication:** {}\n", hints.security.authentication)
    };

    let authz = if hints.security.authorization.is_empty() {
      String::new()
    } else {
      format!("- **Authorization:** {}\n", hints.security.authorization)
    };

    let concerns = if hints.security.concerns.is_empty() {
      String::new()
    } else {
      let concern_list = hints.security.concerns.iter().map(|c| format!("  - {c}")).join("\n");
      format!("- **Concerns:**\n{concern_list}\n")
    };

    format!("### Security Considerations\n\n{auth}{authz}{concerns}\n")
  };

  let libraries = if hints.preferred_libraries.is_empty() {
    String::new()
  } else {
    let lib_list = hints
      .preferred_libraries
      .iter()
      .map(|l| format!("- {l}"))
      .join("\n");

    format!("### Preferred Libraries\n\n{lib_list}\n")
  };

  format!(
    "## Technical Considerations\n\n{architecture}{performance}{error_handling}{entities}{security}{libraries}"
  )
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::intent::types::{
    AIHints, AntiPattern, Behavior, EntityHint, Feature, ImplementationHints, Invariant,
    SecurityHints,
  };

  fn make_test_spec() -> Spec {
    Spec {
      name: "Test API".to_string(),
      description: "A test API for testing".to_string(),
      features: vec![Feature {
        name: "Authentication".to_string(),
        description: "User authentication feature".to_string(),
        behaviors: vec![Behavior {
          name: "login".to_string(),
          intent: "User can log in".to_string(),
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
        constraint: "session.expired == false".to_string(),
      }],
      anti_patterns: vec![AntiPattern {
        name: "Plaintext passwords".to_string(),
        description: "Do not store passwords in plaintext".to_string(),
        why_avoid: "Security risk".to_string(),
        alternative: "Use bcrypt or argon2".to_string(),
      }],
      ai_hints: AIHints {
        implementation: ImplementationHints {
          architecture: "Layered architecture".to_string(),
          performance_notes: "Cache frequently accessed data".to_string(),
          error_handling: "Use Result types".to_string(),
        },
        entities: vec![],
        security: SecurityHints {
          authentication: "JWT".to_string(),
          authorization: "RBAC".to_string(),
          data_sensitivity: String::new(),
          concerns: vec!["SQL injection".to_string()],
        },
        preferred_libraries: vec!["Rust".to_string()],
        style_hints: Vec::new(),
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

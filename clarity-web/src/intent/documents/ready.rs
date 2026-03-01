//! Ready Document Generator
//!
//! Generates implementation-ready documentation from specs.
//!
//! Ported from intent-cli/src/intent/ready_document.gleam

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use itertools::Itertools;

use crate::intent::types::{Behavior, Spec};

/// Generate a ready document from a spec
#[must_use]
pub fn generate_ready_document(spec: &Spec) -> String {
  let sections = [
    generate_header(spec),
    generate_overview(spec),
    generate_features(spec),
    generate_behaviors(spec),
    generate_invariants(spec),
    generate_verification_criteria(),
    generate_anti_patterns(spec),
    generate_implementation_hints(spec),
    generate_security_guidelines(spec),
  ];

  sections.join("\n\n")
}

fn generate_header(spec: &Spec) -> String {
  format!(
    "# Ready Document: {}\nGenerated: {}",
    spec.name,
    get_current_timestamp()
  )
}

fn generate_overview(spec: &Spec) -> String {
  format!("## Overview\n\n**Description:** {}\n\n", spec.description)
}

fn generate_features(spec: &Spec) -> String {
  if spec.features.is_empty() {
    return String::new();
  }

  let features_list = spec
    .features
    .iter()
    .map(|feature| format!("### {}\n\n{}", feature.name, feature.description))
    .join("\n\n");

  format!("## Features\n\n{features_list}")
}

fn generate_behaviors(spec: &Spec) -> String {
  if spec.features.is_empty() {
    return String::new();
  }

  let behaviors_by_feature = spec
    .features
    .iter()
    .map(|feature| {
      if feature.behaviors.is_empty() {
        String::new()
      } else {
        let behaviors = feature
          .behaviors
          .iter()
          .map(generate_behavior_details)
          .join("\n\n");

        if behaviors.is_empty() {
          String::new()
        } else {
          format!("### {}\n\n{}", feature.name, behaviors)
        }
      }
    })
    .filter(|s| !s.is_empty())
    .join("\n\n");

  if behaviors_by_feature.is_empty() {
    String::new()
  } else {
    format!("## Behaviors\n\n{behaviors_by_feature}")
  }
}

fn generate_behavior_details(behavior: &Behavior) -> String {
  let header = format!(
    "#### {}\n\n**Description:** {}\n",
    behavior.name, behavior.description
  );

  let preconditions = if behavior.preconditions.is_empty() {
    String::new()
  } else {
    let pre_list = behavior
      .preconditions
      .iter()
      .map(|p| format!("- {p}"))
      .join("\n");
    format!("\n**Preconditions:**\n{pre_list}")
  };

  let postconditions = if behavior.postconditions.is_empty() {
    String::new()
  } else {
    let post_list = behavior
      .postconditions
      .iter()
      .map(|p| format!("- {p}"))
      .join("\n");
    format!("\n**Postconditions:**\n{post_list}")
  };

  let verification = if behavior.verifications.is_empty() {
    String::new()
  } else {
    // Use new criteria/example fields, fall back to deprecated description for backward compatibility
    #[allow(deprecated)]
    let verifs: Vec<String> = behavior
      .verifications
      .iter()
      .filter(|v| !v.criteria.is_empty() || !v.example.is_empty() || !v.description.is_empty())
      .map(|v| {
        if !v.criteria.is_empty() {
          format!("Criteria: {}", v.criteria.join(", "))
        } else if !v.example.is_empty() {
          format!("Example: {}", v.example)
        } else {
          v.description.clone()
        }
      })
      .collect();
    if verifs.is_empty() {
      String::new()
    } else {
      let verif_str = verifs.join(", ");
      format!("\n**Verification:** {verif_str}")
    }
  };

  format!("{header}{preconditions}{postconditions}{verification}")
}

fn generate_invariants(spec: &Spec) -> String {
  if spec.invariants.is_empty() {
    return String::new();
  }

  let invariants = spec
    .invariants
    .iter()
    .map(|invariant| {
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

      format!(
        "### {}\n\n{}{}",
        invariant.name, invariant.description, criteria
      )
    })
    .join("\n\n");

  format!("## Invariants\n\nThese global rules apply to all behaviors:\n\n{invariants}")
}

fn generate_verification_criteria() -> String {
  "## Verification Criteria

Each behavior should be verified against:

1. **Preconditions**: Verify all preconditions are met before execution
2. **Postconditions**: Verify all postconditions are true after execution
3. **Verification**: Execute all verification criteria specified
4. **Error handling**: Verify error cases are handled correctly
5. **Edge cases**: Test boundary conditions and unusual inputs

### Automated Testing

All behaviors should have automated tests that:
- Verify preconditions are satisfied
- Execute the behavior
- Validate postconditions
- Check all verification criteria
- Test error conditions explicitly"
    .to_string()
}

fn generate_anti_patterns(spec: &Spec) -> String {
  if spec.anti_patterns.is_empty() {
    return String::new();
  }

  let patterns = spec
    .anti_patterns
    .iter()
    .map(|ap| {
      let why = if ap.why_avoid.is_empty() {
        String::new()
      } else {
        format!("\n\n**Why Avoid:** {}", ap.why_avoid)
      };

      let alternative = if ap.alternative.is_empty() {
        String::new()
      } else {
        format!("\n\n**Alternative:** {}", ap.alternative)
      };

      format!(
        "### {}\n\n{}{}{}",
        ap.name, ap.description, why, alternative
      )
    })
    .join("\n\n");

  format!("## Anti-Patterns to Avoid\n\n{patterns}")
}

fn generate_implementation_hints(spec: &Spec) -> String {
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

  let entities = generate_entity_hints(&hints.entities);
  let libraries = generate_library_hints(&hints.preferred_libraries);

  format!(
    "## Implementation Hints\n\n{architecture}{performance}{error_handling}{entities}{libraries}"
  )
}

fn generate_entity_hints(entities: &[crate::intent::types::EntityHint]) -> String {
  if entities.is_empty() {
    return String::new();
  }

  let entity_list = entities
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
        let rel_list = entity
          .relationships
          .iter()
          .map(|r| format!("- {r}"))
          .join("\n");
        format!("\n\n**Relationships:**\n{rel_list}")
      };

      format!(
        "**{}**\n\n{}{}{}",
        entity.name, entity.description, fields, relationships
      )
    })
    .join("\n\n");

  format!("### Entity Models\n\n{entity_list}\n\n")
}

fn generate_library_hints(libraries: &[String]) -> String {
  if libraries.is_empty() {
    return String::new();
  }

  let lib_list = libraries.iter().map(|l| format!("- {l}")).join("\n");

  format!("### Preferred Libraries\n\n{lib_list}")
}

fn generate_security_guidelines(spec: &Spec) -> String {
  let security = &spec.ai_hints.security;

  let password_hashing = if security.password_hashing.is_empty() {
    String::new()
  } else {
    format!("### Password Hashing\n\n{}\n\n", security.password_hashing)
  };

  let jwt_algorithm = if security.jwt_algorithm.is_empty() {
    String::new()
  } else {
    format!("### JWT Algorithm\n\n{}\n\n", security.jwt_algorithm)
  };

  let jwt_expiry = if security.jwt_expiry.is_empty() {
    String::new()
  } else {
    format!("### JWT Expiry\n\n{}\n\n", security.jwt_expiry)
  };

  let rate_limiting = if security.rate_limiting.is_empty() {
    String::new()
  } else {
    format!("### Rate Limiting\n\n{}\n\n", security.rate_limiting)
  };

  if password_hashing.is_empty()
    && jwt_algorithm.is_empty()
    && jwt_expiry.is_empty()
    && rate_limiting.is_empty()
  {
    String::new()
  } else {
    format!(
      "## Security Guidelines\n\n{password_hashing}{jwt_algorithm}{jwt_expiry}{rate_limiting}"
    )
  }
}

/// Get current timestamp in ISO 8601 format
fn get_current_timestamp() -> String {
  // In production, use a proper datetime library like chrono
  // For now, return a placeholder
  "2024-01-15T10:30:00Z".to_string()
}

#[cfg(test)]
mod tests {
  #![allow(deprecated)]

  use super::*;
  use crate::intent::types::{
    AIHints, AntiPattern, Behavior, Feature, ImplementationHints, Invariant, SecurityHints,
    Verification,
  };

  fn make_test_spec() -> Spec {
    Spec {
      name: "Test API".to_string(),
      description: "A test API".to_string(),
      audience: "developers".to_string(),
      version: "1.0.0".to_string(),
      success_criteria: vec![],
      features: vec![Feature {
        name: "Auth".to_string(),
        description: "Authentication".to_string(),
        behaviors: vec![Behavior {
          name: "login".to_string(),
          intent: "User logs in".to_string(),
          description: "User logs in".to_string(),
          notes: String::new(),
          requires: Vec::new(),
          tags: Vec::new(),
          verifications: vec![Verification {
            verification_type: "unit_test".to_string(),
            description: "Test login returns token".to_string(),
            example: String::new(),
            criteria: Vec::new(),
          }],
          preconditions: vec!["User exists".to_string()],
          postconditions: vec!["Session created".to_string()],
        }],
        depends_on: Vec::new(),
      }],
      invariants: vec![Invariant {
        name: "Security".to_string(),
        description: "Must be secure".to_string(),
        criteria: vec!["HTTPS required".to_string()],
      }],
      anti_patterns: vec![AntiPattern {
        name: "Bad Pattern".to_string(),
        description: "Don't do this".to_string(),
        bad_example: serde_json::Value::Null,
        good_example: serde_json::Value::Null,
        why_avoid: "Security risk".to_string(),
        alternative: "Use proper hashing".to_string(),
      }],
      ai_hints: AIHints {
        implementation: ImplementationHints {
          architecture: "Layered".to_string(),
          performance_notes: String::new(),
          error_handling: "Result types".to_string(),
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
  fn test_generate_ready_document_includes_header() {
    let spec = make_test_spec();
    let doc = generate_ready_document(&spec);
    assert!(doc.contains("# Ready Document: Test API"));
  }

  #[test]
  fn test_generate_ready_document_includes_overview() {
    let spec = make_test_spec();
    let doc = generate_ready_document(&spec);
    assert!(doc.contains("## Overview"));
    assert!(doc.contains("**Description:**"));
  }

  #[test]
  fn test_generate_ready_document_includes_features() {
    let spec = make_test_spec();
    let doc = generate_ready_document(&spec);
    assert!(doc.contains("## Features"));
    assert!(doc.contains("### Auth"));
  }

  #[test]
  fn test_generate_ready_document_includes_behaviors() {
    let spec = make_test_spec();
    let doc = generate_ready_document(&spec);
    assert!(doc.contains("## Behaviors"));
    assert!(doc.contains("#### login"));
    assert!(doc.contains("**Description:**"));
  }

  #[test]
  fn test_generate_ready_document_includes_preconditions() {
    let spec = make_test_spec();
    let doc = generate_ready_document(&spec);
    assert!(doc.contains("**Preconditions:**"));
    assert!(doc.contains("User exists"));
  }

  #[test]
  fn test_generate_ready_document_includes_postconditions() {
    let spec = make_test_spec();
    let doc = generate_ready_document(&spec);
    assert!(doc.contains("**Postconditions:**"));
    assert!(doc.contains("Session created"));
  }

  #[test]
  fn test_generate_ready_document_includes_verification() {
    let spec = make_test_spec();
    let doc = generate_ready_document(&spec);
    assert!(doc.contains("**Verification:**"));
    assert!(doc.contains("Test login returns token"));
  }

  #[test]
  fn test_generate_ready_document_includes_invariants() {
    let spec = make_test_spec();
    let doc = generate_ready_document(&spec);
    assert!(doc.contains("## Invariants"));
    assert!(doc.contains("### Security"));
  }

  #[test]
  fn test_generate_ready_document_includes_anti_patterns() {
    let spec = make_test_spec();
    let doc = generate_ready_document(&spec);
    assert!(doc.contains("## Anti-Patterns to Avoid"));
    assert!(doc.contains("### Bad Pattern"));
    assert!(doc.contains("**Why Avoid:**"));
    assert!(doc.contains("**Alternative:**"));
  }

  #[test]
  fn test_generate_ready_document_includes_implementation_hints() {
    let spec = make_test_spec();
    let doc = generate_ready_document(&spec);
    assert!(doc.contains("## Implementation Hints"));
    assert!(doc.contains("### Architecture"));
    assert!(doc.contains("Layered"));
  }

  #[test]
  fn test_generate_ready_document_includes_security() {
    let spec = make_test_spec();
    let doc = generate_ready_document(&spec);
    assert!(doc.contains("## Security Guidelines"));
    assert!(doc.contains("### Password Hashing"));
    assert!(doc.contains("bcrypt"));
  }

  #[test]
  fn test_generate_header() {
    let spec = make_test_spec();
    let header = generate_header(&spec);
    assert!(header.contains("# Ready Document: Test API"));
  }

  #[test]
  fn test_generate_verification_criteria() {
    let criteria = generate_verification_criteria();
    assert!(criteria.contains("## Verification Criteria"));
    assert!(criteria.contains("Preconditions"));
    assert!(criteria.contains("Postconditions"));
    assert!(criteria.contains("Automated Testing"));
  }
}

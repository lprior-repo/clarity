#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// `AIHints` - hints to guide AI code generation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AIHints {
  /// Implementation hints
  #[serde(default)]
  pub implementation: ImplementationHints,
  /// Entity hints for data modeling
  #[serde(default)]
  pub entities: Vec<EntityHint>,
  /// Security considerations
  #[serde(default)]
  pub security: SecurityHints,
  /// Preferred libraries or frameworks
  #[serde(default)]
  pub preferred_libraries: Vec<String>,
  /// Code style preferences
  #[serde(default)]
  pub style_hints: Vec<String>,
  /// Common pitfalls to avoid
  #[serde(default)]
  pub pitfalls: Vec<String>,
}

/// `ImplementationHints` - hints for implementation approach
///
/// This type aligns with the Gleam/CUE schema. Fields present in Gleam/CUE schema:
/// - `suggested_stack` - CUE: `suggested_stack?: [...string]`
/// - `architecture` - CUE: `architecture?: string`
/// - `key_components` - CUE: `key_components?: [...string]`
///
/// Rust-only extensions (not in Gleam/CUE):
/// - `performance_notes` - Performance considerations for document generation
/// - `error_handling` - Error handling approach for document generation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ImplementationHints {
  /// Suggested architecture pattern (e.g., "clean architecture", "microservices")
  /// Maps to CUE field: `architecture?: string`
  #[serde(default)]
  pub architecture: String,

  /// Performance considerations and optimization hints
  /// Rust-only extension for document generation (not in Gleam/CUE)
  #[serde(default)]
  pub performance_notes: String,

  /// Error handling approach (e.g., "Result types with ? operator", "custom error enums")
  /// Rust-only extension for document generation (not in Gleam/CUE)
  #[serde(default)]
  pub error_handling: String,

  /// Suggested tech stack (libraries, frameworks, tools)
  /// Maps to CUE field: `suggested_stack?: [...string]`
  #[serde(default)]
  pub suggested_stack: Vec<String>,

  /// Key components or modules in the system architecture
  /// Maps to CUE field: `key_components?: [...string]`
  #[serde(default)]
  pub key_components: Vec<String>,
}

/// `EntityHint` - hint for data entity modeling
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EntityHint {
  /// Entity name
  pub name: String,
  /// Entity description
  #[serde(default)]
  pub description: String,
  /// Suggested fields
  #[serde(default)]
  pub fields: Vec<String>,
  /// Relationships to other entities
  #[serde(default)]
  pub relationships: Vec<String>,
  /// Additional properties as key-value pairs
  #[serde(default)]
  pub dict: std::collections::HashMap<String, serde_json::Value>,
}

/// `SecurityHints` - security-related considerations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SecurityHints {
  /// Password hashing algorithm recommendation
  #[serde(default)]
  pub password_hashing: String,
  /// JWT algorithm recommendation
  #[serde(default)]
  pub jwt_algorithm: String,
  /// JWT expiry configuration
  #[serde(default)]
  pub jwt_expiry: String,
  /// Rate limiting configuration
  #[serde(default)]
  pub rate_limiting: String,
}

#[cfg(test)]
mod tests {
  use super::AIHints;
  use super::EntityHint;
  use serde_json::json;
  use std::collections::HashMap;

  #[test]
  fn test_ai_hints_default() {
    let hints = AIHints::default();
    assert!(hints.entities.is_empty());
    assert!(hints.preferred_libraries.is_empty());
    assert!(hints.style_hints.is_empty());
  }

  #[test]
  fn test_entity_hint_serialization_roundtrip() {
    let mut dict = HashMap::new();
    dict.insert("key1".to_string(), json!("value1"));
    dict.insert("key2".to_string(), json!(42));
    dict.insert("nested".to_string(), json!({"inner": "data"}));

    let original = EntityHint {
      name: "User".to_string(),
      description: "A user entity".to_string(),
      fields: vec!["id".to_string(), "email".to_string()],
      relationships: vec!["has_many Posts".to_string()],
      dict,
    };

    let json_str = serde_json::to_string(&original).expect("serialization should succeed");
    let deserialized: EntityHint =
      serde_json::from_str(&json_str).expect("deserialization should succeed");

    assert_eq!(original, deserialized);
  }

  #[test]
  fn test_entity_hint_empty_dict_serialization() {
    let original = EntityHint {
      name: "EmptyEntity".to_string(),
      description: String::new(),
      fields: vec![],
      relationships: vec![],
      dict: HashMap::new(),
    };

    let json_str = serde_json::to_string(&original).expect("serialization should succeed");
    let deserialized: EntityHint =
      serde_json::from_str(&json_str).expect("deserialization should succeed");

    assert_eq!(original, deserialized);
    assert!(deserialized.dict.is_empty());
  }

  #[test]
  fn test_entity_hint_dict_with_various_json_types() {
    let mut dict = HashMap::new();
    dict.insert("string_val".to_string(), json!("text"));
    dict.insert("number_val".to_string(), json!(123.45));
    dict.insert("bool_val".to_string(), json!(true));
    dict.insert("null_val".to_string(), json!(null));
    dict.insert("array_val".to_string(), json!([1, 2, 3]));
    dict.insert(
      "object_val".to_string(),
      json!({"nested": {"deep": "value"}}),
    );

    let original = EntityHint {
      name: "ComplexEntity".to_string(),
      description: "Entity with complex dict".to_string(),
      fields: vec![],
      relationships: vec![],
      dict,
    };

    let json_str = serde_json::to_string(&original).expect("serialization should succeed");
    let deserialized: EntityHint =
      serde_json::from_str(&json_str).expect("deserialization should succeed");

    assert_eq!(original, deserialized);
  }

  #[test]
  fn test_entity_hint_default() {
    let hint = EntityHint::default();
    assert!(hint.name.is_empty());
    assert!(hint.description.is_empty());
    assert!(hint.fields.is_empty());
    assert!(hint.relationships.is_empty());
    assert!(hint.dict.is_empty());
  }

  #[test]
  fn test_entity_hint_json_structure() {
    let mut dict = HashMap::new();
    dict.insert("custom_field".to_string(), json!("custom_value"));

    let hint = EntityHint {
      name: "TestEntity".to_string(),
      description: "Test description".to_string(),
      fields: vec!["field1".to_string()],
      relationships: vec!["relates_to Other".to_string()],
      dict,
    };

    let json_value = serde_json::to_value(&hint).expect("should serialize to json value");

    assert_eq!(json_value["name"], "TestEntity");
    assert_eq!(json_value["description"], "Test description");
    assert_eq!(json_value["fields"], json!(["field1"]));
    assert_eq!(json_value["relationships"], json!(["relates_to Other"]));
    assert_eq!(json_value["dict"]["custom_field"], "custom_value");
  }

  #[test]
  fn test_implementation_hints_default() {
    let hints = super::ImplementationHints::default();
    assert!(hints.architecture.is_empty());
    assert!(hints.performance_notes.is_empty());
    assert!(hints.error_handling.is_empty());
    assert!(hints.suggested_stack.is_empty());
    assert!(hints.key_components.is_empty());
  }

  #[test]
  fn test_implementation_hints_serialization_roundtrip() {
    let original = super::ImplementationHints {
      architecture: "microservices".to_string(),
      performance_notes: "Use caching".to_string(),
      error_handling: "Result types".to_string(),
      suggested_stack: vec!["Rust".to_string(), "Tokio".to_string()],
      key_components: vec!["API Gateway".to_string(), "Auth Service".to_string()],
    };

    let json_str = serde_json::to_string(&original).expect("serialization should succeed");
    let deserialized: super::ImplementationHints =
      serde_json::from_str(&json_str).expect("deserialization should succeed");

    assert_eq!(original, deserialized);
  }

  #[test]
  fn test_implementation_hints_json_structure() {
    let hints = super::ImplementationHints {
      architecture: "layered".to_string(),
      performance_notes: String::new(),
      error_handling: "anyhow".to_string(),
      suggested_stack: vec!["serde".to_string()],
      key_components: vec!["Router".to_string(), "Handler".to_string()],
    };

    let json_value = serde_json::to_value(&hints).expect("should serialize to json value");

    assert_eq!(json_value["architecture"], "layered");
    assert_eq!(json_value["performance_notes"], "");
    assert_eq!(json_value["error_handling"], "anyhow");
    assert_eq!(json_value["suggested_stack"], json!(["serde"]));
    assert_eq!(json_value["key_components"], json!(["Router", "Handler"]));
  }
}

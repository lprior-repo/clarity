#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::missing_const_for_fn)]

//! Schema Registry for type management
//!
//! Provides schema storage, retrieval, and validation functionality.
//! All functions return Result<T, E> - no unwraps, no panics.

use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

/// Unique identifier for a schema version
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SchemaId(String);

impl SchemaId {
  /// Create a new SchemaId
  ///
  /// # Errors
  ///
  /// Returns `SchemaRegistryError::InvalidId` if the ID is empty or contains invalid characters
  pub fn new(id: String) -> Result<Self, SchemaRegistryError> {
    if id.trim().is_empty() {
      return Err(SchemaRegistryError::InvalidId(
        "Schema ID cannot be empty".to_string(),
      ));
    }

    if !id.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
      return Err(SchemaRegistryError::InvalidId(format!(
        "Schema ID contains invalid characters: {id}"
      )));
    }

    Ok(Self(id))
  }

  /// Get the ID as a string slice
  #[must_use]
  pub const fn as_str(&self) -> &str {
    self.0.as_str()
  }
}

/// Schema version following semantic versioning
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SchemaVersion(String);

impl SchemaVersion {
  /// Create a new SchemaVersion
  ///
  /// # Errors
  ///
  /// Returns `SchemaRegistryError::InvalidVersion` if the version format is invalid
  pub fn new(version: String) -> Result<Self, SchemaRegistryError> {
    if version.trim().is_empty() {
      return Err(SchemaRegistryError::InvalidVersion(
        "Version cannot be empty".to_string(),
      ));
    }

    // Basic semver validation: must contain at least one dot
    if !version.contains('.') {
      return Err(SchemaRegistryError::InvalidVersion(format!(
        "Version must follow semver format (e.g., 1.0.0): {version}"
      )));
    }

    Ok(Self(version))
  }

  /// Get the version as a string slice
  #[must_use]
  pub const fn as_str(&self) -> &str {
    self.0.as_str()
  }
}

/// JSON schema with metadata
#[derive(Debug, Clone)]
pub struct Schema {
  /// Unique identifier for this schema
  pub id: SchemaId,
  /// Schema version
  pub version: SchemaVersion,
  /// Human-readable name
  pub name: String,
  /// Optional description
  pub description: Option<String>,
  /// JSON schema definition
  pub schema: serde_json::Value,
  /// Creation timestamp
  pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Schema registry for managing schemas
#[derive(Debug, Clone)]
pub struct SchemaRegistry {
  schemas: Arc<HashMap<(SchemaId, SchemaVersion), Schema>>,
}

impl Default for SchemaRegistry {
  fn default() -> Self {
    Self::new()
  }
}

impl SchemaRegistry {
  /// Create a new empty schema registry
  #[must_use]
  pub fn new() -> Self {
    Self {
      schemas: Arc::new(HashMap::new()),
    }
  }

  /// Register a new schema
  ///
  /// # Errors
  ///
  /// Returns `SchemaRegistryError::DuplicateSchema` if a schema with the same ID and version already exists
  pub fn register(&mut self, schema: Schema) -> Result<(), SchemaRegistryError> {
    let key = (schema.id.clone(), schema.version.clone());

    // Check for duplicate using Arc::make_mut to get mutable reference
    let schemas = Arc::make_mut(&mut self.schemas);

    if schemas.contains_key(&key) {
      return Err(SchemaRegistryError::DuplicateSchema {
        id: key.0.as_str().to_string(),
        version: key.1.as_str().to_string(),
      });
    }

    schemas.insert(key, schema);
    Ok(())
  }

  /// Get a schema by ID and version
  ///
  /// # Errors
  ///
  /// Returns `SchemaRegistryError::NotFound` if the schema doesn't exist
  pub fn get(&self, id: &SchemaId, version: &SchemaVersion) -> Result<Schema, SchemaRegistryError> {
    self
      .schemas
      .get(&(id.clone(), version.clone()))
      .cloned()
      .ok_or_else(|| SchemaRegistryError::NotFound {
        id: id.as_str().to_string(),
        version: version.as_str().to_string(),
      })
  }

  /// Get the latest version of a schema
  ///
  /// # Errors
  ///
  /// Returns `SchemaRegistryError::NotFound` if no versions of the schema exist
  pub fn get_latest(&self, id: &SchemaId) -> Result<Schema, SchemaRegistryError> {
    self
      .schemas
      .iter()
      .filter(|((schema_id, _), _)| schema_id == id)
      .max_by_key(|((_, version), _)| version)
      .map(|(_, schema)| schema.clone())
      .ok_or_else(|| SchemaRegistryError::NotFound {
        id: id.as_str().to_string(),
        version: "any".to_string(),
      })
  }

  /// List all schemas
  #[must_use]
  pub fn list_all(&self) -> Vec<Schema> {
    self.schemas.values().cloned().collect()
  }

  /// List all versions of a schema
  #[must_use]
  pub fn list_versions(&self, id: &SchemaId) -> Vec<Schema> {
    self
      .schemas
      .iter()
      .filter(|((schema_id, _), _)| schema_id == id)
      .map(|(_, schema)| schema.clone())
      .collect()
  }

  /// Validate JSON data against a schema
  ///
  /// # Errors
  ///
  /// Returns `SchemaRegistryError::NotFound` if the schema doesn't exist
  /// Returns `SchemaRegistryError::ValidationError` if validation fails
  pub fn validate(
    &self,
    id: &SchemaId,
    version: &SchemaVersion,
    data: &serde_json::Value,
  ) -> Result<(), SchemaRegistryError> {
    let schema = self.get(id, version)?;

    // Perform basic JSON schema validation
    // For now, we'll do a simplified check - full JSON schema validation
    // would require the `jsonschema` crate
    self.validate_against_schema(&schema.schema, data)
  }

  /// Internal validation logic
  fn validate_against_schema(
    &self,
    schema: &serde_json::Value,
    data: &serde_json::Value,
  ) -> Result<(), SchemaRegistryError> {
    // Get the type from schema
    let schema_type = schema.get("type").and_then(|v| v.as_str()).ok_or_else(|| {
      SchemaRegistryError::ValidationError {
        message: "Schema must have a 'type' field".to_string(),
        path: "".to_string(),
      }
    })?;

    // Perform type checking
    match schema_type {
      "object" => {
        if !data.is_object() {
          return Err(SchemaRegistryError::ValidationError {
            message: format!("Expected object, got {}", self.get_type_name(data)),
            path: "/".to_string(),
          });
        }

        // Check required properties
        if let Some(required) = schema.get("required").and_then(|v| v.as_array()) {
          for prop in required {
            if let Some(prop_name) = prop.as_str() {
              if data.get(prop_name).is_none() {
                return Err(SchemaRegistryError::ValidationError {
                  message: format!("Missing required property: {prop_name}"),
                  path: format!("/{prop_name}"),
                });
              }
            }
          }
        }
      }
      "string" => {
        if !data.is_string() {
          return Err(SchemaRegistryError::ValidationError {
            message: format!("Expected string, got {}", self.get_type_name(data)),
            path: "/".to_string(),
          });
        }
      }
      "number" | "integer" => {
        if !data.is_number() {
          return Err(SchemaRegistryError::ValidationError {
            message: format!("Expected number, got {}", self.get_type_name(data)),
            path: "/".to_string(),
          });
        }
      }
      "boolean" => {
        if !data.is_boolean() {
          return Err(SchemaRegistryError::ValidationError {
            message: format!("Expected boolean, got {}", self.get_type_name(data)),
            path: "/".to_string(),
          });
        }
      }
      "array" => {
        if !data.is_array() {
          return Err(SchemaRegistryError::ValidationError {
            message: format!("Expected array, got {}", self.get_type_name(data)),
            path: "/".to_string(),
          });
        }
      }
      _ => {
        return Err(SchemaRegistryError::ValidationError {
          message: format!("Unsupported schema type: {schema_type}"),
          path: "/".to_string(),
        });
      }
    }

    Ok(())
  }

  /// Get the type name of a JSON value
  fn get_type_name(&self, value: &serde_json::Value) -> &str {
    match value {
      serde_json::Value::Null => "null",
      serde_json::Value::Bool(_) => "boolean",
      serde_json::Value::Number(_) => "number",
      serde_json::Value::String(_) => "string",
      serde_json::Value::Array(_) => "array",
      serde_json::Value::Object(_) => "object",
    }
  }
}

/// Errors that can occur in the schema registry
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SchemaRegistryError {
  /// Invalid schema ID
  #[error("Invalid schema ID: {0}")]
  InvalidId(String),

  /// Invalid schema version
  #[error("Invalid schema version: {0}")]
  InvalidVersion(String),

  /// Schema with this ID and version already exists
  #[error("Schema already exists: {id} version {version}")]
  DuplicateSchema { id: String, version: String },

  /// Schema not found
  #[error("Schema not found: {id} version {version}")]
  NotFound { id: String, version: String },

  /// Validation failed
  #[error("Validation failed at {path}: {message}")]
  ValidationError { message: String, path: String },
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
mod tests {
  use super::*;
  use chrono::Utc;

  fn create_test_schema(
    id: &str,
    version: &str,
    name: &str,
    schema_def: serde_json::Value,
  ) -> Schema {
    Schema {
      id: SchemaId::new(id.to_string()).unwrap(),
      version: SchemaVersion::new(version.to_string()).unwrap(),
      name: name.to_string(),
      description: None,
      schema: schema_def,
      created_at: Utc::now(),
    }
  }

  #[test]
  fn test_schema_id_valid() {
    assert!(SchemaId::new("test_schema".to_string()).is_ok());
    assert!(SchemaId::new("test-schema".to_string()).is_ok());
    assert!(SchemaId::new("test_schema_123".to_string()).is_ok());
  }

  #[test]
  fn test_schema_id_empty() {
    let result = SchemaId::new(String::new());
    assert!(result.is_err());
    assert!(matches!(result, Err(SchemaRegistryError::InvalidId(_))));
  }

  #[test]
  fn test_schema_version_valid() {
    assert!(SchemaVersion::new("1.0.0".to_string()).is_ok());
    assert!(SchemaVersion::new("0.1.0".to_string()).is_ok());
    assert!(SchemaVersion::new("1.0".to_string()).is_ok());
  }

  #[test]
  fn test_register_new_schema() {
    let mut registry = SchemaRegistry::new();
    let schema = create_test_schema(
      "user",
      "1.0.0",
      "User Schema",
      serde_json::json!({"type": "object"}),
    );

    let result = registry.register(schema);
    assert!(result.is_ok());
  }

  #[test]
  fn test_get_existing_schema() {
    let mut registry = SchemaRegistry::new();
    let schema = create_test_schema(
      "user",
      "1.0.0",
      "User Schema",
      serde_json::json!({"type": "object"}),
    );
    let id = schema.id.clone();
    let version = schema.version.clone();

    registry.register(schema).unwrap();
    let result = registry.get(&id, &version);

    assert!(result.is_ok());
    let retrieved = result.unwrap();
    assert_eq!(retrieved.name, "User Schema");
  }

  #[test]
  fn test_validate_valid_object() {
    let mut registry = SchemaRegistry::new();
    let schema = create_test_schema(
      "user",
      "1.0.0",
      "User Schema",
      serde_json::json!({
        "type": "object",
        "required": ["name", "email"]
      }),
    );
    let id = schema.id.clone();
    let version = schema.version.clone();

    registry.register(schema).unwrap();

    let valid_data = serde_json::json!({
      "name": "John Doe",
      "email": "john@example.com"
    });

    let result = registry.validate(&id, &version, &valid_data);
    assert!(result.is_ok());
  }

  #[test]
  fn test_validate_missing_required_field() {
    let mut registry = SchemaRegistry::new();
    let schema = create_test_schema(
      "user",
      "1.0.0",
      "User Schema",
      serde_json::json!({
        "type": "object",
        "required": ["name", "email"]
      }),
    );
    let id = schema.id.clone();
    let version = schema.version.clone();

    registry.register(schema).unwrap();

    let invalid_data = serde_json::json!({
      "name": "John Doe"
    });

    let result = registry.validate(&id, &version, &invalid_data);
    assert!(result.is_err());
    assert!(matches!(result, Err(SchemaRegistryError::ValidationError { .. })));
  }
}

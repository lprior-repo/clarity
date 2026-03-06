#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting, clippy::manual_let_else, clippy::match_wild_err_arm, clippy::match_like_matches_macro)]
#![warn(clippy::nursery)]
#![allow(clippy::missing_const_for_fn)]
#![forbid(unsafe_code)]

//! Core KIRK contract type definitions.
//!
//! This module provides type-safe definitions for design-by-contract specifications,
//! following the KIRK (Keep Invariants Regular and Known) methodology.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

/// Domain errors for KIRK contract operations.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum KirkContractError {
  /// Type name already exists in the registry
  #[error("duplicate type: {0}")]
  DuplicateType(String),

  /// Schema reference is invalid or not found
  #[error("invalid schema reference: {0}")]
  InvalidSchema(String),

  /// Circular dependency detected in type definitions
  #[error("circular dependency detected: {0}")]
  CircularDependency(String),

  /// Required field is missing from type definition
  #[error("missing required field: {0}")]
  MissingField(String),

  /// Type definition is incomplete
  #[error("incomplete type definition: {0}")]
  IncompleteDefinition(String),

  /// Validation failed for type constraints
  #[error("validation failed: {0}")]
  ValidationFailed(String),
}

/// Version information for type schemas.
///
/// Schemas are versioned for compatibility tracking and migration support.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractVersion {
  /// Major version (breaking changes)
  pub major: u32,
  /// Minor version (additions)
  pub minor: u32,
  /// Patch version (bug fixes)
  pub patch: u32,
}

impl ContractVersion {
  /// Create a new version.
  #[must_use]
  pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
    Self {
      major,
      minor,
      patch,
    }
  }

  /// Create an initial version (1.0.0).
  #[must_use]
  pub const fn initial() -> Self {
    Self::new(1, 0, 0)
  }

  /// Check if this version is compatible with another.
  /// Compatible if major versions match and this >= other.
  #[must_use]
  pub const fn is_compatible_with(&self, other: &Self) -> bool {
    self.major == other.major
      && (self.minor > other.minor || (self.minor == other.minor && self.patch >= other.patch))
  }
}

impl Default for ContractVersion {
  fn default() -> Self {
    Self::initial()
  }
}

impl std::fmt::Display for ContractVersion {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
  }
}

/// Schema definition for a KIRK contract type.
///
/// Contains metadata and structure information for type validation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeSchema {
  /// Unique name for this type schema
  pub name: String,
  /// Version of this schema
  pub version: ContractVersion,
  /// Human-readable description
  pub description: String,
  /// JSON schema for validation (optional)
  pub json_schema: Option<String>,
}

impl TypeSchema {
  /// Create a new type schema.
  #[must_use]
  pub fn new(name: String, version: ContractVersion, description: String) -> Self {
    Self {
      name,
      version,
      description,
      json_schema: None,
    }
  }

  /// Add a JSON schema for validation.
  #[must_use]
  pub fn with_json_schema(mut self, schema: String) -> Self {
    self.json_schema = Some(schema);
    self
  }
}

/// EARS requirement types for contract specifications.
///
/// Based on the Easy Approach to Requirements Syntax patterns.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EarsRequirement {
  /// Ubiquitous: Always applicable requirement
  /// "THE SYSTEM SHALL provide X"
  Ubiquitous {
    /// The actor (typically "system")
    actor: String,
    /// The action that shall be performed
    action: String,
  },

  /// Event-driven: Triggered by specific events
  /// "WHEN X, THE SYSTEM SHALL Y"
  EventDriven {
    /// The actor
    actor: String,
    /// The triggering event/condition
    trigger: String,
    /// The action to perform
    shall: String,
  },

  /// Unwanted: Behaviors that must NOT occur
  /// "IF X, THE SYSTEM SHALL NOT Y"
  Unwanted {
    /// The actor
    actor: String,
    /// The condition under which this applies
    condition: String,
    /// The action that must NOT occur
    shall_not: String,
    /// Reason why this is forbidden
    because: String,
  },
}

impl EarsRequirement {
  /// Create a ubiquitous requirement.
  #[must_use]
  pub fn ubiquitous(actor: String, action: String) -> Self {
    Self::Ubiquitous { actor, action }
  }

  /// Create an event-driven requirement.
  #[must_use]
  pub fn event_driven(actor: String, trigger: String, shall: String) -> Self {
    Self::EventDriven {
      actor,
      trigger,
      shall,
    }
  }

  /// Create an unwanted behavior specification.
  #[must_use]
  pub fn unwanted(actor: String, condition: String, shall_not: String, because: String) -> Self {
    Self::Unwanted {
      actor,
      condition,
      shall_not,
      because,
    }
  }

  /// Get the actor for this requirement.
  #[must_use]
  pub fn actor(&self) -> &str {
    match self {
      Self::Ubiquitous { actor, .. }
      | Self::EventDriven { actor, .. }
      | Self::Unwanted { actor, .. } => actor,
    }
  }
}

/// EARS section containing categorized requirements.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EarsSection {
  /// Ubiquitous requirements (always applicable)
  pub ubiquitous: Vec<EarsRequirement>,
  /// Event-driven requirements (triggered by events)
  pub event_driven: Vec<EarsRequirement>,
  /// Unwanted behaviors (forbidden actions)
  pub unwanted: Vec<EarsRequirement>,
}

impl EarsSection {
  /// Create an empty EARS section.
  #[must_use]
  pub fn new() -> Self {
    Self::default()
  }

  /// Add a ubiquitous requirement.
  #[must_use]
  pub fn with_ubiquitous(mut self, requirement: EarsRequirement) -> Self {
    self.ubiquitous.push(requirement);
    self
  }

  /// Add an event-driven requirement.
  #[must_use]
  pub fn with_event_driven(mut self, requirement: EarsRequirement) -> Self {
    self.event_driven.push(requirement);
    self
  }

  /// Add an unwanted behavior specification.
  #[must_use]
  pub fn with_unwanted(mut self, requirement: EarsRequirement) -> Self {
    self.unwanted.push(requirement);
    self
  }

  /// Get total count of all requirements.
  #[must_use]
  pub fn total_count(&self) -> usize {
    self.ubiquitous.len() + self.event_driven.len() + self.unwanted.len()
  }

  /// Check if section is empty.
  #[must_use]
  pub fn is_empty(&self) -> bool {
    self.total_count() == 0
  }
}

/// Precondition for a contract operation.
///
/// Preconditions specify requirements that must hold before an operation executes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Precondition {
  /// Unique identifier for this precondition
  pub id: String,
  /// Human-readable description
  pub description: String,
  /// Whether authentication is required
  pub auth_required: bool,
  /// Required input parameters
  pub required_inputs: Vec<String>,
  /// Required system state conditions
  pub system_state: Vec<String>,
}

impl Precondition {
  /// Create a new precondition.
  #[must_use]
  pub fn new(id: String, description: String) -> Self {
    Self {
      id,
      description,
      auth_required: false,
      required_inputs: Vec::new(),
      system_state: Vec::new(),
    }
  }

  /// Mark this precondition as requiring authentication.
  #[must_use]
  pub fn with_auth(mut self) -> Self {
    self.auth_required = true;
    self
  }

  /// Add a required input parameter.
  #[must_use]
  pub fn with_required_input(mut self, input: String) -> Self {
    self.required_inputs.push(input);
    self
  }

  /// Add a system state requirement.
  #[must_use]
  pub fn with_system_state(mut self, state: String) -> Self {
    self.system_state.push(state);
    self
  }
}

/// Postcondition for a contract operation.
///
/// Postconditions specify guarantees that hold after an operation completes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Postcondition {
  /// Unique identifier for this postcondition
  pub id: String,
  /// Human-readable description
  pub description: String,
  /// State changes caused by the operation
  pub state_changes: Vec<String>,
  /// Return value guarantees
  pub return_guarantees: Vec<String>,
}

impl Postcondition {
  /// Create a new postcondition.
  #[must_use]
  pub fn new(id: String, description: String) -> Self {
    Self {
      id,
      description,
      state_changes: Vec::new(),
      return_guarantees: Vec::new(),
    }
  }

  /// Add a state change guarantee.
  #[must_use]
  pub fn with_state_change(mut self, change: String) -> Self {
    self.state_changes.push(change);
    self
  }

  /// Add a return value guarantee.
  #[must_use]
  pub fn with_return_guarantee(mut self, guarantee: String) -> Self {
    self.return_guarantees.push(guarantee);
    self
  }
}

/// Invariant property that must always hold.
///
/// Invariants specify properties that must be maintained at all times.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Invariant {
  /// Unique identifier for this invariant
  pub id: String,
  /// Human-readable description
  pub description: String,
  /// Category of the invariant (e.g., "safety", "consistency", "security")
  pub category: String,
}

impl Invariant {
  /// Create a new invariant.
  #[must_use]
  pub fn new(id: String, description: String, category: String) -> Self {
    Self {
      id,
      description,
      category,
    }
  }

  /// Create a safety invariant.
  #[must_use]
  pub fn safety(id: String, description: String) -> Self {
    Self::new(id, description, "safety".to_string())
  }

  /// Create a consistency invariant.
  #[must_use]
  pub fn consistency(id: String, description: String) -> Self {
    Self::new(id, description, "consistency".to_string())
  }

  /// Create a security invariant.
  #[must_use]
  pub fn security(id: String, description: String) -> Self {
    Self::new(id, description, "security".to_string())
  }
}

/// A complete KIRK contract definition.
///
/// Contains preconditions, postconditions, invariants, and EARS requirements
/// for a system component or operation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KirkContract {
  /// Unique identifier for this contract
  pub id: String,
  /// Human-readable name
  pub name: String,
  /// Version of this contract
  pub version: ContractVersion,
  /// Schema reference for validation
  pub schema: TypeSchema,
  /// Preconditions that must hold before execution
  pub preconditions: Vec<Precondition>,
  /// Postconditions guaranteed after execution
  pub postconditions: Vec<Postcondition>,
  /// Invariants that must always hold
  pub invariants: Vec<Invariant>,
  /// EARS requirements section
  pub ears: EarsSection,
}

impl KirkContract {
  /// Create a new KIRK contract.
  #[must_use]
  pub fn new(id: String, name: String, schema: TypeSchema) -> Self {
    Self {
      id,
      name,
      version: ContractVersion::initial(),
      schema,
      preconditions: Vec::new(),
      postconditions: Vec::new(),
      invariants: Vec::new(),
      ears: EarsSection::new(),
    }
  }

  /// Add a precondition.
  #[must_use]
  pub fn with_precondition(mut self, precondition: Precondition) -> Self {
    self.preconditions.push(precondition);
    self
  }

  /// Add a postcondition.
  #[must_use]
  pub fn with_postcondition(mut self, postcondition: Postcondition) -> Self {
    self.postconditions.push(postcondition);
    self
  }

  /// Add an invariant.
  #[must_use]
  pub fn with_invariant(mut self, invariant: Invariant) -> Self {
    self.invariants.push(invariant);
    self
  }

  /// Set the EARS requirements section.
  #[must_use]
  pub fn with_ears(mut self, ears: EarsSection) -> Self {
    self.ears = ears;
    self
  }

  /// Update the version.
  #[must_use]
  pub fn with_version(mut self, version: ContractVersion) -> Self {
    self.version = version;
    self
  }

  /// Validate this contract for completeness and consistency.
  ///
  /// # Errors
  /// Returns [`KirkContractError`] if required fields are missing,
  /// the schema is invalid, or identifiers are duplicated.
  pub fn validate(&self) -> Result<(), KirkContractError> {
    // Check required fields
    if self.id.is_empty() {
      return Err(KirkContractError::MissingField("id".to_string()));
    }
    if self.name.is_empty() {
      return Err(KirkContractError::MissingField("name".to_string()));
    }

    // Validate schema
    if self.schema.name.is_empty() {
      return Err(KirkContractError::InvalidSchema(
        "schema name is empty".to_string(),
      ));
    }

    // Check for duplicate precondition IDs
    let mut seen_ids = BTreeSet::new();
    for pre in &self.preconditions {
      if !seen_ids.insert(&pre.id) {
        return Err(KirkContractError::ValidationFailed(format!(
          "duplicate precondition id: {}",
          pre.id
        )));
      }
    }

    // Check for duplicate postcondition IDs
    seen_ids.clear();
    for post in &self.postconditions {
      if !seen_ids.insert(&post.id) {
        return Err(KirkContractError::ValidationFailed(format!(
          "duplicate postcondition id: {}",
          post.id
        )));
      }
    }

    // Check for duplicate invariant IDs
    seen_ids.clear();
    for inv in &self.invariants {
      if !seen_ids.insert(&inv.id) {
        return Err(KirkContractError::ValidationFailed(format!(
          "duplicate invariant id: {}",
          inv.id
        )));
      }
    }

    Ok(())
  }
}

/// Registry for KIRK contract types.
///
/// Maintains a collection of registered types with unique names and versions.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeRegistry {
  /// Registered contracts by ID
  contracts: Vec<KirkContract>,
}

impl TypeRegistry {
  /// Create an empty type registry.
  #[must_use]
  pub fn new() -> Self {
    Self::default()
  }

  /// Register a new contract type.
  ///
  /// # Errors
  ///
  /// Returns `DuplicateType` if a contract with the same ID already exists.
  pub fn register(&mut self, contract: KirkContract) -> Result<(), KirkContractError> {
    // Check for duplicate ID
    if self.contracts.iter().any(|c| c.id == contract.id) {
      return Err(KirkContractError::DuplicateType(contract.id));
    }

    // Validate the contract before registration
    contract.validate()?;

    self.contracts.push(contract);
    Ok(())
  }

  /// Look up a contract by ID.
  #[must_use]
  pub fn get(&self, id: &str) -> Option<&KirkContract> {
    self.contracts.iter().find(|c| c.id == id)
  }

  /// Get all registered contract IDs.
  #[must_use]
  pub fn ids(&self) -> Vec<&str> {
    self.contracts.iter().map(|c| c.id.as_str()).collect()
  }

  /// Get the number of registered contracts.
  #[must_use]
  pub fn len(&self) -> usize {
    self.contracts.len()
  }

  /// Check if the registry is empty.
  #[must_use]
  pub fn is_empty(&self) -> bool {
    self.contracts.is_empty()
  }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::float_cmp, clippy::needless_collect, clippy::unnecessary_debug_formatting, clippy::match_same_arms, clippy::option_if_let_else, clippy::suspicious_else_formatting, clippy::manual_let_else, clippy::match_wild_err_arm, clippy::match_like_matches_macro, clippy::needless_pass_by_value)]
mod tests {
  use super::*;

  #[test]
  fn test_version_initial() {
    let v = ContractVersion::initial();
    assert_eq!(v.major, 1);
    assert_eq!(v.minor, 0);
    assert_eq!(v.patch, 0);
    assert_eq!(format!("{v}"), "1.0.0");
  }

  #[test]
  fn test_version_compatibility() {
    let v1 = ContractVersion::new(1, 0, 0);
    let v2 = ContractVersion::new(1, 1, 0);
    let v3 = ContractVersion::new(1, 0, 1);
    let v4 = ContractVersion::new(2, 0, 0);

    // Same major, higher minor is compatible
    assert!(v2.is_compatible_with(&v1));
    // Same major, same minor, higher patch is compatible
    assert!(v3.is_compatible_with(&v1));
    // Different major is not compatible
    assert!(!v4.is_compatible_with(&v1));
    // Lower version is not compatible
    assert!(!v1.is_compatible_with(&v2));
  }

  #[test]
  fn test_ears_requirement_ubiquitous() {
    let req = EarsRequirement::ubiquitous("system".to_string(), "authenticate users".to_string());
    assert_eq!(req.actor(), "system");

    let json = serde_json::to_string(&req);
    assert!(json.is_ok());
    let json_ref = json.as_ref();
    assert!(json_ref.is_ok());
    let Some(json_str) = json_ref.ok() else {
      return;
    };
    let deserialized: Result<EarsRequirement, _> = serde_json::from_str(json_str);
    assert!(deserialized.is_ok());
  }

  #[test]
  fn test_ears_requirement_event_driven() {
    let req = EarsRequirement::event_driven(
      "system".to_string(),
      "a new contract type is defined".to_string(),
      "validate the type structure".to_string(),
    );
    assert_eq!(req.actor(), "system");
  }

  #[test]
  fn test_ears_requirement_unwanted() {
    let req = EarsRequirement::unwanted(
      "system".to_string(),
      "a contract type definition is incomplete".to_string(),
      "register the type".to_string(),
      "incomplete types cause runtime validation failures".to_string(),
    );
    assert_eq!(req.actor(), "system");
  }

  #[test]
  fn test_ears_section_builder() {
    let section = EarsSection::new()
      .with_ubiquitous(EarsRequirement::ubiquitous(
        "system".to_string(),
        "provide type definitions".to_string(),
      ))
      .with_event_driven(EarsRequirement::event_driven(
        "system".to_string(),
        "contract types are serialized".to_string(),
        "produce deterministic JSON output".to_string(),
      ))
      .with_unwanted(EarsRequirement::unwanted(
        "system".to_string(),
        "type constraints are contradictory".to_string(),
        "accept the type definition".to_string(),
        "contradictory constraints make validation impossible".to_string(),
      ));

    assert_eq!(section.ubiquitous.len(), 1);
    assert_eq!(section.event_driven.len(), 1);
    assert_eq!(section.unwanted.len(), 1);
    assert_eq!(section.total_count(), 3);
    assert!(!section.is_empty());
  }

  #[test]
  fn test_ears_section_empty() {
    let section = EarsSection::new();
    assert!(section.is_empty());
    assert_eq!(section.total_count(), 0);
  }

  #[test]
  fn test_precondition_builder() {
    let pre = Precondition::new(
      "pre-1".to_string(),
      "Type system must be initialized".to_string(),
    )
    .with_auth()
    .with_required_input("schema".to_string())
    .with_system_state("registry available".to_string());

    assert_eq!(pre.id, "pre-1");
    assert!(pre.auth_required);
    assert_eq!(pre.required_inputs.len(), 1);
    assert_eq!(pre.system_state.len(), 1);
  }

  #[test]
  fn test_postcondition_builder() {
    let post = Postcondition::new("post-1".to_string(), "All types are registered".to_string())
      .with_state_change("registry updated".to_string())
      .with_return_guarantee("returns success".to_string());

    assert_eq!(post.id, "post-1");
    assert_eq!(post.state_changes.len(), 1);
    assert_eq!(post.return_guarantees.len(), 1);
  }

  #[test]
  fn test_invariant_factories() {
    let safety = Invariant::safety("inv-1".to_string(), "No data loss".to_string());
    assert_eq!(safety.category, "safety");

    let consistency = Invariant::consistency("inv-2".to_string(), "Type names unique".to_string());
    assert_eq!(consistency.category, "consistency");

    let security = Invariant::security("inv-3".to_string(), "Auth required".to_string());
    assert_eq!(security.category, "security");
  }

  #[test]
  fn test_kirk_contract_builder() {
    let schema = TypeSchema::new(
      "TestSchema".to_string(),
      ContractVersion::initial(),
      "Test schema".to_string(),
    );

    let contract = KirkContract::new(
      "contract-1".to_string(),
      "Test Contract".to_string(),
      schema,
    )
    .with_precondition(Precondition::new(
      "pre-1".to_string(),
      "System ready".to_string(),
    ))
    .with_postcondition(Postcondition::new(
      "post-1".to_string(),
      "Operation complete".to_string(),
    ))
    .with_invariant(Invariant::consistency(
      "inv-1".to_string(),
      "Always valid".to_string(),
    ));

    assert_eq!(contract.id, "contract-1");
    assert_eq!(contract.preconditions.len(), 1);
    assert_eq!(contract.postconditions.len(), 1);
    assert_eq!(contract.invariants.len(), 1);
  }

  #[test]
  fn test_kirk_contract_validation_success() {
    let schema = TypeSchema::new(
      "ValidSchema".to_string(),
      ContractVersion::initial(),
      "Valid schema".to_string(),
    );

    let contract = KirkContract::new(
      "valid-contract".to_string(),
      "Valid Contract".to_string(),
      schema,
    );

    let result = contract.validate();
    assert!(result.is_ok());
  }

  #[test]
  fn test_kirk_contract_validation_missing_id() {
    let schema = TypeSchema::new(
      "Schema".to_string(),
      ContractVersion::initial(),
      "Schema".to_string(),
    );

    let contract = KirkContract::new(String::new(), "Name".to_string(), schema);

    let result = contract.validate();
    assert!(matches!(
        result,
        Err(KirkContractError::MissingField(field)) if field == "id"
    ));
  }

  #[test]
  fn test_kirk_contract_validation_missing_name() {
    let schema = TypeSchema::new(
      "Schema".to_string(),
      ContractVersion::initial(),
      "Schema".to_string(),
    );

    let contract = KirkContract::new("id".to_string(), String::new(), schema);

    let result = contract.validate();
    assert!(matches!(
        result,
        Err(KirkContractError::MissingField(field)) if field == "name"
    ));
  }

  #[test]
  fn test_kirk_contract_validation_empty_schema_name() {
    let schema = TypeSchema::new(
      String::new(),
      ContractVersion::initial(),
      "Schema".to_string(),
    );

    let contract = KirkContract::new("id".to_string(), "Name".to_string(), schema);

    let result = contract.validate();
    assert!(matches!(result, Err(KirkContractError::InvalidSchema(_))));
  }

  #[test]
  fn test_kirk_contract_validation_duplicate_precondition_ids() {
    let schema = TypeSchema::new(
      "Schema".to_string(),
      ContractVersion::initial(),
      "Schema".to_string(),
    );

    let contract = KirkContract::new("id".to_string(), "Name".to_string(), schema)
      .with_precondition(Precondition::new("dup".to_string(), "First".to_string()))
      .with_precondition(Precondition::new("dup".to_string(), "Second".to_string()));

    let result = contract.validate();
    assert!(matches!(
      result,
      Err(KirkContractError::ValidationFailed(_))
    ));
  }

  #[test]
  fn test_type_registry_register() {
    let schema = TypeSchema::new(
      "Schema".to_string(),
      ContractVersion::initial(),
      "Schema".to_string(),
    );

    let contract = KirkContract::new("contract-1".to_string(), "Contract".to_string(), schema);

    let mut registry = TypeRegistry::new();
    let result = registry.register(contract);

    assert!(result.is_ok());
    assert_eq!(registry.len(), 1);
  }

  #[test]
  fn test_type_registry_duplicate() {
    let schema = TypeSchema::new(
      "Schema".to_string(),
      ContractVersion::initial(),
      "Schema".to_string(),
    );

    let contract1 = KirkContract::new(
      "same-id".to_string(),
      "Contract 1".to_string(),
      schema.clone(),
    );
    let contract2 = KirkContract::new("same-id".to_string(), "Contract 2".to_string(), schema);

    let mut registry = TypeRegistry::new();
    let result1 = registry.register(contract1);
    assert!(result1.is_ok());

    let result2 = registry.register(contract2);
    assert!(matches!(result2, Err(KirkContractError::DuplicateType(id)) if id == "same-id"));
  }

  #[test]
  fn test_type_registry_get() {
    let schema = TypeSchema::new(
      "Schema".to_string(),
      ContractVersion::initial(),
      "Schema".to_string(),
    );

    let contract = KirkContract::new("find-me".to_string(), "Contract".to_string(), schema);

    let mut registry = TypeRegistry::new();
    let _ = registry.register(contract);

    let found = registry.get("find-me");
    assert!(found.is_some());
    assert_eq!(found.map(|c| c.name.as_str()), Some("Contract"));

    let not_found = registry.get("missing");
    assert!(not_found.is_none());
  }

  #[test]
  fn test_type_registry_ids() {
    let schema = TypeSchema::new(
      "Schema".to_string(),
      ContractVersion::initial(),
      "Schema".to_string(),
    );

    let mut registry = TypeRegistry::new();
    let _ = registry.register(KirkContract::new(
      "id-1".to_string(),
      "C1".to_string(),
      schema.clone(),
    ));
    let _ = registry.register(KirkContract::new(
      "id-2".to_string(),
      "C2".to_string(),
      schema,
    ));

    let ids = registry.ids();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&"id-1"));
    assert!(ids.contains(&"id-2"));
  }

  #[test]
  fn test_kirk_contract_serialization() {
    let schema = TypeSchema::new(
      "TestSchema".to_string(),
      ContractVersion::new(1, 2, 3),
      "A test schema".to_string(),
    );

    let contract = KirkContract::new(
      "test-contract".to_string(),
      "Test Contract".to_string(),
      schema,
    )
    .with_precondition(Precondition::new(
      "pre-1".to_string(),
      "Must be ready".to_string(),
    ))
    .with_invariant(Invariant::safety(
      "inv-1".to_string(),
      "Always safe".to_string(),
    ));

    let json = serde_json::to_string(&contract);
    assert!(json.is_ok());

    let json_ref = json.as_ref();
    assert!(json_ref.is_ok());
    let Some(json_str) = json_ref.ok() else {
      return;
    };
    let deserialized: Result<KirkContract, _> = serde_json::from_str(json_str);
    assert!(deserialized.is_ok());
    assert_eq!(
      deserialized.ok().map(|c| c.id),
      Some("test-contract".to_string())
    );
  }

  #[test]
  fn test_type_schema_with_json_schema() {
    let schema = TypeSchema::new(
      "Schema".to_string(),
      ContractVersion::initial(),
      "Schema".to_string(),
    )
    .with_json_schema(r#"{"type": "object"}"#.to_string());

    assert!(schema.json_schema.is_some());
  }

  #[test]
  fn test_registry_empty() {
    let registry = TypeRegistry::new();
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
  }
}

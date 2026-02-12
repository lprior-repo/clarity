//! Design by Contract Module - Meyer's DbC Framework
//!
//! Implements Bertrand Meyer's Design by Contract methodology:
//! - Preconditions: What must be true BEFORE an operation
//! - Postconditions: What must be true AFTER an operation
//! - Invariants: What must ALWAYS be true for a type/state
//!
//! This module provides compile-time and runtime contract verification
//! for building reliable, self-documenting software components.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::derivable_impls)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

// ============================================================================
// INVARIANT SEVERITY
// ============================================================================

/// Severity level for invariant violations
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvariantSeverity {
  /// Informational - no impact on correctness
  Info,
  /// Warning - potential issue but not critical
  Warning,
  /// Error - correctness compromised
  Error,
  /// Critical - system integrity at risk
  Critical,
}

impl Default for InvariantSeverity {
  fn default() -> Self {
    Self::Warning
  }
}

impl fmt::Display for InvariantSeverity {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Info => write!(f, "Info"),
      Self::Warning => write!(f, "Warning"),
      Self::Error => write!(f, "Error"),
      Self::Critical => write!(f, "Critical"),
    }
  }
}

// ============================================================================
// CONTRACT VIOLATION ERROR
// ============================================================================

/// Errors representing contract violations
#[derive(Clone, Debug, PartialEq, Eq, Error, Serialize, Deserialize)]
pub enum ContractViolation {
  /// A precondition was not satisfied
  #[error("Precondition failed in '{contract_name}': {message}")]
  PreconditionFailed {
    /// Name of the contract
    contract_name: String,
    /// Error message describing the violation
    message: String,
  },

  /// A postcondition was not satisfied
  #[error("Postcondition failed in '{contract_name}': {message}")]
  PostconditionFailed {
    /// Name of the contract
    contract_name: String,
    /// Error message describing the violation
    message: String,
  },

  /// An invariant was violated
  #[error("Invariant violated in '{contract_name}': {description} (severity: {severity})")]
  InvariantViolated {
    /// Name of the contract
    contract_name: String,
    /// Description of the violated invariant
    description: String,
    /// Severity of the violation
    severity: InvariantSeverity,
  },
}

// ============================================================================
// PRECONDITION
// ============================================================================

/// Serializable metadata for a precondition (without the predicate)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreconditionMeta {
  /// Error message when precondition fails
  pub error_message: String,
  /// Optional description of the precondition
  pub description: Option<String>,
}

/// A condition that must be true BEFORE an operation executes.
///
/// Preconditions define the requirements that callers must satisfy.
/// If a precondition fails, the caller is at fault (contract breach).
///
/// # Example
///
/// ```
/// # use clarity_core::pme_lattice::design_by_contract::Precondition;
/// let precondition = Precondition::new(
///     "amount must be positive".to_string(),
///     Box::new(|amount: &i32| *amount > 0)
/// );
///
/// assert!(precondition.check(&100));
/// assert!(!precondition.check(&-50));
/// ```
#[derive(Clone)]
pub struct Precondition<T> {
  /// Error message when precondition fails
  error_message: String,
  /// Optional description of the precondition
  description: Option<String>,
  /// The predicate function that checks the condition
  predicate: Arc<dyn Fn(&T) -> bool + Send + Sync>,
}

impl<T> Precondition<T> {
  /// Create a new precondition with an error message and predicate.
  ///
  /// The predicate returns `true` if the precondition is satisfied.
  #[must_use]
  pub fn new(error_message: String, predicate: Box<dyn Fn(&T) -> bool + Send + Sync>) -> Self {
    Self {
      error_message,
      description: None,
      predicate: Arc::from(predicate),
    }
  }

  /// Add an optional description for documentation purposes.
  #[must_use]
  pub fn with_description(mut self, description: String) -> Self {
    self.description = Some(description);
    self
  }

  /// Check if the precondition is satisfied for the given input.
  #[must_use]
  pub fn check(&self, input: &T) -> bool {
    (self.predicate)(input)
  }

  /// Get the error message for this precondition.
  #[must_use]
  pub fn error_message(&self) -> &str {
    &self.error_message
  }

  /// Get the optional description.
  #[must_use]
  pub fn description(&self) -> Option<&String> {
    self.description.as_ref()
  }

  /// Extract serializable metadata (without predicate).
  #[must_use]
  pub fn to_meta(&self) -> PreconditionMeta {
    PreconditionMeta {
      error_message: self.error_message.clone(),
      description: self.description.clone(),
    }
  }
}

impl<T> fmt::Debug for Precondition<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Precondition")
      .field("error_message", &self.error_message)
      .field("description", &self.description)
      .finish_non_exhaustive()
  }
}

// ============================================================================
// POSTCONDITION
// ============================================================================

/// Serializable metadata for a postcondition (without the predicate)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostconditionMeta {
  /// Error message when postcondition fails
  pub error_message: String,
  /// Optional description of the postcondition
  pub description: Option<String>,
  /// Optional tag for categorization
  pub tag: Option<String>,
}

/// A condition that must be true AFTER an operation completes.
///
/// Postconditions define what the operation guarantees to the caller.
/// If a postcondition fails, the implementation is at fault (contract breach).
///
/// # Example
///
/// ```
/// # use clarity_core::pme_lattice::design_by_contract::Postcondition;
/// let postcondition = Postcondition::new(
///     "result must be sorted".to_string(),
///     Box::new(|result: &Vec<i32>| result.windows(2).all(|w| w[0] <= w[1]))
/// );
///
/// assert!(postcondition.check(&vec![1, 2, 3]));
/// assert!(!postcondition.check(&vec![3, 1, 2]));
/// ```
#[derive(Clone)]
pub struct Postcondition<R> {
  /// Error message when postcondition fails
  error_message: String,
  /// Optional description of the postcondition
  description: Option<String>,
  /// Optional tag for categorization
  tag: Option<String>,
  /// The predicate function that checks the condition
  predicate: Arc<dyn Fn(&R) -> bool + Send + Sync>,
}

impl<R> Postcondition<R> {
  /// Create a new postcondition with an error message and predicate.
  ///
  /// The predicate returns `true` if the postcondition is satisfied.
  #[must_use]
  pub fn new(error_message: String, predicate: Box<dyn Fn(&R) -> bool + Send + Sync>) -> Self {
    Self {
      error_message,
      description: None,
      tag: None,
      predicate: Arc::from(predicate),
    }
  }

  /// Add an optional description for documentation purposes.
  #[must_use]
  pub fn with_description(mut self, description: String) -> Self {
    self.description = Some(description);
    self
  }

  /// Add an optional tag for categorization.
  #[must_use]
  pub fn with_tag(mut self, tag: &str) -> Self {
    self.tag = Some(tag.to_string());
    self
  }

  /// Check if the postcondition is satisfied for the given output.
  #[must_use]
  pub fn check(&self, output: &R) -> bool {
    (self.predicate)(output)
  }

  /// Get the error message for this postcondition.
  #[must_use]
  pub fn error_message(&self) -> &str {
    &self.error_message
  }

  /// Get the optional description.
  #[must_use]
  pub fn description(&self) -> Option<&String> {
    self.description.as_ref()
  }

  /// Get the optional tag.
  #[must_use]
  pub fn tag(&self) -> Option<&String> {
    self.tag.as_ref()
  }

  /// Extract serializable metadata (without predicate).
  #[must_use]
  pub fn to_meta(&self) -> PostconditionMeta {
    PostconditionMeta {
      error_message: self.error_message.clone(),
      description: self.description.clone(),
      tag: self.tag.clone(),
    }
  }
}

impl<R> fmt::Debug for Postcondition<R> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Postcondition")
      .field("error_message", &self.error_message)
      .field("description", &self.description)
      .field("tag", &self.tag)
      .finish_non_exhaustive()
  }
}

// ============================================================================
// INVARIANT
// ============================================================================

/// Serializable metadata for an invariant (without the predicate)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvariantMeta {
  /// Description of what the invariant guarantees
  pub description: String,
  /// Severity level when violated
  pub severity: InvariantSeverity,
}

/// A condition that must ALWAYS be true for a type or state.
///
/// Invariants define the consistency rules that must hold before and after
/// every operation. They represent the "health" of an object or system.
///
/// # Example
///
/// ```
/// # use clarity_core::pme_lattice::design_by_contract::{Invariant, InvariantSeverity};
/// struct BankAccount { balance: i64 }
///
/// let invariant = Invariant::new(
///     "balance must be non-negative".to_string(),
///     Box::new(|account: &BankAccount| account.balance >= 0)
/// ).with_severity(InvariantSeverity::Critical);
///
/// assert!(invariant.check(&BankAccount { balance: 100 }));
/// assert!(!invariant.check(&BankAccount { balance: -1 }));
/// ```
#[derive(Clone)]
pub struct Invariant<S> {
  /// Description of what the invariant guarantees
  description: String,
  /// Severity level when violated
  severity: InvariantSeverity,
  /// The predicate function that checks the condition
  predicate: Arc<dyn Fn(&S) -> bool + Send + Sync>,
}

impl<S> Invariant<S> {
  /// Create a new invariant with a description and predicate.
  ///
  /// The predicate returns `true` if the invariant holds.
  /// Default severity is `InvariantSeverity::Warning`.
  #[must_use]
  pub fn new(description: String, predicate: Box<dyn Fn(&S) -> bool + Send + Sync>) -> Self {
    Self {
      description,
      severity: InvariantSeverity::Warning,
      predicate: Arc::from(predicate),
    }
  }

  /// Set the severity level for this invariant.
  #[must_use]
  pub fn with_severity(mut self, severity: InvariantSeverity) -> Self {
    self.severity = severity;
    self
  }

  /// Check if the invariant holds for the given state.
  #[must_use]
  pub fn check(&self, state: &S) -> bool {
    (self.predicate)(state)
  }

  /// Get the description of this invariant.
  #[must_use]
  pub fn description(&self) -> &str {
    &self.description
  }

  /// Get the severity level.
  #[must_use]
  pub fn severity(&self) -> InvariantSeverity {
    self.severity
  }

  /// Extract serializable metadata (without predicate).
  #[must_use]
  pub fn to_meta(&self) -> InvariantMeta {
    InvariantMeta {
      description: self.description.clone(),
      severity: self.severity,
    }
  }
}

impl<S> fmt::Debug for Invariant<S> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Invariant")
      .field("description", &self.description)
      .field("severity", &self.severity)
      .finish_non_exhaustive()
  }
}

// ============================================================================
// CONTRACT META (SERIALIZABLE)
// ============================================================================

/// Serializable metadata for a contract (without predicates)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractMeta {
  /// Unique identifier
  pub id: Uuid,
  /// Name of the contract
  pub name: String,
  /// Optional description
  pub description: Option<String>,
  /// Optional tag
  pub tag: Option<String>,
  /// Number of preconditions
  pub precondition_count: usize,
  /// Number of postconditions
  pub postcondition_count: usize,
  /// Number of invariants
  pub invariant_count: usize,
}

// ============================================================================
// CONTRACT
// ============================================================================

/// A complete contract combining preconditions, postconditions, and invariants.
///
/// Contracts specify the mutual obligations between a caller and a callee:
/// - The caller must satisfy preconditions
/// - The callee must satisfy postconditions (given preconditions)
/// - Both must preserve invariants
///
/// # Type Parameters
///
/// - `T`: Input type (checked by preconditions)
/// - `R`: Output type (checked by postconditions)
/// - `S`: State type (checked by invariants)
///
/// # Example
///
/// ```
/// # use clarity_core::pme_lattice::design_by_contract::{Contract, Precondition, Postcondition, Invariant};
/// struct State { counter: i32 }
///
/// let contract = Contract::<i32, String, State>::new("divide")
///     .with_precondition(Precondition::new(
///         "divisor must not be zero".to_string(),
///         Box::new(|x: &i32| *x != 0)
///     ))
///     .with_postcondition(Postcondition::new(
///         "result must not be empty".to_string(),
///         Box::new(|s: &String| !s.is_empty())
///     ))
///     .with_invariant(Invariant::new(
///         "counter >= 0".to_string(),
///         Box::new(|s: &State| s.counter >= 0)
///     ));
///
/// // Verify preconditions
/// assert!(contract.verify_preconditions(&5).is_ok());
/// assert!(contract.verify_preconditions(&0).is_err());
/// ```
#[derive(Clone)]
pub struct Contract<T, R, S> {
  /// Unique identifier
  id: Uuid,
  /// Name of the contract
  name: String,
  /// Optional description
  description: Option<String>,
  /// Optional tag
  tag: Option<String>,
  /// Creation timestamp
  created_at: DateTime<Utc>,
  /// Preconditions to check before operation
  preconditions: Vec<Precondition<T>>,
  /// Postconditions to check after operation
  postconditions: Vec<Postcondition<R>>,
  /// Invariants to always maintain
  invariants: Vec<Invariant<S>>,
}

impl<T, R, S> Contract<T, R, S> {
  /// Create a new empty contract with the given name.
  #[must_use]
  pub fn new(name: &str) -> Self {
    Self {
      id: Uuid::new_v4(),
      name: name.to_string(),
      description: None,
      tag: None,
      created_at: Utc::now(),
      preconditions: Vec::new(),
      postconditions: Vec::new(),
      invariants: Vec::new(),
    }
  }

  /// Add an optional description.
  #[must_use]
  pub fn with_description(mut self, description: String) -> Self {
    self.description = Some(description);
    self
  }

  /// Add an optional tag.
  #[must_use]
  pub fn with_tag(mut self, tag: &str) -> Self {
    self.tag = Some(tag.to_string());
    self
  }

  /// Add a precondition.
  #[must_use]
  pub fn with_precondition(mut self, precondition: Precondition<T>) -> Self {
    self.preconditions.push(precondition);
    self
  }

  /// Add a postcondition.
  #[must_use]
  pub fn with_postcondition(mut self, postcondition: Postcondition<R>) -> Self {
    self.postconditions.push(postcondition);
    self
  }

  /// Add an invariant.
  #[must_use]
  pub fn with_invariant(mut self, invariant: Invariant<S>) -> Self {
    self.invariants.push(invariant);
    self
  }

  /// Get the contract's unique identifier.
  #[must_use]
  pub fn id(&self) -> Uuid {
    self.id
  }

  /// Get the contract name.
  #[must_use]
  pub fn name(&self) -> &str {
    &self.name
  }

  /// Get the optional description.
  #[must_use]
  pub fn description(&self) -> Option<&String> {
    self.description.as_ref()
  }

  /// Get the optional tag.
  #[must_use]
  pub fn tag(&self) -> Option<&String> {
    self.tag.as_ref()
  }

  /// Get the creation timestamp.
  #[must_use]
  pub fn created_at(&self) -> DateTime<Utc> {
    self.created_at
  }

  /// Get all preconditions.
  #[must_use]
  pub fn preconditions(&self) -> &[Precondition<T>] {
    &self.preconditions
  }

  /// Get all postconditions.
  #[must_use]
  pub fn postconditions(&self) -> &[Postcondition<R>] {
    &self.postconditions
  }

  /// Get all invariants.
  #[must_use]
  pub fn invariants(&self) -> &[Invariant<S>] {
    &self.invariants
  }

  /// Verify all preconditions against the input.
  ///
  /// # Errors
  ///
  /// Returns `ContractViolation::PreconditionFailed` for the first failing precondition.
  pub fn verify_preconditions(&self, input: &T) -> Result<(), ContractViolation> {
    self
      .preconditions
      .iter()
      .find(|pre| !pre.check(input))
      .map(|pre| ContractViolation::PreconditionFailed {
        contract_name: self.name.clone(),
        message: pre.error_message().to_string(),
      })
      .map_or(Ok(()), Err)
  }

  /// Verify all postconditions against the output.
  ///
  /// # Errors
  ///
  /// Returns `ContractViolation::PostconditionFailed` for the first failing postcondition.
  pub fn verify_postconditions(&self, output: &R) -> Result<(), ContractViolation> {
    self
      .postconditions
      .iter()
      .find(|post| !post.check(output))
      .map(|post| ContractViolation::PostconditionFailed {
        contract_name: self.name.clone(),
        message: post.error_message().to_string(),
      })
      .map_or(Ok(()), Err)
  }

  /// Verify all invariants against the state.
  ///
  /// # Errors
  ///
  /// Returns `ContractViolation::InvariantViolated` for the first failing invariant.
  pub fn verify_invariants(&self, state: &S) -> Result<(), ContractViolation> {
    self
      .invariants
      .iter()
      .find(|inv| !inv.check(state))
      .map(|inv| ContractViolation::InvariantViolated {
        contract_name: self.name.clone(),
        description: inv.description().to_string(),
        severity: inv.severity(),
      })
      .map_or(Ok(()), Err)
  }

  /// Verify preconditions, postconditions, and invariants together.
  ///
  /// Checks in order: preconditions -> invariants -> postconditions
  ///
  /// # Errors
  ///
  /// Returns the first violation encountered.
  pub fn verify_all(&self, input: &T, output: &R, state: &S) -> Result<(), ContractViolation> {
    self
      .verify_preconditions(input)
      .and_then(|()| self.verify_invariants(state))
      .and_then(|()| self.verify_postconditions(output))
  }

  /// Extract serializable metadata (without predicates).
  #[must_use]
  pub fn to_meta(&self) -> ContractMeta {
    ContractMeta {
      id: self.id,
      name: self.name.clone(),
      description: self.description.clone(),
      tag: self.tag.clone(),
      precondition_count: self.preconditions.len(),
      postcondition_count: self.postconditions.len(),
      invariant_count: self.invariants.len(),
    }
  }
}

impl<T, R, S> fmt::Debug for Contract<T, R, S> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Contract")
      .field("id", &self.id)
      .field("name", &self.name)
      .field("description", &self.description)
      .field("tag", &self.tag)
      .field("created_at", &self.created_at)
      .field("precondition_count", &self.preconditions.len())
      .field("postcondition_count", &self.postconditions.len())
      .field("invariant_count", &self.invariants.len())
      .finish_non_exhaustive()
  }
}

// ============================================================================
// UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn invariant_severity_default_is_warning() {
    assert_eq!(InvariantSeverity::default(), InvariantSeverity::Warning);
  }

  #[test]
  fn invariant_severity_display_works() {
    assert_eq!(format!("{}", InvariantSeverity::Info), "Info");
    assert_eq!(format!("{}", InvariantSeverity::Warning), "Warning");
    assert_eq!(format!("{}", InvariantSeverity::Error), "Error");
    assert_eq!(format!("{}", InvariantSeverity::Critical), "Critical");
  }

  #[test]
  fn contract_violation_display_precondition() {
    let violation = ContractViolation::PreconditionFailed {
      contract_name: "test".to_string(),
      message: "must be positive".to_string(),
    };
    let display = format!("{violation}");
    assert!(display.contains("Precondition failed"));
    assert!(display.contains("test"));
    assert!(display.contains("must be positive"));
  }

  #[test]
  fn contract_violation_display_postcondition() {
    let violation = ContractViolation::PostconditionFailed {
      contract_name: "output".to_string(),
      message: "not empty".to_string(),
    };
    let display = format!("{violation}");
    assert!(display.contains("Postcondition failed"));
    assert!(display.contains("output"));
    assert!(display.contains("not empty"));
  }

  #[test]
  fn contract_violation_display_invariant() {
    let violation = ContractViolation::InvariantViolated {
      contract_name: "state".to_string(),
      description: "valid state".to_string(),
      severity: InvariantSeverity::Critical,
    };
    let display = format!("{violation}");
    assert!(display.contains("Invariant violated"));
    assert!(display.contains("state"));
    assert!(display.contains("valid state"));
    assert!(display.contains("Critical"));
  }
}

//! Domain Error Taxonomy
//!
//! Unified error variants for the entire system.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(tag = "type", content = "data")]
pub enum ClarityError {
  /// Errors related to the core domain logic and invariants.
  #[error("Domain Error: {0}")]
  Domain(String),

  /// Errors occurring during data persistence or retrieval.
  #[error("Storage Error: {0}")]
  Storage(String),

  /// Errors from the Lattice analysis engine (EARS, Inversion, etc.)
  #[error("Analysis Error: {0}")]
  Analysis(String),

  /// Errors during the planning and bead generation phase.
  #[error("Planning Error: {0}")]
  Planning(String),

  /// Errors from external integrations (AI providers, network, etc.)
  #[error("External Error: {0}")]
  External(String),

  /// Generic validation error for user input.
  #[error("Validation Error: {0}")]
  Validation(String),

  /// Catch-all for internal system failures.
  #[error("Internal System Error: {0}")]
  Internal(String),
}

impl ClarityError {
  /// Helper to create a storage error from anything that can be a string.
  pub fn storage(msg: impl Into<String>) -> Self {
    Self::Storage(msg.into())
  }

  /// Helper to create an analysis error.
  pub fn analysis(msg: impl Into<String>) -> Self {
    Self::Analysis(msg.into())
  }

  /// Helper to create a planning error.
  pub fn planning(msg: impl Into<String>) -> Self {
    Self::Planning(msg.into())
  }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
  use super::*;

  #[test]
  fn test_error_serialization_roundtrip() {
    let err = ClarityError::Storage("Disk full".to_string());
    let json = serde_json::to_string(&err).unwrap();
    let decoded: ClarityError = serde_json::from_str(&json).unwrap();
    assert_eq!(err, decoded);
  }

  #[test]
  fn test_error_display() {
    let err = ClarityError::Analysis("Invalid EARS".to_string());
    assert_eq!(format!("{err}"), "Analysis Error: Invalid EARS");
  }
}

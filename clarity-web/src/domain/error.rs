//! Domain Error Taxonomy
//!
//! Unified error variants for the entire system.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(tag = "type", content = "data")]
pub enum ClarityError {
  #[error("I/O Error: {0}")]
  Io(String),

  #[error("Serialization Error: {0}")]
  Serialization(String),

  #[error("Internal Error: {0}")]
  Internal(String),
}

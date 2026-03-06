//! Domain Types
//!
//! Canonical types for Answers and Specifications.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Answer {
  pub id: String,
  pub step_id: String,
  pub value: String,
  pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Behavior {
  pub name: String,
  pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Feature {
  pub name: String,
  pub description: String,
  pub behaviors: Vec<Behavior>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Spec {
  pub name: String,
  pub features: Vec<Feature>,
}

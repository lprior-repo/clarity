//! Thesis & Antithesis Generator for PME Discover Phase
//!
//! Generates product theses with required null hypotheses (antitheses) to prevent
//! optimism bias. Every thesis must have a corresponding antithesis that asks
//! "WHY might this fail?" to validate assumptions.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::missing_const_for_fn)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum ThesisAntithesisError {
  #[error("thesis cannot be empty or whitespace")]
  EmptyThesis,

  #[error("antithesis cannot be empty or whitespace")]
  EmptyAntithesis,

  #[error("thesis and antithesis cannot be identical")]
  ThesisEqualsAntithesis,

  #[error("at least one failure mode is required for validation")]
  NoFailureModes,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThesisAntithesisGenerator {
  id: Uuid,
  thesis: String,
  antithesis: String,
  failure_modes: Vec<String>,
  validation_criteria: Vec<String>,
  created_at: DateTime<Utc>,
  updated_at: DateTime<Utc>,
}

impl ThesisAntithesisGenerator {
  /// Create a new thesis/antithesis generator.
  ///
  /// # Errors
  /// Returns `ThesisAntithesisError::EmptyThesis` if thesis is empty or whitespace.
  /// Returns `ThesisAntithesisError::EmptyAntithesis` if antithesis is empty or whitespace.
  /// Returns `ThesisAntithesisError::ThesisEqualsAntithesis` if thesis and antithesis are identical (case-insensitive).
  pub fn new(thesis: &str, antithesis: &str) -> Result<Self, ThesisAntithesisError> {
    let thesis_trimmed = thesis.trim();
    let antithesis_trimmed = antithesis.trim();

    if thesis_trimmed.is_empty() {
      return Err(ThesisAntithesisError::EmptyThesis);
    }

    if antithesis_trimmed.is_empty() {
      return Err(ThesisAntithesisError::EmptyAntithesis);
    }

    if thesis_trimmed.eq_ignore_ascii_case(antithesis_trimmed) {
      return Err(ThesisAntithesisError::ThesisEqualsAntithesis);
    }

    let now = Utc::now();
    Ok(Self {
      id: Uuid::new_v4(),
      thesis: thesis_trimmed.to_string(),
      antithesis: antithesis_trimmed.to_string(),
      failure_modes: Vec::new(),
      validation_criteria: Vec::new(),
      created_at: now,
      updated_at: now,
    })
  }

  #[must_use]
  pub fn with_failure_mode(mut self, mode: &str) -> Self {
    let trimmed = mode.trim();
    if !trimmed.is_empty() {
      self.failure_modes.push(trimmed.to_string());
      self.updated_at = Utc::now();
    }
    self
  }

  #[must_use]
  pub fn with_validation_criterion(mut self, criterion: &str) -> Self {
    let trimmed = criterion.trim();
    if !trimmed.is_empty() {
      self.validation_criteria.push(trimmed.to_string());
      self.updated_at = Utc::now();
    }
    self
  }

  #[must_use]
  pub fn is_valid(&self) -> bool {
    !self.failure_modes.is_empty()
  }

  /// Validate the generator has at least one failure mode.
  ///
  /// # Errors
  /// Returns `ThesisAntithesisError::NoFailureModes` if no failure modes have been added.
  pub fn validate(&self) -> Result<(), ThesisAntithesisError> {
    if self.failure_modes.is_empty() {
      return Err(ThesisAntithesisError::NoFailureModes);
    }
    Ok(())
  }

  #[must_use]
  pub fn generate_antithesis_prompts() -> Vec<String> {
    vec![
      "What evidence would prove this thesis wrong?".to_string(),
      "Under what conditions would users reject this solution?".to_string(),
      "What assumptions might we be making that could fail?".to_string(),
      "Why might the market not exist for this product?".to_string(),
      "What could competitors do that would make this irrelevant?".to_string(),
      "What technical challenges could cause this to fail?".to_string(),
      "Why might users not want to pay for this?".to_string(),
      "What behavioral changes are required that users might resist?".to_string(),
    ]
  }

  #[must_use]
  pub const fn id(&self) -> Uuid {
    self.id
  }

  #[must_use]
  pub fn thesis(&self) -> &str {
    &self.thesis
  }

  #[must_use]
  pub fn antithesis(&self) -> &str {
    &self.antithesis
  }

  #[must_use]
  pub fn failure_modes(&self) -> &[String] {
    &self.failure_modes
  }

  #[must_use]
  pub fn validation_criteria(&self) -> &[String] {
    &self.validation_criteria
  }
}

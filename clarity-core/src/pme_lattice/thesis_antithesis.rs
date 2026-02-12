//! Thesis & Antithesis Generator (bead bd-16qs.1)
//!
//! Generates thesis/antithesis pairs for hypothesis testing in PME Discover phase.
//! Every thesis must have a corresponding antithesis to validate assumptions and
//! prevent optimism bias through dialectical reasoning.

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
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

// ============================================================================
// ERRORS
// ============================================================================

/// Errors for thesis/antithesis operations
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum ThesisAntithesisError {
  /// Statement cannot be empty or whitespace
  #[error("statement cannot be empty or whitespace")]
  EmptyStatement,

  /// Thesis and antithesis cannot be identical
  #[error("thesis and antithesis cannot be identical")]
  ThesisEqualsAntithesis,
}

// ============================================================================
// SYNTHESIS STATUS
// ============================================================================

/// Status of hypothesis synthesis after evaluation
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SynthesisStatus {
  /// Not yet evaluated
  #[default]
  Pending,
  /// Evidence supports the thesis
  ThesisSupported,
  /// Evidence supports the antithesis
  AntithesisSupported,
  /// Evidence is contradictory or insufficient
  Inconclusive,
  /// More data needed before synthesis
  RequiresMoreData,
}

impl fmt::Display for SynthesisStatus {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Pending => write!(f, "Pending"),
      Self::ThesisSupported => write!(f, "Thesis Supported"),
      Self::AntithesisSupported => write!(f, "Antithesis Supported"),
      Self::Inconclusive => write!(f, "Inconclusive"),
      Self::RequiresMoreData => write!(f, "Requires More Data"),
    }
  }
}

// ============================================================================
// THESIS
// ============================================================================

/// A testable proposition or hypothesis about a product decision.
///
/// Represents the positive assertion to be tested against evidence.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Thesis {
  /// Unique identifier
  id: Uuid,
  /// The thesis statement
  statement: String,
  /// Evidence supporting the thesis
  supporting_evidence: Vec<String>,
  /// When the thesis was created
  created_at: DateTime<Utc>,
}

impl Thesis {
  /// Create a new thesis.
  ///
  /// # Errors
  /// Returns `ThesisAntithesisError::EmptyStatement` if statement is empty or whitespace.
  pub fn new(statement: String) -> Result<Self, ThesisAntithesisError> {
    let trimmed = statement.trim();
    if trimmed.is_empty() {
      return Err(ThesisAntithesisError::EmptyStatement);
    }

    Ok(Self {
      id: Uuid::new_v4(),
      statement: trimmed.to_string(),
      supporting_evidence: Vec::new(),
      created_at: Utc::now(),
    })
  }

  /// Get the thesis ID.
  #[must_use]
  pub const fn id(&self) -> Uuid {
    self.id
  }

  /// Get the thesis statement.
  #[must_use]
  pub fn statement(&self) -> &str {
    &self.statement
  }

  /// Get supporting evidence.
  #[must_use]
  pub fn supporting_evidence(&self) -> &[String] {
    &self.supporting_evidence
  }

  /// Get creation timestamp.
  #[must_use]
  pub const fn created_at(&self) -> DateTime<Utc> {
    self.created_at
  }

  /// Add supporting evidence (returns new instance).
  #[must_use]
  pub fn add_evidence(self, evidence: String) -> Self {
    let trimmed = evidence.trim();
    if trimmed.is_empty() {
      return self;
    }

    Self {
      supporting_evidence: self
        .supporting_evidence
        .into_iter()
        .chain(std::iter::once(trimmed.to_string()))
        .collect(),
      ..self
    }
  }
}

// ============================================================================
// THESIS BUILDER
// ============================================================================

/// Builder for creating Thesis instances.
#[derive(Default)]
pub struct ThesisBuilder {
  statement: Option<String>,
  supporting_evidence: Vec<String>,
}

impl ThesisBuilder {
  /// Create a new thesis builder.
  #[must_use]
  pub fn new() -> Self {
    Self::default()
  }

  /// Set the thesis statement.
  #[must_use]
  pub fn statement(mut self, statement: String) -> Self {
    self.statement = Some(statement);
    self
  }

  /// Add supporting evidence.
  #[must_use]
  pub fn supporting_evidence(mut self, evidence: String) -> Self {
    let trimmed = evidence.trim();
    if !trimmed.is_empty() {
      self.supporting_evidence.push(trimmed.to_string());
    }
    self
  }

  /// Build the thesis.
  ///
  /// # Errors
  /// Returns `ThesisAntithesisError::EmptyStatement` if no statement was provided.
  pub fn build(self) -> Result<Thesis, ThesisAntithesisError> {
    let statement = self
      .statement
      .ok_or(ThesisAntithesisError::EmptyStatement)?;

    Thesis::new(statement).map(|thesis| {
      self
        .supporting_evidence
        .into_iter()
        .fold(thesis, |t, e| t.add_evidence(e))
    })
  }
}

// ============================================================================
// ANTITHESIS
// ============================================================================

/// A counter-proposition that challenges a thesis.
///
/// Represents the negative assertion designed to test the thesis's validity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Antithesis {
  /// Unique identifier
  id: Uuid,
  /// The counter-statement
  counter_statement: String,
  /// Evidence attacking the thesis
  attacking_evidence: Vec<String>,
  /// When the antithesis was created
  created_at: DateTime<Utc>,
}

impl Antithesis {
  /// Create a new antithesis.
  ///
  /// # Errors
  /// Returns `ThesisAntithesisError::EmptyStatement` if statement is empty or whitespace.
  pub fn new(counter_statement: String) -> Result<Self, ThesisAntithesisError> {
    let trimmed = counter_statement.trim();
    if trimmed.is_empty() {
      return Err(ThesisAntithesisError::EmptyStatement);
    }

    Ok(Self {
      id: Uuid::new_v4(),
      counter_statement: trimmed.to_string(),
      attacking_evidence: Vec::new(),
      created_at: Utc::now(),
    })
  }

  /// Get the antithesis ID.
  #[must_use]
  pub const fn id(&self) -> Uuid {
    self.id
  }

  /// Get the counter-statement.
  #[must_use]
  pub fn counter_statement(&self) -> &str {
    &self.counter_statement
  }

  /// Get attacking evidence.
  #[must_use]
  pub fn attacking_evidence(&self) -> &[String] {
    &self.attacking_evidence
  }

  /// Get creation timestamp.
  #[must_use]
  pub const fn created_at(&self) -> DateTime<Utc> {
    self.created_at
  }

  /// Add attacking evidence (returns new instance).
  #[must_use]
  pub fn add_evidence(self, evidence: String) -> Self {
    let trimmed = evidence.trim();
    if trimmed.is_empty() {
      return self;
    }

    Self {
      attacking_evidence: self
        .attacking_evidence
        .into_iter()
        .chain(std::iter::once(trimmed.to_string()))
        .collect(),
      ..self
    }
  }
}

// ============================================================================
// ANTITHESIS BUILDER
// ============================================================================

/// Builder for creating Antithesis instances.
#[derive(Default)]
pub struct AntithesisBuilder {
  counter_statement: Option<String>,
  attacking_evidence: Vec<String>,
}

impl AntithesisBuilder {
  /// Create a new antithesis builder.
  #[must_use]
  pub fn new() -> Self {
    Self::default()
  }

  /// Set the counter-statement.
  #[must_use]
  pub fn counter_statement(mut self, statement: String) -> Self {
    self.counter_statement = Some(statement);
    self
  }

  /// Add attacking evidence.
  #[must_use]
  pub fn attacking_evidence(mut self, evidence: String) -> Self {
    let trimmed = evidence.trim();
    if !trimmed.is_empty() {
      self.attacking_evidence.push(trimmed.to_string());
    }
    self
  }

  /// Build the antithesis.
  ///
  /// # Errors
  /// Returns `ThesisAntithesisError::EmptyStatement` if no statement was provided.
  pub fn build(self) -> Result<Antithesis, ThesisAntithesisError> {
    let statement = self
      .counter_statement
      .ok_or(ThesisAntithesisError::EmptyStatement)?;

    Antithesis::new(statement).map(|antithesis| {
      self
        .attacking_evidence
        .into_iter()
        .fold(antithesis, |a, e| a.add_evidence(e))
    })
  }
}

// ============================================================================
// HYPOTHESIS PAIR
// ============================================================================

/// A thesis/antithesis pair with synthesis notes.
///
/// Combines a thesis with its corresponding antithesis and tracks
/// the synthesis process and conclusions.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HypothesisPair {
  /// Unique identifier
  id: Uuid,
  /// The thesis
  thesis: Thesis,
  /// The antithesis
  antithesis: Antithesis,
  /// Notes from synthesis process
  synthesis_notes: Vec<String>,
  /// Current synthesis status
  synthesis_status: SynthesisStatus,
  /// When the pair was created
  created_at: DateTime<Utc>,
  /// When the pair was last updated
  updated_at: DateTime<Utc>,
}

impl HypothesisPair {
  /// Create a new hypothesis pair.
  ///
  /// # Errors
  /// Returns `ThesisAntithesisError::ThesisEqualsAntithesis` if thesis and antithesis
  /// are identical (case-insensitive, trimmed comparison).
  pub fn new(thesis: Thesis, antithesis: Antithesis) -> Result<Self, ThesisAntithesisError> {
    if thesis
      .statement()
      .eq_ignore_ascii_case(antithesis.counter_statement())
    {
      return Err(ThesisAntithesisError::ThesisEqualsAntithesis);
    }

    let now = Utc::now();
    Ok(Self {
      id: Uuid::new_v4(),
      thesis,
      antithesis,
      synthesis_notes: Vec::new(),
      synthesis_status: SynthesisStatus::Pending,
      created_at: now,
      updated_at: now,
    })
  }

  /// Get the pair ID.
  #[must_use]
  pub const fn id(&self) -> Uuid {
    self.id
  }

  /// Get the thesis.
  #[must_use]
  pub fn thesis(&self) -> &Thesis {
    &self.thesis
  }

  /// Get the antithesis.
  #[must_use]
  pub fn antithesis(&self) -> &Antithesis {
    &self.antithesis
  }

  /// Get synthesis notes.
  #[must_use]
  pub fn synthesis_notes(&self) -> &[String] {
    &self.synthesis_notes
  }

  /// Get synthesis status.
  #[must_use]
  pub const fn synthesis_status(&self) -> SynthesisStatus {
    self.synthesis_status
  }

  /// Get creation timestamp.
  #[must_use]
  pub const fn created_at(&self) -> DateTime<Utc> {
    self.created_at
  }

  /// Get last update timestamp.
  #[must_use]
  pub const fn updated_at(&self) -> DateTime<Utc> {
    self.updated_at
  }

  /// Add a synthesis note (returns new instance).
  #[must_use]
  pub fn add_synthesis_note(self, note: String) -> Self {
    let trimmed = note.trim();
    if trimmed.is_empty() {
      return self;
    }

    Self {
      synthesis_notes: self
        .synthesis_notes
        .into_iter()
        .chain(std::iter::once(trimmed.to_string()))
        .collect(),
      updated_at: Utc::now(),
      ..self
    }
  }

  /// Set synthesis status (returns new instance).
  #[must_use]
  pub fn with_status(self, status: SynthesisStatus) -> Self {
    Self {
      synthesis_status: status,
      updated_at: Utc::now(),
      ..self
    }
  }
}

// ============================================================================
// HYPOTHESIS PAIR BUILDER
// ============================================================================

/// Builder for creating HypothesisPair instances.
#[derive(Default)]
pub struct HypothesisPairBuilder {
  thesis_statement: Option<String>,
  thesis_evidence: Vec<String>,
  antithesis_statement: Option<String>,
  antithesis_evidence: Vec<String>,
  synthesis_notes: Vec<String>,
  synthesis_status: SynthesisStatus,
}

impl HypothesisPairBuilder {
  /// Create a new hypothesis pair builder.
  #[must_use]
  pub fn new() -> Self {
    Self::default()
  }

  /// Set the thesis statement.
  #[must_use]
  pub fn thesis_statement(mut self, statement: String) -> Self {
    self.thesis_statement = Some(statement);
    self
  }

  /// Add thesis evidence.
  #[must_use]
  pub fn thesis_evidence(mut self, evidence: String) -> Self {
    let trimmed = evidence.trim();
    if !trimmed.is_empty() {
      self.thesis_evidence.push(trimmed.to_string());
    }
    self
  }

  /// Set the antithesis statement.
  #[must_use]
  pub fn antithesis_statement(mut self, statement: String) -> Self {
    self.antithesis_statement = Some(statement);
    self
  }

  /// Add antithesis evidence.
  #[must_use]
  pub fn antithesis_evidence(mut self, evidence: String) -> Self {
    let trimmed = evidence.trim();
    if !trimmed.is_empty() {
      self.antithesis_evidence.push(trimmed.to_string());
    }
    self
  }

  /// Add a synthesis note.
  #[must_use]
  pub fn synthesis_note(mut self, note: String) -> Self {
    let trimmed = note.trim();
    if !trimmed.is_empty() {
      self.synthesis_notes.push(trimmed.to_string());
    }
    self
  }

  /// Set synthesis status.
  #[must_use]
  pub fn synthesis_status(mut self, status: SynthesisStatus) -> Self {
    self.synthesis_status = status;
    self
  }

  /// Build the hypothesis pair.
  ///
  /// # Errors
  /// Returns `ThesisAntithesisError::EmptyStatement` if thesis or antithesis statement is missing.
  /// Returns `ThesisAntithesisError::ThesisEqualsAntithesis` if thesis and antithesis are identical.
  pub fn build(self) -> Result<HypothesisPair, ThesisAntithesisError> {
    let thesis = ThesisBuilder::new()
      .statement(
        self
          .thesis_statement
          .ok_or(ThesisAntithesisError::EmptyStatement)?,
      )
      .build()?;

    let thesis = self
      .thesis_evidence
      .into_iter()
      .fold(thesis, |t, e| t.add_evidence(e));

    let antithesis = AntithesisBuilder::new()
      .counter_statement(
        self
          .antithesis_statement
          .ok_or(ThesisAntithesisError::EmptyStatement)?,
      )
      .build()?;

    let antithesis = self
      .antithesis_evidence
      .into_iter()
      .fold(antithesis, |a, e| a.add_evidence(e));

    let pair = HypothesisPair::new(thesis, antithesis)?;

    let pair = self
      .synthesis_notes
      .into_iter()
      .fold(pair, |p, n| p.add_synthesis_note(n));

    Ok(pair.with_status(self.synthesis_status))
  }
}

// ============================================================================
// THESIS ANTITHESIS GENERATOR
// ============================================================================

/// Generator for creating and managing thesis/antithesis pairs.
///
/// Provides a collection of hypothesis pairs with filtering and analysis capabilities.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThesisAntithesisGenerator {
  /// Collection of hypothesis pairs
  pairs: Vec<HypothesisPair>,
}

impl ThesisAntithesisGenerator {
  /// Create a new empty generator.
  #[must_use]
  pub fn new() -> Self {
    Self::default()
  }

  /// Get all hypothesis pairs.
  #[must_use]
  pub fn pairs(&self) -> &[HypothesisPair] {
    &self.pairs
  }

  /// Create a new hypothesis pair.
  ///
  /// # Errors
  /// Returns `ThesisAntithesisError::EmptyStatement` if either statement is empty.
  /// Returns `ThesisAntithesisError::ThesisEqualsAntithesis` if statements are identical.
  pub fn create_pair(
    &self,
    thesis_statement: String,
    antithesis_statement: String,
  ) -> Result<HypothesisPair, ThesisAntithesisError> {
    HypothesisPairBuilder::new()
      .thesis_statement(thesis_statement)
      .antithesis_statement(antithesis_statement)
      .build()
  }

  /// Add a hypothesis pair (returns new instance).
  #[must_use]
  pub fn add_pair(self, pair: HypothesisPair) -> Self {
    Self {
      pairs: self
        .pairs
        .into_iter()
        .chain(std::iter::once(pair))
        .collect(),
    }
  }

  /// Get prompts to help generate antithesis statements.
  #[must_use]
  pub fn antithesis_prompts() -> Vec<String> {
    vec![
      "What evidence would prove this thesis wrong?".to_string(),
      "Under what conditions would users reject this solution?".to_string(),
      "What assumptions might we be making that could fail?".to_string(),
      "Why might the market not exist for this product?".to_string(),
      "What could competitors do that would make this irrelevant?".to_string(),
      "What technical challenges could cause this to fail?".to_string(),
      "Why might users not want to pay for this?".to_string(),
      "What behavioral changes are required that users might resist?".to_string(),
      "What would need to be true for the thesis to fail?".to_string(),
      "Who would disagree with this thesis and why?".to_string(),
    ]
  }

  /// Filter pairs by synthesis status.
  #[must_use]
  pub fn filter_by_status(&self, status: SynthesisStatus) -> Vec<&HypothesisPair> {
    self
      .pairs
      .iter()
      .filter(|pair| pair.synthesis_status() == status)
      .collect()
  }

  /// Count pairs by synthesis status.
  #[must_use]
  pub fn count_by_status(&self, status: SynthesisStatus) -> usize {
    self
      .pairs
      .iter()
      .filter(|pair| pair.synthesis_status() == status)
      .count()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn synthesis_status_display_works() {
    assert_eq!(SynthesisStatus::Pending.to_string(), "Pending");
    assert_eq!(
      SynthesisStatus::ThesisSupported.to_string(),
      "Thesis Supported"
    );
    assert_eq!(
      SynthesisStatus::AntithesisSupported.to_string(),
      "Antithesis Supported"
    );
    assert_eq!(SynthesisStatus::Inconclusive.to_string(), "Inconclusive");
    assert_eq!(
      SynthesisStatus::RequiresMoreData.to_string(),
      "Requires More Data"
    );
  }
}

#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(
  clippy::missing_const_for_fn,
  clippy::manual_let_else,
  clippy::match_wild_err_arm,
  clippy::match_like_matches_macro
)]
#![allow(clippy::derive_partial_eq_without_eq)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! `TranscriptStore` trait for persistence abstraction.
//!
//! Provides a trait for storing, retrieving, and managing conversation transcripts
//! with support for incremental updates and atomic operations.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::StorageError;
pub use crate::components::discover::straw_man::{StrawManTrap, StrawManValidation};
pub use crate::components::discover::types::{HolePunchingResults, ScenarioField};

/// Result type for transcript store operations.
pub type TranscriptResult<T> = Result<T, StorageError>;

/// Trait for transcript persistence operations.
///
/// Abstracts storage backend to allow different implementations
/// (redb, `SQLite`, cloud storage, etc.) while maintaining a consistent interface.
///
/// All implementations must provide ACID guarantees:
/// - **Atomicity**: Operations complete fully or not at all
/// - **Consistency**: Data remains valid across operations
/// - **Isolation**: Concurrent operations don't interfere
/// - **Durability**: Committed data survives crashes
#[async_trait]
pub trait TranscriptStore: Send + Sync {
  /// Save a transcript with the given session ID.
  ///
  /// If a transcript already exists for this session, it will be overwritten.
  ///
  /// # Errors
  ///
  /// Returns `StorageError` if:
  /// - Serialization fails
  /// - Database write fails
  /// - Transaction cannot be committed
  async fn save(
    &self,
    session_id: &str,
    transcript: &InterrogationTranscript,
  ) -> TranscriptResult<()>;

  /// Load a transcript by session ID.
  ///
  /// # Returns
  ///
  /// - `Ok(Some(transcript))` if found
  /// - `Ok(None)` if no transcript exists for this session
  ///
  /// # Errors
  ///
  /// Returns `StorageError` if:
  /// - Deserialization fails
  /// - Database read fails
  async fn load(&self, session_id: &str) -> TranscriptResult<Option<InterrogationTranscript>>;

  /// Delete a transcript by session ID.
  ///
  /// # Returns
  ///
  /// Returns `Ok(())` whether or not the transcript existed.
  ///
  /// # Errors
  ///
  /// Returns `StorageError` if the database operation fails.
  async fn delete(&self, session_id: &str) -> TranscriptResult<()>;

  /// List all session IDs in the store.
  ///
  /// # Errors
  ///
  /// Returns `StorageError` if the database operation fails.
  async fn list_sessions(&self) -> TranscriptResult<Vec<String>>;
}

/// Extracted field from AI processing.
///
/// Represents a single piece of information extracted from user input,
/// along with confidence level and metadata.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExtractedField {
  /// The extracted content
  pub content: String,
  /// Confidence level (0.0 = uncertain, 1.0 = very confident)
  pub confidence: f64,
  /// Source of the extraction (e.g., "ai", "user")
  pub source: String,
  /// ISO 8601 timestamp when this field was extracted
  pub extracted_at: String,
}

impl ExtractedField {
  /// Create a new extracted field.
  #[must_use]
  pub fn new(content: String, confidence: f64, source: String) -> Self {
    Self {
      content,
      confidence: confidence.clamp(0.0, 1.0),
      source,
      extracted_at: chrono::Utc::now().to_rfc3339(),
    }
  }

  /// Create an empty field with default values.
  #[must_use]
  pub fn empty() -> Self {
    Self {
      content: String::new(),
      confidence: 0.0,
      source: String::new(),
      extracted_at: chrono::Utc::now().to_rfc3339(),
    }
  }
}

impl Default for ExtractedField {
  fn default() -> Self {
    Self::empty()
  }
}

/// Antithesis response containing null hypothesis points.
///
/// Part of the adversarial reasoning process that challenges
/// the initial problem statement with counter-arguments.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AntithesisResponse {
  /// Exactly 3 null hypothesis points
  pub points: [String; 3],
  /// Quality score (0-1, higher = more specific and actionable)
  pub quality_score: f64,
}

impl AntithesisResponse {
  /// Create a new antithesis response.
  #[must_use]
  pub const fn new(point1: String, point2: String, point3: String, quality_score: f64) -> Self {
    Self {
      points: [point1, point2, point3],
      quality_score: quality_score.clamp(0.0, 1.0),
    }
  }

  /// Create an empty/default antithesis response.
  #[must_use]
  pub const fn empty() -> Self {
    Self {
      points: [String::new(), String::new(), String::new()],
      quality_score: 0.0,
    }
  }
}

impl Default for AntithesisResponse {
  fn default() -> Self {
    Self::empty()
  }
}

/// Main transcript structure for the interrogation flow.
///
/// Contains all extracted fields, adversarial responses, and metadata
/// for a complete Progressive Discover session.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InterrogationTranscript {
  /// Original user input prompt
  pub original_prompt: String,

  /// AI-extracted problem statement
  pub problem: ExtractedField,

  /// Adversarial antithesis response
  pub antithesis: AntithesisResponse,

  /// AI-extracted target persona
  pub persona: ExtractedField,

  /// Straw man validation results
  pub straw_man_validation: StrawManValidation,

  /// AI-extracted solution description
  pub solution: ExtractedField,

  /// VORP (Value of Risk Reduction) justification
  pub vorp_justification: String,

  /// AI-extracted non-persona (who the solution is NOT for)
  pub nonpersona: ExtractedField,

  /// User scenario with emotional context
  pub scenario: ScenarioField,

  /// When the transcript was started
  pub started_at: DateTime<Utc>,

  /// When the transcript was completed (if applicable)
  pub completed_at: Option<DateTime<Utc>>,
}

impl InterrogationTranscript {
  /// Create a new transcript from an initial prompt.
  #[must_use]
  pub fn from_prompt(original_prompt: String) -> Self {
    Self {
      original_prompt,
      problem: ExtractedField::empty(),
      antithesis: AntithesisResponse::empty(),
      persona: ExtractedField::empty(),
      straw_man_validation: StrawManValidation::passing(),
      solution: ExtractedField::empty(),
      vorp_justification: String::new(),
      nonpersona: ExtractedField::empty(),
      scenario: ScenarioField::empty(),
      started_at: chrono::Utc::now(),
      completed_at: None,
    }
  }

  /// Mark the transcript as completed.
  #[must_use]
  pub fn complete(self) -> Self {
    Self {
      completed_at: Some(chrono::Utc::now()),
      ..self
    }
  }

  /// Check if the transcript is completed.
  #[must_use]
  pub const fn is_completed(&self) -> bool {
    self.completed_at.is_some()
  }
}

impl Default for InterrogationTranscript {
  fn default() -> Self {
    Self::from_prompt(String::new())
  }
}

#[cfg(test)]
#[allow(
  clippy::unwrap_used,
  clippy::expect_used,
  clippy::panic,
  clippy::float_cmp,
  clippy::needless_collect,
  clippy::unnecessary_debug_formatting,
  clippy::match_same_arms,
  clippy::option_if_let_else,
  clippy::suspicious_else_formatting,
  clippy::manual_let_else,
  clippy::match_wild_err_arm,
  clippy::match_like_matches_macro,
  clippy::needless_pass_by_value
)]
mod tests {
  use super::*;

  fn assert_approx_eq(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-9);
  }

  #[test]
  fn test_extracted_field_creation() {
    let field = ExtractedField::new("Test content".to_string(), 0.8, "ai".to_string());

    assert_eq!(field.content, "Test content");
    assert_approx_eq(field.confidence, 0.8);
    assert_eq!(field.source, "ai");
  }

  #[test]
  fn test_extracted_field_confidence_clamped() {
    let field = ExtractedField::new(
      "content".to_string(),
      1.5, // Over max
      "ai".to_string(),
    );
    assert_approx_eq(field.confidence, 1.0);

    let field = ExtractedField::new(
      "content".to_string(),
      -0.5, // Under min
      "ai".to_string(),
    );
    assert_approx_eq(field.confidence, 0.0);
  }

  #[test]
  fn test_extracted_field_default() {
    let field = ExtractedField::default();
    assert!(field.content.is_empty());
    assert_approx_eq(field.confidence, 0.0);
  }

  #[test]
  fn test_antithesis_response_creation() {
    let response = AntithesisResponse::new(
      "Point 1".to_string(),
      "Point 2".to_string(),
      "Point 3".to_string(),
      0.75,
    );

    assert_eq!(response.points[0], "Point 1");
    assert_eq!(response.points[1], "Point 2");
    assert_eq!(response.points[2], "Point 3");
    assert_approx_eq(response.quality_score, 0.75);
  }

  #[test]
  fn test_antithesis_response_quality_clamped() {
    let response = AntithesisResponse::new("A".to_string(), "B".to_string(), "C".to_string(), 2.0);
    assert_approx_eq(response.quality_score, 1.0);
  }

  #[test]
  fn test_straw_man_validation_passing() {
    let validation = StrawManValidation::passing();
    assert!(validation.passed);
    assert!(validation.traps_detected.is_empty());
  }

  #[test]
  fn test_straw_man_validation_with_traps() {
    let validation =
      StrawManValidation::new(vec![StrawManTrap::IrrationalActor, StrawManTrap::YourClone]);

    assert!(!validation.passed);
    assert_eq!(validation.traps_detected.len(), 2);
  }

  #[test]
  fn test_hole_punching_results() {
    let results = HolePunchingResults::new();

    assert!(results.discovery_hole.is_none());
    assert!(results.edge_case_hole.is_none());
    assert!(results.motivation_dropoff.is_none());
    assert!(!results.is_complete());
  }

  #[test]
  fn test_scenario_field_creation() {
    let scenario = ScenarioField::new(
      "User opens app".to_string(),
      "Gets instant result".to_string(),
      "Delighted".to_string(),
    );

    assert_eq!(scenario.trigger, "User opens app");
    assert_eq!(scenario.value_moment, "Gets instant result");
    assert_eq!(scenario.feeling, "Delighted");
  }

  #[test]
  fn test_interrogation_transcript_from_prompt() {
    let transcript = InterrogationTranscript::from_prompt("I want to build a todo app".to_string());

    assert_eq!(transcript.original_prompt, "I want to build a todo app");
    assert!(transcript.problem.content.is_empty());
    assert!(transcript.started_at <= chrono::Utc::now());
    assert!(transcript.completed_at.is_none());
    assert!(!transcript.is_completed());
  }

  #[test]
  fn test_interrogation_transcript_complete() {
    let transcript = InterrogationTranscript::from_prompt("Test".to_string());
    let completed = transcript.complete();

    assert!(completed.is_completed());
    assert!(completed.completed_at.is_some());
  }

  #[test]
  fn test_interrogation_transcript_serialization() -> Result<(), serde_json::Error> {
    let transcript = InterrogationTranscript::from_prompt("Build a fitness tracker".to_string());

    let json = serde_json::to_string(&transcript)?;
    let deserialized: InterrogationTranscript = serde_json::from_str(&json)?;

    assert_eq!(deserialized.original_prompt, "Build a fitness tracker");
    Ok(())
  }

  #[test]
  fn test_straw_man_trap_serialization() -> Result<(), serde_json::Error> {
    let trap = StrawManTrap::ManicPixieDreamUser;
    let json = serde_json::to_string(&trap)?;
    assert_eq!(json, "\"ManicPixieDreamUser\"");
    Ok(())
  }

  #[test]
  fn test_full_transcript_round_trip() -> Result<(), serde_json::Error> {
    let mut transcript =
      InterrogationTranscript::from_prompt("I want to build a meditation app".to_string());

    transcript.problem = ExtractedField::new(
      "Users struggle to maintain meditation habits".to_string(),
      0.9,
      "ai".to_string(),
    );
    transcript.persona = ExtractedField::new(
      "Busy professionals aged 25-40".to_string(),
      0.85,
      "ai".to_string(),
    );
    transcript.solution = ExtractedField::new(
      "Micro-meditation app with reminders".to_string(),
      0.8,
      "ai".to_string(),
    );

    let json = serde_json::to_string(&transcript)?;
    let restored: InterrogationTranscript = serde_json::from_str(&json)?;

    assert_eq!(
      restored.problem.content,
      "Users struggle to maintain meditation habits"
    );
    assert_approx_eq(restored.persona.confidence, 0.85);
    assert_eq!(restored.solution.source, "ai");
    Ok(())
  }
}

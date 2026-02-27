#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Confidence level for an answer or extracted information
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
  /// High confidence - explicitly provided by user or verified
  High,
  /// Inferred confidence - derived from context or patterns
  Inferred,
  /// Uncertain confidence - low confidence or requires validation
  Uncertain,
}

/// Answer record stored in the database
/// Extends the core Answer type with additional metadata
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnswerRecord {
  /// Unique identifier for the prompt step
  pub step_id: String,
  /// The answer value
  pub value: String,
  /// ISO 8601 timestamp when answer was recorded
  pub timestamp: String,
  /// Confidence level of this answer
  pub confidence: Confidence,
  /// Whether this answer was AI-generated (vs user-provided)
  pub ai_generated: bool,
}

impl AnswerRecord {
  /// Create a new answer record
  #[must_use]
  pub const fn new(
    step_id: String,
    value: String,
    timestamp: String,
    confidence: Confidence,
    ai_generated: bool,
  ) -> Self {
    Self {
      step_id,
      value,
      timestamp,
      confidence,
      ai_generated,
    }
  }

  /// Create from existing answer with default metadata
  #[must_use]
  pub fn from_answer(step_id: String, value: String, timestamp: String) -> Self {
    Self {
      step_id,
      value,
      timestamp,
      confidence: Confidence::High,
      ai_generated: false,
    }
  }
}

/// Cache for field extraction results
/// Maps input hash to extracted fields to avoid redundant processing
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExtractionCache {
  /// Hash of the input text for cache lookup
  pub input_hash: String,
  /// Extracted fields as JSON object
  pub fields: String,
  /// ISO 8601 timestamp when cache entry was created
  pub timestamp: String,
}

impl ExtractionCache {
  /// Create a new extraction cache entry
  #[must_use]
  pub const fn new(input_hash: String, fields: String, timestamp: String) -> Self {
    Self {
      input_hash,
      fields,
      timestamp,
    }
  }
}

/// Project metadata persisted across sessions
/// Tracks project state and user preferences
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectMetadata {
  /// User's preferred planning mode (e.g., "waterfall", "agile")
  pub mode_preference: String,
  /// Current phase in the planning process
  pub current_phase: String,
  /// ISO 8601 timestamp when project was created
  pub created_at: String,
  /// ISO 8601 timestamp when project was last updated
  pub updated_at: String,
}

impl ProjectMetadata {
  /// Create new project metadata
  #[must_use]
  pub const fn new(
    mode_preference: String,
    current_phase: String,
    created_at: String,
    updated_at: String,
  ) -> Self {
    Self {
      mode_preference,
      current_phase,
      created_at,
      updated_at,
    }
  }

  /// Create with current timestamp for created and updated
  #[must_use]
  pub fn with_current_timestamp(mode_preference: String, current_phase: String) -> Self {
    let now = chrono::Utc::now().to_rfc3339();
    Self {
      mode_preference,
      current_phase,
      created_at: now.clone(),
      updated_at: now,
    }
  }
}

/// Cache for lattice graph computation results
/// Stores phase-specific lattice outputs to avoid recomputation
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LatticeCache {
  /// Phase identifier (e.g., "discover", "define")
  pub phase: String,
  /// Serialized lattice output data
  pub output_data: String,
  /// ISO 8601 timestamp when cache entry was created
  pub timestamp: String,
}

impl LatticeCache {
  /// Create a new lattice cache entry
  #[must_use]
  pub const fn new(phase: String, output_data: String, timestamp: String) -> Self {
    Self {
      phase,
      output_data,
      timestamp,
    }
  }

  /// Create with current timestamp
  #[must_use]
  pub fn with_current_timestamp(phase: String, output_data: String) -> Self {
    Self {
      phase,
      output_data,
      timestamp: chrono::Utc::now().to_rfc3339(),
    }
  }
}

/// Redb table definition constants
/// These define the table names used in the database
pub mod tables {
  /// Table name for AnswerRecord storage
  pub const ANSWERS: &str = "answers";

  /// Table name for ExtractionCache storage
  pub const EXTRACTIONS: &str = "extractions";

  /// Table name for ProjectMetadata storage
  pub const PROJECT_METADATA: &str = "project_metadata";

  /// Table name for LatticeCache storage
  pub const LATTICE_CACHE: &str = "lattice_cache";
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_confidence_serialization() {
    // Test serialization
    let high_json = serde_json::to_string(&Confidence::High).expect("failed to serialize High");
    assert_eq!(high_json, "\"high\"");

    let inferred_json =
      serde_json::to_string(&Confidence::Inferred).expect("failed to serialize Inferred");
    assert_eq!(inferred_json, "\"inferred\"");

    let uncertain_json =
      serde_json::to_string(&Confidence::Uncertain).expect("failed to serialize Uncertain");
    assert_eq!(uncertain_json, "\"uncertain\"");

    // Test deserialization
    let high: Confidence = serde_json::from_str("\"high\"").expect("failed to deserialize High");
    assert_eq!(high, Confidence::High);

    let inferred: Confidence =
      serde_json::from_str("\"inferred\"").expect("failed to deserialize Inferred");
    assert_eq!(inferred, Confidence::Inferred);

    let uncertain: Confidence =
      serde_json::from_str("\"uncertain\"").expect("failed to deserialize Uncertain");
    assert_eq!(uncertain, Confidence::Uncertain);
  }

  #[test]
  fn test_answer_record_serialization() {
    let record = AnswerRecord::new(
      "step-1".to_string(),
      "Test answer".to_string(),
      "2024-02-25T12:00:00Z".to_string(),
      Confidence::High,
      false,
    );

    let json = serde_json::to_string(&record).expect("failed to serialize AnswerRecord");

    let deserialized: AnswerRecord =
      serde_json::from_str(&json).expect("failed to deserialize AnswerRecord");

    assert_eq!(deserialized, record);
  }

  #[test]
  fn test_answer_record_from_answer() {
    let record = AnswerRecord::from_answer(
      "step-2".to_string(),
      "User provided answer".to_string(),
      "2024-02-25T12:00:00Z".to_string(),
    );

    assert_eq!(record.step_id, "step-2");
    assert_eq!(record.value, "User provided answer");
    assert_eq!(record.confidence, Confidence::High);
    assert!(!record.ai_generated);
  }

  #[test]
  fn test_extraction_cache_serialization() {
    let cache = ExtractionCache::new(
      "hash-123".to_string(),
      r#"{"field": "value"}"#.to_string(),
      "2024-02-25T12:00:00Z".to_string(),
    );

    let json = serde_json::to_string(&cache).expect("failed to serialize ExtractionCache");

    let deserialized: ExtractionCache =
      serde_json::from_str(&json).expect("failed to deserialize ExtractionCache");

    assert_eq!(deserialized, cache);
  }

  #[test]
  fn test_project_metadata_serialization() {
    let metadata = ProjectMetadata::new(
      "agile".to_string(),
      "discover".to_string(),
      "2024-02-25T10:00:00Z".to_string(),
      "2024-02-25T12:00:00Z".to_string(),
    );

    let json = serde_json::to_string(&metadata).expect("failed to serialize ProjectMetadata");

    let deserialized: ProjectMetadata =
      serde_json::from_str(&json).expect("failed to deserialize ProjectMetadata");

    assert_eq!(deserialized, metadata);
  }

  #[test]
  fn test_project_metadata_with_current_timestamp() {
    let before = chrono::Utc::now();
    let metadata =
      ProjectMetadata::with_current_timestamp("waterfall".to_string(), "define".to_string());
    let after = chrono::Utc::now();

    assert_eq!(metadata.mode_preference, "waterfall");
    assert_eq!(metadata.current_phase, "define");

    // Parse timestamps to verify they're within expected range
    let created = metadata
      .created_at
      .parse::<chrono::DateTime<chrono::Utc>>()
      .expect("failed to parse created_at");
    let updated = metadata
      .updated_at
      .parse::<chrono::DateTime<chrono::Utc>>()
      .expect("failed to parse updated_at");

    assert!(created >= before && created <= after);
    assert!(updated >= before && updated <= after);
  }

  #[test]
  fn test_lattice_cache_serialization() {
    let cache = LatticeCache::new(
      "discover".to_string(),
      r#"{"nodes": [], "edges": []}"#.to_string(),
      "2024-02-25T12:00:00Z".to_string(),
    );

    let json = serde_json::to_string(&cache).expect("failed to serialize LatticeCache");

    let deserialized: LatticeCache =
      serde_json::from_str(&json).expect("failed to deserialize LatticeCache");

    assert_eq!(deserialized, cache);
  }

  #[test]
  fn test_lattice_cache_with_current_timestamp() {
    let before = chrono::Utc::now();
    let cache = LatticeCache::with_current_timestamp(
      "develop".to_string(),
      r#"{"output": "data"}"#.to_string(),
    );
    let after = chrono::Utc::now();

    assert_eq!(cache.phase, "develop");
    assert_eq!(cache.output_data, r#"{"output": "data"}"#);

    // Parse timestamp to verify it's within expected range
    let timestamp = cache
      .timestamp
      .parse::<chrono::DateTime<chrono::Utc>>()
      .expect("failed to parse timestamp");

    assert!(timestamp >= before && timestamp <= after);
  }

  #[test]
  fn test_confidence_equality() {
    assert_eq!(Confidence::High, Confidence::High);
    assert_eq!(Confidence::Inferred, Confidence::Inferred);
    assert_eq!(Confidence::Uncertain, Confidence::Uncertain);

    assert_ne!(Confidence::High, Confidence::Inferred);
    assert_ne!(Confidence::Inferred, Confidence::Uncertain);
    assert_ne!(Confidence::High, Confidence::Uncertain);
  }

  #[test]
  fn test_answer_record_with_ai_generated() {
    let ai_record = AnswerRecord::new(
      "step-3".to_string(),
      "AI suggested answer".to_string(),
      "2024-02-25T12:00:00Z".to_string(),
      Confidence::Inferred,
      true,
    );

    assert!(ai_record.ai_generated);
    assert_eq!(ai_record.confidence, Confidence::Inferred);

    let json = serde_json::to_string(&ai_record).expect("failed to serialize AI-generated record");

    let deserialized: AnswerRecord =
      serde_json::from_str(&json).expect("failed to deserialize AI-generated record");

    assert_eq!(deserialized, ai_record);
    assert!(deserialized.ai_generated);
  }

  #[test]
  fn test_table_constants() {
    assert_eq!(tables::ANSWERS, "answers");
    assert_eq!(tables::EXTRACTIONS, "extractions");
    assert_eq!(tables::PROJECT_METADATA, "project_metadata");
    assert_eq!(tables::LATTICE_CACHE, "lattice_cache");
  }
}

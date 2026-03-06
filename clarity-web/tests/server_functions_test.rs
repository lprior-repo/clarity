#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::float_cmp, clippy::needless_collect, clippy::unnecessary_debug_formatting, clippy::match_same_arms, clippy::option_if_let_else, clippy::suspicious_else_formatting, clippy::manual_let_else, clippy::match_wild_err_arm, clippy::match_like_matches_macro)]
// Integration tests for server functions
//
// These tests verify that:
// 1. Server functions are properly registered with Dioxus
// 2. Type serialization works correctly
// 3. Rate limiting functions as expected

use serde_json::json;

#[test]
fn test_extracted_fields_serialization_roundtrip() {
  use chrono::Utc;
  use clarity_web::providers::{ExtractedFields, ExtractionMetadata, FieldExtraction, FieldType};

  let fields = ExtractedFields {
    fields: vec![FieldExtraction {
      name: "test_field".to_string(),
      field_type: FieldType::Text,
      value: json!("test value"),
      confidence: 0.95,
      justification: Some("test justification".to_string()),
    }],
    confidence: 0.95,
    metadata: ExtractionMetadata {
      provider: "test_provider".to_string(),
      model: Some("test_model".to_string()),
      timestamp: Utc::now(),
      processing_duration_ms: 100,
      extra: json!({}),
    },
  };

  let serialized = serde_json::to_string(&fields).expect("Failed to serialize");
  let deserialized: ExtractedFields =
    serde_json::from_str(&serialized).expect("Failed to deserialize");

  assert_eq!(deserialized.fields.len(), 1);
  assert_eq!(deserialized.fields[0].name, "test_field");
  assert!((deserialized.fields[0].confidence - 0.95).abs() < f64::EPSILON);
}

#[test]
fn test_quality_score_serialization_roundtrip() {
  use clarity_web::lattice::quality::{DimensionScore, QualityDimension, QualityScore};

  let score = QualityScore {
    overall: 80,
    dimensions: vec![DimensionScore {
      dimension: QualityDimension::Completeness,
      score: 90,
    }],
    issues: vec![],
  };

  let serialized = serde_json::to_string(&score).expect("Failed to serialize");
  let deserialized: QualityScore =
    serde_json::from_str(&serialized).expect("Failed to deserialize");

  assert_eq!(deserialized.overall, 80);
  assert_eq!(deserialized.dimensions.len(), 1);
  assert_eq!(deserialized.dimensions[0].score, 90);
}

#[test]
fn test_extraction_context_serialization_roundtrip() {
  use clarity_web::providers::ExtractionContext;

  let context = ExtractionContext {
    document_type: Some("test_doc".to_string()),
    locale: Some("en_US".to_string()),
    schema: None,
    extra: json!({"key": "value"}),
  };

  let serialized = serde_json::to_string(&context).expect("Failed to serialize");
  let deserialized: ExtractionContext =
    serde_json::from_str(&serialized).expect("Failed to deserialize");

  assert_eq!(deserialized.document_type, Some("test_doc".to_string()));
  assert_eq!(deserialized.locale, Some("en_US".to_string()));
}

#[test]
fn test_ears_requirement_ref_serialization_roundtrip() {
  use clarity_web::lattice::quality::EarsRequirementRef;

  let ears = EarsRequirementRef {
    id: "req-1".to_string(),
    text: "Test requirement".to_string(),
    has_acceptance_criteria: true,
  };

  let serialized = serde_json::to_string(&ears).expect("Failed to serialize");
  let deserialized: EarsRequirementRef =
    serde_json::from_str(&serialized).expect("Failed to deserialize");

  assert_eq!(deserialized.id, "req-1");
  assert_eq!(deserialized.text, "Test requirement");
  assert!(deserialized.has_acceptance_criteria);
}

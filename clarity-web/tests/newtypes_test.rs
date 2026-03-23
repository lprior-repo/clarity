//! Test for AnswerId newtype
//!
//! This test file drives the implementation of validated newtypes for domain identifiers.

/// Test that AnswerId exists and can be constructed from a valid string.
#[test]
fn answer_id_can_be_constructed_from_valid_string() {
  use clarity_web::domain::AnswerId;

  let id = AnswerId::new("answer-123".to_string());
  assert!(
    id.is_ok(),
    "AnswerId should accept a valid non-empty string"
  );

  let answer_id = id.expect("valid AnswerId");
  assert_eq!(answer_id.as_str(), "answer-123");
}

/// Test that AnswerId rejects empty strings.
#[test]
fn answer_id_rejects_empty_string() {
  use clarity_web::domain::AnswerId;

  let result = AnswerId::new(String::new());
  assert!(result.is_err(), "AnswerId should reject empty string");
}

/// Test that StepId can be constructed from a valid string.
#[test]
fn step_id_can_be_constructed_from_valid_string() {
  use clarity_web::domain::StepId;

  let id = StepId::new("step-456".to_string());
  assert!(id.is_ok(), "StepId should accept a valid non-empty string");

  let step_id = id.expect("valid StepId");
  assert_eq!(step_id.as_str(), "step-456");
}

/// Test that BeadId can be constructed from a valid string.
#[test]
fn bead_id_can_be_constructed_from_valid_string() {
  use clarity_web::domain::BeadId;

  let id = BeadId::new("bead-789".to_string());
  assert!(id.is_ok(), "BeadId should accept a valid non-empty string");

  let bead_id = id.expect("valid BeadId");
  assert_eq!(bead_id.as_str(), "bead-789");
}

/// Test that AnswerValue can be empty.
#[test]
fn answer_value_can_be_empty() {
  use clarity_web::domain::AnswerValue;

  let v = AnswerValue::new(String::new());
  assert!(v.is_empty(), "AnswerValue should allow empty strings");
}

/// Test that Timestamp validates ISO-8601 format.
#[test]
fn timestamp_accepts_valid_iso8601() {
  use clarity_web::domain::Timestamp;

  let ts = Timestamp::new("2024-03-10T12:30:45+00:00".to_string());
  assert!(ts.is_ok(), "Timestamp should accept valid ISO-8601");
}

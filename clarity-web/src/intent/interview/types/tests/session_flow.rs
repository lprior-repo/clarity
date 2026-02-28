use std::collections::HashMap;

use crate::intent::interview::types::{
  Answer, InterviewSession, InterviewSessionError, InterviewStage, Perspective, Profile,
};

fn make_session() -> InterviewSession {
  InterviewSession::new(
    "test".to_string(),
    Profile::Api,
    "2026-02-27T00:00:00Z".to_string(),
  )
}

fn answer(question_id: &str, round: u32) -> Answer {
  Answer {
    question_id: question_id.to_string(),
    question_text: "What?".to_string(),
    perspective: Perspective::User,
    round,
    response: "Answer".to_string(),
    extracted: HashMap::new(),
    confidence: 0.9,
    notes: String::new(),
    timestamp: "2026-02-27T00:01:00Z".to_string(),
  }
}

#[test]
fn add_answer_validation() {
  let mut session = make_session();
  assert!(session
    .add_answer(answer("q1", 1), "2026-02-27T00:01:00Z")
    .is_ok());
  assert_eq!(session.answers.len(), 1);

  let duplicate = session.add_answer(answer("q1", 1), "2026-02-27T00:02:00Z");
  assert!(matches!(
      duplicate,
      Err(InterviewSessionError::DuplicateAnswer { question_id, round: 1 }) if question_id == "q1"
  ));

  let empty_timestamp = session.add_answer(answer("q2", 1), "");
  assert!(matches!(
    empty_timestamp,
    Err(InterviewSessionError::EmptyTimestamp)
  ));
}

#[test]
fn add_answer_rejects_wrong_state_or_round() {
  let mut paused = make_session();
  paused.stage = InterviewStage::Paused;
  assert!(matches!(
    paused.add_answer(answer("q1", 1), "2026-02-27T00:01:00Z"),
    Err(InterviewSessionError::SessionPaused)
  ));

  let mut complete = make_session();
  complete.stage = InterviewStage::Complete;
  assert!(matches!(
    complete.add_answer(answer("q1", 1), "2026-02-27T00:01:00Z"),
    Err(InterviewSessionError::AlreadyComplete)
  ));

  let mut round_mismatch = make_session();
  let result = round_mismatch.add_answer(answer("q1", 5), "2026-02-27T00:01:00Z");
  assert!(matches!(
    result,
    Err(InterviewSessionError::RoundMismatch {
      answer_round: 5,
      current_round: 1,
    })
  ));
}

#[test]
fn complete_round_transitions_stages() {
  let mut session = make_session();

  assert!(session.complete_round("t1").is_ok());
  assert_eq!(session.stage, InterviewStage::Discovery);
  assert!(session.complete_round("t2").is_ok());
  assert_eq!(session.stage, InterviewStage::Discovery);
  assert!(session.complete_round("t3").is_ok());
  assert_eq!(session.stage, InterviewStage::Refinement);
  assert!(session.complete_round("t4").is_ok());
  assert_eq!(session.stage, InterviewStage::Validation);
  assert!(session.complete_round("t5").is_ok());
  assert_eq!(session.stage, InterviewStage::Complete);
  assert_eq!(session.completed_at, Some("t5".to_string()));
}

#[test]
fn complete_phase_behaves_as_expected() {
  let mut session = make_session();

  assert!(session.complete_phase(1, "t1").is_ok());
  assert_eq!(session.current_phase, 2);
  assert_eq!(session.completed_phases, vec![1]);

  assert!(session.complete_phase(1, "t2").is_ok());
  assert_eq!(session.completed_phases, vec![1]);

  assert!(session.complete_phase(3, "t3").is_ok());
  assert_eq!(session.current_phase, 2);
  assert!(session.completed_phases.contains(&3));

  assert!(matches!(
    session.complete_phase(0, "t4"),
    Err(InterviewSessionError::InvalidPhaseNumber { phase_number: 0 })
  ));
  assert!(matches!(
    session.complete_phase(4, ""),
    Err(InterviewSessionError::EmptyTimestamp)
  ));
}

#[test]
fn calculate_confidence_with_long_response_and_extracted_fields() {
  let mut extracted = HashMap::new();
  extracted.insert("name".to_string(), "test".to_string());

  let confidence = InterviewSession::calculate_confidence(
    "This is a very long response that has more than fifty characters in it!",
    &extracted,
  );

  assert!((confidence - 0.85).abs() < 0.001);
}

#[test]
fn calculate_confidence_with_short_response_and_extracted_fields() {
  let mut extracted = HashMap::new();
  extracted.insert("name".to_string(), "test".to_string());

  let confidence = InterviewSession::calculate_confidence("Short", &extracted);

  assert!((confidence - 0.6).abs() < 0.001);
}

#[test]
fn calculate_confidence_with_long_response_and_no_extracted_fields() {
  let extracted = HashMap::new();

  let confidence = InterviewSession::calculate_confidence(
    "This is a very long response that has more than fifty characters in it!",
    &extracted,
  );

  assert!((confidence - 0.6).abs() < 0.001);
}

#[test]
fn calculate_confidence_boundary_at_50_chars() {
  let with_extracted = {
    let mut m = HashMap::new();
    m.insert("key".to_string(), "value".to_string());
    m
  };

  let confidence_50 = InterviewSession::calculate_confidence(
    "12345678901234567890123456789012345678901234567890",
    &with_extracted,
  );
  assert!((confidence_50 - 0.6).abs() < 0.001);

  let confidence_51 = InterviewSession::calculate_confidence(
    "123456789012345678901234567890123456789012345678901",
    &with_extracted,
  );
  assert!((confidence_51 - 0.85).abs() < 0.001);
}

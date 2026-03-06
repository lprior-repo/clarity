#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::float_cmp, clippy::needless_collect, clippy::unnecessary_debug_formatting, clippy::match_same_arms, clippy::option_if_let_else, clippy::suspicious_else_formatting, clippy::manual_let_else, clippy::match_wild_err_arm, clippy::match_like_matches_macro)]
use crate::intent::interview::types::{
  Answer, Conflict, ConflictDetectionError, ConflictResolution, InterviewSession, Profile,
};

fn make_session() -> InterviewSession {
  InterviewSession::new(
    "test-session".to_string(),
    Profile::Api,
    "2026-02-27T00:00:00Z".to_string(),
  )
}

#[test]
fn detect_conflicts_requires_session_id() {
  let mut session = InterviewSession::default();
  let result = session.detect_conflicts();
  assert!(matches!(
    result,
    Err(ConflictDetectionError::EmptySessionId)
  ));
}

#[test]
fn detect_conflicts_requires_non_empty_question_id() {
  let mut session = make_session();
  session.answers.push(Answer {
    question_id: String::new(),
    question_text: "Q1".to_string(),
    response: "fast response".to_string(),
    ..Answer::default()
  });

  let result = session.detect_conflicts();
  assert!(matches!(
    result,
    Err(ConflictDetectionError::EmptyQuestionId(0))
  ));
}

#[test]
fn detects_cap_conflict() {
  let mut session = make_session();
  session.answers.push(Answer {
    question_id: "q-perf".to_string(),
    question_text: "Performance requirements".to_string(),
    response: "We need fast response times with low latency".to_string(),
    ..Answer::default()
  });
  session.answers.push(Answer {
    question_id: "q-data".to_string(),
    question_text: "Data requirements".to_string(),
    response: "Data must be consistent and accurate at all times".to_string(),
    ..Answer::default()
  });

  let result = session.detect_conflicts();
  assert!(result.is_ok());

  if let Ok(conflicts) = result {
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].id, "conflict-cap-0");
    assert_eq!(
      conflicts[0].between,
      ("q-perf".to_string(), "q-data".to_string())
    );
  } else {
    panic!("Should not be Ok");
  }
}

#[test]
fn detects_anonymous_audit_conflict() {
  let mut session = make_session();
  session.answers.push(Answer {
    question_id: "q-privacy".to_string(),
    question_text: "Privacy requirements".to_string(),
    response: "Users must remain anonymous".to_string(),
    ..Answer::default()
  });
  session.answers.push(Answer {
    question_id: "q-audit".to_string(),
    question_text: "Audit requirements".to_string(),
    response: "We need full audit trail for compliance".to_string(),
    ..Answer::default()
  });

  let result = session.detect_conflicts();
  assert!(result.is_ok());

  if let Ok(conflicts) = result {
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].id, "conflict-anonymous-audit-0");
  } else {
    panic!("Should not be Ok");
  }
}

#[test]
fn resolve_conflict_rejects_empty_id() {
  let mut session = make_session();
  let result = session.resolve_conflict("", 0);
  assert!(matches!(
    result,
    Err(ConflictDetectionError::EmptyConflictId)
  ));
}

#[test]
fn resolve_conflict_handles_validation_and_success() {
  let mut session = make_session();
  session.conflicts.push(Conflict {
    id: "conflict-1".to_string(),
    between: ("a".to_string(), "b".to_string()),
    description: "test".to_string(),
    impact: "test".to_string(),
    options: vec![
      ConflictResolution {
        option: "opt1".to_string(),
        description: "option 1".to_string(),
        tradeoffs: "tradeoffs".to_string(),
        recommendation: false,
      },
      ConflictResolution {
        option: "opt2".to_string(),
        description: "option 2".to_string(),
        tradeoffs: "tradeoffs".to_string(),
        recommendation: true,
      },
    ],
    chosen: None,
  });

  let negative = session.resolve_conflict("conflict-1", -1);
  assert!(matches!(
    negative,
    Err(ConflictDetectionError::NegativeOptionIndex(-1))
  ));

  let out_of_bounds = session.resolve_conflict("conflict-1", 9);
  assert!(matches!(
    out_of_bounds,
    Err(ConflictDetectionError::InvalidOptionIndex { .. })
  ));

  let ok = session.resolve_conflict("conflict-1", 1);
  assert!(ok.is_ok());
  assert_eq!(session.conflicts[0].chosen, Some(1));

  let already = session.resolve_conflict("conflict-1", 0);
  assert!(matches!(
      already,
      Err(ConflictDetectionError::ConflictAlreadyResolved(conflict_id)) if conflict_id == "conflict-1"
  ));
}

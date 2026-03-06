#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::float_cmp, clippy::needless_collect, clippy::unnecessary_debug_formatting, clippy::match_same_arms, clippy::option_if_let_else, clippy::suspicious_else_formatting, clippy::manual_let_else, clippy::match_wild_err_arm, clippy::match_like_matches_macro)]
use std::collections::HashMap;

use crate::intent::interview::types::{
  Answer, Gap, InterviewError, InterviewSession, InterviewSessionError, Perspective, Profile,
};

fn make_session(profile: Profile) -> InterviewSession {
  InterviewSession::new(
    "test-session".to_string(),
    profile,
    "2026-02-27T00:00:00Z".to_string(),
  )
}

#[test]
fn detect_gaps_handles_no_answers() {
  let session = make_session(Profile::Api);
  let gaps = session.detect_gaps();

  assert_eq!(gaps.len(), 5);
  assert!(gaps.iter().all(|gap| gap.blocking));
  assert!(gaps.iter().all(|gap| !gap.resolved));
}

#[test]
fn detect_gaps_handles_partial_answers() {
  let mut session = make_session(Profile::Api);
  let mut extracted = HashMap::new();
  extracted.insert(
    "base_url".to_string(),
    "https://api.example.com".to_string(),
  );
  extracted.insert("auth_method".to_string(), "Bearer".to_string());

  session.answers.push(Answer {
    question_id: "q1".to_string(),
    question_text: "What is the base URL?".to_string(),
    perspective: Perspective::Developer,
    round: 1,
    response: "The base URL is https://api.example.com with Bearer auth".to_string(),
    extracted,
    confidence: 0.9,
    notes: String::new(),
    timestamp: "2026-02-27T00:00:00Z".to_string(),
  });

  let gaps = session.detect_gaps();
  assert_eq!(gaps.len(), 3);
  assert!(gaps.iter().all(|gap| gap.field != "base_url"));
  assert!(gaps.iter().all(|gap| gap.field != "auth_method"));
}

#[test]
fn get_blocking_gaps_filters_correctly() {
  let mut session = make_session(Profile::Api);
  session.gaps = vec![
    Gap {
      id: "gap-1".to_string(),
      field: "field1".to_string(),
      blocking: true,
      resolved: false,
      ..Gap::default()
    },
    Gap {
      id: "gap-2".to_string(),
      field: "field2".to_string(),
      blocking: true,
      resolved: true,
      ..Gap::default()
    },
    Gap {
      id: "gap-3".to_string(),
      field: "field3".to_string(),
      blocking: false,
      resolved: false,
      ..Gap::default()
    },
  ];

  let blocking = session.get_blocking_gaps();
  assert_eq!(blocking.len(), 1);
  assert_eq!(blocking[0].id, "gap-1");
}

#[test]
fn resolve_gap_validates_and_updates_state() {
  let mut session = make_session(Profile::Api);
  session.gaps.push(Gap {
    id: "gap-base_url".to_string(),
    field: "base_url".to_string(),
    blocking: true,
    resolved: false,
    resolution: String::new(),
    ..Gap::default()
  });

  let ok = session.resolve_gap("gap-base_url", "https://api.example.com");
  assert!(ok.is_ok());
  assert!(session.gaps[0].resolved);
  assert_eq!(session.gaps[0].resolution, "https://api.example.com");

  assert_eq!(
    session.resolve_gap("", "x"),
    Err(InterviewError::EmptyGapId)
  );
  assert_eq!(
    session.resolve_gap("   ", "x"),
    Err(InterviewError::EmptyGapId)
  );
  assert_eq!(
    session.resolve_gap("gap-base_url", ""),
    Err(InterviewError::EmptyResolution)
  );
  assert_eq!(
    session.resolve_gap("missing", "value"),
    Err(InterviewError::GapNotFound("missing".to_string()))
  );
}

#[test]
fn can_proceed_blocks_on_unresolved_blocking_gap() {
  let mut session = make_session(Profile::Api);
  session.gaps.push(Gap {
    id: "gap-1".to_string(),
    field: "test".to_string(),
    description: "Missing".to_string(),
    blocking: true,
    resolved: false,
    ..Gap::default()
  });

  let result = session.can_proceed();
  assert!(matches!(
      result,
      Err(InterviewSessionError::BlockingGapsUnresolved { count: 1, gap_ids })
      if gap_ids == vec!["gap-1".to_string()]
  ));
}

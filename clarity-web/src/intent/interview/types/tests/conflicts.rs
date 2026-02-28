use crate::intent::interview::types::models::{ConflictState, ConflictStateError};
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
    assert!(false);
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
    assert!(false);
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
    state: ConflictState::Pending,
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
  assert_eq!(session.conflicts[0].chosen_index(), Some(1));

  let already = session.resolve_conflict("conflict-1", 0);
  assert!(matches!(
      already,
      Err(ConflictDetectionError::ConflictAlreadyResolved(conflict_id)) if conflict_id == "conflict-1"
  ));
}

// ============================================
// Exhaustive match tests for ConflictState
// ============================================

#[test]
fn conflict_state_pending_is_not_resolved() {
  let state = ConflictState::Pending;
  assert!(!state.is_resolved());
  assert!(state.chosen_index().is_none());
}

#[test]
fn conflict_state_resolved_is_resolved() {
  let state = ConflictState::Resolved { chosen_index: 1 };
  assert!(state.is_resolved());
  assert_eq!(state.chosen_index(), Some(1));
}

#[test]
fn conflict_state_resolve_rejects_negative_index() {
  let state = ConflictState::Pending;
  let result = state.resolve(-1, 2);
  assert!(result.is_err());
}

#[test]
fn conflict_state_resolve_rejects_out_of_bounds() {
  let state = ConflictState::Pending;
  let result = state.resolve(5, 2);
  assert!(result.is_err());
}

#[test]
fn conflict_state_resolve_rejects_already_resolved() {
  let state = ConflictState::Resolved { chosen_index: 0 };
  let result = state.resolve(1, 2);
  assert!(result.is_err());
}

#[test]
fn conflict_state_resolve_succeeds_with_valid_index() {
  let state = ConflictState::Pending;
  let result = state.resolve(1, 3);
  assert!(result.is_ok());
  let new_state = result.unwrap();
  assert!(new_state.is_resolved());
  assert_eq!(new_state.chosen_index(), Some(1));
}

#[test]
fn conflict_is_resolved_delegates_to_state() {
  let pending_conflict = Conflict {
    state: ConflictState::Pending,
    ..Conflict::default()
  };
  let resolved_conflict = Conflict {
    state: ConflictState::Resolved { chosen_index: 0 },
    ..Conflict::default()
  };

  assert!(!pending_conflict.is_resolved());
  assert!(resolved_conflict.is_resolved());
}

#[test]
fn conflict_chosen_index_delegates_to_state() {
  let pending_conflict = Conflict {
    state: ConflictState::Pending,
    ..Conflict::default()
  };
  let resolved_conflict = Conflict {
    state: ConflictState::Resolved { chosen_index: 2 },
    ..Conflict::default()
  };

  assert!(pending_conflict.chosen_index().is_none());
  assert_eq!(resolved_conflict.chosen_index(), Some(2));
}

// ============================================
// P0: State Machine Validation Tests
// ============================================

#[test]
fn conflict_state_one_way_transition_enforced() {
  // P0: Cannot re-resolve an already resolved conflict
  let resolved = ConflictState::Resolved { chosen_index: 0 };

  let result = resolved.resolve(1, 5);
  assert!(result.is_err());
  assert_eq!(result, Err(ConflictStateError::AlreadyResolved));
}

#[test]
fn conflict_state_can_transition_to_validates_one_way() {
  let pending = ConflictState::Pending;
  let resolved = ConflictState::Resolved { chosen_index: 0 };

  // Pending can transition to Resolved
  assert!(pending.can_transition_to(&resolved));

  // Resolved cannot transition back to Pending
  assert!(!resolved.can_transition_to(&ConflictState::Pending));

  // Same state is valid (no-op)
  assert!(pending.can_transition_to(&ConflictState::Pending));
  assert!(resolved.can_transition_to(&resolved));
}

#[test]
fn conflict_state_rejects_empty_options() {
  // P1: Cannot resolve with no options
  let pending = ConflictState::Pending;
  let result = pending.resolve(0, 0);
  assert!(result.is_err());
  assert_eq!(result, Err(ConflictStateError::EmptyOptions));
}

#[test]
fn conflict_state_rejects_out_of_bounds_even_with_zero_index() {
  // When options exist, index 0 is valid
  let pending = ConflictState::Pending;
  let result = pending.resolve(0, 1);
  assert!(result.is_ok());

  // But with no options, even index 0 is invalid
  let pending2 = ConflictState::Pending;
  let result2 = pending2.resolve(0, 0);
  assert_eq!(result2, Err(ConflictStateError::EmptyOptions));
}

#[test]
fn conflict_state_validate_catches_negative_index() {
  // Valid state
  let valid_resolved = ConflictState::Resolved { chosen_index: 0 };
  assert!(valid_resolved.validate().is_ok());

  // Invalid: negative index
  let invalid_resolved = ConflictState::Resolved { chosen_index: -1 };
  assert_eq!(
    invalid_resolved.validate(),
    Err(ConflictStateError::NegativeIndex(-1))
  );

  // Pending state is always valid
  assert!(ConflictState::Pending.validate().is_ok());
}

#[test]
fn conflict_state_validate_bounds_full_check() {
  // Valid bounds
  let resolved = ConflictState::Resolved { chosen_index: 2 };
  assert!(resolved.validate_bounds(5).is_ok());
  assert!(resolved.validate_bounds(3).is_ok());

  // Out of bounds
  assert!(resolved.validate_bounds(2).is_err());
  assert!(resolved.validate_bounds(1).is_err());

  // Empty options
  let resolved2 = ConflictState::Resolved { chosen_index: 0 };
  assert_eq!(
    resolved2.validate_bounds(0),
    Err(ConflictStateError::EmptyOptions)
  );

  // Negative index
  let invalid = ConflictState::Resolved { chosen_index: -5 };
  assert_eq!(
    invalid.validate_bounds(10),
    Err(ConflictStateError::NegativeIndex(-5))
  );

  // Pending is always valid
  assert!(ConflictState::Pending.validate_bounds(0).is_ok());
  assert!(ConflictState::Pending.validate_bounds(100).is_ok());
}

#[test]
fn conflict_state_is_pending_predicate() {
  assert!(ConflictState::Pending.is_pending());
  assert!(!ConflictState::Resolved { chosen_index: 0 }.is_pending());
}

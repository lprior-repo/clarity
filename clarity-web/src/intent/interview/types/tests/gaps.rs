use std::collections::HashMap;

use crate::intent::interview::types::models::{GapState, GapStateError};
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
  assert!(gaps.iter().all(|gap| !gap.is_resolved()));
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
      state: GapState::Open,
      ..Gap::default()
    },
    Gap {
      id: "gap-2".to_string(),
      field: "field2".to_string(),
      blocking: true,
      state: GapState::Resolved {
        resolution: "done".to_string(),
      },
      ..Gap::default()
    },
    Gap {
      id: "gap-3".to_string(),
      field: "field3".to_string(),
      blocking: false,
      state: GapState::Open,
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
    state: GapState::Open,
    ..Gap::default()
  });

  let ok = session.resolve_gap("gap-base_url", "https://api.example.com");
  assert!(ok.is_ok());
  assert!(session.gaps[0].is_resolved());
  assert_eq!(
    session.gaps[0].resolution(),
    Some("https://api.example.com")
  );

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
    state: GapState::Open,
    ..Gap::default()
  });

  let result = session.can_proceed();
  assert!(matches!(
      result,
      Err(InterviewSessionError::BlockingGapsUnresolved { count: 1, gap_ids })
      if gap_ids == vec!["gap-1".to_string()]
  ));
}

// ============================================
// Exhaustive match tests for GapState
// ============================================

#[test]
fn gap_state_open_is_not_resolved() {
  let state = GapState::Open;
  assert!(!state.is_resolved());
  assert!(state.resolution().is_none());
}

#[test]
fn gap_state_resolved_is_resolved() {
  let state = GapState::Resolved {
    resolution: "fixed".to_string(),
  };
  assert!(state.is_resolved());
  assert_eq!(state.resolution(), Some("fixed"));
}

#[test]
fn gap_state_resolve_validates_non_empty() {
  let state = GapState::Open;
  let result = state.resolve(String::new());
  assert!(result.is_err());
}

#[test]
fn gap_state_resolve_validates_whitespace() {
  let state = GapState::Open;
  let result = state.resolve("   ".to_string());
  assert!(result.is_err());
}

#[test]
fn gap_state_resolve_succeeds_with_valid_text() {
  let state = GapState::Open;
  let result = state.resolve("valid resolution".to_string());
  if let Ok(new_state) = result {
    assert!(new_state.is_resolved());
    assert_eq!(new_state.resolution(), Some("valid resolution"));
  } else {
    panic!("resolve should succeed for valid text");
  }
}

#[test]
fn gap_is_resolved_delegates_to_state() {
  let open_gap = Gap {
    state: GapState::Open,
    ..Gap::default()
  };
  let resolved_gap = Gap {
    state: GapState::Resolved {
      resolution: "done".to_string(),
    },
    ..Gap::default()
  };

  assert!(!open_gap.is_resolved());
  assert!(resolved_gap.is_resolved());
}

#[test]
fn gap_resolution_delegates_to_state() {
  let open_gap = Gap {
    state: GapState::Open,
    ..Gap::default()
  };
  let resolved_gap = Gap {
    state: GapState::Resolved {
      resolution: "done".to_string(),
    },
    ..Gap::default()
  };

  assert!(open_gap.resolution().is_none());
  assert_eq!(resolved_gap.resolution(), Some("done"));
}

// ============================================
// P0: State Machine Validation Tests
// ============================================

#[test]
fn gap_state_one_way_transition_enforced() {
  // P0: Cannot re-resolve an already resolved gap
  let resolved = GapState::Resolved {
    resolution: "first resolution".to_string(),
  };

  let result = resolved.resolve("second resolution".to_string());
  assert!(result.is_err());
  assert_eq!(result, Err(GapStateError::AlreadyResolved));
}

#[test]
fn gap_state_can_transition_to_validates_one_way() {
  let open = GapState::Open;
  let resolved = GapState::Resolved {
    resolution: "done".to_string(),
  };

  // Open can transition to Resolved
  assert!(open.can_transition_to(&resolved));

  // Resolved cannot transition back to Open
  assert!(!resolved.can_transition_to(&GapState::Open));

  // Same state is valid (no-op)
  assert!(open.can_transition_to(&GapState::Open));
  assert!(resolved.can_transition_to(&resolved));
}

#[test]
fn gap_state_validate_catches_empty_resolution() {
  // P1: Resolution must be non-empty
  let invalid_resolved = GapState::Resolved {
    resolution: String::new(),
  };
  assert_eq!(
    invalid_resolved.validate(),
    Err(GapStateError::EmptyResolution)
  );

  let whitespace_resolved = GapState::Resolved {
    resolution: "   ".to_string(),
  };
  assert_eq!(
    whitespace_resolved.validate(),
    Err(GapStateError::EmptyResolution)
  );

  // Valid state
  let valid_resolved = GapState::Resolved {
    resolution: "valid".to_string(),
  };
  assert!(valid_resolved.validate().is_ok());

  // Open state is always valid
  assert!(GapState::Open.validate().is_ok());
}

#[test]
fn gap_state_is_open_predicate() {
  assert!(GapState::Open.is_open());
  assert!(!GapState::Resolved {
    resolution: "x".to_string()
  }
  .is_open());
}
